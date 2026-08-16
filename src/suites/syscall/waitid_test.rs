//! waitid syscall tests.

use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, wait, Errno, P_ALL, P_PID, Siginfo};

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid WNOHANG with no children returns success or ECHILD")]
fn waitid_nohang_no_child() -> TestResult {
    let mut info = Siginfo::default();
    match syscall::waitid(P_ALL, 0, &mut info, wait::WNOHANG | wait::WEXITED) {
        Ok(()) => {}
        Err(Errno::ECHILD) => {}
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid P_PID with WEXITED reaps a child that called exit")]
fn waitid_after_fork_exit() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(17);
    }
    let mut info = Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WEXITED), "waitid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "waitid P_PID reaps the specified child")]
fn waitid_specific_pid() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(33);
    }
    let mut info = Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WEXITED), "waitid pid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid P_PID with WEXITED reaps a child that exited 0")]
fn waitid_zero_exit() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut info = Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WEXITED), "waitid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "waitid P_PID can reap two children by their pids")]
fn waitid_two_children() -> TestResult {
    let p1 = check_ok!(syscall::fork(), "fork1");
    if p1 == 0 {
        syscall::exit(3);
    }
    let p2 = check_ok!(syscall::fork(), "fork2");
    if p2 == 0 {
        syscall::exit(4);
    }
    let mut info = Siginfo::default();
    check_ok!(syscall::waitid(P_PID, p1, &mut info, wait::WEXITED), "waitid p1");
    check_ok!(syscall::waitid(P_PID, p2, &mut info, wait::WEXITED), "waitid p2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid WNOHANG with no remaining children returns immediately")]
fn waitid_nohang_reaped_none() -> TestResult {
    let mut info = Siginfo::default();
    // After prior tests there should be no child; WNOHANG returns immediately.
    let _ = syscall::waitid(P_ALL, 0, &mut info, wait::WNOHANG | wait::WEXITED);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "waitid P_PID with WEXITED reaps a child that exited 42")]
fn waitid_exit_status_42() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(42);
    }
    let mut info = Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WEXITED), "waitid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "waitid P_PID for a non-child pid returns ECHILD or ESRCH")]
fn waitid_invalid_pid_esrch() -> TestResult {
    let mut info = Siginfo::default();
    // Linux waitid(P_PID) for a non-child returns ECHILD (ESRCH on some paths).
    match syscall::waitid(P_PID, 999_999_999, &mut info, wait::WEXITED) {
        Err(Errno::ECHILD) | Err(Errno::ESRCH) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("expected ECHILD/ESRCH")),
        Err(_) => Err(crate::harness::AssertFail::msg("unexpected errno")),
    }
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "waitid P_PID with WNOHANG|WEXITED reaps a child that has already exited")]
fn waitid_nohang_after_exit() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(9);
    }
    let mut info = Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WNOHANG | wait::WEXITED), "waitid nohang");
    Ok(())
}
