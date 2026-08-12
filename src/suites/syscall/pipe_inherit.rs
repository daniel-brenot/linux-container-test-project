//! Inherited empty pipes across `fork` / `exec` / `poll`.
//!
//! Shells often leave pipe ends open when forking external commands. A child
//! that blocks on an empty pipe while the parent `waitpid`s deadlocks if the
//! runtime wrongly deschedules the child (e.g. cooperative yield on read/poll)
//! instead of letting `poll` time out or `read` see EOF after the writer closes.
//! Detached sessions (`setsid`) are included — the same pattern as a PTY shell
//! spawning `ls`.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, poll, wait, Errno, F_OK, POLLIN};

fn wait_exit_status(pid: i32) -> Result<i32, crate::harness::AssertFail> {
    let mut status = 0;
    // Bound wait: poll WNOHANG with nanosleep so a guest deadlock fails the test
    // instead of hanging the suite forever.
    for _ in 0..200 {
        match syscall::wait4(pid, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid => {
                check!(syscall::wifexited(status), "exited");
                return Ok(syscall::wexitstatus(status));
            }
            Ok(_) | Err(Errno::ECHILD) => {}
            Err(_) => return Err(crate::harness::AssertFail::msg("wait4")),
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000, // 50ms
        };
        let _ = syscall::nanosleep(&req);
    }
    // Last blocking wait would hang; kill and fail.
    let _ = syscall::kill(pid, 9);
    let _ = syscall::wait4(pid, &mut status, 0);
    Err(crate::harness::AssertFail::msg("child hung (pipe/wait deadlock?)"))
}

