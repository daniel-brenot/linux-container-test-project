//! Live `fork`+`exec` helpers that speak first on an IPC channel.
//!
//! Theia plugin-host and Node `child_process.fork` create a socketpair, fork,
//! exec a child that must stay alive, and read a handshake the *child* writes.
//! A runtime that invents an immediate zombie (exit 0, EOF on the channel)
//! or refuses to run the new image (exit 127) fails these cases. Existing
//! echo helpers write only after the parent, so they do not catch that.
//!
//! A second class of failure is nest restore: the parent HTTP listen must
//! still be accept()'able (via epoll) while the nested helper is running.
//! After the workbench page has loaded, an already-accepted TCP connection
//! (the frontend websocket) must keep serving, and epoll must go idle rather
//! than livelock on leftover socketpair POLLOUT.

use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::runtime;
use crate::syscall::{
    self, clock, epoll, fcntl_cmd, map, oflag, poll, prot, wait, SockAddrIn, AF_INET, AF_UNIX,
    EPOLLET, EPOLLIN, EPOLLOUT, EPOLL_CTL_ADD, F_OK, POLLIN, POLLHUP, SIGKILL, SOCK_CLOEXEC,
    SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR,
};

fn self_exe(buf: &mut [u8; 256]) -> Result<usize, crate::harness::AssertFail> {
    let n = check_ok!(
        syscall::readlink(b"/proc/self/exe\0", buf),
        "readlink /proc/self/exe"
    );
    check!(n > 0 && n < buf.len(), "exe path len");
    buf[n] = 0;
    Ok(n + 1)
}

fn spawn_hello_helper(exe: &[u8]) -> Result<(i32, i32), crate::harness::AssertFail> {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(a);
        if syscall::dup2(b, 3).is_err() {
            syscall::exit(125);
        }
        if b != 3 {
            let _ = syscall::close(b);
        }
        let env0 = b"CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let mut arg0 = [0u8; 256];
        arg0[..exe.len()].copy_from_slice(exe);
        let flag = b"--ipc-channel-hello\0";
        let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(exe, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    Ok((a, pid))
}

fn child_still_running(pid: i32) -> Result<bool, crate::harness::AssertFail> {
    let mut status = 0;
    match syscall::wait4(pid, &mut status, wait::WNOHANG) {
        Ok(0) => Ok(true),
        Ok(p) if p == pid => Ok(false),
        Ok(_) => Ok(true),
        Err(syscall::Errno::ECHILD) => Ok(false),
        Err(_) => Err(crate::harness::AssertFail::msg("wait4")),
    }
}

fn read_hello(fd: i32) -> Result<(), crate::harness::AssertFail> {
    let mut got = [0u8; 5];
    let mut filled = 0usize;
    for _ in 0..50 {
        let mut fds = [poll::PollFd {
            fd,
            events: POLLIN | POLLHUP,
            revents: 0,
        }];
        let pr = check_ok!(syscall::poll(&mut fds, 100), "poll hello");
        if pr == 0 {
            continue;
        }
        if fds[0].revents & POLLHUP != 0 && fds[0].revents & POLLIN == 0 {
            return Err(crate::harness::AssertFail::msg("ipc eof"));
        }
        match syscall::read(fd, &mut got[filled..]) {
            Ok(0) => return Err(crate::harness::AssertFail::msg("ipc eof")),
            Ok(n) => {
                filled += n;
                if filled >= 5 {
                    check_eq!(&got[..5], b"HELLO", "hello");
                    return Ok(());
                }
            }
            Err(syscall::Errno::EAGAIN) => {}
            Err(_) => return Err(crate::harness::AssertFail::msg("read hello")),
        }
    }
    Err(crate::harness::AssertFail::msg("hello timeout"))
}

fn reap_or_kill(pid: i32) {
    let mut status = 0;
    for _ in 0..50 {
        match syscall::wait4(pid, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid => return,
            _ => {
                let req = syscall::Timespec {
                    tv_sec: 0,
                    tv_nsec: 20_000_000,
                };
                let _ = syscall::nanosleep(&req);
            }
        }
    }
    let _ = syscall::kill(pid, SIGKILL);
    let _ = syscall::wait4(pid, &mut status, 0);
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a fork+exec helper that writes HELLO first on fd 3 stays alive (waitpid WNOHANG is 0)"
)]
fn spawn_hello_child_still_alive() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (sock, pid) = spawn_hello_helper(&exe[..exe_len])?;
    // Instant-zombie fork reaps immediately with exit 0; a refused exec reaps
    // with 127. Either way the plugin-host analogue is already dead.
    check!(child_still_running(pid)?, "child already exited");
    let _ = syscall::close(sock);
    reap_or_kill(pid);
    Ok(())
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a fork+exec helper writes HELLO on the IPC socket before the parent writes"
)]
fn spawn_hello_child_speaks_first() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (sock, pid) = spawn_hello_helper(&exe[..exe_len])?;
    check!(child_still_running(pid)?, "child already exited");
    read_hello(sock)?;
    check!(child_still_running(pid)?, "exited after hello");
    let _ = syscall::close(sock);
    reap_or_kill(pid);
    Ok(())
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "while a live IPC helper is running the parent can still accept a loopback TCP connection"
)]
fn spawn_hello_parent_still_accepts_tcp() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (sock, pid) = spawn_hello_helper(&exe[..exe_len])?;
    check!(child_still_running(pid)?, "child already exited");
    read_hello(sock)?;

    let srv = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "socket"
    );
    let one = 1i32.to_ne_bytes();
    check_ok!(
        syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one),
        "SO_REUSEADDR"
    );
    let addr = SockAddrIn::loopback(0);
    check_ok!(syscall::bind(srv, &addr), "bind");
    check_ok!(syscall::listen(srv, 8), "listen");
    let bound = check_ok!(syscall::getsockname_in(srv), "getsockname");
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::send(cli, b"web", 0), "send");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(acc, &mut buf, 0), "recv"), 3, "len");
    check_eq!(&buf[..3], b"web", "payload");
    check!(child_still_running(pid)?, "helper died during accept");
    let _ = syscall::close(acc);
    let _ = syscall::close(cli);
    let _ = syscall::close(srv);
    let _ = syscall::close(sock);
    reap_or_kill(pid);
    Ok(())
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "libuv-style stdout pipe plus IPC: the execed helper writes ready on stdout and HELLO on fd 3"
)]
fn spawn_hello_stdout_pipe_and_ipc() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (ipc_a, ipc_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "ipc");
    let (out_r, out_w) = check_ok!(syscall::pipe2(0), "stdout pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(ipc_a);
        let _ = syscall::close(out_r);
        if syscall::dup2(ipc_b, 3).is_err() || syscall::dup2(out_w, 1).is_err() {
            syscall::exit(125);
        }
        if ipc_b != 3 {
            let _ = syscall::close(ipc_b);
        }
        if out_w != 1 {
            let _ = syscall::close(out_w);
        }
        let env0 = b"CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let mut arg0 = [0u8; 256];
        arg0[..exe_len].copy_from_slice(&exe[..exe_len]);
        let flag = b"--ipc-channel-hello\0";
        let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(&exe[..exe_len], &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(ipc_b);
    let _ = syscall::close(out_w);
    check!(child_still_running(pid)?, "child already exited");
    read_hello(ipc_a)?;
    check!(child_still_running(pid)?, "exited after hello");
    let _ = syscall::close(out_r);
    let _ = syscall::close(ipc_a);
    reap_or_kill(pid);
    Ok(())
}

fn set_nonblock(fd: i32) -> Result<(), crate::harness::AssertFail> {
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "F_GETFL");
    check_ok!(
        syscall::fcntl(
            fd,
            fcntl_cmd::F_SETFL,
            (fl as i32 | oflag::O_NONBLOCK) as usize
        ),
        "F_SETFL O_NONBLOCK"
    );
    Ok(())
}

fn listen_loopback() -> Result<(i32, SockAddrIn), crate::harness::AssertFail> {
    let srv = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "listen socket"
    );
    let one = 1i32.to_ne_bytes();
    check_ok!(
        syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one),
        "SO_REUSEADDR"
    );
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "bind");
    check_ok!(syscall::listen(srv, 128), "listen");
    set_nonblock(srv)?;
    let bound = check_ok!(syscall::getsockname_in(srv), "getsockname");
    Ok((srv, bound))
}

/// Theia/Node `child_process.fork` shape: unix socketpairs for stdio + IPC +
/// spawn-error, TCP listen left open, child `dup2`s the listen socket onto
/// fd 4 (xvisor always steals fd 4 into the nested child). Exec a nestable
/// `sh -c` that blocks on the IPC fd so the child stays alive.
fn spawn_nested_plugin_host_shape(
    listen: i32,
) -> Result<(i32, i32, [i32; 4]), crate::harness::AssertFail> {
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "/bin/sh missing");
    let (ipc_a, ipc_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "ipc");
    let (in_a, in_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "stdin");
    let (out_a, out_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "stdout");
    let (err_a, err_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "stderr");
    let (st_a, st_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "spawn-error");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(ipc_a);
        let _ = syscall::close(in_a);
        let _ = syscall::close(out_a);
        let _ = syscall::close(err_a);
        let _ = syscall::close(st_a);
        // Park listen + IPC above the stdio/channel slots before dup2, so
        // overwriting fd 3/4 cannot clobber the source descriptors.
        let listen_keep = match syscall::fcntl(listen, fcntl_cmd::F_DUPFD, 20) {
            Ok(fd) => fd as i32,
            Err(_) => syscall::exit(124),
        };
        let ipc_keep = match syscall::fcntl(ipc_b, fcntl_cmd::F_DUPFD, 20) {
            Ok(fd) => fd as i32,
            Err(_) => syscall::exit(124),
        };
        if syscall::dup2(in_b, 0).is_err()
            || syscall::dup2(out_b, 1).is_err()
            || syscall::dup2(err_b, 2).is_err()
            || syscall::dup2(ipc_keep, 3).is_err()
            || syscall::dup2(listen_keep, 4).is_err()
        {
            syscall::exit(125);
        }
        if in_b != 0 {
            let _ = syscall::close(in_b);
        }
        if out_b != 1 {
            let _ = syscall::close(out_b);
        }
        if err_b != 2 {
            let _ = syscall::close(err_b);
        }
        if ipc_b != 3 {
            let _ = syscall::close(ipc_b);
        }
        if ipc_keep != 3 {
            let _ = syscall::close(ipc_keep);
        }
        if listen_keep != 4 {
            let _ = syscall::close(listen_keep);
        }
        if listen != 4 {
            let _ = syscall::close(listen);
        }
        if st_b != 4 {
            let _ = syscall::close(st_b);
        }
        let env0 = b"NODE_CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        // Builtin `read` blocks on fd 3 until the parent writes or hangs up.
        // Not stubbed as echo/ls/true, so a cooperative runtime must nest it.
        let arg2 = b"read line <&3\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(ipc_b);
    let _ = syscall::close(in_b);
    let _ = syscall::close(out_b);
    let _ = syscall::close(err_b);
    let _ = syscall::close(st_b);
    Ok((pid, ipc_a, [in_a, out_a, err_a, st_a]))
}

