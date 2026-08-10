//! Timer conformance grids: clock_*/timer_*/nanosleep.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, clock, Errno, Itimerspec, Sigevent, Timespec, TIMER_ABSTIME, TFD_CLOEXEC, TFD_NONBLOCK, SIGEV_NONE};

fn valid_nsec(ts: &Timespec) -> bool { ts.tv_nsec >= 0 && ts.tv_nsec < 1_000_000_000 }

#[crate::lctp_test(suite = posix)]
fn timc_gettime_rt_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_rt_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_rt_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_rt_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_rt_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_rt_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_rt_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_rt_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_rt_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_rt_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mono_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mono_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mono_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mono_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mono_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mono_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mono_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mono_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mono_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mono_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_raw_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_raw_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_raw_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_raw_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_raw_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_raw_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_raw_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_raw_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_raw_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_raw_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_RAW) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_boot_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_boot_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_boot_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_boot_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_boot_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_boot_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_boot_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_BOOTTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_boot_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_BOOTTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_boot_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_BOOTTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_boot_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_BOOTTIME) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_coarse_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_coarse_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_coarse_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_coarse_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_coarse_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_coarse_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_coarse_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_coarse_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_coarse_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_coarse_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_REALTIME_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mcoarse_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mcoarse_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mcoarse_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mcoarse_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mcoarse_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_mcoarse_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_COARSE), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mcoarse_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mcoarse_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mcoarse_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_mcoarse_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_MONOTONIC_COARSE) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_pcpu_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_pcpu_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_pcpu_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_pcpu_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_pcpu_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_pcpu_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_pcpu_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_PROCESS_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_pcpu_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_PROCESS_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_pcpu_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_PROCESS_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_pcpu_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_PROCESS_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_tcpu_1() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_tcpu_2() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_tcpu_3() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_tcpu_4() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_tcpu_5() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_gettime_tcpu_6() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID), "get");
    check!(valid_nsec(&t), "nsec");
    check!(t.tv_sec >= 0, "sec");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_tcpu_1() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_THREAD_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_tcpu_2() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_THREAD_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_tcpu_3() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_THREAD_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_getres_tcpu_4() -> TestResult {
    match syscall::clock_getres(clock::CLOCK_THREAD_CPUTIME_ID) {
        Ok(r) => {
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("res")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_nanosleep_1() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 0 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_nanosleep_2() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_nanosleep_3() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 5000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_nanosleep_4() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 10000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_nanosleep_5() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 50000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_6() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 100000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_7() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 250000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_8() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 500000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_9() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 750000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_10() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1000000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_11() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1500000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_12() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 2000000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_13() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 3000000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_14() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 5000000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_15() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 8000000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_nanosleep_16() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 10000000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_rt100000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 100000 };
    match syscall::clock_nanosleep(clock::CLOCK_REALTIME, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_rt500000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 500000 };
    match syscall::clock_nanosleep(clock::CLOCK_REALTIME, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_rt1000000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1000000 };
    match syscall::clock_nanosleep(clock::CLOCK_REALTIME, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_rt2000000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 2000000 };
    match syscall::clock_nanosleep(clock::CLOCK_REALTIME, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_mono100000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 100000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_mono500000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 500000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_mono1000000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 1000000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_rel_mono2000000() -> TestResult {
    let req = Timespec { tv_sec: 0, tv_nsec: 2000000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("cns")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_1() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_2() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_3() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_4() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_5() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_6() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_7() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_cns_abs_mono_8() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec { tv_sec: now.tv_sec, tv_nsec: (now.tv_nsec + 1_000_000) % 1_000_000_000 };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("abs")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_1() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_2() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_3() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_4() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_5() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_6() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_7() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_8() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_9() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_10() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_11() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timer_create_del_12() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => { check_ok!(syscall::timer_delete(tid), "del"); }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("create")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_1() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 1, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_2() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 2, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_3() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 3, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_4() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 4, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_5() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 5, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_6() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 6, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_7() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 7, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_8() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 8, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_9() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 9, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn timc_timer_setget_10() -> TestResult {
    let sev = Sigevent { sigev_notify: SIGEV_NONE, ..Sigevent::default() };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() { return Ok(()); }
    let new = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 10, tv_nsec: 0 },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = match syscall::timer_gettime(tid) {
                Ok(v) => v,
                Err(_) => { let _ = syscall::timer_delete(tid); return Ok(()); }
            };
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let _ = syscall::timer_delete(tid);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_1() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_2() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_3() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_4() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_5() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_6() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_7() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_8() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_9() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_create_10() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_cloexec_1() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_cloexec_2() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_cloexec_3() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_cloexec_4() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_cloexec_5() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_cloexec_6() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_nonblock_1() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_nonblock_2() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_nonblock_3() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_nonblock_4() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_nonblock_5() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_timerfd_nonblock_6() -> TestResult {
    match syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK) {
        Ok(fd) => { check_ok!(syscall::close(fd), "c"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("tfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_1() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_2() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_3() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_4() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_5() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_6() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_7() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn timc_mono_le_8() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec), "le");
    Ok(())
}