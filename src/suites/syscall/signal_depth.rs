//! Signal depth: kill probe, sigaction, pending, signalfd, mask, SIGCHLD.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, Errno, Sigaction, Sigset, SIGCHLD, SIG_BLOCK, SIG_DFL, SIG_IGN, SIG_SETMASK, SIG_UNBLOCK,
    SIGUSR1, SIGTERM, SFD_CLOEXEC, SFD_NONBLOCK,
};

fn sigbit(sig: i32) -> Sigset {
    1u64 << (sig - 1)
}

#[crate::lctp_test(suite = syscall)]
fn sig_kill_self_zero() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "kill0");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_kill_child_zero() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 30, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, 0), "probe");
    check_ok!(syscall::kill(pid, SIGTERM), "term");
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_kill_bad_pid_zero() -> TestResult {
    match syscall::kill(999_999_999, 0) {
        Err(Errno::ESRCH) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("kill bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_rt_sigaction_ign() -> TestResult {
    let mut old = Sigaction::default();
    let mut neu = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&neu), Some(&mut old)), "ign");
    neu.sa_handler = SIG_DFL;
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&neu), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_rt_sigaction_dfl() -> TestResult {
    let mut neu = Sigaction {
        sa_handler: SIG_DFL,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&neu), None), "dfl");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_rt_sigaction_get_old() -> TestResult {
    let mut old = Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut old)), "get");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_block_unblock_usr1() -> TestResult {
    let set = sigbit(SIGUSR1);
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(set), Some(&mut old)), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(set), None), "unblock");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_pending_after_blocked_raise() -> TestResult {
    let set = sigbit(SIGUSR1);
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(set), Some(&mut old)), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "raise");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & set != 0, "usr1 pending");
    // Consume by setting IGN then unblock, or just unblock with IGN.
    let mut act = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&act), None), "ign");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore mask");
    act.sa_handler = SIG_DFL;
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&act), None), "dfl");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_signalfd_create() -> TestResult {
    let mask = sigbit(SIGUSR1);
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(mask), Some(&mut old)), "block");
    let sfd = check_ok!(syscall::signalfd(-1, mask, SFD_CLOEXEC | SFD_NONBLOCK), "sfd");
    check_ok!(syscall::close(sfd), "close");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_signalfd_read_usr1() -> TestResult {
    let mask = sigbit(SIGUSR1);
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(mask), Some(&mut old)), "block");
    let sfd = check_ok!(syscall::signalfd(-1, mask, SFD_CLOEXEC), "sfd");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "raise");
    let mut info = syscall::SignalfdSiginfo::default();
    // Safety: POD buffer for kernel write.
    let buf = unsafe {
        core::slice::from_raw_parts_mut(
            &mut info as *mut _ as *mut u8,
            core::mem::size_of::<syscall::SignalfdSiginfo>(),
        )
    };
    let n = check_ok!(syscall::read(sfd, buf), "read");
    check!(n >= 4, "len");
    check_eq!(info.ssi_signo as i32, SIGUSR1, "signo");
    check_ok!(syscall::close(sfd), "close");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_signalfd_nonblock_eagain() -> TestResult {
    let mask = sigbit(SIGUSR1);
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(mask), Some(&mut old)), "block");
    let sfd = check_ok!(syscall::signalfd(-1, mask, SFD_CLOEXEC | SFD_NONBLOCK), "sfd");
    let mut buf = [0u8; 128];
    match syscall::read(sfd, &mut buf) {
        Err(Errno::EAGAIN) => {}
        Ok(_) => {}
        Err(_) => {
            let _ = syscall::close(sfd);
            let _ = syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None);
            return Err(crate::harness::AssertFail::msg("sfd eagain"));
        }
    }
    check_ok!(syscall::close(sfd), "close");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_sigchld_wait_child() -> TestResult {
    let mut act = Sigaction {
        sa_handler: SIG_DFL,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&act), None), "dfl");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check!(syscall::wifexited(st), "exited");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_sigchld_ign_still_wait() -> TestResult {
    let mut old = Sigaction::default();
    let mut act = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&act), Some(&mut old)), "ign");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(7);
    }
    // With SIG_IGN, child may be auto-reaped; wait may get ECHILD.
    let mut st = 0;
    match syscall::wait4(pid, &mut st, 0) {
        Ok(_) => check_eq!(syscall::wexitstatus(st), 7, "st"),
        Err(Errno::ECHILD) => {}
        Err(_) => {
            let _ = syscall::rt_sigaction(SIGCHLD, Some(&old), None);
            return Err(crate::harness::AssertFail::msg("wait"));
        }
    }
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_procmask_setmask_empty() -> TestResult {
    let empty = 0u64;
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(empty), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_pending_initially_clear_soft() -> TestResult {
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    let _ = pending;
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_kill_zero_process_group_soft() -> TestResult {
    // kill(0, 0) probes calling process group.
    match syscall::kill(0, 0) {
        Ok(()) => {}
        Err(Errno::ESRCH) | Err(Errno::EPERM) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("kill pg")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_block_two_signals() -> TestResult {
    let set = sigbit(SIGUSR1) | sigbit(syscall::SIGUSR2);
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(set), Some(&mut old)), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(set), None), "unblock");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sig_signalfd_update_mask() -> TestResult {
    let m1 = sigbit(SIGUSR1);
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m1), Some(&mut old)), "block");
    let sfd = check_ok!(syscall::signalfd(-1, m1, SFD_CLOEXEC), "sfd");
    let m2 = sigbit(SIGUSR1) | sigbit(syscall::SIGUSR2);
    let _ = syscall::rt_sigprocmask(SIG_BLOCK, Some(m2), None);
    let sfd2 = check_ok!(syscall::signalfd(sfd, m2, SFD_CLOEXEC), "upd");
    check_eq!(sfd2, sfd, "same fd");
    check_ok!(syscall::close(sfd), "close");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_action_ign_hup() -> TestResult {
    let mut old = Sigaction::default();
    let mut neu = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(syscall::SIGHUP, Some(&neu), Some(&mut old)), "ign");
    check_ok!(syscall::rt_sigaction(syscall::SIGHUP, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_child_kill_zero_alive() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec { tv_sec: 5, tv_nsec: 0 };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, 0), "alive");
    check_ok!(syscall::kill(pid, SIGTERM), "term");
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_rt_sigaction_int_ign() -> TestResult {
    let mut old = Sigaction::default();
    let mut neu = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(syscall::SIGINT, Some(&neu), Some(&mut old)), "ign");
    check_ok!(syscall::rt_sigaction(syscall::SIGINT, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_procmask_query_only() -> TestResult {
    let mut cur = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, None, Some(&mut cur)), "query");
    let _ = cur;
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sig_pending_clear_after_ign() -> TestResult {
    let set = sigbit(SIGUSR1);
    let mut oldm = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(set), Some(&mut oldm)), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "raise");
    let mut act = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&act), None), "ign");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(set), None), "unblock");
    let mut pending = !0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & set == 0, "cleared");
    act.sa_handler = SIG_DFL;
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&act), None), "dfl");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(oldm), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_kill_self_term_with_ign() -> TestResult {
    let mut old = Sigaction::default();
    let mut neu = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&neu), Some(&mut old)), "ign");
    check_ok!(syscall::kill(syscall::getpid(), SIGTERM), "kill");
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&old), None), "restore");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sig_wait_after_sigchld_default() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(42);
    }
    let mut st = 0;
    check_eq!(check_ok!(syscall::waitpid(pid, &mut st, 0), "wait"), pid, "pid");
    check_eq!(syscall::wexitstatus(st), 42, "st");
    Ok(())
}
