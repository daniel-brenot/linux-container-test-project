//! ppoll(2) tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, poll, Timespec, POLLIN};

#[crate::lctp_test(suite = syscall)]
fn ppoll_pipe_readable() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, b"x"), "write");
    let mut pfd = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let ts = Timespec {
        tv_sec: 1,
        tv_nsec: 0,
    };
    let n = check_ok!(syscall::ppoll(&mut pfd, Some(&ts), None), "ppoll");
    check_eq!(n, 1, "ready");
    check!(pfd[0].revents & POLLIN != 0, "POLLIN");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn ppoll_timeout_empty() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut pfd = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let n = check_ok!(syscall::ppoll(&mut pfd, Some(&ts), None), "ppoll");
    check_eq!(n, 0, "timeout");
    check_eq!(pfd[0].revents, 0, "no revents");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ppoll_null_timeout_immediate() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, b"z"), "write");
    let mut pfd = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    // NULL timeout means wait forever; data already ready so returns immediately.
    let n = check_ok!(syscall::ppoll(&mut pfd, None, None), "ppoll");
    check_eq!(n, 1, "ready");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ppoll_zero_timespec() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut pfd = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let n = check_ok!(syscall::ppoll(&mut pfd, Some(&ts), None), "ppoll");
    check_eq!(n, 0, "not ready");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn ppoll_two_fds_one_ready() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "pipe1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w2, b"y"), "write");
    let mut pfd = [
        poll::PollFd {
            fd: r1,
            events: POLLIN,
            revents: 0,
        },
        poll::PollFd {
            fd: r2,
            events: POLLIN,
            revents: 0,
        },
    ];
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 50_000_000,
    };
    let n = check_ok!(syscall::ppoll(&mut pfd, Some(&ts), None), "ppoll");
    check_eq!(n, 1, "one ready");
    check!(pfd[1].revents & POLLIN != 0, "r2 POLLIN");
    check_ok!(syscall::close(r1), "close r1");
    check_ok!(syscall::close(w1), "close w1");
    check_ok!(syscall::close(r2), "close r2");
    check_ok!(syscall::close(w2), "close w2");
    Ok(())
}
