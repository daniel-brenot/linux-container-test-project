//! unshare(2) tests (careful: only in forked children).

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, CLONE_FILES, CLONE_NEWUSER, Errno};

#[crate::lctp_test(suite = syscall)]
fn unshare_clone_files_in_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        match syscall::unshare(CLONE_FILES) {
            Ok(()) => syscall::exit(0),
            Err(_) => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "unshare CLONE_FILES");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn unshare_newuser_soft_eperm() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        match syscall::unshare(CLONE_NEWUSER) {
            Ok(()) => syscall::exit(0),
            Err(Errno::EPERM) | Err(Errno::EINVAL) => syscall::exit(0),
            Err(_) => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "NEWUSER ok or EPERM");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn unshare_clone_files_twice_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::unshare(CLONE_FILES).is_err() {
            syscall::exit(1);
        }
        if syscall::unshare(CLONE_FILES).is_err() {
            syscall::exit(2);
        }
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "twice");
    Ok(())
}
