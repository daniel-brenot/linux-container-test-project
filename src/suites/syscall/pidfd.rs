//! pidfd_open + pidfd_send_signal tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, SIGKILL, SIGTERM};

#[crate::lctp_test(suite = syscall)]
fn pidfd_open_self() -> TestResult {
    let pid = syscall::getpid();
    let fd = check_ok!(syscall::pidfd_open(pid, 0), "pidfd_open");
    check!(fd >= 0, "fd");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pidfd_send_signal_sigkill_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "pidfd_open");
    check_ok!(
        syscall::pidfd_send_signal(pfd, SIGKILL, None, 0),
        "send_signal"
    );
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGKILL, "SIGKILL");
    check_ok!(syscall::close(pfd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pidfd_send_signal_sigterm_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "pidfd_open");
    check_ok!(
        syscall::pidfd_send_signal(pfd, SIGTERM, None, 0),
        "send_signal"
    );
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGTERM, "SIGTERM");
    check_ok!(syscall::close(pfd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pidfd_open_child_then_kill() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        // Block forever on a pipe read so we are interruptible by SIGKILL.
        let (r, w) = match syscall::pipe2(0) {
            Ok(p) => p,
            Err(_) => syscall::exit(1),
        };
        let _ = w;
        let mut buf = [0u8; 1];
        let _ = syscall::read(r, &mut buf);
        syscall::exit(0);
    }
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "pidfd_open");
    check_ok!(
        syscall::pidfd_send_signal(pfd, SIGKILL, None, 0),
        "kill"
    );
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "signaled");
    check_ok!(syscall::close(pfd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pidfd_send_signal_zero_probe() -> TestResult {
    let pid = syscall::getpid();
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "pidfd_open");
    // Signal 0 is a permission/existence probe and must not deliver a signal.
    check_ok!(syscall::pidfd_send_signal(pfd, 0, None, 0), "probe");
    check_ok!(syscall::close(pfd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pidfd_getfd_from_child_soft() -> TestResult {
    use crate::syscall::{Errno, AF_UNIX, SOCK_STREAM};

    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(a);
        // Park until parent finishes pidfd_getfd.
        let req = syscall::Timespec {
            tv_sec: 30,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        let _ = syscall::close(b);
        syscall::exit(0);
    }
    let _ = syscall::close(b);
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "pidfd_open");
    match syscall::pidfd_getfd(pfd, b, 0) {
        Ok(fd) => {
            check!(fd >= 0, "got fd");
            check_ok!(syscall::close(fd), "close got");
        }
        Err(Errno::EPERM) | Err(Errno::ENOSYS) | Err(Errno::EBADF) | Err(Errno::EINVAL) => {}
        Err(_) => {
            let _ = syscall::close(pfd);
            let _ = syscall::close(a);
            let _ = syscall::kill(pid, SIGKILL);
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            return Err(crate::harness::AssertFail::msg("pidfd_getfd"));
        }
    }
    check_ok!(syscall::kill(pid, SIGKILL), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_ok!(syscall::close(pfd), "close pfd");
    check_ok!(syscall::close(a), "close a");
    Ok(())
}
