//! Scheduling / TPS-ish: sched_yield, getaffinity, getscheduler, getpriority.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, PRIO_PROCESS, SCHED_OTHER};

macro_rules! yield_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            for _ in 0..$n {
                check_ok!(syscall::sched_yield(), "yield");
            }
            Ok(())
        }
    };
}

yield_n!(tps_yield_1, 1);
yield_n!(tps_yield_2, 2);
yield_n!(tps_yield_4, 4);
yield_n!(tps_yield_8, 8);
yield_n!(tps_yield_16, 16);
yield_n!(tps_yield_32, 32);

#[crate::lctp_test(suite = posix)]
fn tps_getaffinity_self() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    let any = mask.iter().any(|&b| b != 0);
    check!(any, "some cpu");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getaffinity_pid() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(
        syscall::sched_getaffinity(syscall::getpid(), &mut mask),
        "aff"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getscheduler_self() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getscheduler_pid() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(syscall::getpid()), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getpriority_self() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice range");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getpriority_pid() -> TestResult {
    let p = check_ok!(
        syscall::getpriority(PRIO_PROCESS, syscall::getpid()),
        "prio"
    );
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getpriority_zero_eq_pid() -> TestResult {
    let a = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "a");
    let b = check_ok!(
        syscall::getpriority(PRIO_PROCESS, syscall::getpid()),
        "b"
    );
    check_eq!(a, b, "same");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_yield_between_prio() -> TestResult {
    let a = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "a");
    check_ok!(syscall::sched_yield(), "y");
    let b = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "b");
    check_eq!(a, b, "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getscheduler_twice() -> TestResult {
    let a = check_ok!(syscall::sched_getscheduler(0), "a");
    let b = check_ok!(syscall::sched_getscheduler(0), "b");
    check_eq!(a, b, "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_affinity_mask_stable() -> TestResult {
    let mut a = [0u8; 64];
    let mut b = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut a), "a");
    check_ok!(syscall::sched_getaffinity(0, &mut b), "b");
    check_eq!(&a[..], &b[..], "same");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getpriority_bad_which() -> TestResult {
    match syscall::getpriority(99, 0) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(_) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix)]
fn tps_getpriority_esrch_soft() -> TestResult {
    match syscall::getpriority(PRIO_PROCESS, 999_999_999) {
        Err(Errno::ESRCH) => Ok(()),
        Ok(_) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn tps_child_scheduler() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        match syscall::sched_getscheduler(0) {
            Ok(p) if p == SCHED_OTHER => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "ex");
    check_eq!(syscall::wexitstatus(status), 0, "ok");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_child_priority() -> TestResult {
    let parent = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        match syscall::getpriority(PRIO_PROCESS, 0) {
            Ok(c) if c == parent => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wexitstatus(status), 0, "same nice");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_yield_ok() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_affinity_small_buf_soft() -> TestResult {
    let mut mask = [0u8; 8];
    match syscall::sched_getaffinity(0, &mut mask) {
        Ok(()) => check!(mask.iter().any(|&b| b != 0), "bit"),
        Err(Errno::EINVAL) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getscheduler_bad_pid_soft() -> TestResult {
    match syscall::sched_getscheduler(999_999_999) {
        Err(Errno::ESRCH) | Err(Errno::EINVAL) => Ok(()),
        Ok(_) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn tps_yield_then_affinity() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_prio_stable_twice() -> TestResult {
    let a = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "a");
    let b = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "b");
    check_eq!(a, b, "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_many_yields_full() -> TestResult {
    for _ in 0..64 {
        check_ok!(syscall::sched_yield(), "y");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_scheduler_other_constant() -> TestResult {
    check_eq!(SCHED_OTHER, 0, "const");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_getaffinity_after_fork() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let mut mask = [0u8; 64];
        match syscall::sched_getaffinity(0, &mut mask) {
            Ok(()) => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wexitstatus(status), 0, "ok");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_yield_interleaved_prio() -> TestResult {
    for _ in 0..4 {
        let _ = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
        check_ok!(syscall::sched_yield(), "y");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_getscheduler_child_pid() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let pol = check_ok!(syscall::sched_getscheduler(pid), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_affinity_len_64() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_prio_and_sched_together() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let s = check_ok!(syscall::sched_getscheduler(0), "s");
    check!(p >= -20 && p <= 19, "nice");
    check_eq!(s, SCHED_OTHER, "pol");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_yield_no_errno() -> TestResult {
    for _ in 0..3 {
        match syscall::sched_yield() {
            Ok(()) => {}
            Err(_) => return Err(crate::harness::AssertFail::msg("yield")),
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_affinity_cpu0_or_any() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    // At least one bit set somewhere.
    check!(mask.iter().copied().any(|b| b != 0), "cpus");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn tps_getpid_positive() -> TestResult {
    check!(syscall::getpid() > 0, "pid");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn tps_round_robin_queries() -> TestResult {
    for _ in 0..5 {
        check_ok!(syscall::sched_yield(), "y");
        let _ = check_ok!(syscall::sched_getscheduler(0), "s");
        let _ = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    }
    Ok(())
}
