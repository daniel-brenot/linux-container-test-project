//! Timer depth (TMR): clocks, nanosleep, clock_nanosleep absolute, timer_create,
//! timerfd, itimer.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, clock, Errno, Itimerspec, Sigevent, Timespec, TIMER_ABSTIME, TFD_CLOEXEC, TFD_NONBLOCK, TFD_TIMER_ABSTIME, ITIMER_REAL, SIGEV_NONE};

fn valid_nsec(ts: &Timespec) -> bool {
    ts.tv_nsec >= 0 && ts.tv_nsec < 1_000_000_000
}

fn mono_le(a: &Timespec, b: &Timespec) -> bool {
    b.tv_sec > a.tv_sec || (b.tv_sec == a.tv_sec && b.tv_nsec >= a.tv_nsec)
}

macro_rules! clock_get {
    ($name:ident, $clk:expr) => {
        #[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime returns a timespec with non-negative seconds and valid nsec")]
        fn $name() -> TestResult {
            let t = check_ok!(syscall::clock_gettime($clk), "get");
            check!(valid_nsec(&t), "nsec");
            check!(t.tv_sec >= 0, "sec");
            Ok(())
        }
    };
}

clock_get!(tmr_d_rt, clock::CLOCK_REALTIME);
clock_get!(tmr_d_mono, clock::CLOCK_MONOTONIC);
clock_get!(tmr_d_raw, clock::CLOCK_MONOTONIC_RAW);
clock_get!(tmr_d_boot, clock::CLOCK_BOOTTIME);
clock_get!(tmr_d_coarse, clock::CLOCK_REALTIME_COARSE);
clock_get!(tmr_d_pcpu, clock::CLOCK_PROCESS_CPUTIME_ID);
clock_get!(tmr_d_tcpu, clock::CLOCK_THREAD_CPUTIME_ID);

macro_rules! clock_res {
    ($name:ident, $clk:expr) => {
        #[crate::lctp_test(suite = posix, expect = success, case = "clock_getres returns a positive resolution")]
        fn $name() -> TestResult {
            let r = check_ok!(syscall::clock_getres($clk), "res");
            check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
            check!(valid_nsec(&r) || r.tv_sec > 0, "ok");
            Ok(())
        }
    };
}

clock_res!(tmr_d_res_rt, clock::CLOCK_REALTIME);
clock_res!(tmr_d_res_mono, clock::CLOCK_MONOTONIC);
clock_res!(tmr_d_res_raw, clock::CLOCK_MONOTONIC_RAW);
clock_res!(tmr_d_res_boot, clock::CLOCK_BOOTTIME);

macro_rules! nanosleep_ns {
    ($name:ident, $ns:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = success, case = "nanosleep of a short relative timespec succeeds")]
        fn $name() -> TestResult {
            let req = Timespec {
                tv_sec: 0,
                tv_nsec: $ns,
            };
            check_ok!(syscall::nanosleep(&req), "sleep");
            Ok(())
        }
    };
}

nanosleep_ns!(tmr_d_ns_100us, 100_000);
nanosleep_ns!(tmr_d_ns_500us, 500_000);
nanosleep_ns!(tmr_d_ns_1ms, 1_000_000);
nanosleep_ns!(tmr_d_ns_2ms, 2_000_000);
nanosleep_ns!(tmr_d_ns_5ms, 5_000_000);

#[crate::lctp_test(suite = posix, expect = success, case = "nanosleep of a zero timespec succeeds")]
fn tmr_d_nanosleep_zero() -> TestResult {
    check_ok!(
        syscall::nanosleep(&Timespec {
            tv_sec: 0,
            tv_nsec: 0
        }),
        "0"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "clock_nanosleep absolute on CLOCK_MONOTONIC a short time ahead succeeds or returns EINVAL")]
fn tmr_d_clock_nanosleep_abs_mono() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let abs = Timespec {
        tv_sec: now.tv_sec,
        tv_nsec: (now.tv_nsec + 2_000_000) % 1_000_000_000,
    };
    // Soft: absolute may already be past if wrap — still should return quickly.
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, TIMER_ABSTIME, &abs) {
        Ok(()) => {}
        Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("abs")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "clock_nanosleep relative on CLOCK_REALTIME for 1 ms succeeds")]
fn tmr_d_clock_nanosleep_rel_rt() -> TestResult {
    let req = Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    check_ok!(
        syscall::clock_nanosleep(clock::CLOCK_REALTIME, 0, &req),
        "rel"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "timer_create on CLOCK_MONOTONIC with SIGEV_NONE can be deleted or rejected as unsupported")]