fn http_roundtrip(srv: i32, bound: &SockAddrIn) -> Result<(), crate::harness::AssertFail> {
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, bound), "connect after nest");
    let mut pfds = [poll::PollFd {
        fd: srv,
        events: POLLIN,
        revents: 0,
    }];
    check!(
        check_ok!(syscall::poll(&mut pfds, 2000), "poll listen") >= 1,
        "listen not readable after nest"
    );
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::send(cli, b"GET /", 0), "send");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(acc, &mut buf, 0), "recv"), 5, "len");
    check_eq!(&buf[..5], b"GET /", "payload");
    let _ = syscall::close(acc);
    let _ = syscall::close(cli);
    Ok(())
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "after fork+exec of a live nested helper that inherits the TCP listen on fd 4, the parent listen fd still accepts"
)]
fn spawn_nested_child_parent_tcp_listen_survives() -> TestResult {
    // Theia plugin-host nest restores the parent and steals fd 3 (IPC) plus
    // whatever sits at fd 4. If fd 4 is a TCP socket, identifying it by
    // Darwin fstat (dev=0, ino=0 for every TCP fd) closes the HTTP listen
    // in the parent: the page connects into the kernel backlog and never
    // gets accept()'d.
    let (srv, bound) = listen_loopback()?;
    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid)?, "child already exited");
    check_ok!(
        syscall::fcntl(srv, fcntl_cmd::F_GETFL, 0),
        "listen fd closed by nest"
    );
    http_roundtrip(srv, &bound)?;
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    Ok(())
}

/// Theia spawns plugin-host, then later `uv_spawn`s again (pty, git, …). The
/// second fork used to arm COW, suspend UV workers that held libc locks, and
/// leave the parent spinning inside `uv_spawn` so HTTP stopped accepting.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a second plugin-host-shaped fork+exec still leaves the parent TCP listen accepting"
)]
fn spawn_second_nested_plugin_host_parent_still_accepts_http() -> TestResult {
    let (srv, bound) = listen_loopback()?;
    let (pid1, ipc1, extra1) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid1)?, "first child already exited");
    http_roundtrip(srv, &bound)?;
    let _ = syscall::close(ipc1);
    for fd in extra1 {
        let _ = syscall::close(fd);
    }
    reap_or_kill(pid1);
    let (pid2, ipc2, extra2) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid2)?, "second child already exited");
    http_roundtrip(srv, &bound)?;
    let _ = syscall::close(ipc2);
    for fd in extra2 {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid2);
    Ok(())
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "after a nested plugin-host-shaped fork+exec, epoll_wait still reports the TCP listen so HTTP can be accepted"
)]
fn spawn_nested_child_parent_epoll_still_accepts_http() -> TestResult {
    // Node HTTP is nonblocking listen + epoll (EPOLLIN|EPOLLET) while libuv
    // also watches IPC/stdio socketpairs with EPOLLIN|EPOLLOUT|EPOLLET.
    // A nest restore that drops the listen fd or leaves POLLOUT always-ready
    // wedges the event loop: curl hangs with 0 bytes and the page never loads.
    let (srv, bound) = listen_loopback()?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLET,
        data: srv as u64,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, srv, &mut ev), "add listen");

    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid)?, "child already exited");

    ev.events = EPOLLIN | EPOLLOUT | EPOLLET;
    ev.data = ipc as u64;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, ipc, &mut ev), "add ipc");
    for &fd in &extra {
        ev.data = fd as u64;
        let _ = syscall::epoll_ctl(ep, EPOLL_CTL_ADD, fd, &mut ev);
    }

    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, &bound), "connect");

    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    let mut saw_listen = false;
    for _ in 0..20 {
        let n = check_ok!(syscall::epoll_wait(ep, &mut out, 100), "epoll_wait");
        for e in out.iter().take(n) {
            if e.data == srv as u64 && e.events & EPOLLIN != 0 {
                saw_listen = true;
            }
        }
        if saw_listen {
            break;
        }
    }
    check!(saw_listen, "listen not ready in epoll after nest");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::send(cli, b"web", 0), "send");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(acc, &mut buf, 0), "recv"), 3, "len");
    check_eq!(&buf[..3], b"web", "payload");

    let _ = syscall::close(acc);
    let _ = syscall::close(cli);
    let _ = syscall::close(ep);
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    Ok(())
}

fn add_socket_to_epoll(ep: i32, fd: i32, events: u32) -> Result<(), crate::harness::AssertFail> {
    let mut ev = epoll::EpollEvent {
        events,
        data: fd as u64,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, fd, &mut ev), "epoll add");
    Ok(())
}

/// Wait until `epoll_wait` reports `want_bit` for `want_data`, skipping other
/// ready fds (libuv watches many always-writable socketpairs).
fn wait_epoll_bit(
    ep: i32,
    want_data: u64,
    want_bit: u32,
    iters: u32,
) -> Result<bool, crate::harness::AssertFail> {
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    for _ in 0..iters {
        let n = check_ok!(syscall::epoll_wait(ep, &mut out, 100), "epoll_wait");
        for e in out.iter().take(n) {
            if e.data == want_data && e.events & want_bit != 0 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn epoll_event_is_live_io(e: &epoll::EpollEvent) -> bool {
    e.events & (EPOLLIN | EPOLLOUT) != 0 && e.events & syscall::EPOLLHUP == 0
}

fn timespec_ms(t: &syscall::Timespec) -> i64 {
    t.tv_sec.saturating_mul(1000).saturating_add(t.tv_nsec / 1_000_000)
}

/// Drain edge-triggered leftovers with timeout 0, then a short wait must block.
///
/// Returning 0 events immediately is not idle: Darwin `poll` still sees
/// POLLOUT on leftover socketpairs after EPOLLET has disarmed them. Linux
/// `epoll_wait` sleeps until a new edge; a runtime that returns in ~0ms
/// busy-loops Theia after plugin-host nest (page loaded, then frozen).
fn epoll_goes_idle(ep: i32) -> Result<bool, crate::harness::AssertFail> {
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    let mut drained = false;
    for _ in 0..64 {
        let n = check_ok!(syscall::epoll_wait(ep, &mut out, 0), "epoll drain");
        if n == 0 || !out.iter().take(n).any(epoll_event_is_live_io) {
            drained = true;
            break;
        }
    }
    if !drained {
        return Ok(false);
    }
    let t0 = check_ok!(
        syscall::clock_gettime(clock::CLOCK_MONOTONIC),
        "clock before idle"
    );
    let n2 = check_ok!(syscall::epoll_wait(ep, &mut out, 50), "epoll idle");
    let t1 = check_ok!(
        syscall::clock_gettime(clock::CLOCK_MONOTONIC),
        "clock after idle"
    );
    if n2 != 0 && out.iter().take(n2).any(epoll_event_is_live_io) {
        return Ok(false);
    }
    Ok(timespec_ms(&t1).saturating_sub(timespec_ms(&t0)) >= 20)
}

fn recv_exact(fd: i32, want: &[u8]) -> Result<(), crate::harness::AssertFail> {
    let mut buf = [0u8; 8];
    check!(want.len() <= buf.len(), "recv_exact buf");
    check_eq!(
        check_ok!(syscall::recv(fd, &mut buf[..want.len()], 0), "recv"),
        want.len(),
        "recv len"
    );
    check_eq!(&buf[..want.len()], want, "recv payload");
    Ok(())
}

/// Watch IPC/stdio the way libuv does *before* nest, accept a TCP client
/// (page load), nest plugin-host, then use the same connection.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "an accepted TCP connection kept in epoll across plugin-host nest still reads and writes (workbench websocket after page load)"
)]
fn spawn_nested_existing_tcp_still_serves() -> TestResult {
    // Theia serves HTML, then keeps the frontend websocket accepted while
    // plugin-host fork+exec nests. Tests that only accept *after* nest miss
    // a restore that closes or wedges the already-accepted fd: the page
    // appears, then "reconnecting channel" / frozen layout.
    let (srv, bound) = listen_loopback()?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, srv, EPOLLIN | EPOLLET)?;

    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, &bound), "connect page");
    check!(
        wait_epoll_bit(ep, srv as u64, EPOLLIN, 20)?,
        "listen not ready before nest"
    );
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept page");
    set_nonblock(acc)?;
    add_socket_to_epoll(ep, acc, EPOLLIN | EPOLLOUT | EPOLLET)?;

    check_ok!(syscall::send(cli, b"PAGE", 0), "send page");
    check!(
        wait_epoll_bit(ep, acc as u64, EPOLLIN, 20)?,
        "accepted fd not readable for page"
    );
    recv_exact(acc, b"PAGE")?;
    check_ok!(syscall::send(acc, b"OK", 0), "send ok");
    let mut echo = [0u8; 2];
    check_eq!(check_ok!(syscall::recv(cli, &mut echo, 0), "cli recv"), 2, "ok len");
    check_eq!(&echo, b"OK", "ok");

    // libuv already watches spawn/IPC socketpairs before fork. Both ends of a
    // leftover pair stay in the parent and are typically POLLOUT-ready.
    let (left_a, left_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "leftover");
    add_socket_to_epoll(ep, left_a, EPOLLIN | EPOLLOUT | EPOLLET)?;
    add_socket_to_epoll(ep, left_b, EPOLLIN | EPOLLOUT | EPOLLET)?;

    // Observe not-ready so EPOLLET rearms POLLIN before nest (libuv does this
    // by returning to epoll_wait after draining the websocket).
    let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    let _ = syscall::epoll_wait(ep, &mut drain, 0);

    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid)?, "child already exited");
    add_socket_to_epoll(ep, ipc, EPOLLIN | EPOLLOUT | EPOLLET)?;
    for &fd in &extra {
        let _ = add_socket_to_epoll(ep, fd, EPOLLIN | EPOLLOUT | EPOLLET);
    }

    check_ok!(
        syscall::fcntl(acc, fcntl_cmd::F_GETFL, 0),
        "accepted fd closed by nest"
    );
    check_ok!(syscall::send(cli, b"PING", 0), "send ping");
    check!(
        wait_epoll_bit(ep, acc as u64, EPOLLIN, 20)?,
        "accepted fd not readable after nest"
    );
    recv_exact(acc, b"PING")?;
    check_ok!(syscall::send(acc, b"PONG", 0), "send pong");
    let mut pong = [0u8; 4];
    check_eq!(check_ok!(syscall::recv(cli, &mut pong, 0), "cli pong"), 4, "pong len");
    check_eq!(&pong, b"PONG", "pong");

    http_roundtrip(srv, &bound)?;

    let _ = syscall::close(acc);
    let _ = syscall::close(cli);
    let _ = syscall::close(left_a);
    let _ = syscall::close(left_b);
    let _ = syscall::close(ep);
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    Ok(())
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "after plugin-host nest with leftover socketpairs in epoll, a 50ms epoll_wait blocks instead of returning immediately"
)]
fn spawn_nested_epoll_goes_idle() -> TestResult {
    // libuv watches spawn socketpairs with EPOLLIN|EPOLLOUT|EPOLLET. After the
    // edge is consumed, Linux epoll_wait(timeout) sleeps — it does not poll
    // host POLLOUT and return 0 in a tight loop. A runtime that returns
    // immediately pegs a core and the loaded page stops handling the websocket.
    //
    // Only watch live socketpair ends (both held in the parent). The spawn-error
    // pipe is already hung up after nest and would make epoll_wait return
    // immediately with EPOLLHUP on Linux too.
    let (srv, bound) = listen_loopback()?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, srv, EPOLLIN | EPOLLET)?;
    let (left_a, left_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "leftover");
    add_socket_to_epoll(ep, left_a, EPOLLIN | EPOLLOUT | EPOLLET)?;
    add_socket_to_epoll(ep, left_b, EPOLLIN | EPOLLOUT | EPOLLET)?;

    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    let still = child_still_running(pid);
    let leftover_ok = still.as_ref().ok().copied() == Some(true)
        && syscall::send(left_a, b"x", 0).is_ok();
    let idle = if leftover_ok {
        // Consume the byte we just sent so POLLIN is not a live edge.
        let mut b = [0u8; 1];
        let _ = syscall::recv(left_b, &mut b, 0);
        let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 16];
        let _ = syscall::epoll_wait(ep, &mut drain, 0);
        epoll_goes_idle(ep)
    } else {
        Ok(false)
    };
    let http = match idle {
        Ok(true) => http_roundtrip(srv, &bound),
        _ => Ok(()),
    };

    let _ = syscall::close(left_a);
    let _ = syscall::close(left_b);
    let _ = syscall::close(ep);
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    check!(still?, "child already exited");
    check!(leftover_ok, "leftover socketpair closed by nest");
    check!(idle?, "epoll idle wait did not block after nest");
    http?;
    Ok(())
}

