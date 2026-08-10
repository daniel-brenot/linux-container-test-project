//! POSIX signal delivery semantics.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, SIGINT, SIGTERM};

#[crate::lctp_test(suite = posix)]
fn signal_kill_self_zero() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "kill 0");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn signal_child_sigterm_reap() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 120, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGTERM, "sig");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn signal_child_sigint_reap() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 120, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGINT), "kill INT");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGINT, "sig");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn signal_wtermsig_matches() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 120, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wtermsig(status), SIGTERM, "wtermsig");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn signal_exit_not_signaled() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(3);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    check!(!syscall::wifsignaled(status), "not signaled");
    check_eq!(syscall::wexitstatus(status), 3, "code");
    Ok(())
}
