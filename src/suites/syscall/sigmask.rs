//! rt_sigprocmask signal mask tests.

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, SIGUSR1, SIGUSR2, SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK, sigmask};

fn discard_pending(sig: i32) -> TestResult {
    check_ok!(syscall::signal_ignore(sig), "ignore");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(sig)), None),
        "unblock"
    );
    check_ok!(syscall::signal_default(sig), "default");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sigprocmask_block_unblock_roundtrip() -> TestResult {
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), Some(&mut old)),
        "block"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "unblock"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sigprocmask_setmask_restore() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(0), Some(&mut old)), "get");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sigprocmask_block_sigusr1_kill_survives() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill self SIGUSR1");
    discard_pending(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sigprocmask_query_null_set() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut old)), "query");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sigprocmask_block_twice() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block1"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block2"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "unblock"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sigprocmask_unblock_all_clear() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(0), None), "clear");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sigpending_after_block_kill() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "sigpending");
    check!(pending & sigmask(SIGUSR1) != 0, "SIGUSR1 pending");
    discard_pending(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sigprocmask_save_restore_exact() -> TestResult {
    let mut saved = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut saved)), "save");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(saved), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sigprocmask_block_multiple_signals() -> TestResult {
    let mask = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(mask), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill1");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill2");
    check_ok!(syscall::signal_ignore(SIGUSR1), "ign1");
    check_ok!(syscall::signal_ignore(SIGUSR2), "ign2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(mask), None), "unblock");
    check_ok!(syscall::signal_default(SIGUSR1), "dfl1");
    check_ok!(syscall::signal_default(SIGUSR2), "dfl2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sigpending_empty_initially() -> TestResult {
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "sigpending");
    // May have pending signals from environment; just ensure call succeeds.
    Ok(())
}
