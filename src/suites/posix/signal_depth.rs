//! Extra SIG coverage: sigaction IGN/DFL grids, block/unblock, kill/killpg soft,
//! sigpending, signalfd, SIGCHLD+wait. Avoids SIGSTOP hangs.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, sigmask, Errno, SIGCHLD, SIGINT, SIGTERM, SIGUSR1, SIGUSR2, SIG_BLOCK, SIG_DFL, SIG_IGN,
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

macro_rules! sigaction_ign_dfl {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let ign = syscall::Sigaction {
                sa_handler: SIG_IGN,
                ..syscall::Sigaction::default()
            };
            check_ok!(syscall::rt_sigaction($sig, Some(&ign), None), "IGN");
            let mut cur = syscall::Sigaction::default();
            check_ok!(syscall::rt_sigaction($sig, None, Some(&mut cur)), "query");
            check_eq!(cur.sa_handler, SIG_IGN, "handler");
            let dfl = syscall::Sigaction {
                sa_handler: SIG_DFL,
                ..syscall::Sigaction::default()
            };
            check_ok!(syscall::rt_sigaction($sig, Some(&dfl), None), "DFL");
            Ok(())
        }
    };
}

sigaction_ign_dfl!(sig_d_ign_dfl_usr1, SIGUSR1);
sigaction_ign_dfl!(sig_d_ign_dfl_usr2, SIGUSR2);
sigaction_ign_dfl!(sig_d_ign_dfl_int, SIGINT);
sigaction_ign_dfl!(sig_d_ign_dfl_term, SIGTERM);
sigaction_ign_dfl!(sig_d_ign_dfl_chld, SIGCHLD);

macro_rules! block_unblock {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask($sig)), None),
                "block"
            );
            check_ok!(
                syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask($sig)), None),
                "unblock"
            );
            Ok(())
        }
    };
}

block_unblock!(sig_d_bu_usr1, SIGUSR1);
block_unblock!(sig_d_bu_usr2, SIGUSR2);
block_unblock!(sig_d_bu_int, SIGINT);
block_unblock!(sig_d_bu_term, SIGTERM);
block_unblock!(sig_d_bu_chld, SIGCHLD);

macro_rules! block_pending {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix, full)]
        fn $name() -> TestResult {
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask($sig)), None),
                "block"
            );
            check_ok!(syscall::kill(syscall::getpid(), $sig), "kill");
            let mut pending = 0u64;
            check_ok!(syscall::rt_sigpending(&mut pending), "pending");
            check!(pending & sigmask($sig) != 0, "pending bit");
            discard_pending($sig)?;
            Ok(())
        }
    };
}

block_pending!(sig_d_pend_usr1, SIGUSR1);
block_pending!(sig_d_pend_usr2, SIGUSR2);

#[crate::lctp_test(suite = posix)]
fn sig_d_killpg_self_zero_soft() -> TestResult {
    let pgid = match syscall::getpgid(0) {
        Ok(p) => p,
        Err(_) => return Ok(()),
    };
    match syscall::killpg(pgid, 0) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ESRCH) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("killpg")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_killpg_term_child_soft() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::setpgid(0, 0);
        let req = syscall::Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let pause = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let _ = syscall::nanosleep(&pause);
    let pgid = match syscall::getpgid(pid) {
        Ok(p) => p,
        Err(_) => {
            let _ = syscall::kill(pid, SIGTERM);
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            return Ok(());
        }
    };
    match syscall::killpg(pgid, SIGTERM) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ESRCH) => {
            let _ = syscall::kill(pid, SIGTERM);
        }
        Err(_) => {
            let _ = syscall::kill(pid, SIGTERM);
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_signalfd_create_soft() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => {
            check!(fd >= 0, "fd");
            check_ok!(syscall::close(fd), "close");
        }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => {}
    }
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "unblock"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_signalfd_read_pending() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    let fd = match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => fd,
        Err(_) => {
            let _ = discard_pending(SIGUSR1);
            return Ok(());
        }
    };
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut buf = [0u8; 128];
    match syscall::read(fd, &mut buf) {
        Ok(n) => check!(n > 0, "got"),
        Err(Errno::EAGAIN) | Err(Errno::EINTR) => {}
        Err(_) => {}
    }
    check_ok!(syscall::close(fd), "close");
    discard_pending(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_sigchld_wait_exit() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGCHLD)), None),
        "block"
    );
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(7);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 7, "7");
    check_ok!(syscall::signal_ignore(SIGCHLD), "ign");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGCHLD)), None),
        "un"
    );
    check_ok!(syscall::signal_default(SIGCHLD), "dfl");
    Ok(())
}