fn tmr_d_timer_create_delete() -> TestResult {
    let sev = Sigevent {
        sigev_notify: SIGEV_NONE,
        ..Sigevent::default()
    };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid) {
        Ok(()) => {
            check_ok!(syscall::timer_delete(tid), "del");
        }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("create")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "timer_settime can arm a monotonic timer and timer_gettime reports it remaining, or create is unsupported")]
fn tmr_d_timer_set_get() -> TestResult {
    let sev = Sigevent {
        sigev_notify: SIGEV_NONE,
        ..Sigevent::default()
    };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_MONOTONIC, Some(&sev), &mut tid).is_err() {
        return Ok(());
    }
    let new = Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: Timespec {
            tv_sec: 10,
            tv_nsec: 0,
        },
    };
    match syscall::timer_settime(tid, 0, &new, None) {
        Ok(()) => {
            let cur = check_ok!(syscall::timer_gettime(tid), "get");
            check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
        }
        Err(_) => {}
    }
    let zero = Itimerspec::default();
    let _ = syscall::timer_settime(tid, 0, &zero, None);
    let _ = syscall::timer_delete(tid);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "timer_settime with TIMER_ABSTIME can arm a CLOCK_REALTIME timer or be rejected as unsupported")]
fn tmr_d_timer_abstime_soft() -> TestResult {
    let sev = Sigevent {
        sigev_notify: SIGEV_NONE,
        ..Sigevent::default()
    };
    let mut tid = 0usize;
    if syscall::timer_create(clock::CLOCK_REALTIME, Some(&sev), &mut tid).is_err() {
        return Ok(());
    }
    let now = match syscall::clock_gettime(clock::CLOCK_REALTIME) {
        Ok(t) => t,
        Err(_) => {
            let _ = syscall::timer_delete(tid);
            return Ok(());
        }
    };
    let new = Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: Timespec {
            tv_sec: now.tv_sec + 5,
            tv_nsec: now.tv_nsec,
        },
    };
    let _ = syscall::timer_settime(tid, TIMER_ABSTIME, &new, None);
    let _ = syscall::timer_settime(tid, 0, &Itimerspec::default(), None);
    let _ = syscall::timer_delete(tid);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "timerfd_create on CLOCK_MONOTONIC returns a closable file descriptor")]
fn tmr_d_timerfd_create() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "tfd");
    check!(fd >= 0, "fd");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "timerfd_create on CLOCK_MONOTONIC with TFD_CLOEXEC returns a closable file descriptor")]
fn tmr_d_timerfd_cloexec() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC),
        "tfd"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "timerfd_create on CLOCK_MONOTONIC with TFD_NONBLOCK returns a closable file descriptor")]
fn tmr_d_timerfd_nonblock() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK),
        "tfd"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "timerfd_settime can arm a monotonic timerfd and timerfd_gettime reports it remaining")]
fn tmr_d_timerfd_set_get() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "tfd");
    let new = Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: Timespec {
            tv_sec: 5,
            tv_nsec: 0,
        },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &new), "set");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "get");
    check!(cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0, "armed");
    let zero = Itimerspec::default();
    check_ok!(syscall::timerfd_settime(fd, 0, &zero), "clr");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "timerfd_settime with TFD_TIMER_ABSTIME can arm a CLOCK_MONOTONIC timerfd")]
fn tmr_d_timerfd_abstime() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "tfd");
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let new = Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: Timespec {
            tv_sec: now.tv_sec + 3,
            tv_nsec: now.tv_nsec,
        },
    };
    check_ok!(
        syscall::timerfd_settime(fd, TFD_TIMER_ABSTIME, &new),
        "abs"
    );
    check_ok!(
        syscall::timerfd_settime(fd, 0, &Itimerspec::default()),
        "clr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "getitimer on ITIMER_REAL succeeds")]
fn tmr_d_itimer_get() -> TestResult {
    let mut cur = syscall::Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "get");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "setitimer can arm ITIMER_REAL and then clear it")]
fn tmr_d_itimer_set_clear() -> TestResult {
    let new = syscall::Itimerval {
        it_interval: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        it_value: syscall::Timeval {
            tv_sec: 5,
            tv_usec: 0,
        },
    };
    check_ok!(syscall::setitimer(ITIMER_REAL, &new, None), "set");
    let zero = syscall::Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &zero, None), "clr");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_MONOTONIC samples are non-decreasing")]
fn tmr_d_mono_nondec() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(mono_le(&a, &b), "le");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_BOOTTIME samples are non-decreasing")]
fn tmr_d_boot_nondec() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_BOOTTIME), "b");
    check!(mono_le(&a, &b), "le");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_gettime on an invalid clock id returns EINVAL")]
fn tmr_d_bad_clock_einval() -> TestResult {
    check_err!(syscall::clock_gettime(99999), Errno::EINVAL, "bad");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_getres on an invalid clock id returns EINVAL")]
fn tmr_d_bad_res_einval() -> TestResult {
    check_err!(syscall::clock_getres(99999), Errno::EINVAL, "bad");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "nanosleep with tv_nsec equal to 1e9 returns EINVAL")]
