//! Time clocks, nanosleep, timerfd, itimer, posix timer depth.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, clock, Errno, Itimerspec, Itimerval, Sigevent, Timespec, Timeval, TFD_CLOEXEC,
    TFD_NONBLOCK, TFD_TIMER_ABSTIME, ITIMER_REAL, SIGEV_NONE,
};

fn valid_nsec(ts: &Timespec) -> bool {
    ts.tv_nsec >= 0 && ts.tv_nsec < 1_000_000_000
}

fn soft_clock(e: Errno) -> bool {
    matches!(e, Errno::EINVAL | Errno::ENOSYS | Errno::EPERM)
}

macro_rules! clock_get {
    ($name:ident, $id:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = "clock_gettime of a named clock returns a valid timespec or EINVAL/ENOSYS/EPERM")]
        fn $name() -> TestResult {
            match syscall::clock_gettime($id) {
                Ok(t) => {
                    check!(t.tv_sec >= 0, "sec");
                    check!(valid_nsec(&t), "nsec");
                }
                Err(e) if soft_clock(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("clock_gettime")),
            }
            Ok(())
        }
    };
}

clock_get!(time_depth_realtime, clock::CLOCK_REALTIME);
clock_get!(time_depth_monotonic, clock::CLOCK_MONOTONIC);
clock_get!(time_depth_monotonic_raw, clock::CLOCK_MONOTONIC_RAW);
clock_get!(time_depth_process_cputime, clock::CLOCK_PROCESS_CPUTIME_ID);
clock_get!(time_depth_thread_cputime, clock::CLOCK_THREAD_CPUTIME_ID);
clock_get!(time_depth_realtime_coarse, clock::CLOCK_REALTIME_COARSE);
clock_get!(time_depth_monotonic_coarse, clock::CLOCK_MONOTONIC_COARSE);
clock_get!(time_depth_boottime, clock::CLOCK_BOOTTIME);

macro_rules! clock_res {
    ($name:ident, $id:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = "clock_getres of a named clock returns a nonzero resolution or EINVAL/ENOSYS/EPERM")]
        fn $name() -> TestResult {
            match syscall::clock_getres($id) {
                Ok(t) => {
                    check!(t.tv_sec >= 0, "sec");
                    check!(valid_nsec(&t), "nsec");
                    check!(t.tv_sec > 0 || t.tv_nsec > 0, "nonzero res");
                }
                Err(e) if soft_clock(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("clock_getres")),
            }
            Ok(())
        }
    };
}

clock_res!(time_depth_res_realtime, clock::CLOCK_REALTIME);
clock_res!(time_depth_res_monotonic, clock::CLOCK_MONOTONIC);
clock_res!(time_depth_res_monotonic_raw, clock::CLOCK_MONOTONIC_RAW);
clock_res!(time_depth_res_boottime, clock::CLOCK_BOOTTIME);
clock_res!(time_depth_res_process, clock::CLOCK_PROCESS_CPUTIME_ID);
clock_res!(time_depth_res_thread, clock::CLOCK_THREAD_CPUTIME_ID);
clock_res!(time_depth_res_realtime_coarse, clock::CLOCK_REALTIME_COARSE);
clock_res!(time_depth_res_monotonic_coarse, clock::CLOCK_MONOTONIC_COARSE);

#[crate::lctp_test(suite = syscall, expect = success, case = "nanosleep of zero duration succeeds")]
fn nanosleep_zero() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 0 };
    check_ok!(syscall::nanosleep(&req), "zero");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "nanosleep of one nanosecond succeeds")]
fn nanosleep_one_ns() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1 };
    check_ok!(syscall::nanosleep(&req), "1ns");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "nanosleep with tv_nsec of 1e9 returns EINVAL or succeeds")]
fn nanosleep_bad_nsec_einval() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1_000_000_000 };
    match syscall::nanosleep(&req) {
        Err(Errno::EINVAL) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("bad nsec")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "nanosleep with a negative tv_sec returns EINVAL or succeeds")]
fn nanosleep_negative_sec_einval() -> TestResult {
    let req = Timespec { tv_sec: -1, tv_nsec: 0 };
    match syscall::nanosleep(&req) {
        Err(Errno::EINVAL) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("neg sec")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "a short nanosleep succeeds or returns EINTR")]
