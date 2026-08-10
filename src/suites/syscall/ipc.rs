//! IPC syscall tests (pipes, socketpairs, eventfd).

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, oflag, AF_UNIX, SOCK_STREAM, Errno};

#[crate::lctp_test(suite = syscall)]
fn pipe2_roundtrip() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let msg = b"pipe-data";
    check_eq!(check_ok!(syscall::write(w, msg), "write"), msg.len(), "wlen");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), msg.len(), "rlen");
    check!(&buf[..msg.len()] == msg, "data");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pipe2_cloexec() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_CLOEXEC), "pipe2 cloexec");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pipe2_nonblock() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_NONBLOCK), "pipe2 nonblock");
    let fl = check_ok!(syscall::fcntl(r, crate::syscall::fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & oflag::O_NONBLOCK != 0, "O_NONBLOCK on read");
    // Empty nonblocking read returns EAGAIN.
    let mut buf = [0u8; 1];
    check_err!(syscall::read(r, &mut buf), Errno::EAGAIN, "EAGAIN");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pipe2_partial_write() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, b"AB"), "write");
    let mut one = [0u8; 1];
    check_eq!(check_ok!(syscall::read(r, &mut one), "read1"), 1, "one");
    check_eq!(one[0], b'A', "byte");
    check_eq!(check_ok!(syscall::read(r, &mut one), "read2"), 1, "two");
    check_eq!(one[0], b'B', "byte2");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn socketpair_stream() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    check_ok!(syscall::send(a, b"ping", 0), "send");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::recv(b, &mut buf, 0), "recv"), 4, "rlen");
    check_eq!(&buf, b"ping", "payload");
    check_ok!(syscall::shutdown(a, syscall::SHUT_RDWR), "shutdown a");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn socketpair_cloexec() -> TestResult {
    let (a, b) = check_ok!(
        syscall::socketpair(AF_UNIX, SOCK_STREAM | oflag::O_CLOEXEC, 0),
        "socketpair cloexec"
    );
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn eventfd_write_read() -> TestResult {
    let efd = check_ok!(syscall::eventfd(0, 0), "eventfd");
    let val: u64 = 42;
    let bytes = val.to_le_bytes();
    check_eq!(check_ok!(syscall::write(efd, &bytes), "write"), 8, "wlen");
    let mut out = [0u8; 8];
    check_eq!(check_ok!(syscall::read(efd, &mut out), "read"), 8, "rlen");
    check_eq!(u64::from_le_bytes(out), 42, "value");
    check_ok!(syscall::close(efd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn eventfd_increment() -> TestResult {
    let efd = check_ok!(syscall::eventfd(5, 0), "eventfd init 5");
    let one = 1u64.to_le_bytes();
    check_ok!(syscall::write(efd, &one), "inc");
    let mut out = [0u8; 8];
    check_ok!(syscall::read(efd, &mut out), "read");
    check_eq!(u64::from_le_bytes(out), 6, "counter");
    check_ok!(syscall::close(efd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn eventfd_cloexec() -> TestResult {
    let efd = check_ok!(syscall::eventfd(0, oflag::O_CLOEXEC), "eventfd cloexec");
    check_ok!(syscall::close(efd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn eventfd_write_twice_accumulates() -> TestResult {
    let efd = check_ok!(syscall::eventfd(0, 0), "eventfd");
    let one = 1u64.to_le_bytes();
    check_ok!(syscall::write(efd, &one), "write1");
    check_ok!(syscall::write(efd, &one), "write2");
    let mut out = [0u8; 8];
    check_ok!(syscall::read(efd, &mut out), "read");
    check_eq!(u64::from_le_bytes(out), 2, "accumulated");
    check_ok!(syscall::close(efd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn eventfd_semaphore_style() -> TestResult {
    let efd = check_ok!(
        syscall::eventfd(0, crate::syscall::EFD_SEMAPHORE),
        "eventfd sem"
    );
    let one = 1u64.to_le_bytes();
    check_ok!(syscall::write(efd, &one), "w1");
    check_ok!(syscall::write(efd, &one), "w2");
    let mut out = [0u8; 8];
    check_ok!(syscall::read(efd, &mut out), "r1");
    check_eq!(u64::from_le_bytes(out), 1, "sem read1");
    check_ok!(syscall::read(efd, &mut out), "r2");
    check_eq!(u64::from_le_bytes(out), 1, "sem read2");
    check_ok!(syscall::close(efd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn eventfd_semaphore_nonblock_eagain() -> TestResult {
    let efd = check_ok!(
        syscall::eventfd(
            0,
            crate::syscall::EFD_SEMAPHORE | crate::syscall::EFD_NONBLOCK
        ),
        "sem nb"
    );
    let mut out = [0u8; 8];
    check_err!(syscall::read(efd, &mut out), Errno::EAGAIN, "empty");
    check_ok!(syscall::close(efd), "close");
    Ok(())
}