/// Parent-side libuv spawn-error socket: read end must stay open and return EOF
/// after the child execs. Closing both ends (Darwin socketpair ends share
/// `st_ino`) makes `read(status)` EBADF and Node/Theia's event loop livelocks.
fn spawn_error_read_eof(fd: i32) -> Result<(), crate::harness::AssertFail> {
    check_ok!(
        syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0),
        "spawn-error read still open"
    );
    let mut buf = [0u8; 8];
    for _ in 0..50 {
        let mut fds = [poll::PollFd {
            fd,
            events: POLLIN | POLLHUP,
            revents: 0,
        }];
        let pr = check_ok!(syscall::poll(&mut fds, 100), "poll spawn-error");
        if pr == 0 {
            continue;
        }
        match syscall::read(fd, &mut buf) {
            Ok(0) => return Ok(()),
            Ok(_) => return Err(crate::harness::AssertFail::msg("spawn-error had data")),
            Err(syscall::Errno::EAGAIN) => {}
            Err(syscall::Errno::EBADF) => {
                return Err(crate::harness::AssertFail::msg("spawn-error read EBADF"));
            }
            Err(_) => return Err(crate::harness::AssertFail::msg("spawn-error read")),
        }
    }
    Err(crate::harness::AssertFail::msg("spawn-error eof timeout"))
}

fn wait_exited(pid: i32) -> Result<i32, crate::harness::AssertFail> {
    let mut status = 0;
    for _ in 0..50 {
        match syscall::wait4(pid, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid => return Ok(status),
            Ok(0) => {
                let req = syscall::Timespec {
                    tv_sec: 0,
                    tv_nsec: 20_000_000,
                };
                let _ = syscall::nanosleep(&req);
            }
            Ok(_) => {}
            Err(syscall::Errno::ECHILD) => {
                return Err(crate::harness::AssertFail::msg("wait ECHILD"));
            }
            Err(_) => return Err(crate::harness::AssertFail::msg("wait4")),
        }
    }
    Err(crate::harness::AssertFail::msg("child did not exit"))
}

/// libuv `uv_spawn` creates a CLOEXEC socketpair, `fork`s, and `read`s the
/// parent end for a status byte or EOF. A runtime that drops the parent's
/// *read* end (matching Darwin `st_ino` of both socketpair ends) fails this.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "after fork+exec of /bin/true, the parent spawn-error socketpair read end still returns EOF rather than EBADF"
)]
fn spawn_error_read_eof_after_true() -> TestResult {
    check_ok!(syscall::access(b"/bin/true\0", F_OK), "/bin/true missing");
    let (st_a, st_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "spawn-error");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(st_a);
        let arg0 = b"/bin/true\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let envp = [core::ptr::null::<u8>()];
        let _ = syscall::execve(b"/bin/true\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(st_b);
    let _ = wait_exited(pid);
    let eof = spawn_error_read_eof(st_a);
    let _ = syscall::close(st_a);
    eof?;
    Ok(())
}

/// Same spawn-error handshake with a parked plugin-host helper (no nested
/// busybox). The parent read end must survive cooperative fork restore.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "after fork+exec of a parked plugin-host helper, the parent spawn-error socketpair read end still returns EOF rather than EBADF"
)]
fn spawn_error_read_eof_after_plugin_host_hold() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (st_a, st_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "spawn-error");
    let (ipc_a, ipc_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "ipc");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(st_a);
        let _ = syscall::close(ipc_a);
        if syscall::dup2(ipc_b, 3).is_err() {
            syscall::exit(125);
        }
        if ipc_b != 3 {
            let _ = syscall::close(ipc_b);
        }
        if st_b != 3 {
            let _ = syscall::close(st_b);
        }
        let env0 = b"NODE_CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let mut arg0 = [0u8; 256];
        arg0[..exe_len].copy_from_slice(&exe[..exe_len]);
        let flag = b"--plugin-host-hold\0";
        let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(&exe[..exe_len], &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(st_b);
    let _ = syscall::close(ipc_b);
    let still = child_still_running(pid);
    let eof = spawn_error_read_eof(st_a);
    let _ = syscall::close(st_a);
    let _ = syscall::close(ipc_a);
    reap_or_kill(pid);
    check!(still?, "helper already exited");
    eof?;
    Ok(())
}

/// Same handshake after a live plugin-host-shaped nest: the parent must keep
/// the spawn-error read end, a leftover UV-style socketpair, and the HTTP
/// listen. Inode-matching restore closes all three when they share Darwin
/// identity, and the workbench freezes in epoll_wait after the page loads.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "after plugin-host nest, spawn-error read returns EOF, leftover socketpair still works, epoll goes idle, and HTTP still accepts"
)]
fn spawn_error_eof_after_plugin_host_nest() -> TestResult {
    let (srv, bound) = listen_loopback()?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, srv, EPOLLIN | EPOLLET)?;
    let (left_a, left_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "leftover");
    add_socket_to_epoll(ep, left_a, EPOLLIN | EPOLLOUT | EPOLLET)?;
    add_socket_to_epoll(ep, left_b, EPOLLIN | EPOLLOUT | EPOLLET)?;

    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    let st_a = extra[3];
    let still = child_still_running(pid)?;
    let leftover_ok = syscall::send(left_a, b"x", 0).is_ok();
    let eof = spawn_error_read_eof(st_a);
    let idle = if leftover_ok {
        let mut b = [0u8; 1];
        let _ = syscall::recv(left_b, &mut b, 0);
        let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 16];
        let _ = syscall::epoll_wait(ep, &mut drain, 0);
        epoll_goes_idle(ep)
    } else {
        Ok(false)
    };
    let http = match idle {
        Ok(true) => http_roundtrip(srv, &bound),
        _ => Ok(()),
    };

    let _ = syscall::close(left_a);
    let _ = syscall::close(left_b);
    let _ = syscall::close(ep);
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    check!(still, "child already exited");
    check!(leftover_ok, "leftover socketpair closed by nest");
    eof?;
    check!(idle?, "epoll idle wait did not block after nest");
    http?;
    Ok(())
}

