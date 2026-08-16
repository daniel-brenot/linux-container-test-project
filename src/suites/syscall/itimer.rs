//! setitimer / getitimer tests.

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Itimerval, Timeval, Timespec, ITIMER_REAL, SIGALRM};

fn disarm() -> Result<(), crate::harness::AssertFail> {
    let zero = Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &zero, None), "disarm");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getitimer ITIMER_REAL reports zeros when the timer is disarmed")]
fn getitimer_disarmed_zero() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGALRM), "SIG_IGN");
    disarm()?;
    let mut cur = Itimerval {
        it_interval: Timeval {
            tv_sec: 1,
            tv_usec: 1,
        },
        it_value: Timeval {
            tv_sec: 1,
            tv_usec: 1,
        },
    };
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "getitimer");
    check!(cur.it_value.tv_sec == 0 && cur.it_value.tv_usec == 0, "value zero");
    check!(
        cur.it_interval.tv_sec == 0 && cur.it_interval.tv_usec == 0,
        "interval zero"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getitimer reports a positive remaining time after arming ITIMER_REAL")]
fn setitimer_real_get_remaining() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGALRM), "SIG_IGN");
    let new = Itimerval {
        it_interval: Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        it_value: Timeval {
            tv_sec: 0,
            tv_usec: 500_000,
        },
    };
    check_ok!(syscall::setitimer(ITIMER_REAL, &new, None), "setitimer");
    let mut cur = Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "getitimer");
    // Remaining time should be positive and not exceed the requested value.
    let rem_us = cur.it_value.tv_sec * 1_000_000 + cur.it_value.tv_usec;
    check!(rem_us > 0, "still armed");
    check!(rem_us <= 500_000, "not overshot");
    disarm()?;
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setitimer returns the previous remaining ITIMER_REAL value")]
fn setitimer_returns_old() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGALRM), "SIG_IGN");
    disarm()?;
    let first = Itimerval {
        it_interval: Timeval::default(),
        it_value: Timeval {
            tv_sec: 1,
            tv_usec: 0,
        },
    };
    check_ok!(syscall::setitimer(ITIMER_REAL, &first, None), "arm");
    let second = Itimerval {
        it_interval: Timeval::default(),
        it_value: Timeval {
            tv_sec: 2,
            tv_usec: 0,
        },
    };
    let mut old = Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &second, Some(&mut old)), "replace");
    let old_us = old.it_value.tv_sec * 1_000_000 + old.it_value.tv_usec;
    check!(old_us > 0, "old remaining");
    check!(old_us <= 1_000_000, "old <= 1s");
    disarm()?;
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "setitimer retains the configured ITIMER_REAL interval")]
fn setitimer_interval_retained() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGALRM), "SIG_IGN");
    let new = Itimerval {
        it_interval: Timeval {
            tv_sec: 0,
            tv_usec: 200_000,
        },
        it_value: Timeval {
            tv_sec: 0,
            tv_usec: 500_000,
        },
    };
    check_ok!(syscall::setitimer(ITIMER_REAL, &new, None), "set");
    let mut cur = Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "get");
    check!(
        cur.it_interval.tv_sec == 0 && cur.it_interval.tv_usec == 200_000,
        "interval retained"
    );
    let rem_us = cur.it_value.tv_sec * 1_000_000 + cur.it_value.tv_usec;
    check!(rem_us > 0 && rem_us <= 500_000, "still armed");
    // Brief sleep; timer should remain armed (interval or remaining value).
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let _ = syscall::nanosleep(&ts);
    let mut after = Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut after), "get after");
    check!(
        after.it_interval.tv_usec == 200_000,
        "interval after sleep"
    );
    let after_us = after.it_value.tv_sec * 1_000_000 + after.it_value.tv_usec;
    check!(after_us > 0, "still counting");
    disarm()?;
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setitimer with a zero value clears ITIMER_REAL")]
fn setitimer_clear() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGALRM), "SIG_IGN");
    let armed = Itimerval {
        it_interval: Timeval::default(),
        it_value: Timeval {
            tv_sec: 5,
            tv_usec: 0,
        },
    };
    check_ok!(syscall::setitimer(ITIMER_REAL, &armed, None), "arm");
    disarm()?;
    let mut cur = Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "get");
    check!(cur.it_value.tv_sec == 0 && cur.it_value.tv_usec == 0, "cleared");
    Ok(())
}
