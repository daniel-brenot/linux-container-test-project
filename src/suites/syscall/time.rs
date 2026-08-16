//! Time-related syscall tests.

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, clock};

fn valid_nsec(ts: &syscall::Timespec) -> bool {
    ts.tv_nsec >= 0 && ts.tv_nsec < 1_000_000_000
}

#[crate::lctp_test(suite = syscall, expect = success, case = "clock_gettime CLOCK_REALTIME returns a plausible wall-clock time")]
fn clock_realtime() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "realtime");
    check!(t.tv_sec > 1_600_000_000, "realtime too small");
    check!(valid_nsec(&t), "bad nsec");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "clock_gettime CLOCK_MONOTONIC returns a non-negative valid timespec")]
fn clock_monotonic() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "monotonic");
    check!(t.tv_sec >= 0, "negative monotonic");
    check!(valid_nsec(&t), "bad nsec");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "clock_gettime CLOCK_MONOTONIC_RAW returns a non-negative valid timespec")]
fn clock_monotonic_raw() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "raw");
    check!(t.tv_sec >= 0, "negative raw");
    check!(valid_nsec(&t), "bad nsec");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "clock_gettime CLOCK_PROCESS_CPUTIME_ID returns a non-negative valid timespec")]
fn clock_process_cputime() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID), "cputime");
    check!(t.tv_sec >= 0, "negative cputime sec");
    check!(valid_nsec(&t), "bad nsec");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "nanosleep of one millisecond succeeds")]
fn nanosleep_short() -> TestResult {
    let req = syscall::Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    check_ok!(syscall::nanosleep(&req), "nanosleep");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "clock_nanosleep on CLOCK_MONOTONIC for two milliseconds succeeds")]
fn clock_nanosleep_monotonic() -> TestResult {
    let req = syscall::Timespec { tv_sec: 0, tv_nsec: 2_000_000 };
    check_ok!(
        syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req),
        "clock_nanosleep"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "gettimeofday returns a plausible tv_sec and an in-range tv_usec")]
fn gettimeofday() -> TestResult {
    let tv = check_ok!(syscall::gettimeofday(), "gettimeofday");
    check!(tv.tv_sec > 1_600_000_000, "tv_sec small");
    check!(tv.tv_usec >= 0 && tv.tv_usec < 1_000_000, "tv_usec range");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "clock_gettime CLOCK_MONOTONIC returns tv_nsec in 0..1e9")]
fn timespec_nsec_range() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "mono");
    check!(valid_nsec(&t), "nsec out of range");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "two CLOCK_MONOTONIC samples are non-decreasing")]
fn monotonic_non_decreasing() -> TestResult {
    let t1 = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "t1");
    let t2 = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "t2");
    check!(
        t2.tv_sec > t1.tv_sec || (t2.tv_sec == t1.tv_sec && t2.tv_nsec >= t1.tv_nsec),
        "monotonic went backwards"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "clock_settime CLOCK_REALTIME succeeds or returns EPERM/EACCES/EINVAL")]
fn clock_settime_realtime_eperm() -> TestResult {
    use crate::syscall::Errno;
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "gettime");
    match syscall::clock_settime(clock::CLOCK_REALTIME, &now) {
        Err(Errno::EPERM) | Err(Errno::EACCES) => Ok(()),
        Ok(()) => Ok(()), // privileged container — allowed
        Err(Errno::EINVAL) => Ok(()),
        Err(_) => Err(crate::harness::AssertFail::msg("clock_settime errno")),
    }
}