fn exec_plugin_host_hold() -> ! {
    let mut exe = [0u8; 256];
    let Ok(exe_len) = self_exe(&mut exe) else {
        syscall::exit(127);
    };
    let mut arg0 = [0u8; 256];
    arg0[..exe_len].copy_from_slice(&exe[..exe_len]);
    let flag = b"--plugin-host-hold\0";
    let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
    let envp = [core::ptr::null::<u8>()];
    let _ = syscall::execve(&exe[..exe_len], &argv, &envp);
    syscall::exit(127);
}

fn cow_slice_mut(addr: usize, len: usize) -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, len) }
}

fn cow_slice(addr: usize, len: usize) -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(addr as *const u8, len) }
}

/// 1MiB private anon mapping: child fill must not be visible in the parent
/// after `fork`+`exec`. Cooperative fork without heap restore lets Node/Theia
/// `abort()` (guest 134) once the parent continues.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a 1MiB MAP_PRIVATE anonymous mapping keeps the parent's canary after the child overwrites it and execs"
)]
fn mmap_anon_canary_survives_fork_exec() -> TestResult {
    let len = 1024 * 1024usize;
    let addr = check_ok!(
        syscall::mmap(
            0,
            len,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    cow_slice_mut(addr, len).fill(0xA5);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        cow_slice_mut(addr, len).fill(0x5A);
        exec_plugin_host_hold();
    }
    let parent_ok = cow_slice(addr, len).iter().all(|&b| b == 0xA5);
    reap_or_kill(pid);
    let _ = syscall::munmap(addr, len);
    check!(parent_ok, "parent mmap saw child's stores after fork+exec");
    Ok(())
}

fn drain_zombies() {
    let mut status = 0;
    loop {
        match syscall::wait4(-1, &mut status, wait::WNOHANG) {
            Ok(0) | Err(syscall::Errno::ECHILD) => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}

/// Theia/Node classic `fork` COW used to livelock the child on a PAGEZERO
/// store (`fork cow armed` with no restore) so the parent's HTTP listen died
/// — workbench reload / second `uv_spawn`. A live TCP listen must still
/// accept after the child overwrites ELF data and execs.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "after a child overwrites ELF data and execs, the parent TCP listen still accepts"
)]
fn low_gva_store_after_fork_exec_parent_http() -> TestResult {
    let (srv, bound) = listen_loopback()?;
    http_roundtrip(srv, &bound)?;
    unsafe {
        ELF_DATA_CANARY.fill(0xA5);
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            ELF_DATA_CANARY.fill(0x5A);
        }
        core::hint::black_box(unsafe { &ELF_DATA_CANARY });
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"true\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let envp = [core::ptr::null::<u8>()];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    core::hint::black_box(unsafe { &ELF_DATA_CANARY });
    let parent_ok = unsafe { ELF_DATA_CANARY.iter().all(|&b| b == 0xA5) };
    let http_ok = http_roundtrip(srv, &bound);
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    drain_zombies();
    check!(
        parent_ok,
        "parent ELF data saw child's stores after fork+exec"
    );
    http_ok.map_err(|_| crate::harness::AssertFail::msg("http after low-GVA fork+exec"))?;
    Ok(())
}

/// Fork COW must not leave the parent's pages PROT_READ.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "the parent can still write a private anonymous mapping after the child overwrites it and execs"
)]
fn mmap_anon_writable_after_fork_exec() -> TestResult {
    let len = 64 * 1024usize;
    let addr = check_ok!(
        syscall::mmap(
            0,
            len,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    cow_slice_mut(addr, len).fill(0x11);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        cow_slice_mut(addr, len).fill(0x22);
        exec_plugin_host_hold();
    }
    cow_slice_mut(addr, len).fill(0x33);
    let wrote = cow_slice(addr, len).iter().all(|&b| b == 0x33);
    reap_or_kill(pid);
    let _ = syscall::munmap(addr, len);
    check!(wrote, "parent mmap not writable after fork+exec");
    Ok(())
}

/// brk heap is not an anonymous mmap; glibc small malloc lives here and the
/// child's execve path mutates it.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "bytes on the brk heap keep the parent's canary after the child overwrites them and execs"
)]
fn brk_canary_survives_fork_exec() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "brk query");
    let need = cur + 4096;
    let got = check_ok!(syscall::brk(need), "brk grow");
    check!(got >= need, "brk did not grow");
    cow_slice_mut(cur, 4096).fill(0xA5);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        cow_slice_mut(cur, 4096).fill(0x5A);
        exec_plugin_host_hold();
    }
    let parent_ok = cow_slice(cur, 4096).iter().all(|&b| b == 0xA5);
    reap_or_kill(pid);
    let _ = syscall::brk(cur);
    check!(parent_ok, "parent brk heap saw child's stores after fork+exec");
    Ok(())
}

/// Locals in the current frame must survive cooperative child exec (`uv_spawn`
/// pipe fds live here). Too little restore livelocks; this is the small window.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a stack local in the parent fork frame keeps its value after the child overwrites its copy and execs"
)]
fn stack_local_survives_fork_exec() -> TestResult {
    let mut local = [0xAAu8; 64];
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        local.fill(0xBB);
        core::hint::black_box(&local);
        exec_plugin_host_hold();
    }
    core::hint::black_box(&local);
    let parent_ok = local.iter().all(|&b| b == 0xAA);
    reap_or_kill(pid);
    check!(
        parent_ok,
        "parent stack local saw child's stores after fork+exec"
    );
    Ok(())
}

/// glibc malloc_state / tcache live in libc `.data`/`.bss` (file-backed
/// `MAP_PRIVATE`), not in brk or anonymous mmap. Child `execve` malloc must
/// not leave those pages as the child left them or the parent `abort()`s (134).
static mut ELF_DATA_CANARY: [u8; 8192] = [0; 8192];

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "ELF data/BSS canary bytes keep the parent's value after the child overwrites them and execs"
)]
fn elf_data_canary_survives_fork_exec() -> TestResult {
    unsafe {
        ELF_DATA_CANARY.fill(0xA5);
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            ELF_DATA_CANARY.fill(0x5A);
        }
        core::hint::black_box(unsafe { &ELF_DATA_CANARY });
        exec_plugin_host_hold();
    }
    core::hint::black_box(unsafe { &ELF_DATA_CANARY });
    let parent_ok = unsafe { ELF_DATA_CANARY.iter().all(|&b| b == 0xA5) };
    reap_or_kill(pid);
    check!(
        parent_ok,
        "parent ELF data saw child's stores after fork+exec"
    );
    Ok(())
}

/// Many private anonymous maps must not starve ELF data out of the fork-COW
/// cap (Theia filled 192 regions and then abort/SIGILL'd after plugin-host nest).
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "ELF data canary survives fork+exec even when many scattered anonymous maps were dirtied"
)]
fn elf_data_canary_survives_fork_exec_with_many_anon_maps() -> TestResult {
    const N: usize = 220;
    const PAGE: usize = 0x4000;
    let mut maps = [0usize; N];
    let mut n = 0usize;
    let mut i = 0usize;
    while i < N {
        let hint = 0x0000_0070_0000_0000usize + i * 0x2_0000;
        match syscall::mmap(
            hint,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_FIXED,
            -1,
            0,
        ) {
            Ok(addr) => {
                maps[n] = addr;
                n += 1;
                cow_slice_mut(addr, PAGE).fill(0xA5);
            }
            Err(_) => {}
        }
        i += 1;
    }
    unsafe {
        ELF_DATA_CANARY.fill(0xA5);
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            ELF_DATA_CANARY.fill(0x5A);
        }
        let mut j = 0usize;
        while j < n {
            cow_slice_mut(maps[j], PAGE).fill(0x5A);
            j += 1;
        }
        exec_plugin_host_hold();
    }
    let parent_ok = unsafe { ELF_DATA_CANARY.iter().all(|&b| b == 0xA5) };
    reap_or_kill(pid);
    let mut j = 0usize;
    while j < n {
        let _ = syscall::munmap(maps[j], PAGE);
        j += 1;
    }
    check!(
        parent_ok,
        "parent ELF data lost among many anon COW regions after fork+exec"
    );
    Ok(())
}

/// Fork COW of file-backed ELF data must not leave those pages PROT_READ.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "the parent can still write ELF data/BSS after the child overwrites it and execs"
)]
fn elf_data_writable_after_fork_exec() -> TestResult {
    unsafe {
        ELF_DATA_CANARY.fill(0x11);
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            ELF_DATA_CANARY.fill(0x22);
        }
        exec_plugin_host_hold();
    }
    unsafe {
        ELF_DATA_CANARY.fill(0x33);
    }
    let wrote = unsafe { ELF_DATA_CANARY.iter().all(|&b| b == 0x33) };
    reap_or_kill(pid);
    check!(wrote, "parent ELF data not writable after fork+exec");
    Ok(())
}

/// Same as ELF data, but via an explicit file-backed `MAP_PRIVATE` mmap
/// (ld.so maps libc this way after the main executable image).
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a file-backed MAP_PRIVATE mapping keeps the parent's canary after the child overwrites it and execs"
)]
fn file_private_mmap_canary_survives_fork_exec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"cow\0", 0o644), "create");
    let len = 64 * 1024usize;
    let zeros = [0u8; 4096];
    let mut wrote = 0usize;
    while wrote < len {
        let n = check_ok!(syscall::write(fd, &zeros), "write");
        check!(n > 0, "short write");
        wrote += n;
    }
    let addr = check_ok!(
        syscall::mmap(
            0,
            len,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE,
            fd,
            0
        ),
        "mmap file"
    );
    let _ = syscall::close(fd);
    cow_slice_mut(addr, len).fill(0xA5);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        cow_slice_mut(addr, len).fill(0x5A);
        exec_plugin_host_hold();
    }
    let parent_ok = cow_slice(addr, len).iter().all(|&b| b == 0xA5);
    reap_or_kill(pid);
    let _ = syscall::munmap(addr, len);
    check!(
        parent_ok,
        "parent file-backed mmap saw child's stores after fork+exec"
    );
    Ok(())
}