#[crate::lctp_test(suite = syscall)]
fn pipe_eof_when_writer_closed() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(w);
        let mut b = [0u8; 8];
        match syscall::read(r, &mut b) {
            Ok(0) => syscall::exit(0), // EOF
            Ok(_) => syscall::exit(2),
            Err(_) => syscall::exit(3),
        }
    }
    let _ = syscall::close(r);
    let _ = syscall::close(w); // no writers → child read returns 0
    let code = wait_exit_status(pid)?;
    check_eq!(code, 0, "expected EOF");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pipe_poll_timeout_writer_held() -> TestResult {
    // Parent keeps the write end open (empty pipe). Child `poll`s with a short
    // timeout, then exits. Must complete — yielding the child into the parent's
    // wait forever is the failure mode.
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(w);
        let mut fds = [poll::PollFd {
            fd: r,
            events: POLLIN,
            revents: 0,
        }];
        match syscall::poll(&mut fds, 100) {
            Ok(0) => syscall::exit(0), // timed out as expected
            Ok(_) => syscall::exit(2),
            Err(_) => syscall::exit(3),
        }
    }
    let _ = syscall::close(r);
    // Hold `w` open until the child finishes.
    let code = wait_exit_status(pid)?;
    let _ = syscall::close(w);
    check_eq!(code, 0, "poll timeout");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pipe_poll_then_exec_writer_held() -> TestResult {
    // Same as above, but the child must reach `execve` after the timed poll —
    // shells fork, probe fds, then exec the external command.
    let true_path: &[u8] = if syscall::access(b"/bin/true\0", F_OK).is_ok() {
        b"/bin/true\0"
    } else if syscall::access(b"/usr/bin/true\0", F_OK).is_ok() {
        b"/usr/bin/true\0"
    } else {
        return Ok(());
    };
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(w);
        let mut fds = [poll::PollFd {
            fd: r,
            events: POLLIN,
            revents: 0,
        }];
        let _ = syscall::poll(&mut fds, 50);
        // Leave `r` inherited (not CLOEXEC) — external cmds often see leftover fds.
        let arg0 = b"true\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(true_path, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(r);
    let code = wait_exit_status(pid)?;
    let _ = syscall::close(w);
    check_eq!(code, 0, "exec after poll");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pipe_setsid_poll_then_exec() -> TestResult {
    // Detached session + empty inherited pipe + exec (PTY shell spawn shape).
    let true_path: &[u8] = if syscall::access(b"/bin/true\0", F_OK).is_ok() {
        b"/bin/true\0"
    } else if syscall::access(b"/usr/bin/true\0", F_OK).is_ok() {
        b"/usr/bin/true\0"
    } else {
        return Ok(()); // soft
    };
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(w);
        let _ = syscall::setsid();
        let mut fds = [poll::PollFd {
            fd: r,
            events: POLLIN,
            revents: 0,
        }];
        let _ = syscall::poll(&mut fds, 50);
        let arg0 = b"true\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(true_path, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(r);
    let code = wait_exit_status(pid)?;
    let _ = syscall::close(w);
    check_eq!(code, 0, "setsid exec");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pipe_inherit_exec_without_touching() -> TestResult {
    // Child inherits an empty pipe and execs without reading it.
    let true_path: &[u8] = if syscall::access(b"/bin/true\0", F_OK).is_ok() {
        b"/bin/true\0"
    } else if syscall::access(b"/usr/bin/true\0", F_OK).is_ok() {
        b"/usr/bin/true\0"
    } else {
        return Ok(());
    };
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(w);
        let arg0 = b"true\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(true_path, &argv, &envp);
        let _ = r;
        syscall::exit(127);
    }
    let _ = syscall::close(r);
    let code = wait_exit_status(pid)?;
    let _ = syscall::close(w);
    check_eq!(code, 0, "exec with inherited pipe");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pipe_shell_pipeline_completes() -> TestResult {
    // Pipeline needs both sides to run; wrong yield on the empty pipe side
    // deadlocks the shell's wait.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let (out_r, out_w) = check_ok!(syscall::pipe2(0), "out");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(out_r);
        if syscall::dup2(out_w, 1).is_err() {
            syscall::exit(125);
        }
        let _ = syscall::close(out_w);
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"printf xy | cat\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(out_w);
    let mut buf = [0u8; 16];
    let mut n = 0usize;
    for _ in 0..40 {
        match syscall::read(out_r, &mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => {
                n += k;
                if n >= 2 {
                    break;
                }
            }
            Err(_) => break,
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    let _ = syscall::close(out_r);
    let code = wait_exit_status(pid)?;
    check_eq!(code, 0, "pipeline status");
    check!(n >= 2 && buf[0] == b'x' && buf[1] == b'y', "pipeline out");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pipe_shell_external_ls_completes() -> TestResult {
    // Shell forks an external binary (`ls`); must reap without hanging.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let (out_r, out_w) = check_ok!(syscall::pipe2(0), "out");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(out_r);
        if syscall::dup2(out_w, 1).is_err() || syscall::dup2(out_w, 2).is_err() {
            syscall::exit(125);
        }
        let _ = syscall::close(out_w);
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"ls /\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(out_w);
    let mut buf = [0u8; 512];
    let mut n = 0usize;
    for _ in 0..60 {
        match syscall::read(out_r, &mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => {
                n += k;
                if n >= 3 {
                    // keep draining a bit
                }
            }
            Err(_) => break,
        }
        let mut st = 0;
        if syscall::wait4(pid, &mut st, wait::WNOHANG).ok() == Some(pid) {
            while n < buf.len() {
                match syscall::read(out_r, &mut buf[n..]) {
                    Ok(0) | Err(_) => break,
                    Ok(k) => n += k,
                }
            }
            let _ = syscall::close(out_r);
            check!(syscall::wifexited(st), "exited");
            check_eq!(syscall::wexitstatus(st), 0, "ls status");
            check!(contains_root_entry(&buf[..n]), "ls listing");
            return Ok(());
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    let _ = syscall::close(out_r);
    let code = wait_exit_status(pid)?;
    check_eq!(code, 0, "ls status");
    check!(contains_root_entry(&buf[..n]), "ls listing");
    Ok(())
}

fn contains_root_entry(buf: &[u8]) -> bool {
    // Common rootfs entries — enough to prove `ls /` ran.
    contains(buf, b"etc") || contains(buf, b"bin") || contains(buf, b"usr") || contains(buf, b"tmp")
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return false;
    }
    for i in 0..=hay.len() - needle.len() {
        if &hay[i..i + needle.len()] == needle {
            return true;
        }
    }
    false
}
