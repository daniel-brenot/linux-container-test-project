//! Scheduler and priority syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, PRIO_PROCESS, SCHED_OTHER};

fn affinity_has_cpu(mask: &[u8]) -> bool {
    mask.iter().any(|&b| b != 0)
}

#[crate::lctp_test(suite = syscall)]
fn sched_getaffinity_self_nonempty() -> TestResult {
    let mut mask = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "getaffinity");
    check!(affinity_has_cpu(&mask), "empty cpumask");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_getaffinity_pid_self() -> TestResult {
    let mut mask = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(syscall::getpid(), &mut mask), "getaffinity pid");
    check!(affinity_has_cpu(&mask), "empty cpumask");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_getaffinity_small_buffer() -> TestResult {
    let mut mask = [0u8; 8];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "getaffinity small");
    check!(affinity_has_cpu(&mask), "cpu in small mask");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_getaffinity_large_buffer() -> TestResult {
    let mut mask = [0u8; 256];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "getaffinity large");
    check!(affinity_has_cpu(&mask), "cpu in large mask");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_getscheduler_self() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "getscheduler");
    check_eq!(pol, SCHED_OTHER, "policy");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_getscheduler_pid() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(syscall::getpid()), "getscheduler pid");
    check_eq!(pol, SCHED_OTHER, "policy");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getpriority_self() -> TestResult {
    let nice = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "getpriority");
    check!(nice >= -20 && nice <= 19, "nice range");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getpriority_pid() -> TestResult {
    let nice = check_ok!(syscall::getpriority(PRIO_PROCESS, syscall::getpid()), "getpriority pid");
    check!(nice >= -20 && nice <= 19, "nice range");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sched_getaffinity_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let mut mask = [0u8; 128];
        if syscall::sched_getaffinity(0, &mut mask).is_ok() && affinity_has_cpu(&mask) {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "child affinity");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn getpriority_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        match syscall::getpriority(PRIO_PROCESS, 0) {
            Ok(n) if n >= -20 && n <= 19 => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "child priority");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_getscheduler_zero_pid() -> TestResult {
    let a = check_ok!(syscall::sched_getscheduler(0), "sched 0");
    let b = check_ok!(syscall::sched_getscheduler(syscall::getpid()), "sched pid");
    check_eq!(a, b, "0 vs pid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getpriority_zero_vs_pid() -> TestResult {
    let a = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio 0");
    let b = check_ok!(syscall::getpriority(PRIO_PROCESS, syscall::getpid()), "prio pid");
    check_eq!(a, b, "0 vs pid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sched_getaffinity_twice_stable() -> TestResult {
    let mut m1 = [0u8; 128];
    let mut m2 = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut m1), "a1");
    check_ok!(syscall::sched_getaffinity(0, &mut m2), "a2");
    check_eq!(m1, m2, "stable mask");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_setaffinity_same_mask() -> TestResult {
    let mut mask = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    check_ok!(syscall::sched_setaffinity(0, &mask), "set same");
    let mut again = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut again), "get again");
    check!(affinity_has_cpu(&again), "still nonempty");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_setaffinity_pid_self() -> TestResult {
    let mut mask = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(syscall::getpid(), &mut mask), "get");
    check_ok!(
        syscall::sched_setaffinity(syscall::getpid(), &mask),
        "set pid"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sched_setaffinity_roundtrip() -> TestResult {
    let mut mask = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    check_ok!(syscall::sched_setaffinity(0, &mask), "set");
    check_ok!(syscall::sched_setaffinity(0, &mask), "set twice");
    Ok(())
}
