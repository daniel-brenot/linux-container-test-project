//! pselect6(2) tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, FdSet, Timespec};

#[crate::lctp_test(suite = syscall, expect = success, case = "pselect6 reports a pipe that already has data as readable")]
fn pselect6_pipe_readable() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, b"x"), "write");
    let mut rfds = FdSet::zero();
    rfds.set(r);
    let ts = Timespec {
        tv_sec: 1,
        tv_nsec: 0,
    };
    let n = check_ok!(
        syscall::pselect6(r + 1, Some(&mut rfds), None, None, Some(&ts), None),
        "pselect6"
    );
    check_eq!(n, 1, "ready");
    check!(rfds.is_set(r), "fd set");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pselect6 of an empty pipe with a short timeout returns 0")]
fn pselect6_timeout() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut rfds = FdSet::zero();
    rfds.set(r);
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let n = check_ok!(
        syscall::pselect6(r + 1, Some(&mut rfds), None, None, Some(&ts), None),
        "pselect6"
    );
    check_eq!(n, 0, "timeout");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pselect6 with a zero timeout on an empty pipe returns 0")]
fn pselect6_zero_timeout() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut rfds = FdSet::zero();
    rfds.set(r);
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let n = check_ok!(
        syscall::pselect6(r + 1, Some(&mut rfds), None, None, Some(&ts), None),
        "pselect6"
    );
    check_eq!(n, 0, "not ready");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "pselect6 reports an empty pipe write end as writable")]
fn pselect6_writefds_pipe() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut wfds = FdSet::zero();
    wfds.set(w);
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 50_000_000,
    };
    let n = check_ok!(
        syscall::pselect6(w + 1, None, Some(&mut wfds), None, Some(&ts), None),
        "pselect6"
    );
    check!(n >= 1, "writable");
    check!(wfds.is_set(w), "w set");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pselect6 with all fd sets NULL and a short timeout returns 0")]
fn pselect6_null_sets() -> TestResult {
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    let n = check_ok!(
        syscall::pselect6(0, None, None, None, Some(&ts), None),
        "pselect6"
    );
    check_eq!(n, 0, "no fds");
    Ok(())
}