/// Cooperative fork pauses UV-style workers while the child runs. They must
/// resume; a leftover `thread_suspend` freezes Node after plugin-host nest.
static COW_WORKER_KEEP: AtomicU32 = AtomicU32::new(0);
static COW_WORKER_TICKS: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn cow_tick_worker(_arg: *mut u8) -> i32 {
    while COW_WORKER_KEEP.load(Ordering::SeqCst) != 0 {
        COW_WORKER_TICKS.fetch_add(1, Ordering::SeqCst);
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 2_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    0
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a live CLONE_THREAD worker keeps ticking after the parent fork+execs a helper"
)]
fn clone_worker_keeps_ticking_across_fork_exec() -> TestResult {
    COW_WORKER_KEEP.store(1, Ordering::SeqCst);
    COW_WORKER_TICKS.store(0, Ordering::SeqCst);
    let worker = match runtime::spawn_thread(cow_tick_worker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => return Ok(()),
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };
    let mut spins = 0u32;
    while COW_WORKER_TICKS.load(Ordering::SeqCst) == 0 && spins < 200 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }
    let started = COW_WORKER_TICKS.load(Ordering::SeqCst) > 0;
    let before = COW_WORKER_TICKS.load(Ordering::SeqCst);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        exec_plugin_host_hold();
    }
    let wait = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 30_000_000,
    };
    let _ = syscall::nanosleep(&wait);
    let after = COW_WORKER_TICKS.load(Ordering::SeqCst);
    reap_or_kill(pid);
    COW_WORKER_KEEP.store(0, Ordering::SeqCst);
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(e) => {
            return Err(crate::harness::AssertFail::msg(e.name()));
        }
    }
    check!(started, "worker never ran");
    check!(
        after > before,
        "worker did not tick after fork+exec (left suspended?)"
    );
    Ok(())
}

static COW_GETPID_KEEP: AtomicU32 = AtomicU32::new(0);
static COW_GETPID_TICKS: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn cow_getpid_worker(_arg: *mut u8) -> i32 {
    while COW_GETPID_KEEP.load(Ordering::SeqCst) != 0 {
        let _ = syscall::getpid();
        COW_GETPID_TICKS.fetch_add(1, Ordering::SeqCst);
    }
    0
}

/// Watchdog `force_thread_state` racing cooperative-fork worker suspend
/// SIGILL'd nested Node (Theia plugin-host IPC exit 132).
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a live worker spinning getpid survives parent fork+exec without killing the process"
)]
fn clone_worker_getpid_survives_fork_exec() -> TestResult {
    COW_GETPID_KEEP.store(1, Ordering::SeqCst);
    COW_GETPID_TICKS.store(0, Ordering::SeqCst);
    let worker = match runtime::spawn_thread(cow_getpid_worker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => return Ok(()),
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };
    let mut spins = 0u32;
    while COW_GETPID_TICKS.load(Ordering::SeqCst) == 0 && spins < 200 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }
    let started = COW_GETPID_TICKS.load(Ordering::SeqCst) > 0;
    let before = COW_GETPID_TICKS.load(Ordering::SeqCst);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        exec_plugin_host_hold();
    }
    let wait = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 50_000_000,
    };
    let _ = syscall::nanosleep(&wait);
    let after = COW_GETPID_TICKS.load(Ordering::SeqCst);
    reap_or_kill(pid);
    COW_GETPID_KEEP.store(0, Ordering::SeqCst);
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(e) => {
            return Err(crate::harness::AssertFail::msg(e.name()));
        }
    }
    check!(started, "getpid worker never ran");
    check!(
        after > before,
        "getpid worker did not survive fork+exec"
    );
    Ok(())
}

static COW_CLOCK_KEEP: AtomicU32 = AtomicU32::new(0);
static COW_CLOCK_TICKS: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn cow_clock_worker(_arg: *mut u8) -> i32 {
    while COW_CLOCK_KEEP.load(Ordering::SeqCst) != 0 {
        let _ = syscall::clock_gettime(syscall::clock::CLOCK_MONOTONIC);
        COW_CLOCK_TICKS.fetch_add(1, Ordering::SeqCst);
    }
    0
}

/// Watchdog used to `handle()` `clock_gettime` on its own thread for a worker
/// stuck on `brk #0x100`, which SIGILL'd nested Node (plugin-host IPC 132).
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a live worker spinning clock_gettime survives parent fork+exec without killing the process"
)]
fn clone_worker_clock_gettime_survives_fork_exec() -> TestResult {
    COW_CLOCK_KEEP.store(1, Ordering::SeqCst);
    COW_CLOCK_TICKS.store(0, Ordering::SeqCst);
    let worker = match runtime::spawn_thread(cow_clock_worker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => return Ok(()),
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };
    let mut spins = 0u32;
    while COW_CLOCK_TICKS.load(Ordering::SeqCst) == 0 && spins < 200 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }
    let started = COW_CLOCK_TICKS.load(Ordering::SeqCst) > 0;
    let before = COW_CLOCK_TICKS.load(Ordering::SeqCst);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        exec_plugin_host_hold();
    }
    let wait = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 50_000_000,
    };
    let _ = syscall::nanosleep(&wait);
    let after = COW_CLOCK_TICKS.load(Ordering::SeqCst);
    reap_or_kill(pid);
    COW_CLOCK_KEEP.store(0, Ordering::SeqCst);
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(e) => {
            return Err(crate::harness::AssertFail::msg(e.name()));
        }
    }
    check!(started, "clock_gettime worker never ran");
    check!(
        after > before,
        "clock_gettime worker did not survive fork+exec"
    );
    Ok(())
}

static COW_FUTEX_KEEP: AtomicU32 = AtomicU32::new(0);
static COW_FUTEX_TICKS: AtomicU32 = AtomicU32::new(0);
static COW_FUTEX_WORD: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn cow_futex_worker(_arg: *mut u8) -> i32 {
    while COW_FUTEX_KEEP.load(Ordering::SeqCst) != 0 {
        // libuv workers sit in FUTEX_WAIT. Watchdog trampoline on leftover
        // `brk #0x100` with nr=98 SIGILL'd the Darwin process (Theia parent
        // and nested plugin-host both exited 132).
        let timeout = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::futex_wait(&COW_FUTEX_WORD, 0, Some(&timeout));
        COW_FUTEX_TICKS.fetch_add(1, Ordering::SeqCst);
    }
    0
}

/// Watchdog trampoline used to `bl` without saving guest LR, so the libc
/// syscall wrapper `ret`'d into host text and SIGILL'd Node (Theia parent
/// EXIT 132 after `unblock-retry … nr=98`). A worker in `futex_wait` across
/// fork+exec must keep ticking without killing the process.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a live worker spinning futex_wait survives parent fork+exec without killing the process"
)]
fn clone_worker_futex_survives_fork_exec() -> TestResult {
    COW_FUTEX_KEEP.store(1, Ordering::SeqCst);
    COW_FUTEX_TICKS.store(0, Ordering::SeqCst);
    COW_FUTEX_WORD.store(0, Ordering::SeqCst);
    let worker = match runtime::spawn_thread(cow_futex_worker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => return Ok(()),
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };
    let mut spins = 0u32;
    while COW_FUTEX_TICKS.load(Ordering::SeqCst) == 0 && spins < 200 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }
    let started = COW_FUTEX_TICKS.load(Ordering::SeqCst) > 0;
    let before = COW_FUTEX_TICKS.load(Ordering::SeqCst);
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        exec_plugin_host_hold();
    }
    let wait = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 80_000_000,
    };
    let _ = syscall::nanosleep(&wait);
    let after = COW_FUTEX_TICKS.load(Ordering::SeqCst);
    reap_or_kill(pid);
    COW_FUTEX_KEEP.store(0, Ordering::SeqCst);
    let _ = syscall::futex_wake(&COW_FUTEX_WORD, 1);
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(e) => {
            return Err(crate::harness::AssertFail::msg(e.name()));
        }
    }
    check!(started, "futex worker never ran");
    check!(
        after > before,
        "futex worker did not survive fork+exec"
    );
    Ok(())
}

static WORKER_KEEP: AtomicU32 = AtomicU32::new(0);
static WORKER_TICKS: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn nest_tcp_worker(_arg: *mut u8) -> i32 {
    while WORKER_KEEP.load(Ordering::SeqCst) != 0 {
        WORKER_TICKS.fetch_add(1, Ordering::SeqCst);
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 2_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    0
}

#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "with a live CLONE_THREAD worker, an accepted TCP connection still serves after plugin-host nest"
)]
fn spawn_nested_existing_tcp_with_live_worker() -> TestResult {
    // Node/Theia keep libuv pool threads running across child_process.fork.
    // A cooperative nest that rewinds the worker stack or shares the child's
    // fd table with those threads wedges the workbench after the page loads.
    WORKER_KEEP.store(1, Ordering::SeqCst);
    WORKER_TICKS.store(0, Ordering::SeqCst);
    let worker = match runtime::spawn_thread(nest_tcp_worker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => return Ok(()),
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };
    let result = spawn_nested_existing_tcp_with_live_worker_body();
    WORKER_KEEP.store(0, Ordering::SeqCst);
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(e) => {
            if result.is_ok() {
                return Err(crate::harness::AssertFail::msg(e.name()));
            }
        }
    }
    result
}

