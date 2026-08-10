//! POSIX clock_gettime / clock_getres / nanosleep semantics.

use crate::check;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, clock, Errno};

fn valid_nsec(ts: &syscall::Timespec) -> bool {
    ts.tv_nsec >= 0 && ts.tv_nsec < 1_000_000_000
}

fn mono_le(a: &syscall::Timespec, b: &syscall::Timespec) -> bool {
    b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec)
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_realtime_get() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "realtime");
    check!(t.tv_sec > 1_600_000_000, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_monotonic_get() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "mono");
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_monotonic_raw_get() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "raw");
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_process_cputime_get() -> TestResult {
    let t = check_ok!(
        syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID),
        "cputime"
    );
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_thread_cputime_get() -> TestResult {
    let t = check_ok!(
        syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID),
        "thread cputime"
    );
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_realtime_getres() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_REALTIME), "getres");
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0 && r.tv_nsec < 1_000_000_000, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_monotonic_getres() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "getres");
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_monotonic_raw_getres() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW), "getres");
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_process_cputime_getres() -> TestResult {
    let r = check_ok!(
        syscall::clock_getres(clock::CLOCK_PROCESS_CPUTIME_ID),
        "getres"
    );
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_thread_cputime_getres() -> TestResult {
    let r = check_ok!(
        syscall::clock_getres(clock::CLOCK_THREAD_CPUTIME_ID),
        "getres"
    );
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_getres_bad_clock_einval() -> TestResult {
    check_err!(syscall::clock_getres(99999), Errno::EINVAL, "bad clock");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_gettime_bad_clock_einval() -> TestResult {
    check_err!(syscall::clock_gettime(99999), Errno::EINVAL, "bad clock");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_monotonic_non_decreasing() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(mono_le(&a, &b), "non-decreasing");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_monotonic_raw_non_decreasing() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "b");
    check!(mono_le(&a, &b), "non-decreasing");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_realtime_advances_soft() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "a");
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 2_000_000,
    };
    let _ = syscall::nanosleep(&req);
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "b");
    check!(mono_le(&a, &b), "realtime advanced or equal");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn timers_nanosleep_1ms() -> TestResult {
    let before = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "before");
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    check_ok!(syscall::nanosleep(&req), "nanosleep");
    let after = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "after");
    check!(mono_le(&before, &after), "time moved");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn timers_nanosleep_10ms() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 10_000_000,
    };
    check_ok!(syscall::nanosleep(&req), "nanosleep");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_nanosleep_zero() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    check_ok!(syscall::nanosleep(&req), "zero");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_nanosleep_bad_nsec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000_000,
    };
    check_err!(syscall::nanosleep(&req), Errno::EINVAL, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_nanosleep_neg_nsec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: -1,
    };
    check_err!(syscall::nanosleep(&req), Errno::EINVAL, "neg nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_nanosleep_neg_sec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: -1,
        tv_nsec: 0,
    };
    check_err!(syscall::nanosleep(&req), Errno::EINVAL, "neg sec");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn timers_clock_nanosleep_monotonic() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    check_ok!(
        syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req),
        "clock_nanosleep"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_nanosleep_bad_clock() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    check_err!(
        syscall::clock_nanosleep(99999, 0, &req),
        Errno::EINVAL,
        "bad clock"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_nanosleep_bad_nsec() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 2_000_000_000,
    };
    check_err!(
        syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req),
        Errno::EINVAL,
        "bad nsec"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_gettimeofday_sec() -> TestResult {
    let tv = check_ok!(syscall::gettimeofday(), "gettimeofday");
    check!(tv.tv_sec > 1_600_000_000, "sec");
    check!(tv.tv_usec >= 0 && tv.tv_usec < 1_000_000, "usec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_realtime_nsec_in_range() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "rt");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_monotonic_nsec_in_range() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "m");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_getres_realtime_le_1s() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_REALTIME), "res");
    check!(r.tv_sec == 0 && r.tv_nsec < 1_000_000_000, "fine");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_getres_monotonic_le_1s() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "res");
    check!(r.tv_sec == 0 && r.tv_nsec < 1_000_000_000, "fine");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn timers_monotonic_after_sleep() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 5_000_000,
    };
    check_ok!(syscall::nanosleep(&req), "sleep");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    let delta_ns = (b.tv_sec - a.tv_sec) * 1_000_000_000 + (b.tv_nsec - a.tv_nsec);
    check!(delta_ns >= 1_000_000, "at least ~1ms");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_cputime_non_decreasing() -> TestResult {
    let a = check_ok!(
        syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID),
        "a"
    );
    let b = check_ok!(
        syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID),
        "b"
    );
    check!(mono_le(&a, &b), "cputime");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_thread_cputime_non_decreasing() -> TestResult {
    let a = check_ok!(
        syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID),
        "a"
    );
    let b = check_ok!(
        syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID),
        "b"
    );
    check!(mono_le(&a, &b), "thread cputime");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_realtime_two_samples_ok() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "b");
    check!(valid_nsec(&a) && valid_nsec(&b), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_getres_twice_stable() -> TestResult {
    let a = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "b");
    check!(a.tv_sec == b.tv_sec && a.tv_nsec == b.tv_nsec, "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn timers_nanosleep_sub_ms() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 100_000,
    };
    check_ok!(syscall::nanosleep(&req), "100us");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_clock_nanosleep_realtime() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    check_ok!(
        syscall::clock_nanosleep(clock::CLOCK_REALTIME, 0, &req),
        "rt sleep"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_getres_raw_positive() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW), "res");
    check!(r.tv_nsec > 0 || r.tv_sec > 0, "positive");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn timers_monotonic_sec_nonneg() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "m");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn timers_triple_monotonic_samples() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    let c = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "c");
    check!(mono_le(&a, &b) && mono_le(&b, &c), "ordered");
    Ok(())
}
