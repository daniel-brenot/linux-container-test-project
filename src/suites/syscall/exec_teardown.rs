//! Nested helper teardown / "reload" lifecycle.
//!
//! When a parent kills a nested same-image helper that holds an IPC channel,
//! the peer socket must hang up and a subsequent spawn must get a fresh
//! working channel. Leaving a detached helper alive after guest kill surfaces
//! as a closed/stale transport on the next registration.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, poll, wait, AF_UNIX, POLLIN, POLLHUP, SIGKILL, SIGTERM, SOCK_STREAM,
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

fn spawn_hold_helper(exe: &[u8]) -> Result<(i32, i32), crate::harness::AssertFail> {
    // Returns (parent_sock, child_pid).
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
        let _ = syscall::setsid();
        let env0 = b"CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let mut arg0 = [0u8; 256];
        let n = exe.len();
        arg0[..n].copy_from_slice(exe);
        let flag = b"--ipc-channel-hold\0";
        let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(exe, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    Ok((a, pid))
}

fn spawn_echo_helper(exe: &[u8]) -> Result<(i32, i32), crate::harness::AssertFail> {
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
        let n = exe.len();
        arg0[..n].copy_from_slice(exe);
        let flag = b"--ipc-channel-echo\0";
        let argv = [arg0.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(exe, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    Ok((a, pid))
}

fn reap_killed(pid: i32) -> Result<i32, crate::harness::AssertFail> {
    let mut status = 0;
    for _ in 0..100 {
        match syscall::wait4(pid, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid => return Ok(status),
            Ok(0) => {}
            Ok(_) | Err(syscall::Errno::ECHILD) => {}
            Err(_) => return Err(crate::harness::AssertFail::msg("wait4")),
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 20_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    Err(crate::harness::AssertFail::msg("kill did not reap"))
}

fn channel_hung_up(fd: i32) -> Result<bool, crate::harness::AssertFail> {
    let mut fds = [poll::PollFd {
        fd,
        events: POLLIN | POLLHUP,
        revents: 0,
    }];
    let _ = syscall::poll(&mut fds, 500);
    if fds[0].revents & POLLHUP != 0 {
        return Ok(true);
    }
    let mut buf = [0u8; 8];
    match syscall::read(fd, &mut buf) {
        Ok(0) => Ok(true),
        Ok(_) => Ok(false),
        Err(syscall::Errno::ECONNRESET)
        | Err(syscall::Errno::EPIPE)
        | Err(syscall::Errno::EIO) => Ok(true),
        Err(syscall::Errno::EAGAIN) => Ok(false),
        Err(_) => Ok(true), // treat other errors as dead channel
    }
}

#[crate::lctp_test(suite = syscall, expect = success, case = "SIGKILL of a nested helper holding an IPC socket makes the parent observe hangup")]
fn exec_kill_helper_hangs_up_channel() -> TestResult {
    // Guest kill of a nested ET_EXEC helper must close its IPC sockets so the
    // parent observes hangup — not a live peer on a "zombie" mapping.
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (sock, pid) = spawn_hold_helper(&exe[..exe_len])?;

    // Prove the helper is alive and holding the channel.
    let mut fds = [poll::PollFd {
        fd: sock,
        events: POLLIN | POLLHUP,
        revents: 0,
    }];
    check_eq!(check_ok!(syscall::poll(&mut fds, 50), "poll live"), 0, "idle");

    check_ok!(syscall::kill(pid, SIGKILL), "SIGKILL");
    let status = reap_killed(pid)?;
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGKILL, "SIGKILL");

    check!(channel_hung_up(sock)?, "channel still open after kill");
    let _ = syscall::close(sock);
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a second wait4 after reaping a killed helper returns ECHILD or no further status")]
fn exec_kill_helper_second_wait_echild() -> TestResult {
    // Waiter must not invent a second zombie after kill already reaped.
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (sock, pid) = spawn_hold_helper(&exe[..exe_len])?;
    check_ok!(syscall::kill(pid, SIGKILL), "kill");
    let _ = reap_killed(pid)?;
    let mut status = 0;
    match syscall::wait4(pid, &mut status, wait::WNOHANG) {
        Err(syscall::Errno::ECHILD) => {}
        Ok(0) => {
            // No status change; also acceptable if pid is fully gone from
            // the wait set — try one more blocking-style probe.
            match syscall::wait4(pid, &mut status, wait::WNOHANG) {
                Err(syscall::Errno::ECHILD) | Ok(0) => {}
                Ok(_) => return Err(crate::harness::AssertFail::msg("double reap")),
                Err(_) => return Err(crate::harness::AssertFail::msg("wait errno")),
            }
        }
        Ok(_) => return Err(crate::harness::AssertFail::msg("double reap pid")),
        Err(_) => return Err(crate::harness::AssertFail::msg("wait errno")),
    }
    let _ = syscall::close(sock);
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "after killing helper A, a new echo helper on a fresh socket round-trips a payload")]
fn exec_reload_kill_then_new_helper() -> TestResult {
    // Reload shape: tear down helper A (kill), then register helper B with a
    // new channel. Old sockets must be dead; new channel must work.
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;

    let (sock_a, pid_a) = spawn_hold_helper(&exe[..exe_len])?;
    check_ok!(syscall::kill(pid_a, SIGTERM), "SIGTERM");
    // If TERM is ignored by a stuck guest, escalate.
    let mut status = 0;
    let mut reaped = false;
    for _ in 0..50 {
        match syscall::wait4(pid_a, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid_a => {
                reaped = true;
                break;
            }
            _ => {}
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 20_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    if !reaped {
        check_ok!(syscall::kill(pid_a, SIGKILL), "escalate");
        let _ = reap_killed(pid_a)?;
    }
    check!(channel_hung_up(sock_a)?, "old channel live");
    let _ = syscall::close(sock_a);

    let (sock_b, pid_b) = spawn_echo_helper(&exe[..exe_len])?;
    check_ok!(syscall::write(sock_b, b"reload"), "write new");
    let mut fds = [poll::PollFd {
        fd: sock_b,
        events: POLLIN,
        revents: 0,
    }];
    check!(check_ok!(syscall::poll(&mut fds, 5000), "poll new") >= 1, "rdy");
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::read(sock_b, &mut buf), "read new");
    check_eq!(n, 6, "len");
    check_eq!(&buf[..6], b"reload", "payload");
    let _ = syscall::close(sock_b);
    let mut st = 0;
    for _ in 0..100 {
        match syscall::wait4(pid_b, &mut st, wait::WNOHANG) {
            Ok(p) if p == pid_b => break,
            _ => {
                let req = syscall::Timespec {
                    tv_sec: 0,
                    tv_nsec: 20_000_000,
                };
                let _ = syscall::nanosleep(&req);
            }
        }
    }
    check!(syscall::wifexited(st), "echo exited");
    check_eq!(syscall::wexitstatus(st), 0, "echo status");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "SIGKILL of an execed sleep that holds a socketpair end hangs up the peer, or sleep is absent")]
fn exec_sigkill_sleep_peer_eof() -> TestResult {
    // Same hangup property with a plain external binary holding the fd.
    let sleep = if syscall::access(b"/bin/sleep\0", syscall::F_OK).is_ok() {
        b"/bin/sleep\0" as &[u8]
    } else if syscall::access(b"/usr/bin/sleep\0", syscall::F_OK).is_ok() {
        b"/usr/bin/sleep\0"
    } else {
        return Ok(());
    };
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(a);
        // Leave `b` open so the socket stays alive until we die.
        let arg0 = b"sleep\0";
        let arg1 = b"30\0";
        let argv = [arg0.as_ptr(), arg1.as_ptr(), core::ptr::null()];
        let envp = [core::ptr::null::<u8>()];
        let _ = b;
        let _ = syscall::execve(sleep, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    check_ok!(syscall::kill(pid, SIGKILL), "kill");
    let status = reap_killed(pid)?;
    check!(syscall::wifsignaled(status), "signaled");
    check!(channel_hung_up(a)?, "eof");
    let _ = syscall::close(a);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "two kill-then-spawn cycles each hang up the old channel and echo on a new one")]
fn exec_reload_two_cycles() -> TestResult {
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    for _ in 0..2 {
        let (sock, pid) = spawn_hold_helper(&exe[..exe_len])?;
        check_ok!(syscall::kill(pid, SIGKILL), "kill");
        let _ = reap_killed(pid)?;
        check!(channel_hung_up(sock)?, "hup");
        let _ = syscall::close(sock);

        let (sock2, pid2) = spawn_echo_helper(&exe[..exe_len])?;
        check_ok!(syscall::write(sock2, b"ok"), "w");
        let mut fds = [poll::PollFd {
            fd: sock2,
            events: POLLIN,
            revents: 0,
        }];
        check!(check_ok!(syscall::poll(&mut fds, 5000), "p") >= 1, "rdy");
        let mut buf = [0u8; 8];
        check_eq!(check_ok!(syscall::read(sock2, &mut buf), "r"), 2, "n");
        let _ = syscall::close(sock2);
        let mut st = 0;
        for _ in 0..100 {
            if syscall::wait4(pid2, &mut st, wait::WNOHANG).ok() == Some(pid2) {
                break;
            }
            let req = syscall::Timespec {
                tv_sec: 0,
                tv_nsec: 20_000_000,
            };
            let _ = syscall::nanosleep(&req);
        }
        check_eq!(syscall::wexitstatus(st), 0, "st");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "closing the parent end of a helper channel lets the holding helper exit")]
fn exec_parent_close_then_helper_exits() -> TestResult {
    // Parent teardown of the channel (close) should let a holding helper observe
    // EOF and exit cleanly — mirrors host dropping IPC on session end.
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (sock, pid) = spawn_hold_helper(&exe[..exe_len])?;
    let _ = syscall::close(sock);
    let mut status = 0;
    for _ in 0..100 {
        match syscall::wait4(pid, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid => {
                check!(syscall::wifexited(status), "exited");
                return Ok(());
            }
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
    Err(crate::harness::AssertFail::msg("helper did not exit on hup"))
}