fn spawn_nested_existing_tcp_with_live_worker_body() -> TestResult {
    let mut spins = 0u32;
    while WORKER_TICKS.load(Ordering::SeqCst) == 0 && spins < 200 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }
    check!(
        WORKER_TICKS.load(Ordering::SeqCst) > 0,
        "worker never ran"
    );

    let (srv, bound) = listen_loopback()?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, srv, EPOLLIN | EPOLLET)?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, &bound), "connect");
    check!(
        wait_epoll_bit(ep, srv as u64, EPOLLIN, 20)?,
        "listen not ready"
    );
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept");
    set_nonblock(acc)?;
    add_socket_to_epoll(ep, acc, EPOLLIN | EPOLLOUT | EPOLLET)?;
    check_ok!(syscall::send(cli, b"PAGE", 0), "send page");
    check!(
        wait_epoll_bit(ep, acc as u64, EPOLLIN, 20)?,
        "page not readable"
    );
    recv_exact(acc, b"PAGE")?;
    let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    let _ = syscall::epoll_wait(ep, &mut drain, 0);

    let ticks_before = WORKER_TICKS.load(Ordering::SeqCst);
    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid)?, "child already exited");

    check_ok!(syscall::send(cli, b"PING", 0), "send ping");
    check!(
        wait_epoll_bit(ep, acc as u64, EPOLLIN, 20)?,
        "accepted fd not readable after nest with worker"
    );
    recv_exact(acc, b"PING")?;
    check_ok!(syscall::send(acc, b"PONG", 0), "send pong");
    let mut pong = [0u8; 4];
    check_eq!(check_ok!(syscall::recv(cli, &mut pong, 0), "cli pong"), 4, "pong len");
    check_eq!(&pong, b"PONG", "pong");

    spins = 0;
    while WORKER_TICKS.load(Ordering::SeqCst) <= ticks_before && spins < 200 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }
    check!(
        WORKER_TICKS.load(Ordering::SeqCst) > ticks_before,
        "worker stuck after nest"
    );

    let _ = syscall::close(acc);
    let _ = syscall::close(cli);
    let _ = syscall::close(ep);
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    Ok(())
}

/// Bytes already on an accepted TCP socket at fork must still be readable
/// after nest restore (browser may pipeline the next HTTP/websocket frame
/// while plugin-host is spawning).
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "TCP bytes queued on an accepted connection before plugin-host nest are still readable afterwards"
)]
fn spawn_nested_tcp_bytes_queued_before_nest() -> TestResult {
    let (srv, bound) = listen_loopback()?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, srv, EPOLLIN | EPOLLET)?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, &bound), "connect");
    check!(
        wait_epoll_bit(ep, srv as u64, EPOLLIN, 20)?,
        "listen not ready"
    );
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept");
    set_nonblock(acc)?;
    add_socket_to_epoll(ep, acc, EPOLLIN | EPOLLOUT | EPOLLET)?;
    check_ok!(syscall::send(cli, b"PAGE", 0), "send page");
    check!(
        wait_epoll_bit(ep, acc as u64, EPOLLIN, 20)?,
        "page not readable"
    );
    recv_exact(acc, b"PAGE")?;
    let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    let _ = syscall::epoll_wait(ep, &mut drain, 0);

    check_ok!(syscall::send(cli, b"PING", 0), "queue ping before nest");
    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid)?, "child already exited");
    check!(
        wait_epoll_bit(ep, acc as u64, EPOLLIN, 20)?,
        "queued bytes not readable after nest"
    );
    recv_exact(acc, b"PING")?;

    let _ = syscall::close(acc);
    let _ = syscall::close(cli);
    let _ = syscall::close(ep);
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    Ok(())
}

/// Opening the Theia workbench uses several connections at once (HTML, bundle,
/// websocket). All of them must survive plugin-host nest.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "three accepted TCP connections all still read and write after plugin-host nest"
)]
fn spawn_nested_three_accepted_tcp_still_serve() -> TestResult {
    let (srv, bound) = listen_loopback()?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, srv, EPOLLIN | EPOLLET)?;

    let mut clis = [-1i32; 3];
    let mut accs = [-1i32; 3];
    for i in 0..3 {
        clis[i] = check_ok!(
            syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
            "client"
        );
        check_ok!(syscall::connect(clis[i], &bound), "connect");
        check!(
            wait_epoll_bit(ep, srv as u64, EPOLLIN, 20)?,
            "listen not ready"
        );
        accs[i] = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept");
        set_nonblock(accs[i])?;
        add_socket_to_epoll(ep, accs[i], EPOLLIN | EPOLLOUT | EPOLLET)?;
        check_ok!(syscall::send(clis[i], b"PAGE", 0), "send page");
        check!(
            wait_epoll_bit(ep, accs[i] as u64, EPOLLIN, 20)?,
            "page not readable"
        );
        recv_exact(accs[i], b"PAGE")?;
    }
    let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    let _ = syscall::epoll_wait(ep, &mut drain, 0);

    let (pid, ipc, extra) = spawn_nested_plugin_host_shape(srv)?;
    check!(child_still_running(pid)?, "child already exited");

    for i in 0..3 {
        check_ok!(syscall::send(clis[i], b"PING", 0), "send ping");
        check!(
            wait_epoll_bit(ep, accs[i] as u64, EPOLLIN, 20)?,
            "accepted fd not readable after nest"
        );
        recv_exact(accs[i], b"PING")?;
        check_ok!(syscall::send(accs[i], b"PONG", 0), "send pong");
        let mut pong = [0u8; 4];
        check_eq!(
            check_ok!(syscall::recv(clis[i], &mut pong, 0), "cli pong"),
            4,
            "pong len"
        );
        check_eq!(&pong, b"PONG", "pong");
    }

    for i in 0..3 {
        let _ = syscall::close(accs[i]);
        let _ = syscall::close(clis[i]);
    }
    let _ = syscall::close(ep);
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    Ok(())
}

static WORKER_EP: AtomicI32 = AtomicI32::new(-1);
static WORKER_PIPE_R: AtomicI32 = AtomicI32::new(-1);
static WORKER_PIPE_W: AtomicI32 = AtomicI32::new(-1);

unsafe extern "C" fn nest_epoll_worker(_arg: *mut u8) -> i32 {
    let ep = WORKER_EP.load(Ordering::SeqCst);
    let r = WORKER_PIPE_R.load(Ordering::SeqCst);
    let mut buf = [0u8; 8];
    while WORKER_KEEP.load(Ordering::SeqCst) != 0 {
        let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 4];
        match syscall::epoll_wait(ep, &mut out, 50) {
            Ok(n) => {
                for e in out.iter().take(n) {
                    if e.data == r as u64 && e.events & EPOLLIN != 0 {
                        let _ = syscall::read(r, &mut buf);
                        WORKER_TICKS.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
            Err(_) => {
                let req = syscall::Timespec {
                    tv_sec: 0,
                    tv_nsec: 2_000_000,
                };
                let _ = syscall::nanosleep(&req);
            }
        }
    }
    0
}

/// Node/Theia UV pool threads sit in epoll_wait while the main thread
/// fork+execs plugin-host. Those workers must still see pipe POLLIN after nest.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a live CLONE_THREAD worker blocked in epoll_wait still wakes after plugin-host nest"
)]
fn spawn_nested_worker_epoll_still_wakes() -> TestResult {
    WORKER_KEEP.store(1, Ordering::SeqCst);
    WORKER_TICKS.store(0, Ordering::SeqCst);
    let (pr, pw) = check_ok!(syscall::pipe2(0), "pipe");
    set_nonblock(pr)?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, pr, EPOLLIN | EPOLLET)?;
    WORKER_EP.store(ep, Ordering::SeqCst);
    WORKER_PIPE_R.store(pr, Ordering::SeqCst);

    let worker = match runtime::spawn_thread(nest_epoll_worker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => {
            let _ = syscall::close(ep);
            let _ = syscall::close(pr);
            let _ = syscall::close(pw);
            return Ok(());
        }
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };

    check_ok!(syscall::write(pw, b"a"), "prime worker");
    let mut spins = 0u32;
    while WORKER_TICKS.load(Ordering::SeqCst) == 0 && spins < 200 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }

    let (srv, _bound) = listen_loopback()?;
    let ticks_before = WORKER_TICKS.load(Ordering::SeqCst);
    let nest = spawn_nested_plugin_host_shape(srv);
    let woke = nest.as_ref().ok().map(|_| {
        let _ = syscall::write(pw, b"b");
        let mut s = 0u32;
        while WORKER_TICKS.load(Ordering::SeqCst) <= ticks_before && s < 200 {
            let req = syscall::Timespec {
                tv_sec: 0,
                tv_nsec: 1_000_000,
            };
            let _ = syscall::nanosleep(&req);
            s += 1;
        }
        WORKER_TICKS.load(Ordering::SeqCst) > ticks_before
    });

    WORKER_KEEP.store(0, Ordering::SeqCst);
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(_) => {}
    }
    let _ = syscall::close(ep);
    let _ = syscall::close(pr);
    let _ = syscall::close(pw);

    let (pid, ipc, extra) = nest?;
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);
    check!(ticks_before > 0, "worker never ran");
    check!(woke.unwrap_or(false), "worker epoll did not wake after nest");
    Ok(())
}

/// libuv reads a stream until EAGAIN, then `epoll_wait`s. If the peer writes
/// in that gap, Linux still reports EPOLLIN. xvisor disarms ET POLLIN on the
/// first wait and only rearms it when a later wait observes "not ready" — so
/// data that arrives after EAGAIN is stuck and Theia's websocket freezes.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "EPOLLET POLLIN rearms after recv EAGAIN so a socketpair write before the next wait is still reported"
)]
fn spawn_epoll_et_socket_in_rearm_after_eagain() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    set_nonblock(a)?;
    set_nonblock(b)?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, a, EPOLLIN | EPOLLET)?;

    check_ok!(syscall::send(b, b"x", 0), "prime");
    check!(wait_epoll_bit(ep, a as u64, EPOLLIN, 20)?, "first in");
    let mut tmp = [0u8; 8];
    loop {
        match syscall::recv(a, &mut tmp, 0) {
            Ok(0) => break,
            Ok(_) => {}
            Err(syscall::Errno::EAGAIN) => break,
            Err(_) => return Err(crate::harness::AssertFail::msg("recv drain")),
        }
    }

    check_ok!(syscall::send(b, b"y", 0), "after eagain");
    check!(
        wait_epoll_bit(ep, a as u64, EPOLLIN, 20)?,
        "POLLIN missing after recv EAGAIN then peer write"
    );
    recv_exact(a, b"y")?;
    let _ = syscall::close(ep);
    let _ = syscall::close(a);
    let _ = syscall::close(b);
    Ok(())
}