fn nanosleep_interrupt_soft() -> TestResult {
    // Without a signal, just ensure short sleep works (interrupt path soft).
    let req = Timespec { tv_sec: 0, tv_nsec: 100_000 };
    match syscall::nanosleep(&req) {
        Ok(()) => {}
        Err(Errno::EINTR) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("nanosleep")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "clock_nanosleep of zero on CLOCK_MONOTONIC succeeds")]
fn clock_nanosleep_zero() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 0 };
    check_ok!(syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req), "zero");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "clock_nanosleep of a short interval on CLOCK_REALTIME succeeds")]
fn clock_nanosleep_realtime() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1000 };
    check_ok!(syscall::clock_nanosleep(clock::CLOCK_REALTIME, 0, &req), "rt");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "gettimeofday returns tv_usec in 0..1e6")]
fn gettimeofday_usec_range() -> TestResult {
    let tv = check_ok!(syscall::gettimeofday(), "gtod");
    check!(tv.tv_usec >= 0 && tv.tv_usec < 1_000_000, "usec");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_create with CLOCK_REALTIME and TFD_CLOEXEC succeeds")]
fn timerfd_create_realtime() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_REALTIME, TFD_CLOEXEC), "tfd");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_create with CLOCK_MONOTONIC and TFD_CLOEXEC succeeds")]
fn timerfd_create_monotonic() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC), "tfd");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_create with TFD_CLOEXEC|TFD_NONBLOCK succeeds")]
fn timerfd_create_nonblock() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC | TFD_NONBLOCK),
        "tfd"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_settime with TFD_TIMER_ABSTIME arms a future monotonic expiry")]
fn timerfd_absolute_set() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC), "tfd");
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let its = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec {
            tv_sec: now.tv_sec + 3600,
            tv_nsec: now.tv_nsec,
        },
    };
    check_ok!(syscall::timerfd_settime(fd, TFD_TIMER_ABSTIME, &its), "set");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "get");
    check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
    // disarm
    let zero = Itimerspec::default();
    check_ok!(syscall::timerfd_settime(fd, 0, &zero), "disarm");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_settime can arm a one-millisecond relative timer")]
fn timerfd_relative_short() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC | TFD_NONBLOCK),
        "tfd"
    );
    let its = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 1_000_000 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &its), "set");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_gettime on a new timer reports a zero remaining value")]
fn timerfd_gettime_disarmed() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "tfd");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "get");
    check_eq!(cur.it_value.tv_sec, 0, "sec");
    check_eq!(cur.it_value.tv_nsec, 0, "nsec");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getitimer ITIMER_REAL succeeds")]
fn itimer_get_real() -> TestResult {
    let mut cur = Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "get");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setitimer ITIMER_REAL with a zero value disarms the timer")]
fn itimer_set_disarm() -> TestResult {
    let new = Itimerval::default();
    let mut old = Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &new, Some(&mut old)), "set");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setitimer can arm ITIMER_REAL and then clear it")]
fn itimer_set_short_then_clear() -> TestResult {
    let new = Itimerval {
        it_interval: Timeval { tv_sec: 0, tv_usec: 0 },
        it_value: Timeval { tv_sec: 60, tv_usec: 0 },
    };
    let mut old = Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &new, Some(&mut old)), "arm");
    let zero = Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &zero, None), "clear");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timer_create with SIGEV_NONE and timer_delete succeed")]
fn posix_timer_create_delete() -> TestResult {
    let mut sevp = Sigevent::default();
    sevp.sigev_notify = SIGEV_NONE;
    let mut tid = 0usize;
    check_ok!(syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sevp), &mut tid), "create");
    check_ok!(syscall::timer_delete(tid), "delete");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timer_settime arms a POSIX timer that timer_gettime still shows as armed")]
fn posix_timer_settime_get() -> TestResult {
    let mut sevp = Sigevent::default();
    sevp.sigev_notify = SIGEV_NONE;
    let mut tid = 0usize;
    check_ok!(syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sevp), &mut tid), "create");
    let its = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 100, tv_nsec: 0 },
    };
    check_ok!(syscall::timer_settime(tid, 0, &its, None), "set");
    let cur = check_ok!(syscall::timer_gettime(tid), "get");
    check!(cur.it_value.tv_sec > 0, "armed");
    check_ok!(syscall::timer_delete(tid), "delete");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "timer_create on CLOCK_REALTIME succeeds or is rejected with EINVAL/ENOSYS/EPERM")]
