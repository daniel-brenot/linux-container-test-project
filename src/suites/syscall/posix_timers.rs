//! POSIX interval timers (timer_create / settime / gettime / delete).

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, clock, Errno, Itimerspec, Sigevent, SIGEV_NONE, Timespec};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::EINVAL | Errno::ENOMEM
    )
}

fn make_sigev_none() -> Sigevent {
    let mut sev = Sigevent::default();
    sev.sigev_notify = SIGEV_NONE;
    sev
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "timer_create with SIGEV_NONE succeeds or is rejected with ENOSYS/EPERM/EINVAL")]
fn timer_create_monotonic_sigev_none() -> TestResult {
    let sev = make_sigev_none();
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => {
            check_ok!(syscall::timer_delete(tid), "delete");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("timer_create")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "timer_settime arms a POSIX timer that timer_gettime still shows as armed")]
fn timer_settime_gettime_roundtrip() -> TestResult {
    let sev = make_sigev_none();
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => {}
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("create")),
    }
    let new_val = Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        },
    };
    match syscall::timer_settime(tid, 0, &new_val, None) {
        Ok(()) => {}
        Err(e) if soft(e) => {
            let _ = syscall::timer_delete(tid);
            return Ok(());
        }
        Err(_) => {
            let _ = syscall::timer_delete(tid);
            return Err(crate::harness::AssertFail::msg("settime"));
        }
    }
    let cur = match syscall::timer_gettime(tid) {
        Ok(c) => c,
        Err(e) if soft(e) => {
            let _ = syscall::timer_delete(tid);
            return Ok(());
        }
        Err(_) => {
            let _ = syscall::timer_delete(tid);
            return Err(crate::harness::AssertFail::msg("gettime"));
        }
    };
    check!(
        cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0,
        "armed"
    );
    check_ok!(syscall::timer_delete(tid), "delete");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "timer_settime can shorten the remaining time of a POSIX timer")]
fn timer_rearm_updates_remaining() -> TestResult {
    let sev = make_sigev_none();
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => {}
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("create")),
    }
    let long = Itimerspec {
        it_interval: Timespec::default(),
        it_value: Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        },
    };
    if syscall::timer_settime(tid, 0, &long, None).is_err() {
        let _ = syscall::timer_delete(tid);
        return Ok(());
    }
    let short = Itimerspec {
        it_interval: Timespec::default(),
        it_value: Timespec {
            tv_sec: 5,
            tv_nsec: 0,
        },
    };
    check_ok!(syscall::timer_settime(tid, 0, &short, None), "rearm");
    let cur = check_ok!(syscall::timer_gettime(tid), "gettime");
    check!(cur.it_value.tv_sec <= 5, "shortened");
    check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "still armed");
    check_ok!(syscall::timer_delete(tid), "delete");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "a second timer_delete returns EINVAL/EPERM or is otherwise unsupported")]
fn timer_delete_twice_soft() -> TestResult {
    let sev = make_sigev_none();
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => {}
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("create")),
    }
    check_ok!(syscall::timer_delete(tid), "delete");
    match syscall::timer_delete(tid) {
        Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("second delete ok")),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("second delete errno")),
    }
    Ok(())
}
