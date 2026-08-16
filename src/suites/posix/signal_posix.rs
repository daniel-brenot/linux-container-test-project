//! POSIX signal delivery semantics (no pthreads).

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, sigmask, SIGCHLD, SIGINT, SIGTERM, SIGUSR1, SIGUSR2, SIG_BLOCK, SIG_DFL, SIG_IGN,
    SIG_SETMASK, SIG_UNBLOCK,
};

fn discard_pending(sig: i32) -> TestResult {
    check_ok!(syscall::signal_ignore(sig), "ignore");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(sig)), None),
        "unblock"
    );
    check_ok!(syscall::signal_default(sig), "default");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn signal_kill_self_zero() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "kill 0");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "kill of a child with SIGTERM is reported as a signaled wait status")]
fn signal_child_sigterm_reap() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 120,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGTERM, "sig");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "kill of a child with SIGINT is reported as a signaled wait status")]
fn signal_child_sigint_reap() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 120,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGINT), "kill INT");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGINT, "sig");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "wtermsig of a SIGTERM-killed child equals SIGTERM")]
fn signal_wtermsig_matches() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 120,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wtermsig(status), SIGTERM, "wtermsig");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "a child that calls exit is reported as exited and not signaled")]
fn signal_exit_not_signaled() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(3);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    check!(!syscall::wifsignaled(status), "not signaled");
    check_eq!(syscall::wexitstatus(status), 3, "code");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds twice")]
fn signal_kill_self_zero_twice() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "k1");
    check_ok!(syscall::kill(syscall::getpid(), 0), "k2");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 and then unblock it")]
fn signal_sigprocmask_block_unblock() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "unblock"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can save the mask, block SIGUSR1, and restore the old mask")]
fn signal_sigprocmask_setmask_restore() -> TestResult {
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut old)),
        "save"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None),
        "restore"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn signal_sigpending_after_block_kill() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "USR1 pending");
    discard_pending(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "an ignored pending SIGUSR1 is discarded when unblocked without terminating the process")]
fn signal_ignore_sigusr1_unblock_pending() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    // SIG_IGN before unblock so pending delivery does not terminate.
    check_ok!(syscall::signal_ignore(SIGUSR1), "IGN");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "unblock"
    );
    check_ok!(syscall::signal_default(SIGUSR1), "DFL");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "an ignored pending SIGUSR2 is discarded when unblocked without terminating the process")]
fn signal_ignore_sigusr2_unblock_pending() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    check_ok!(syscall::signal_ignore(SIGUSR2), "IGN");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None),
        "unblock"
    );
    check_ok!(syscall::signal_default(SIGUSR2), "DFL");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1, then SIG_DFL, then restore the previous action")]
fn signal_sigaction_ignore_restore() -> TestResult {
    let mut old = syscall::Sigaction::default();
    let ign = syscall::Sigaction {
        sa_handler: SIG_IGN,
        ..syscall::Sigaction::default()
    };
    check_ok!(
        syscall::rt_sigaction(SIGUSR1, Some(&ign), Some(&mut old)),
        "set IGN"
    );
    let dfl = syscall::Sigaction {
        sa_handler: SIG_DFL,
        ..syscall::Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "set DFL");
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can query the current action for SIGUSR1")]
fn signal_sigaction_query() -> TestResult {
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "wait reaps an exited child after SIGCHLD is blocked")]
fn signal_child_death_sigchld_wait() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGCHLD)), None),
        "block CHLD"
    );
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    // Soft: SIGCHLD may or may not still be pending after wait reaped it.
    let _ = pending & sigmask(SIGCHLD);
    check_ok!(syscall::signal_ignore(SIGCHLD), "IGN CHLD");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGCHLD)), None),
        "unblock"
    );
    check_ok!(syscall::signal_default(SIGCHLD), "DFL");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "a child that ignores SIGUSR1 is not terminated by SIGUSR1 and can still be killed with SIGTERM")]
