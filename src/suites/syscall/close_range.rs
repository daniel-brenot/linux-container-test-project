//! close_range(2) tests.

use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::syscall::{self, Errno};

#[crate::lctp_test(suite = syscall)]
fn close_range_dup_fds() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d1 = check_ok!(syscall::dup(fd), "dup1");
    let d2 = check_ok!(syscall::dup(fd), "dup2");
    let first = d1.min(d2) as u32;
    let last = d1.max(d2) as u32;
    check_ok!(syscall::close_range(first, last, 0), "close_range");
    check_err!(syscall::write(d1, b"x"), Errno::EBADF, "d1 closed");
    check_err!(syscall::write(d2, b"x"), Errno::EBADF, "d2 closed");
    // Original may or may not be in range.
    if fd as u32 >= first && fd as u32 <= last {
        check_err!(syscall::write(fd, b"x"), Errno::EBADF, "fd closed");
    } else {
        check_ok!(syscall::close(fd), "close fd");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn close_range_single_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d = check_ok!(syscall::dup(fd), "dup");
    check_ok!(syscall::close_range(d as u32, d as u32, 0), "close_range");
    check_err!(syscall::close(d), Errno::EBADF, "already closed");
    check_ok!(syscall::close(fd), "close orig");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn close_range_high_dups() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d1 = check_ok!(syscall::dup3(fd, 80, 0), "dup3 80");
    let d2 = check_ok!(syscall::dup3(fd, 81, 0), "dup3 81");
    let d3 = check_ok!(syscall::dup3(fd, 82, 0), "dup3 82");
    check_ok!(syscall::close_range(80, 82, 0), "close_range");
    check_err!(syscall::write(d1, b"x"), Errno::EBADF, "80");
    check_err!(syscall::write(d2, b"x"), Errno::EBADF, "81");
    check_err!(syscall::write(d3, b"x"), Errno::EBADF, "82");
    check_ok!(syscall::close(fd), "close orig");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn close_range_empty_span() -> TestResult {
    // first > last is EINVAL on Linux.
    match syscall::close_range(10, 5, 0) {
        Err(Errno::EINVAL) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("expected EINVAL")),
        Err(_) => return Err(crate::harness::AssertFail::msg("unexpected errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn close_range_preserves_outside() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let keep = check_ok!(tmp.create_file(b"keep", 0o644), "keep");
    let drop = check_ok!(syscall::dup3(keep, 90, 0), "dup high");
    check_ok!(syscall::close_range(90, 90, 0), "close_range");
    check_err!(syscall::write(drop, b"x"), Errno::EBADF, "dropped");
    check_ok!(syscall::write(keep, b"ok"), "kept");
    check_ok!(syscall::close(keep), "close keep");
    Ok(())
}