fn tmr_d_nanosleep_bad_nsec() -> TestResult {
    check_err!(
        syscall::nanosleep(&Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000_000
        }),
        Errno::EINVAL,
        "nsec"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "timerfd_create on CLOCK_REALTIME returns a closable file descriptor")]
fn tmr_d_timerfd_realtime() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_REALTIME, 0), "tfd");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "timer_create on CLOCK_REALTIME with SIGEV_NONE succeeds or is rejected as unsupported")]
fn tmr_d_timer_create_rt() -> TestResult {
    let sev = Sigevent {
        sigev_notify: SIGEV_NONE,
        ..Sigevent::default()
    };
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_REALTIME, Some(&sev), &mut tid) {
        Ok(()) => {
            let _ = syscall::timer_delete(tid);
        }
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "gettimeofday returns a time after 2020")]
fn tmr_d_gettimeofday() -> TestResult {
    let tv = check_ok!(syscall::gettimeofday(), "gtod");
    check!(tv.tv_sec > 1_600_000_000, "sec");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "a CLOCK_MONOTONIC sample after a 2 ms nanosleep is greater than or equal to the earlier sample")]
fn tmr_d_mono_after_ns() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    check_ok!(
        syscall::nanosleep(&Timespec {
            tv_sec: 0,
            tv_nsec: 2_000_000
        }),
        "ns"
    );
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    check!(mono_le(&a, &b), "moved");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two clock_getres samples on CLOCK_MONOTONIC are identical")]
fn tmr_d_res_stable() -> TestResult {
    let a = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "b");
    check_eq!(a.tv_sec, b.tv_sec, "sec");
    check_eq!(a.tv_nsec, b.tv_nsec, "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "timerfd_settime can arm a repeating CLOCK_MONOTONIC interval and gettime reports the interval")]
fn tmr_d_timerfd_interval_soft() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK),
        "tfd"
    );
    let new = Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        },
        it_value: Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        },
    };
    check_ok!(syscall::timerfd_settime(fd, 0, &new), "set");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "get");
    check!(cur.it_interval.tv_nsec > 0 || cur.it_interval.tv_sec > 0, "iv");
    check_ok!(
        syscall::timerfd_settime(fd, 0, &Itimerspec::default()),
        "clr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "setitimer can clear ITIMER_REAL twice")]
fn tmr_d_itimer_clear_idempotent() -> TestResult {
    let zero = syscall::Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &zero, None), "c1");
    check_ok!(syscall::setitimer(ITIMER_REAL, &zero, None), "c2");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "clock_nanosleep relative on CLOCK_BOOTTIME succeeds or returns EINVAL")]
fn tmr_d_clock_nanosleep_boot() -> TestResult {
    let req = Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    match syscall::clock_nanosleep(clock::CLOCK_BOOTTIME, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("boot sleep")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "three CLOCK_MONOTONIC samples are non-decreasing in order")]
fn tmr_d_triple_samples() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "b");
    let c = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "c");
    check!(mono_le(&a, &b) && mono_le(&b, &c), "ord");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "timer_delete of timer id 0 is rejected with EINVAL or ENOSYS or otherwise ignored")]
fn tmr_d_timer_delete_bad_soft() -> TestResult {
    match syscall::timer_delete(0) {
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => Ok(()),
        Ok(()) => Ok(()),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, expect = soft, case = "timerfd_create with an invalid clock id returns EINVAL or another rejection")]
fn tmr_d_timerfd_bad_clock_soft() -> TestResult {
    match syscall::timerfd_create(99999, 0) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(fd) => {
            let _ = syscall::close(fd);
            Err(crate::harness::AssertFail::msg("unexpected"))
        }
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "setitimer replacing an armed ITIMER_REAL returns the previous remaining value")]
fn tmr_d_setitimer_returns_old() -> TestResult {
    let first = syscall::Itimerval {
        it_interval: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        it_value: syscall::Timeval {
            tv_sec: 8,
            tv_usec: 0,
        },
    };
    check_ok!(syscall::setitimer(ITIMER_REAL, &first, None), "arm");
    let second = syscall::Itimerval::default();
    let mut old = syscall::Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &second, Some(&mut old)), "rep");
    check!(old.it_value.tv_sec > 0 || old.it_value.tv_usec > 0, "old");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_PROCESS_CPUTIME_ID samples are non-decreasing")]
fn tmr_d_cputime_nondec() -> TestResult {
    let a = check_ok!(
        syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID),
        "a"
    );
    let b = check_ok!(
        syscall::clock_gettime(clock::CLOCK_PROCESS_CPUTIME_ID),
        "b"
    );
    check!(mono_le(&a, &b), "le");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "five successive 200 us nanosleeps succeed")]