fn signal_kill_child_sigusr1_ignored() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::signal_ignore(SIGUSR1);
        let req = syscall::Timespec {
            tv_sec: 120,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    // Give child a moment to install IGN.
    let pause = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let _ = syscall::nanosleep(&pause);
    check_ok!(syscall::kill(pid, SIGUSR1), "kill USR1");
    check_ok!(syscall::kill(pid, SIGTERM), "kill TERM");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "signaled");
    check_eq!(syscall::wtermsig(status), SIGTERM, "TERM");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn signal_sigprocmask_query_null_set() -> TestResult {
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut old)),
        "query"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigpending succeeds and fills a pending-signal mask")]
fn signal_sigpending_succeeds() -> TestResult {
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "blocked SIGUSR1 and SIGUSR2 sent to self both appear in rt_sigpending")]
fn signal_block_both_usr_pending() -> TestResult {
    let mask = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(mask), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "u1");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "u2");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "usr1");
    check!(pending & sigmask(SIGUSR2) != 0, "usr2");
    check_ok!(syscall::signal_ignore(SIGUSR1), "ign1");
    check_ok!(syscall::signal_ignore(SIGUSR2), "ign2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(mask), None), "un");
    check_ok!(syscall::signal_default(SIGUSR1), "d1");
    check_ok!(syscall::signal_default(SIGUSR2), "d2");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN then SIG_DFL on SIGUSR2")]
fn signal_sigaction_ign_then_dfl_usr2() -> TestResult {
    let ign = syscall::Sigaction {
        sa_handler: SIG_IGN,
        ..syscall::Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let dfl = syscall::Sigaction {
        sa_handler: SIG_DFL,
        ..syscall::Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "waitpid reports a child that exits 0 as exited with status 0")]
fn signal_child_exit_zero_wait() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::waitpid(pid, &mut status, 0), "waitpid");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "0");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds as an existence check")]
fn signal_kill_zero_other_self() -> TestResult {
    // kill(pid, 0) existence check on self.
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "with SIGCHLD ignored, waitpid of an exited child succeeds or returns ECHILD")]
fn signal_sigchld_ign_then_fork_wait() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGCHLD), "IGN");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    // With SIG_IGN, child may be auto-reaped; wait may return ECHILD — soft.
    let mut status = 0;
    match syscall::waitpid(pid, &mut status, 0) {
        Ok(_) => {}
        Err(crate::syscall::Errno::ECHILD) => {}
        Err(_) => {
            let _ = syscall::signal_default(SIGCHLD);
            return Err(crate::harness::AssertFail::msg("wait"));
        }
    }
    check_ok!(syscall::signal_default(SIGCHLD), "DFL");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can clear the signal mask")]
fn signal_mask_clear() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(0), None), "clear");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1 and read it back")]
fn signal_rt_sigaction_roundtrip_handler() -> TestResult {
    let mut old = syscall::Sigaction::default();
    let ign = syscall::Sigaction {
        sa_handler: SIG_IGN,
        ..syscall::Sigaction::default()
    };
    check_ok!(
        syscall::rt_sigaction(SIGUSR1, Some(&ign), Some(&mut old)),
        "set"
    );
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "get");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "a blocked SIGUSR1 sent to self does not terminate the process")]
fn signal_block_sigusr1_survives_kill() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    discard_pending(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "a SIGTERM-killed child is reported as signaled and not exited")]
fn signal_child_sigterm_wexitstatus_not_used() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "kill");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(!syscall::wifexited(status), "not exited");
    check!(syscall::wifsignaled(status), "signaled");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "unblocking an ignored pending SIGUSR1 clears it from rt_sigpending")]
fn signal_pending_cleared_after_ign_unblock() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    check_ok!(syscall::signal_ignore(SIGUSR1), "IGN");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "unblock"
    );
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) == 0, "cleared");
    check_ok!(syscall::signal_default(SIGUSR1), "DFL");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn signal_sigprocmask_block_twice() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "b1"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "b2"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "u"
    );
    Ok(())
}