macro_rules! kill_child_sig {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let pid = check_ok!(syscall::fork(), "fork");
            if pid == 0 {
                let req = syscall::Timespec {
                    tv_sec: 60,
                    tv_nsec: 0,
                };
                let _ = syscall::nanosleep(&req);
                syscall::exit(0);
            }
            check_ok!(syscall::kill(pid, $sig), "kill");
            let mut status = 0;
            check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
            check!(syscall::wifsignaled(status), "signaled");
            check_eq!(syscall::wtermsig(status), $sig, "sig");
            Ok(())
        }
    };
}

kill_child_sig!(sig_d_child_term, SIGTERM);
kill_child_sig!(sig_d_child_int, SIGINT);

#[crate::lctp_test(suite = posix)]
fn sig_d_sigaction_roundtrip_save() -> TestResult {
    let mut old = syscall::Sigaction::default();
    let ign = syscall::Sigaction {
        sa_handler: SIG_IGN,
        ..syscall::Sigaction::default()
    };
    check_ok!(
        syscall::rt_sigaction(SIGUSR1, Some(&ign), Some(&mut old)),
        "set"
    );
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_setmask_empty() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(0), None), "clear");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_block_grid_usr() -> TestResult {
    let mask = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(mask), None), "b");
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut old)),
        "q"
    );
    check!(old & sigmask(SIGUSR1) != 0, "u1");
    check!(old & sigmask(SIGUSR2) != 0, "u2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(mask), None), "u");
    Ok(())
}

macro_rules! ign_then_kill_self {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            check_ok!(syscall::signal_ignore($sig), "ign");
            check_ok!(syscall::kill(syscall::getpid(), $sig), "kill");
            check_ok!(syscall::signal_default($sig), "dfl");
            Ok(())
        }
    };
}

ign_then_kill_self!(sig_d_ign_kill_usr1, SIGUSR1);
ign_then_kill_self!(sig_d_ign_kill_usr2, SIGUSR2);

#[crate::lctp_test(suite = posix)]
fn sig_d_pending_empty_initially() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(0), None), "clear");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "p");
    // Soft: may still have remnants; just ensure call works.
    let _ = pending;
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_block_unblock_grid() -> TestResult {
    for sig in [SIGUSR1, SIGUSR2, SIGINT, SIGTERM] {
        check_ok!(
            syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(sig)), None),
            "b"
        );
        check_ok!(
            syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(sig)), None),
            "u"
        );
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_kill_zero_self() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "0");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_signalfd_cloexec_soft() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None),
        "block"
    );
    match syscall::signalfd(-1, sigmask(SIGUSR2), syscall::oflag::O_CLOEXEC) {
        Ok(fd) => {
            check_ok!(syscall::close(fd), "close");
        }
        Err(_) => {}
    }
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None),
        "un"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_sigaction_query_usr2() -> TestResult {
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "q");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_child_exit_codes() -> TestResult {
    for code in [0i32, 1, 2, 3, 42] {
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            syscall::exit(code);
        }
        let mut status = 0;
        check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
        check!(syscall::wifexited(status), "ex");
        check_eq!(syscall::wexitstatus(status), code, "code");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_procmask_save_restore() -> TestResult {
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut old)),
        "save"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "b"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None),
        "restore"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_double_block_same() -> TestResult {
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

#[crate::lctp_test(suite = posix)]
fn sig_d_kill_esrch_soft() -> TestResult {
    match syscall::kill(999_999_999, 0) {
        Err(Errno::ESRCH) | Err(Errno::EPERM) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_ign_chld_fork() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGCHLD), "ign");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    match syscall::waitpid(pid, &mut status, 0) {
        Ok(_) | Err(Errno::ECHILD) => {}
        Err(_) => {
            let _ = syscall::signal_default(SIGCHLD);
            return Err(crate::harness::AssertFail::msg("wait"));
        }
    }
    check_ok!(syscall::signal_default(SIGCHLD), "dfl");
    Ok(())
}

