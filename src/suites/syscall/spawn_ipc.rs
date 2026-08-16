//! Live `fork`+`exec` helpers that speak first on an IPC channel.
//!
//! Theia plugin-host and Node `child_process.fork` create a socketpair, fork,
//! exec a child that must stay alive, and read a handshake the *child* writes.
//! A runtime that invents an immediate zombie (exit 0, EOF on the channel)
//! or refuses to run the new image (exit 127) fails these cases. Existing
//! echo helpers write only after the parent, so they do not catch that.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, poll, wait, SockAddrIn, AF_INET, AF_UNIX, POLLIN, POLLHUP, SIGKILL, SOCK_CLOEXEC,
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