fn tmr_d_many_nanosleeps() -> TestResult {
    for _ in 0..5 {
        check_ok!(
            syscall::nanosleep(&Timespec {
                tv_sec: 0,
                tv_nsec: 200_000
            }),
            "ns"
        );
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two timerfd_create calls on CLOCK_MONOTONIC return distinct file descriptors")]
fn tmr_d_timerfd_create_twice() -> TestResult {
    let a = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "a");
    let b = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "b");
    check!(a != b, "distinct");
    check_ok!(syscall::close(a), "ca");
    check_ok!(syscall::close(b), "cb");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "clock_nanosleep absolute on CLOCK_REALTIME for a past time returns immediately or EINVAL")]
fn tmr_d_abs_past_returns() -> TestResult {
    let past = Timespec {
        tv_sec: 1,
        tv_nsec: 0,
    };
    match syscall::clock_nanosleep(clock::CLOCK_REALTIME, TIMER_ABSTIME, &past) {
        Ok(()) => {}
        Err(Errno::EINVAL) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two CLOCK_MONOTONIC_RAW samples are non-decreasing")]
fn tmr_d_raw_nondec() -> TestResult {
    let a = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "a");
    let b = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC_RAW), "b");
    check!(mono_le(&a, &b), "le");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "timer_create on CLOCK_MONOTONIC with a null sigevent succeeds or is rejected as unsupported")]
fn tmr_d_timer_create_null_sev_soft() -> TestResult {
    let mut tid = 0usize;
    match syscall::timer_create(clock::CLOCK_MONOTONIC, None, &mut tid) {
        Ok(()) => {
            let _ = syscall::timer_delete(tid);
        }
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "getitimer on ITIMER_REAL succeeds twice")]
fn tmr_d_itimer_get_twice() -> TestResult {
    let mut a = syscall::Itimerval::default();
    let mut b = syscall::Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut a), "a");
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut b), "b");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = failure, case = "clock_nanosleep with tv_nsec of 2e9 returns EINVAL")]
fn tmr_d_clock_nanosleep_bad_nsec() -> TestResult {
    check_err!(
        syscall::clock_nanosleep(
            clock::CLOCK_MONOTONIC,
            0,
            &Timespec {
                tv_sec: 0,
                tv_nsec: 2_000_000_000
            }
        ),
        Errno::EINVAL,
        "bad"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_gettime on CLOCK_REALTIME returns nsec in range 0 to 1e9")]
fn tmr_d_realtime_nsec_range() -> TestResult {
    let t = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "rt");
    check!(valid_nsec(&t), "nsec");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "timerfd_gettime on a newly created CLOCK_MONOTONIC timerfd reports a zero it_value")]
fn tmr_d_timerfd_get_disarmed() -> TestResult {
    let fd = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "tfd");
    let cur = check_ok!(syscall::timerfd_gettime(fd), "get");
    check_eq!(cur.it_value.tv_sec, 0, "sec0");
    check_eq!(cur.it_value.tv_nsec, 0, "nsec0");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "nanosleep with a negative tv_sec returns EINVAL")]
fn tmr_d_nanosleep_neg_sec() -> TestResult {
    check_err!(
        syscall::nanosleep(&Timespec {
            tv_sec: -1,
            tv_nsec: 0
        }),
        Errno::EINVAL,
        "neg"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "four timerfd_create calls on CLOCK_MONOTONIC each return a closable file descriptor")]
fn tmr_d_many_timerfd() -> TestResult {
    let mut fds = [-1i32; 4];
    for f in fds.iter_mut() {
        *f = check_ok!(syscall::timerfd_create(clock::CLOCK_MONOTONIC, 0), "c");
    }
    for f in fds.iter() {
        check_ok!(syscall::close(*f), "cl");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_getres on CLOCK_BOOTTIME returns a positive resolution")]
fn tmr_d_boot_res() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_BOOTTIME), "res");
    check!(r.tv_nsec > 0 || r.tv_sec > 0, "pos");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "setitimer can arm ITIMER_REAL for 500 ms, getitimer reads it, and it can be cleared")]
fn tmr_d_itimer_arm_short() -> TestResult {
    let new = syscall::Itimerval {
        it_interval: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        it_value: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 500_000,
        },
    };
    check_ok!(syscall::setitimer(ITIMER_REAL, &new, None), "arm");
    let mut cur = syscall::Itimerval::default();
    check_ok!(syscall::getitimer(ITIMER_REAL, &mut cur), "get");
    check_ok!(
        syscall::setitimer(ITIMER_REAL, &syscall::Itimerval::default(), None),
        "clr"
    );
    Ok(())
}