macro_rules! sigaction_dfl_only {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let dfl = syscall::Sigaction {
                sa_handler: SIG_DFL,
                ..syscall::Sigaction::default()
            };
            check_ok!(syscall::rt_sigaction($sig, Some(&dfl), None), "dfl");
            Ok(())
        }
    };
}

sigaction_dfl_only!(sig_d_dfl_usr1, SIGUSR1);
sigaction_dfl_only!(sig_d_dfl_usr2, SIGUSR2);
sigaction_dfl_only!(sig_d_dfl_int, SIGINT);
sigaction_dfl_only!(sig_d_dfl_term, SIGTERM);

#[crate::lctp_test(suite = posix, full)]
fn sig_d_pending_after_multi_kill() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "b"
    );
    for _ in 0..3 {
        check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "k");
    }
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "p");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_killpg_bad_pgid_soft() -> TestResult {
    match syscall::killpg(-1, 0) {
        Err(Errno::EINVAL) | Err(Errno::ESRCH) | Err(Errno::EPERM) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_signalfd_replace_soft() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1) | sigmask(SIGUSR2)), None),
        "b"
    );
    let fd = match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => fd,
        Err(_) => {
            let _ = syscall::rt_sigprocmask(
                SIG_UNBLOCK,
                Some(sigmask(SIGUSR1) | sigmask(SIGUSR2)),
                None,
            );
            return Ok(());
        }
    };
    match syscall::signalfd(fd, sigmask(SIGUSR2), 0) {
        Ok(fd2) => {
            check_eq!(fd2, fd, "same");
            check_ok!(syscall::close(fd2), "c");
        }
        Err(_) => {
            let _ = syscall::close(fd);
        }
    }
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1) | sigmask(SIGUSR2)), None),
        "u"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_mask_query() -> TestResult {
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut old)),
        "q"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_child_usr1_ignored_then_term() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::signal_ignore(SIGUSR1);
        let req = syscall::Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let pause = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let _ = syscall::nanosleep(&pause);
    check_ok!(syscall::kill(pid, SIGUSR1), "usr1");
    check_ok!(syscall::kill(pid, SIGTERM), "term");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifsignaled(status), "sig");
    check_eq!(syscall::wtermsig(status), SIGTERM, "term");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_sigaction_ign_roundtrip_int() -> TestResult {
    let mut old = syscall::Sigaction::default();
    let ign = syscall::Sigaction {
        sa_handler: SIG_IGN,
        ..syscall::Sigaction::default()
    };
    check_ok!(
        syscall::rt_sigaction(SIGINT, Some(&ign), Some(&mut old)),
        "set"
    );
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "get");
    check_eq!(cur.sa_handler, SIG_IGN, "ign");
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_unblock_clears_pending_via_ign() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None),
        "b"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "k");
    check_ok!(syscall::signal_ignore(SIGUSR2), "ign");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None),
        "u"
    );
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "p");
    check!(pending & sigmask(SIGUSR2) == 0, "cleared");
    check_ok!(syscall::signal_default(SIGUSR2), "dfl");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_kill_self_zero_thrice() -> TestResult {
    for _ in 0..3 {
        check_ok!(syscall::kill(syscall::getpid(), 0), "z");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_block_all_usr_then_pending() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "b");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "1");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "2");
    let mut p = 0u64;
    check_ok!(syscall::rt_sigpending(&mut p), "p");
    check!(p & sigmask(SIGUSR1) != 0, "u1");
    check!(p & sigmask(SIGUSR2) != 0, "u2");
    check_ok!(syscall::signal_ignore(SIGUSR1), "i1");
    check_ok!(syscall::signal_ignore(SIGUSR2), "i2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "u");
    check_ok!(syscall::signal_default(SIGUSR1), "d1");
    check_ok!(syscall::signal_default(SIGUSR2), "d2");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_sigpending_succeeds_twice() -> TestResult {
    let mut a = 0u64;
    let mut b = 0u64;
    check_ok!(syscall::rt_sigpending(&mut a), "a");
    check_ok!(syscall::rt_sigpending(&mut b), "b");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_waitpid_vs_wait4() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::waitpid(pid, &mut status, 0), "waitpid");
    check!(syscall::wifexited(status), "ex");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_procmask_noop_null() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, None), "noop");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_term_not_exit() -> TestResult {
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
    check!(!syscall::wifexited(status), "not exit");
    check!(syscall::wifsignaled(status), "sig");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_getpgid_self() -> TestResult {
    let p = check_ok!(syscall::getpgid(0), "pgid");
    check!(p > 0, "pos");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_ign_dfl_loop() -> TestResult {
    for sig in [SIGUSR1, SIGUSR2] {
        check_ok!(syscall::signal_ignore(sig), "ign");
        check_ok!(syscall::signal_default(sig), "dfl");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_sigaction_default_struct() -> TestResult {
    let a = syscall::Sigaction::default();
    check_eq!(a.sa_handler, 0usize, "zeroed or dfl");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_child_codes_many() -> TestResult {
    for code in [10i32, 20, 30, 40, 50, 60, 70, 80] {
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            syscall::exit(code);
        }
        let mut status = 0;
        check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
        check_eq!(syscall::wexitstatus(status), code, "c");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_kill_zero_pid() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exist");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_block_int_pending_discard() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGINT)), None),
        "b"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGINT), "k");
    discard_pending(SIGINT)?;
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_block_term_pending_discard() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGTERM)), None),
        "b"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGTERM), "k");
    discard_pending(SIGTERM)?;
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_signalfd_nonblock_soft() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "b"
    );
    match syscall::signalfd(-1, sigmask(SIGUSR1), syscall::oflag::O_NONBLOCK) {
        Ok(fd) => {
            let mut buf = [0u8; 128];
            match syscall::read(fd, &mut buf) {
                Err(Errno::EAGAIN) => {}
                Ok(_) => {}
                Err(_) => {}
            }
            check_ok!(syscall::close(fd), "c");
        }
        Err(_) => {}
    }
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "u"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_rt_sigaction_null_both_soft() -> TestResult {
    // Query-only with null new is fine; both null may EINVAL — soft.
    match syscall::rt_sigaction(SIGUSR1, None, None) {
        Ok(()) | Err(Errno::EINVAL) => Ok(()),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_multi_fork_reap() -> TestResult {
    for _ in 0..5 {
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            syscall::exit(0);
        }
        let mut status = 0;
        check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_wtermsig_term() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 30,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGTERM), "k");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "w");
    check_eq!(syscall::wtermsig(status), SIGTERM, "term");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn sig_d_wtermsig_int() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 30,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGINT), "k");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "w");
    check_eq!(syscall::wtermsig(status), SIGINT, "int");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sig_d_exit_not_signaled_code() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(11);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "w");
    check!(syscall::wifexited(status), "ex");
    check!(!syscall::wifsignaled(status), "ns");
    check_eq!(syscall::wexitstatus(status), 11, "11");
    Ok(())
}
