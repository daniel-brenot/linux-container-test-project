//! Signal delivery syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, SIGKILL, SIGTERM};

#[crate::lctp_test(suite = syscall)]
fn kill_self_zero() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "kill(self,0)");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn kill_child_sigterm() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        // Child waits to be killed.
        let req = syscall::Timespec { tv_sec: 60, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(99);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "kill SIGTERM");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGTERM, "SIGTERM");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn kill_child_sigkill() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 60, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGKILL), "kill SIGKILL");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGKILL, "SIGKILL");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn kill_invalid_pid() -> TestResult {
    match syscall::kill(999_999_999, 0) {
        Ok(()) => {}
        Err(crate::syscall::Errno::ESRCH) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("kill invalid pid errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn wait_signaled_sigterm() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 60, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "WIFSIGNALED");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn wait_signaled_sigkill() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 60, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGKILL), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "WIFSIGNALED");
    check_eq!(syscall::wtermsig(status), SIGKILL, "term sig");
    Ok(())
}
