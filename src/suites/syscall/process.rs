//! Process-related syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, wait, Errno};

#[crate::lctp_test(suite = syscall)]
fn fork_exit_status() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(42);
    }
    let mut status = 0;
    check_eq!(check_ok!(syscall::wait4(pid, &mut status, 0), "wait4"), pid, "pid");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 42, "status");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn fork_exit_zero() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "status zero");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn fork_getppid() -> TestResult {
    let parent = syscall::getpid();
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::getppid() == parent {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "ppid mismatch");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getpid_stable() -> TestResult {
    let a = syscall::getpid();
    let b = syscall::getpid();
    check!(a > 0, "getpid <= 0");
    check_eq!(a, b, "getpid unstable");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getppid_non_negative() -> TestResult {
    // PID 1 (typical container entrypoint) has ppid 0.
    check!(syscall::getppid() >= 0, "getppid negative");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getuid_euid_match() -> TestResult {
    check_eq!(syscall::getuid(), syscall::geteuid(), "uid != euid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getgid_egid_match() -> TestResult {
    check_eq!(syscall::getgid(), syscall::getegid(), "gid != egid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn wait_nohang_nochild() -> TestResult {
    let mut status = 0;
    match syscall::wait4(-1, &mut status, wait::WNOHANG) {
        Ok(0) => {}
        Err(Errno::ECHILD) => {}
        Ok(pid) => {
            // Reap unexpected child if any.
            let _ = syscall::wait4(pid, &mut status, 0);
            return Err(crate::harness::AssertFail::msg("unexpected child"));
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("wait WNOHANG errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn wait4_reap_zombie() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(7);
    }
    let mut status = 0;
    check_eq!(check_ok!(syscall::wait4(pid, &mut status, 0), "wait4"), pid, "pid");
    check_eq!(syscall::wexitstatus(status), 7, "status");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn fork_double_child() -> TestResult {
    for code in [11i32, 22] {
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            syscall::exit(code);
        }
        let mut status = 0;
        check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
        check_eq!(syscall::wexitstatus(status), code, "status");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn gettid_equals_pid() -> TestResult {
    check_eq!(syscall::gettid(), syscall::getpid(), "tid != pid");
    Ok(())
}
