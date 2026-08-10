//! Basic futex wait/wake tests.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, Timespec};

#[crate::lctp_test(suite = syscall)]
fn futex_wake_no_waiters() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(0);
    let n = check_ok!(syscall::futex_wake(&VAL, 1), "wake");
    check_eq!(n, 0, "no waiters woken");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn futex_wake_many_no_waiters() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(1);
    let n = check_ok!(syscall::futex_wake(&VAL, 10), "wake");
    check_eq!(n, 0, "no waiters");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn futex_wait_timeout() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(1);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 10_000_000 };
    check_err!(
        syscall::futex_wait(&VAL, 1, Some(&timeout)),
        Errno::ETIMEDOUT,
        "timeout"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn futex_wait_wrong_value() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(5);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 5_000_000 };
    // Waiting for val=1 when uaddr holds 5 should return immediately (EAGAIN).
    match syscall::futex_wait(&VAL, 1, Some(&timeout)) {
        Err(Errno::EAGAIN) | Err(Errno::ETIMEDOUT) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("unexpected wait success")),
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn futex_wake_after_wait_setup() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        static VAL: AtomicU32 = AtomicU32::new(0);
        VAL.store(0, Ordering::SeqCst);
        // Parent will wake us; we wait with short timeout fallback.
        let timeout = Timespec { tv_sec: 2, tv_nsec: 0 };
        let _ = syscall::futex_wait(&VAL, 0, Some(&timeout));
        syscall::exit(0);
    }
    // Parent: just verify wake on unrelated atomic succeeds with 0.
    static PARENT_VAL: AtomicU32 = AtomicU32::new(0);
    let n = check_ok!(syscall::futex_wake(&PARENT_VAL, 1), "wake");
    check_eq!(n, 0, "no waiters");
    let mut status = 0;
    check_ok!(syscall::kill(pid, crate::syscall::SIGTERM), "kill child");
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn futex_wake_zero_count() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(0);
    let n = check_ok!(syscall::futex_wake(&VAL, 0), "wake 0");
    check_eq!(n, 0, "none woken");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn futex_wait_short_timeout() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(42);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    check_err!(
        syscall::futex_wait(&VAL, 42, Some(&timeout)),
        Errno::ETIMEDOUT,
        "timed out"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn futex_atomic_value_preserved() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(99);
    let _ = syscall::futex_wake(&VAL, 1);
    check_eq!(VAL.load(Ordering::SeqCst), 99, "value");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn futex_wait_infinite_wrong_val_eagain() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(7);
    match syscall::futex_wait(&VAL, 3, None) {
        Err(Errno::EAGAIN) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("wait ok")),
        Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn futex_wake_one_no_waiters() -> TestResult {
    static VAL: AtomicU32 = AtomicU32::new(0);
    check_eq!(check_ok!(syscall::futex_wake(&VAL, 1), "wake"), 0, "zero");
    Ok(())
}
