//! signalfd4 tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, sigmask, SignalfdSiginfo, SFD_CLOEXEC, SIGUSR1, SIGUSR2, SIG_BLOCK, SIG_UNBLOCK,
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

#[crate::lctp_test(suite = syscall)]
fn signalfd_create_cloexec() -> TestResult {
    let mask = sigmask(SIGUSR1);
    let fd = check_ok!(syscall::signalfd(-1, mask, SFD_CLOEXEC), "signalfd");
    let flags = check_ok!(syscall::fcntl(fd, syscall::fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(flags & syscall::FD_CLOEXEC as usize != 0, "CLOEXEC");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn signalfd_read_sigusr1() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    let fd = check_ok!(
        syscall::signalfd(-1, sigmask(SIGUSR1), SFD_CLOEXEC),
        "signalfd"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill self");
    let mut info = SignalfdSiginfo::default();
    let n = check_ok!(
        syscall::read(
            fd,
            unsafe {
                core::slice::from_raw_parts_mut(
                    &mut info as *mut SignalfdSiginfo as *mut u8,
                    core::mem::size_of::<SignalfdSiginfo>(),
                )
            }
        ),
        "read"
    );
    check_eq!(n, core::mem::size_of::<SignalfdSiginfo>(), "info size");
    check_eq!(info.ssi_signo, SIGUSR1 as u32, "ssi_signo");
    check_eq!(info.ssi_pid, syscall::getpid() as u32, "ssi_pid");
    check_ok!(syscall::close(fd), "close");
    // Signal consumed by signalfd; unblock safely after ignore.
    discard_pending(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn signalfd_two_signals() -> TestResult {
    let mask = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(mask), None), "block");
    let fd = check_ok!(syscall::signalfd(-1, mask, SFD_CLOEXEC), "signalfd");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill1");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill2");
    let mut info = SignalfdSiginfo::default();
    let buf = unsafe {
        core::slice::from_raw_parts_mut(
            &mut info as *mut SignalfdSiginfo as *mut u8,
            core::mem::size_of::<SignalfdSiginfo>(),
        )
    };
    check_ok!(syscall::read(fd, buf), "read1");
    let s1 = info.ssi_signo;
    check_ok!(syscall::read(fd, buf), "read2");
    let s2 = info.ssi_signo;
    check!(
        (s1 == SIGUSR1 as u32 && s2 == SIGUSR2 as u32)
            || (s1 == SIGUSR2 as u32 && s2 == SIGUSR1 as u32),
        "both signals"
    );
    check_ok!(syscall::close(fd), "close");
    discard_pending(SIGUSR1)?;
    discard_pending(SIGUSR2)?;
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn signalfd_replace_mask() -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1) | sigmask(SIGUSR2)), None),
        "block"
    );
    let fd = check_ok!(
        syscall::signalfd(-1, sigmask(SIGUSR1), SFD_CLOEXEC),
        "signalfd"
    );
    let fd2 = check_ok!(
        syscall::signalfd(fd, sigmask(SIGUSR2), SFD_CLOEXEC),
        "replace"
    );
    check_eq!(fd2, fd, "same fd");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut info = SignalfdSiginfo::default();
    check_ok!(
        syscall::read(
            fd,
            unsafe {
                core::slice::from_raw_parts_mut(
                    &mut info as *mut SignalfdSiginfo as *mut u8,
                    core::mem::size_of::<SignalfdSiginfo>(),
                )
            }
        ),
        "read"
    );
    check_eq!(info.ssi_signo, SIGUSR2 as u32, "usr2");
    check_ok!(syscall::close(fd), "close");
    discard_pending(SIGUSR1)?;
    discard_pending(SIGUSR2)?;
    Ok(())
}
