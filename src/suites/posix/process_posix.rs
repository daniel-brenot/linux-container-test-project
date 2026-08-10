//! POSIX process group, times, and priority tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, PRIO_PROCESS};

#[crate::lctp_test(suite = posix)]
fn posix_getpgid_zero() -> TestResult {
    let pgid = check_ok!(syscall::getpgid(0), "getpgid");
    check!(pgid > 0, "pgid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_getpgid_matches_pid() -> TestResult {
    let a = check_ok!(syscall::getpgid(0), "pgid 0");
    let b = check_ok!(syscall::getpgid(syscall::getpid()), "pgid pid");
    check_eq!(a, b, "match");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_times_non_negative() -> TestResult {
    let t = check_ok!(syscall::times(), "times");
    check!(t.tms_utime >= 0, "utime");
    check!(t.tms_stime >= 0, "stime");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_times_child_fields() -> TestResult {
    let t = check_ok!(syscall::times(), "times");
    check!(t.tms_cutime >= 0, "cutime");
    check!(t.tms_cstime >= 0, "cstime");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_getpriority_self() -> TestResult {
    let nice = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "priority");
    check!(nice >= -20 && nice <= 19, "nice");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_getpriority_pid() -> TestResult {
    let nice = check_ok!(syscall::getpriority(PRIO_PROCESS, syscall::getpid()), "priority");
    check!(nice >= -20 && nice <= 19, "nice");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn posix_times_after_fork() -> TestResult {
    let t0 = check_ok!(syscall::times(), "times0");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    let t1 = check_ok!(syscall::times(), "times1");
    check!(t1.tms_cutime >= t0.tms_cutime, "cutime grew");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_getsid_self() -> TestResult {
    let sid = check_ok!(syscall::getsid(0), "getsid");
    check!(sid > 0, "sid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_getresuid_consistent() -> TestResult {
    let (r, e, s) = check_ok!(syscall::getresuid(), "getresuid");
    check_eq!(r, e, "uid");
    check_eq!(e, s, "saved");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_getresgid_consistent() -> TestResult {
    let (r, e, s) = check_ok!(syscall::getresgid(), "getresgid");
    check_eq!(r, e, "gid");
    check_eq!(e, s, "saved");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn posix_priority_stable() -> TestResult {
    let a = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p1");
    let b = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p2");
    check_eq!(a, b, "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn posix_pgid_positive() -> TestResult {
    check!(check_ok!(syscall::getpgid(0), "pgid") > 0, "positive");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn posix_child_same_pgid() -> TestResult {
    let pgid = check_ok!(syscall::getpgid(0), "pgid");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::getpgid(0).ok() == Some(pgid) {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "same pgid");
    Ok(())
}