/// libuv writes until EAGAIN, the peer may drain the buffer before the next
/// `epoll_wait`, and Linux still reports EPOLLOUT. xvisor keeps POLLOUT
/// disarmed while the socket is writable, so Theia never finishes flushing
/// `bundle.js` / the frontend websocket and the page freezes.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "EPOLLET POLLOUT rearms after send EAGAIN so a drained socketpair is writable again"
)]
fn spawn_epoll_et_socket_out_rearm_after_eagain() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    set_nonblock(a)?;
    set_nonblock(b)?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, a, EPOLLOUT | EPOLLET)?;

    check!(wait_epoll_bit(ep, a as u64, EPOLLOUT, 20)?, "first out");
    let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 4];
    let _ = syscall::epoll_wait(ep, &mut drain, 0);

    let chunk = [0u8; 4096];
    let mut filled = false;
    for _ in 0..4096 {
        match syscall::send(a, &chunk, 0) {
            Ok(0) => break,
            Ok(_) => {}
            Err(syscall::Errno::EAGAIN) => {
                filled = true;
                break;
            }
            Err(_) => return Err(crate::harness::AssertFail::msg("send fill")),
        }
    }
    check!(filled, "send never returned EAGAIN");

    let mut buf = [0u8; 4096];
    loop {
        match syscall::recv(b, &mut buf, 0) {
            Ok(0) => break,
            Ok(_) => {}
            Err(syscall::Errno::EAGAIN) => break,
            Err(_) => return Err(crate::harness::AssertFail::msg("recv drain")),
        }
    }

    check!(
        wait_epoll_bit(ep, a as u64, EPOLLOUT, 50)?,
        "POLLOUT missing after send EAGAIN then peer drain"
    );
    let _ = syscall::close(ep);
    let _ = syscall::close(a);
    let _ = syscall::close(b);
    Ok(())
}

/// Theia `http.Server.listen(port, '0.0.0.0')` binds INADDR_ANY. Isolated
/// runtimes that rewrite unpublished ports to loopback must still accept from
/// 127.0.0.1, or the "Theia app listening" callback never becomes reachable.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "TCP bind/listen on INADDR_ANY still accepts a 127.0.0.1 client (Theia --hostname=0.0.0.0)"
)]
fn spawn_listen_inaddr_any_accepts_loopback() -> TestResult {
    let srv = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "socket"
    );
    let one = 1i32.to_ne_bytes();
    check_ok!(
        syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one),
        "SO_REUSEADDR"
    );
    check_ok!(syscall::bind(srv, &SockAddrIn::any(0)), "bind INADDR_ANY");
    check_ok!(syscall::listen(srv, 128), "listen");
    let bound = check_ok!(syscall::getsockname_in(srv), "getsockname");
    check!(bound.port_host() != 0, "ephemeral port");
    let mut dest = SockAddrIn::loopback(bound.port_host());
    if bound.sin_addr != 0 {
        dest.sin_addr = bound.sin_addr;
    }
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, &dest), "connect loopback to INADDR_ANY");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::send(cli, b"hi", 0), "send");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::recv(acc, &mut buf, 0), "recv"), 2, "len");
    check_eq!(&buf, b"hi", "payload");
    let _ = syscall::close(acc);
    let _ = syscall::close(cli);
    let _ = syscall::close(srv);
    Ok(())
}

unsafe extern "C" fn nest_ppoll_blocker(_arg: *mut u8) -> i32 {
    let r = WORKER_PIPE_R.load(Ordering::SeqCst);
    let ready = WORKER_PIPE_W.load(Ordering::SeqCst);
    let _ = syscall::write(ready, b"x");
    let ts = syscall::Timespec {
        tv_sec: 2,
        tv_nsec: 0,
    };
    while WORKER_KEEP.load(Ordering::SeqCst) != 0 {
        let mut fds = [poll::PollFd {
            fd: r,
            events: POLLIN,
            revents: 0,
        }];
        let _ = syscall::ppoll(&mut fds, Some(&ts), None);
    }
    0
}

/// Node `listen(port, '0.0.0.0')` does `dns.lookup` on a UV worker (`ppoll`)
/// then bind+listen on the main thread. If the emulator holds a global lock
/// across that worker ppoll, Theia never prints "Theia app listening".
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a CLONE_THREAD worker blocked in ppoll must not delay the parent's TCP bind+listen"
)]
fn spawn_worker_ppoll_does_not_block_parent_listen() -> TestResult {
    WORKER_KEEP.store(1, Ordering::SeqCst);
    let (idle_r, idle_w) = check_ok!(syscall::pipe2(0), "idle pipe");
    let (ready_r, ready_w) = check_ok!(syscall::pipe2(0), "ready pipe");
    WORKER_PIPE_R.store(idle_r, Ordering::SeqCst);
    WORKER_PIPE_W.store(ready_w, Ordering::SeqCst);

    let worker = match runtime::spawn_thread(nest_ppoll_blocker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => {
            let _ = syscall::close(idle_r);
            let _ = syscall::close(idle_w);
            let _ = syscall::close(ready_r);
            let _ = syscall::close(ready_w);
            return Ok(());
        }
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };

    let mut go = [0u8; 1];
    set_nonblock(ready_r)?;
    let mut saw = false;
    for _ in 0..10_000 {
        match syscall::read(ready_r, &mut go) {
            Ok(n) if n > 0 => {
                saw = true;
                break;
            }
            Err(syscall::Errno::EAGAIN) => {
                let mut s = 0u32;
                while s < 10_000 {
                    core::hint::spin_loop();
                    s += 1;
                }
            }
            Err(_) => break,
            Ok(_) => {}
        }
    }
    check!(saw, "worker never started");
    let mut spins = 0u32;
    while spins < 5_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    let t0 = check_ok!(
        syscall::clock_gettime(clock::CLOCK_MONOTONIC),
        "clock before listen"
    );

    let srv = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "socket"
    );
    let one = 1i32.to_ne_bytes();
    let _ = syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one);
    let bind_r = syscall::bind(srv, &SockAddrIn::any(0));
    let listen_r = syscall::listen(srv, 8);
    let t1 = check_ok!(
        syscall::clock_gettime(clock::CLOCK_MONOTONIC),
        "clock after listen"
    );

    WORKER_KEEP.store(0, Ordering::SeqCst);
    let _ = syscall::write(idle_w, b"x");
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(_) => {}
    }
    let _ = syscall::close(srv);
    let _ = syscall::close(idle_r);
    let _ = syscall::close(idle_w);
    let _ = syscall::close(ready_r);
    let _ = syscall::close(ready_w);

    check_ok!(bind_r, "bind blocked by worker ppoll");
    check_ok!(listen_r, "listen blocked by worker ppoll");
    let elapsed = timespec_ms(&t1).saturating_sub(timespec_ms(&t0));
    check!(
        elapsed < 500,
        "parent bind+listen waited on worker ppoll"
    );
    Ok(())
}

fn spawn_theia_ipc_epoll_helper(exe: &[u8]) -> Result<(i32, i32), crate::harness::AssertFail> {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(a);
        if syscall::dup2(b, 3).is_err() {
            syscall::exit(125);
        }
        if b != 3 {
            let _ = syscall::close(b);
        }
        let env0 = b"NODE_CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let mut arg0 = [0u8; 256];
        arg0[..exe.len()].copy_from_slice(exe);
        // "plugin-host" in argv so a cooperative runtime nests this like Theia.
        let flag = b"--plugin-host-epoll-idle\0";
        let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(exe, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    Ok((a, pid))
}

fn spawn_plugin_host_fd4_pipe(
    exe: &[u8],
) -> Result<(i32, i32), crate::harness::AssertFail> {
    let (pr, pw) = check_ok!(syscall::pipe2(0), "fd4 pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(pr);
        if syscall::dup2(pw, 4).is_err() {
            syscall::exit(125);
        }
        if pw != 4 {
            let _ = syscall::close(pw);
        }
        let envp = [core::ptr::null::<u8>()];
        let mut arg0 = [0u8; 256];
        arg0[..exe.len()].copy_from_slice(exe);
        let flag = b"--plugin-host-fd4\0";
        let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(exe, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(pw);
    Ok((pr, pid))
}

fn read_exact_msg(fd: i32, want: &[u8]) -> Result<(), crate::harness::AssertFail> {
    check!(want.len() <= 8, "read_exact_msg len");
    let mut got = [0u8; 8];
    let mut filled = 0usize;
    for _ in 0..80 {
        let mut fds = [poll::PollFd {
            fd,
            events: POLLIN | POLLHUP,
            revents: 0,
        }];
        let pr = check_ok!(syscall::poll(&mut fds, 100), "poll msg");
        if pr == 0 {
            continue;
        }
        if fds[0].revents & POLLHUP != 0 && fds[0].revents & POLLIN == 0 {
            return Err(crate::harness::AssertFail::msg("eof before msg"));
        }
        match syscall::read(fd, &mut got[filled..want.len()]) {
            Ok(0) => return Err(crate::harness::AssertFail::msg("eof before msg")),
            Ok(n) => {
                filled += n;
                if filled >= want.len() {
                    check_eq!(&got[..want.len()], want, "msg");
                    return Ok(());
                }
            }
            Err(syscall::Errno::EAGAIN) => {}
            Err(_) => return Err(crate::harness::AssertFail::msg("read msg")),
        }
    }
    Err(crate::harness::AssertFail::msg("msg timeout"))
}

fn read_idleok(fd: i32) -> Result<(), crate::harness::AssertFail> {
    read_exact_msg(fd, b"IDLEOK")
}

/// Theia plugin-host is a nested Node that inherits the IPC socketpair and
/// immediately epolls it with EPOLLIN|EPOLLOUT|EPOLLET. The nested process
/// must go idle and speak on the channel — a POLLOUT livelock never writes
/// IDLEOK and the workbench freezes at "Restoring the layout state".
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "nested plugin-host-shaped child with EPOLLET on inherited IPC goes idle and writes IDLEOK"
)]
fn spawn_theia_nested_ipc_epoll_goes_idle() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (sock, pid) = spawn_theia_ipc_epoll_helper(&exe[..exe_len])?;
    let hello = read_idleok(sock);
    let alive = child_still_running(pid);
    let _ = syscall::close(sock);
    reap_or_kill(pid);
    hello?;
    check!(alive?, "nested idle helper already exited");
    Ok(())
}

