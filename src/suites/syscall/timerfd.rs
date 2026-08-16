//! timerfd syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, clock, fcntl_cmd, oflag, FD_CLOEXEC, Itimerspec, TFD_CLOEXEC, Timespec};

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_create with CLOCK_MONOTONIC succeeds")]
fn timerfd_create_monotonic() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "create");
    check!(fd >= 0, "bad fd");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_create with CLOCK_REALTIME succeeds")]
fn timerfd_create_realtime() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_REALTIME, 0), "create");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_create with TFD_CLOEXEC sets FD_CLOEXEC")]
fn timerfd_create_cloexec() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC),
        "create cloexec"
    );
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(fl & FD_CLOEXEC as usize != 0, "cloexec not set");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_settime can arm a one-second relative timer")]
fn timerfd_settime_relative() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "create");
    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 1, tv_nsec: 0 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &new_val), "settime");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_gettime reports a nonzero remaining value after arming")]
fn timerfd_gettime_after_set() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "create");
    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 10, tv_nsec: 0 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &new_val), "settime");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "gettime");
    check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "zero remaining");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_gettime on a new timer reports a zero remaining value")]
fn timerfd_gettime_disarmed() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "create");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "gettime");
    check_eq!(cur.it_value.tv_sec, 0, "disarmed sec");
    check_eq!(cur.it_value.tv_nsec, 0, "disarmed nsec");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "read after a short timerfd expiry returns an expiration count")]
fn timerfd_read_expiry() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, oflag::O_NONBLOCK),
        "create nb"
    );
    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 10_000_000 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &new_val), "settime");
    let req = Timespec { tv_sec: 0, tv_nsec: 50_000_000 };
    check_ok!(syscall::nanosleep(&req), "sleep");
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::read(fd, &mut buf), "read expiry");
    check_eq!(n, 8, "expiry size");
    let exp = u64::from_ne_bytes(buf);
    check!(exp >= 1, "expirations");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "timerfd_settime with a repeating interval is visible via timerfd_gettime")]
fn timerfd_short_interval() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "create");
    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 5_000_000 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 5_000_000 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &new_val), "settime");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "gettime");
    check!(cur.it_interval.tv_nsec > 0, "interval set");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_settime with a zero it_value disarms the timer")]
fn timerfd_disarm_with_zero() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "create");
    let arm = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 60, tv_nsec: 0 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &arm), "arm");
    let disarm = Itimerspec::default();
    check_ok!(syscall::timerfd_settime(fd, 0, &disarm), "disarm");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "gettime");
    check_eq!(cur.it_value.tv_sec, 0, "disarmed");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "timerfd_create with TFD_CLOEXEC and O_NONBLOCK sets FD_CLOEXEC")]
fn timerfd_create_cloexec_nonblock() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC | oflag::O_NONBLOCK),
        "create"
    );
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(fl & FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "timerfd_settime can arm a 100-millisecond relative timer")]
fn timerfd_settime_short_relative() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "create");
    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 100_000_000 },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &new_val), "settime");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
