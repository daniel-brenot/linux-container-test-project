//! Networking and poll/epoll syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, epoll, oflag, poll, AF_UNIX, EPOLL_CTL_ADD, EPOLLIN, POLLIN, SOCK_STREAM,
};

#[crate::lctp_test(suite = syscall)]
fn poll_pipe_readable() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, b"x"), "write");
    let mut pfd = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut pfd, 1000), "poll");
    check_eq!(n, 1, "poll count");
    check!(pfd[0].revents & POLLIN != 0, "POLLIN");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn poll_pipe_timeout() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut pfd = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut pfd, 10), "poll timeout");
    check_eq!(n, 0, "no events");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn epoll_pipe_readable() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let ep = check_ok!(syscall::epoll_create1(oflag::O_CLOEXEC), "epoll_create1");
    let mut ev = epoll::EpollEvent { events: EPOLLIN, data: 0 };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "epoll_ctl");
    check_ok!(syscall::write(w, b"z"), "write");
    let mut events = [epoll::EpollEvent { events: 0, data: 0 }; 4];
    let n = check_ok!(syscall::epoll_wait(ep, &mut events, 100), "epoll_wait");
    check_eq!(n, 1, "epoll count");
    check!(events[0].events & EPOLLIN != 0, "EPOLLIN");
    check_ok!(syscall::close(ep), "close ep");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn socketpair_send_recv_all() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let msg = b"stream-msg";
    check_eq!(check_ok!(syscall::send(a, msg, 0), "send"), msg.len(), "slen");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::recv(b, &mut buf, 0), "recv"), msg.len(), "rlen");
    check!(&buf[..msg.len()] == msg, "data");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn socketpair_half_close() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    check_ok!(syscall::shutdown(a, syscall::SHUT_WR), "shutdown wr");
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::read(b, &mut buf), "read after shutdown");
    check_eq!(n, 0, "EOF");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn socketpair_large_message() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let msg = [0xABu8; 512];
    check_eq!(check_ok!(syscall::send(a, &msg, 0), "send"), msg.len(), "slen");
    let mut buf = [0u8; 512];
    let mut got = 0usize;
    while got < msg.len() {
        let n = check_ok!(syscall::recv(b, &mut buf[got..], 0), "recv");
        if n == 0 {
            break;
        }
        got += n;
    }
    check_eq!(got, msg.len(), "full message");
    check_eq!(&buf, &msg, "payload");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}
