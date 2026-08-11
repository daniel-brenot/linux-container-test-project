//! `epoll_create1` / `epoll_ctl` / `epoll_wait` behavioural coverage.
//!
//! Exercises registration semantics (`EEXIST` / `ENOENT`), event delivery,
//! oneshot/edge/level quirks, and fd lifecycle — not just a single happy-path
//! ADD that a no-op `epoll_ctl` could falsely satisfy.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, epoll, fcntl_cmd, Errno, EFD_CLOEXEC, EFD_NONBLOCK, EPOLLET, EPOLLIN, EPOLLONESHOT,
    EPOLLOUT, EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, FD_CLOEXEC,
};

fn close_all(fds: &[i32]) {
    for &fd in fds {
        let _ = syscall::close(fd);
    }
}

#[crate::lctp_test(suite = syscall)]
fn epoll_create1_cloexec() -> TestResult {
    let ep = check_ok!(syscall::epoll_create1(EPOLL_CLOEXEC), "create1");
    let flags = check_ok!(syscall::fcntl(ep, fcntl_cmd::F_GETFD, 0), "getfd");
    check!(flags as i32 & FD_CLOEXEC != 0, "CLOEXEC");
    check_ok!(syscall::close(ep), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_add_duplicate_eexist() -> TestResult {
    // Duplicate ADD must fail with EEXIST; success here means registration is
    // not tracking membership (e.g. a no-op epoll_ctl).
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: r as u64,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev),
        Errno::EEXIST,
        "dup ADD"
    );
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_mod_without_add_enoent() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 1,
    };
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev),
        Errno::ENOENT,
        "MOD without ADD"
    );
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_del_without_add_enoent() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 0,
    };
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_DEL, r, &mut ev),
        Errno::ENOENT,
        "DEL without ADD"
    );
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_add_mod_del_add() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 7,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    ev.events = EPOLLIN | EPOLLOUT;
    ev.data = 8;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev), "mod");
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_DEL, r, &mut ev), "del");
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev),
        Errno::ENOENT,
        "mod after del"
    );
    ev.events = EPOLLIN;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "re-add");
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev),
        Errno::EEXIST,
        "re-add dup"
    );
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_bad_epfd_ebadf() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 0,
    };
    check_err!(
        syscall::epoll_ctl(-1, EPOLL_CTL_ADD, r, &mut ev),
        Errno::EBADF,
        "bad epfd"
    );
    close_all(&[r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_bad_fd_ebadf() -> TestResult {
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 0,
    };
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_ADD, -1, &mut ev),
        Errno::EBADF,
        "bad fd"
    );
    check_ok!(syscall::close(ep), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_eventfd_add_wait() -> TestResult {
    let efd = check_ok!(
        syscall::eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK),
        "eventfd"
    );
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 0xDEAD_BEEF,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, efd, &mut ev), "add");
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_ADD, efd, &mut ev),
        Errno::EEXIST,
        "dup ADD"
    );
    let one: u64 = 1;
    check_ok!(syscall::write(efd, &one.to_ne_bytes()), "write");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 1000), "wait");
    check_eq!(n, 1, "ready");
    check_eq!(out[0].data, 0xDEAD_BEEF, "user data");
    check!(out[0].events & EPOLLIN != 0, "EPOLLIN");
    close_all(&[ep, efd]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_wait_data_preserved_across_mod() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 111,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    ev.data = 222;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev), "mod");
    check_ok!(syscall::write(w, b"x"), "write");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 1000), "wait");
    check_eq!(n, 1, "n");
    check_eq!(out[0].data, 222, "data after mod");
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_wait_two_ready() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r1, &mut ev), "add1");
    ev.data = 2;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r2, &mut ev), "add2");
    check_ok!(syscall::write(w1, b"a"), "w1");
    check_ok!(syscall::write(w2, b"b"), "w2");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 4];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 1000), "wait");
    check_eq!(n, 2, "both ready");
    let mut saw1 = false;
    let mut saw2 = false;
    for e in out.iter().take(n) {
        if e.data == 1 {
            saw1 = true;
        }
        if e.data == 2 {
            saw2 = true;
        }
        check!(e.events & EPOLLIN != 0, "in");
    }
    check!(saw1 && saw2, "both data");
    close_all(&[ep, r1, w1, r2, w2]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_ctl_pipe_write_epollout() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLOUT,
        data: 9,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, w, &mut ev), "add");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 0), "wait");
    check_eq!(n, 1, "writable");
    check!(out[0].events & EPOLLOUT != 0, "EPOLLOUT");
    check_eq!(out[0].data, 9, "data");
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_level_rearms_without_mod() -> TestResult {
    // Level-triggered: unread data stays ready across waits.
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"z"), "write");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w1"), 1, "first");
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w2"), 1, "still ready");
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_oneshot_needs_rearm() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLONESHOT,
        data: 3,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"z"), "write");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w1"), 1, "once");
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w2"), 0, "disarmed");
    ev.events = EPOLLIN | EPOLLONESHOT;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev), "rearm");
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w3"), 1, "after rearm");
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn epoll_et_edge_then_silent() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLET,
        data: 4,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"z"), "write");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w1"), 1, "edge");
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w2"), 0, "no re-edge");
    // New data should produce another edge.
    check_ok!(syscall::write(w, b"y"), "write2");
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "w3"), 1, "new edge");
    close_all(&[ep, r, w]);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn epoll_ctl_two_fds_independent_eexist() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r1, &mut ev), "add1");
    ev.data = 2;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r2, &mut ev), "add2");
    ev.data = 1;
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r1, &mut ev),
        Errno::EEXIST,
        "dup fd1"
    );
    ev.data = 2;
    check_err!(
        syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r2, &mut ev),
        Errno::EEXIST,
        "dup fd2"
    );
    close_all(&[ep, r1, w1, r2, w2]);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn epoll_hup_after_peer_close() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 5,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::close(w), "close writer");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 1000), "wait");
    check_eq!(n, 1, "hup/in");
    // Peer close typically surfaces as EPOLLHUP and/or EPOLLIN (EOF).
    check!(
        out[0].events & (EPOLLIN | syscall::EPOLLHUP) != 0,
        "hangup bits"
    );
    close_all(&[ep, r]);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn epoll_del_then_no_event() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 6,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_DEL, r, &mut ev), "del");
    check_ok!(syscall::write(w, b"x"), "write");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 0), "wait"), 0, "gone");
    close_all(&[ep, r, w]);
    Ok(())
}
