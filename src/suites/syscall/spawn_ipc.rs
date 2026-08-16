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

use core::sync::atomic::{AtomicU32, Ordering};

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::runtime;
use crate::syscall::{
    self, clock, epoll, fcntl_cmd, oflag, poll, wait, SockAddrIn, AF_INET, AF_UNIX, EPOLLET,
    EPOLLIN, EPOLLOUT, EPOLL_CTL_ADD, F_OK, POLLIN, POLLHUP, SIGKILL, SOCK_CLOEXEC, SOCK_STREAM,
    SOL_SOCKET, SO_REUSEADDR,
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