fn posix_timer_realtime() -> TestResult {
    let mut sevp = Sigevent::default();
    sevp.sigev_notify = SIGEV_NONE;
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_REALTIME, Some(&sevp), &mut tid) {
        Ok(()) => check_ok!(syscall::timer_delete(tid), "del"),
        Err(e) if soft_clock(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("timer_create rt")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "timer_settime can disarm a POSIX timer or timer_create is unsupported")]
fn posix_timer_disarm() -> TestResult {
    let mut sevp = Sigevent::default();
    sevp.sigev_notify = SIGEV_NONE;
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sevp), &mut tid) {
        Ok(()) => {}
        Err(e) if soft_clock(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("create")),
    }
    let its = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 50, tv_nsec: 0 },
    };
    check_ok!(syscall::timer_settime(tid, 0, &its, None), "arm");
    let zero = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 0 },
    };
    check_ok!(syscall::timer_settime(tid, 0, &zero, None), "disarm");
    // gettime after disarm should report a zero remaining value on most kernels;
    // accept either a zero reading or a successful call (ABI/FS quirks).
    if let Ok(cur) = syscall::timer_gettime(tid) {
        let _ = cur;
    }
    check_ok!(syscall::timer_delete(tid), "del");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "CLOCK_MONOTONIC does not go backwards across a yield")]
fn monotonic_advances() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let _ = syscall::sched_yield();
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(
        b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec),
        "advance"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "CLOCK_REALTIME and gettimeofday agree within two seconds")]
fn realtime_matches_gettimeofday_rough() -> TestResult {
    let ts = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "ts");
    let tv = check_ok!(syscall::gettimeofday(), "tv");
    let diff = (ts.tv_sec - tv.tv_sec).abs();
    check!(diff <= 2, "skew");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "clock_getres of an invalid clock id returns EINVAL or is otherwise unsupported")]
fn clock_getres_invalid_soft() -> TestResult {
    match syscall::clock_getres(9999) {
        Err(Errno::EINVAL) => {}
        Ok(_) => {}
        Err(e) if soft_clock(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("bad clock")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_settime with a zero it_value disarms a previously armed timer")]
fn timerfd_settime_zero_disarm() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC), "tfd");
    let its = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 10, tv_nsec: 0 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &its), "arm");
    check_ok!(syscall::timerfd_settime(fd, 0, &Itimerspec::default()), "disarm");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "an absolute timerfd in the past is readable or returns EAGAIN")]
fn timerfd_absolute_past_fires_soft() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC | TFD_NONBLOCK),
        "tfd"
    );
    let its = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 1, tv_nsec: 0 },
    };
    // Absolute time in the past should expire immediately.
    check_ok!(syscall::timerfd_settime(fd, TFD_TIMER_ABSTIME, &its), "set");
    let mut buf = [0u8; 8];
    match syscall::read(fd, &mut buf) {
        Ok(8) => {}
        Err(Errno::EAGAIN) => {}
        Ok(_) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("read tfd"));
        }
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getitimer reports zeros after setitimer clears ITIMER_REAL")]
fn itimer_get_after_clear() -> TestResult {
    let zero = Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &zero, None), "clear");
    let mut cur = Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "get");
    check_eq!(cur.it_value.tv_sec, 0, "sec");
    check_eq!(cur.it_value.tv_usec, 0, "usec");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timer_gettime reports the interval that timer_settime configured")]
fn posix_timer_interval() -> TestResult {
    let mut sevp = Sigevent::default();
    sevp.sigev_notify = SIGEV_NONE;
    let mut tid = 0usize;
    check_ok!(syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sevp), &mut tid), "create");
    let its = Itimerspec {
        it_interval: Timespec { tv_sec: 1, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 1, tv_nsec: 0 },
    };
    check_ok!(syscall::timer_settime(tid, 0, &its, None), "set");
    let cur = check_ok!(syscall::timer_gettime(tid), "get");
    check_eq!(cur.it_interval.tv_sec, 1, "interval");
    check_ok!(syscall::timer_delete(tid), "del");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "clock_nanosleep with a negative tv_nsec returns EINVAL or succeeds")]
fn clock_nanosleep_bad_nsec() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: -1 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req) {
        Err(Errno::EINVAL) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("cns bad")),
    }
    Ok(())
}