/// Theia `child_process.fork({ stdio: [pipe,pipe,pipe,ipc,overlapped] })` puts
/// BinaryMessagePipe on fd 4, which is a pipe rather than a socket. Nesting
/// that drops fd 4 unless inherited pipes are installed in the child.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "nested plugin-host-shaped child inherits an extra stdio pipe on fd 4 and writes FD4OK"
)]
fn spawn_nested_fd4_pipe_survives() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (fd4, pid) = spawn_plugin_host_fd4_pipe(&exe[..exe_len])?;
    let fd4ok = read_exact_msg(fd4, b"FD4OK");
    let alive = child_still_running(pid);
    let _ = syscall::close(fd4);
    reap_or_kill(pid);
    fd4ok?;
    check!(alive?, "fd4 helper already exited");
    Ok(())
}

unsafe extern "C" fn nest_epoll_block_worker(_arg: *mut u8) -> i32 {
    let ep = WORKER_EP.load(Ordering::SeqCst);
    let r = WORKER_PIPE_R.load(Ordering::SeqCst);
    let mut buf = [0u8; 8];
    while WORKER_KEEP.load(Ordering::SeqCst) != 0 {
        let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 4];
        // Node UV pool threads use epoll_wait(-1), not a 50ms poll. The
        // blocking trampoline must survive parent fork+exec swapping fd tables.
        match syscall::epoll_wait(ep, &mut out, -1) {
            Ok(n) => {
                for e in out.iter().take(n) {
                    if e.data == r as u64 && e.events & EPOLLIN != 0 {
                        let _ = syscall::read(r, &mut buf);
                        WORKER_TICKS.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
            Err(_) => {
                let req = syscall::Timespec {
                    tv_sec: 0,
                    tv_nsec: 2_000_000,
                };
                let _ = syscall::nanosleep(&req);
            }
        }
    }
    0
}

/// UV pool threads sit in epoll_wait(-1) while the main thread nests
/// plugin-host. Closing the original host fds at fork makes that wait see
/// POLLNVAL and the parent event loop livelocks after the page loads.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "a CLONE_THREAD worker blocked in epoll_wait(-1) still wakes after plugin-host nest, and leftover ET sockets go idle"
)]
fn spawn_nested_worker_epoll_infinite_goes_idle() -> TestResult {
    WORKER_KEEP.store(1, Ordering::SeqCst);
    WORKER_TICKS.store(0, Ordering::SeqCst);
    let (pr, pw) = check_ok!(syscall::pipe2(0), "pipe");
    set_nonblock(pr)?;
    let ep = check_ok!(syscall::epoll_create1(0), "epoll");
    add_socket_to_epoll(ep, pr, EPOLLIN | EPOLLET)?;
    let (left_a, left_b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "leftover");
    add_socket_to_epoll(ep, left_a, EPOLLIN | EPOLLOUT | EPOLLET)?;
    add_socket_to_epoll(ep, left_b, EPOLLIN | EPOLLOUT | EPOLLET)?;
    let mut drain = [epoll::EpollEvent { events: 0, data: 0 }; 16];
    let _ = syscall::epoll_wait(ep, &mut drain, 0);
    WORKER_EP.store(ep, Ordering::SeqCst);
    WORKER_PIPE_R.store(pr, Ordering::SeqCst);

    let worker = match runtime::spawn_thread(nest_epoll_block_worker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => {
            let _ = syscall::close(ep);
            let _ = syscall::close(pr);
            let _ = syscall::close(pw);
            let _ = syscall::close(left_a);
            let _ = syscall::close(left_b);
            return Ok(());
        }
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };

    check_ok!(syscall::write(pw, b"a"), "prime worker");
    let mut spins = 0u32;
    while WORKER_TICKS.load(Ordering::SeqCst) == 0 && spins < 400 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        };
        let _ = syscall::nanosleep(&req);
        spins += 1;
    }

    let (srv, bound) = listen_loopback()?;
    let ticks_before = WORKER_TICKS.load(Ordering::SeqCst);
    let nest = spawn_nested_plugin_host_shape(srv);
    let after = nest.as_ref().ok().map(|_| {
        let leftover_ok = syscall::send(left_a, b"x", 0).is_ok();
        let mut b = [0u8; 1];
        let _ = syscall::recv(left_b, &mut b, 0);
        let _ = syscall::epoll_wait(ep, &mut drain, 0);
        let idle = epoll_goes_idle(ep);
        let http = if leftover_ok && idle.as_ref().ok() == Some(&true) {
            http_roundtrip(srv, &bound)
        } else {
            Ok(())
        };
        let _ = syscall::write(pw, b"b");
        let mut s = 0u32;
        while WORKER_TICKS.load(Ordering::SeqCst) <= ticks_before && s < 400 {
            let req = syscall::Timespec {
                tv_sec: 0,
                tv_nsec: 1_000_000,
            };
            let _ = syscall::nanosleep(&req);
            s += 1;
        }
        (
            leftover_ok,
            idle,
            http,
            WORKER_TICKS.load(Ordering::SeqCst) > ticks_before,
        )
    });

    WORKER_KEEP.store(0, Ordering::SeqCst);
    let _ = syscall::write(pw, b"c");
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(_) => {}
    }
    let _ = syscall::close(left_a);
    let _ = syscall::close(left_b);
    let _ = syscall::close(ep);
    let _ = syscall::close(pr);
    let _ = syscall::close(pw);

    let (pid, ipc, extra) = nest?;
    let _ = syscall::close(ipc);
    for fd in extra {
        let _ = syscall::close(fd);
    }
    let _ = syscall::close(srv);
    reap_or_kill(pid);

    check!(ticks_before > 0, "worker never ran before nest");
    let (leftover_ok, idle, http, woke) = after.ok_or(crate::harness::AssertFail::msg("nest failed"))?;
    check!(leftover_ok, "leftover socketpair closed by nest");
    check!(idle?, "epoll idle wait did not block after nest with worker in epoll_wait(-1)");
    http?;
    check!(woke, "worker epoll_wait(-1) did not wake after nest");
    Ok(())
}

/// A second bind of an occupied TCP port must fail with EADDRINUSE quickly,
/// even if a UV-style worker is blocked in ppoll. Hanging here is what the
/// Theia listen callback looks like when port 3000 is already taken.
#[crate::lctp_test(
    suite = syscall,
    expect = success,
    case = "second TCP bind of a live listen port returns EADDRINUSE without waiting on a worker ppoll"
)]
fn spawn_second_bind_eaddrinuse_is_fast() -> TestResult {
    WORKER_KEEP.store(1, Ordering::SeqCst);
    let (idle_r, idle_w) = check_ok!(syscall::pipe2(0), "idle pipe");
    let (ready_r, ready_w) = check_ok!(syscall::pipe2(0), "ready pipe");
    WORKER_PIPE_R.store(idle_r, Ordering::SeqCst);
    WORKER_PIPE_W.store(ready_w, Ordering::SeqCst);

    let worker = match runtime::spawn_thread(nest_ppoll_blocker, core::ptr::null_mut()) {
        Ok(t) => t,
        Err(e) if runtime::thread_unavailable(e) => {
            let _ = syscall::close(idle_r);
            let _ = syscall::close(idle_w);
            let _ = syscall::close(ready_r);
            let _ = syscall::close(ready_w);
            return Ok(());
        }
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    };

    let mut go = [0u8; 1];
    set_nonblock(ready_r)?;
    let mut saw = false;
    for _ in 0..10_000 {
        match syscall::read(ready_r, &mut go) {
            Ok(n) if n > 0 => {
                saw = true;
                break;
            }
            Err(syscall::Errno::EAGAIN) => {
                let mut s = 0u32;
                while s < 10_000 {
                    core::hint::spin_loop();
                    s += 1;
                }
            }
            Err(_) => break,
            Ok(_) => {}
        }
    }
    check!(saw, "worker never started");

    let srv = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "listen socket"
    );
    let one = 1i32.to_ne_bytes();
    let _ = syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one);
    check_ok!(syscall::bind(srv, &SockAddrIn::any(0)), "first bind");
    check_ok!(syscall::listen(srv, 8), "listen");
    let bound = check_ok!(syscall::getsockname_in(srv), "getsockname");

    let srv2 = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "second socket"
    );
    let _ = syscall::setsockopt(srv2, SOL_SOCKET, SO_REUSEADDR, &one);
    let t0 = check_ok!(
        syscall::clock_gettime(clock::CLOCK_MONOTONIC),
        "clock before second bind"
    );
    let bind2 = syscall::bind(srv2, &SockAddrIn::any(bound.port_host()));
    let t1 = check_ok!(
        syscall::clock_gettime(clock::CLOCK_MONOTONIC),
        "clock after second bind"
    );

    WORKER_KEEP.store(0, Ordering::SeqCst);
    let _ = syscall::write(idle_w, b"x");
    match runtime::join_thread(worker) {
        Ok(()) => {}
        Err(e) if runtime::thread_unavailable(e) || e == syscall::Errno::ETIMEDOUT => {}
        Err(_) => {}
    }
    let _ = syscall::close(srv);
    let _ = syscall::close(srv2);
    let _ = syscall::close(idle_r);
    let _ = syscall::close(idle_w);
    let _ = syscall::close(ready_r);
    let _ = syscall::close(ready_w);

    match bind2 {
        Err(e) if e == syscall::Errno::EADDRINUSE => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("second bind succeeded")),
        Err(_) => return Err(crate::harness::AssertFail::msg("second bind errno")),
    }
    let elapsed = timespec_ms(&t1).saturating_sub(timespec_ms(&t0));
    check!(elapsed < 200, "second bind blocked on worker ppoll");
    Ok(())
}
