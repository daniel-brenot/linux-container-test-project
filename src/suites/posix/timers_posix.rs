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

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_REALTIME returns a timespec after 2020 with valid nsec")]
fn timers_clock_realtime_get() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "realtime");
    check!(t.tv_sec > 1_600_000_000, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_MONOTONIC returns a timespec with non-negative seconds and valid nsec")]
fn timers_clock_monotonic_get() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "mono");
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_MONOTONIC_RAW returns a timespec with non-negative seconds and valid nsec")]
fn timers_clock_monotonic_raw_get() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "raw");
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_PROCESS_CPUTIME_ID returns a timespec with non-negative seconds and valid nsec")]
fn timers_clock_process_cputime_get() -> TestResult {
    let t = check_ok!(
        syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID),
        "cputime"
    );
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_THREAD_CPUTIME_ID returns a timespec with non-negative seconds and valid nsec")]
fn timers_clock_thread_cputime_get() -> TestResult {
    let t = check_ok!(
        syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID),
        "thread cputime"
    );
    check!(t.tv_sec >= 0, "sec");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_REALTIME returns a sub-second positive nsec resolution")]
fn timers_clock_realtime_getres() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_REALTIME), "getres");
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0 && r.tv_nsec < 1_000_000_000, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_MONOTONIC returns a zero-second positive nsec resolution")]
fn timers_clock_monotonic_getres() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "getres");
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_MONOTONIC_RAW returns a zero-second positive nsec resolution")]
fn timers_clock_monotonic_raw_getres() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW), "getres");
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_PROCESS_CPUTIME_ID returns a zero-second positive nsec resolution")]
fn timers_clock_process_cputime_getres() -> TestResult {
    let r = check_ok!(
        syscall::clock_getres(clock::CLOCK_PROCESS_CPUTIME_ID),
        "getres"
    );
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_THREAD_CPUTIME_ID returns a zero-second positive nsec resolution")]
fn timers_clock_thread_cputime_getres() -> TestResult {
    let r = check_ok!(
        syscall::clock_getres(clock::CLOCK_THREAD_CPUTIME_ID),
        "getres"
    );
    check!(r.tv_sec == 0, "sec0");
    check!(r.tv_nsec > 0, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_getres on an invalid clock id returns EINVAL")]
fn timers_getres_bad_clock_einval() -> TestResult {
    check_err!(syscall::clock_getres(99999), Errno::EINVAL, "bad clock");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_gettime on an invalid clock id returns EINVAL")]
fn timers_gettime_bad_clock_einval() -> TestResult {
    check_err!(syscall::clock_gettime(99999), Errno::EINVAL, "bad clock");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_MONOTONIC samples are non-decreasing")]
fn timers_monotonic_non_decreasing() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(mono_le(&a, &b), "non-decreasing");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_MONOTONIC_RAW samples are non-decreasing")]
fn timers_monotonic_raw_non_decreasing() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "b");
    check!(mono_le(&a, &b), "non-decreasing");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "a CLOCK_REALTIME sample after a short nanosleep is greater than or equal to the earlier sample")]
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

#[crate::lctp_test(suite = posix, full, expect = success, case = "nanosleep of 1 ms returns and CLOCK_MONOTONIC has not gone backwards")]
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

#[crate::lctp_test(suite = posix, full, expect = success, case = "nanosleep of 10 ms succeeds")]
fn timers_nanosleep_10ms() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 10_000_000,
    };
    check_ok!(syscall::nanosleep(&req), "nanosleep");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "nanosleep of a zero timespec succeeds")]
fn timers_nanosleep_zero() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    check_ok!(syscall::nanosleep(&req), "zero");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "nanosleep with tv_nsec equal to 1e9 returns EINVAL")]
fn timers_nanosleep_bad_nsec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000_000,
    };
    check_err!(syscall::nanosleep(&req), Errno::EINVAL, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "nanosleep with a negative tv_nsec returns EINVAL")]
fn timers_nanosleep_neg_nsec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: -1,
    };
    check_err!(syscall::nanosleep(&req), Errno::EINVAL, "neg nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "nanosleep with a negative tv_sec returns EINVAL")]
fn timers_nanosleep_neg_sec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: -1,
        tv_nsec: 0,
    };
    check_err!(syscall::nanosleep(&req), Errno::EINVAL, "neg sec");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "clock_nanosleep relative on CLOCK_MONOTONIC for 1 ms succeeds")]
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

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_nanosleep on an invalid clock id returns EINVAL")]
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

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_nanosleep with tv_nsec of 2e9 returns EINVAL")]
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

#[crate::lctp_test(suite = posix, expect = success, case = "gettimeofday returns a time after 2020 with usec in range 0 to 1000000")]
fn timers_gettimeofday_sec() -> TestResult {
    let tv = check_ok!(syscall::gettimeofday(), "gettimeofday");
    check!(tv.tv_sec > 1_600_000_000, "sec");
    check!(tv.tv_usec >= 0 && tv.tv_usec < 1_000_000, "usec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_REALTIME returns nsec in range 0 to 1e9")]
fn timers_realtime_nsec_in_range() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "rt");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_MONOTONIC returns nsec in range 0 to 1e9")]
fn timers_monotonic_nsec_in_range() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "m");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_REALTIME reports a sub-second resolution")]
fn timers_getres_realtime_le_1s() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_REALTIME), "res");
    check!(r.tv_sec == 0 && r.tv_nsec < 1_000_000_000, "fine");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_MONOTONIC reports a sub-second resolution")]
fn timers_getres_monotonic_le_1s() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "res");
    check!(r.tv_sec == 0 && r.tv_nsec < 1_000_000_000, "fine");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "the CLOCK_MONOTONIC clock advances by at least about 1 ms across a 5 ms nanosleep")]
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

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_PROCESS_CPUTIME_ID samples are non-decreasing")]
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

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_THREAD_CPUTIME_ID samples are non-decreasing")]
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

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_REALTIME samples both have nsec in range")]
fn timers_realtime_two_samples_ok() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "b");
    check!(valid_nsec(&a) && valid_nsec(&b), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two clock_getres samples on CLOCK_MONOTONIC are identical")]
fn timers_getres_twice_stable() -> TestResult {
    let a = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "b");
    check!(a.tv_sec == b.tv_sec && a.tv_nsec == b.tv_nsec, "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "nanosleep of 100 us succeeds")]
fn timers_nanosleep_sub_ms() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 100_000,
    };
    check_ok!(syscall::nanosleep(&req), "100us");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_nanosleep relative on CLOCK_REALTIME for 1 ms succeeds")]
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

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_MONOTONIC_RAW returns a positive resolution")]
fn timers_getres_raw_positive() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW), "res");
    check!(r.tv_nsec > 0 || r.tv_sec > 0, "positive");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_MONOTONIC returns a non-negative tv_sec")]
fn timers_monotonic_sec_nonneg() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "m");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "three CLOCK_MONOTONIC samples are non-decreasing in order")]
fn timers_triple_monotonic_samples() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    let c = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "c");
    check!(mono_le(&a, &b) && mono_le(&b, &c), "ordered");
    Ok(())
}
