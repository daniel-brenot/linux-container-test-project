//! sync and syncfs tests.

use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{self};

#[crate::lctp_test(suite = syscall)]
fn sync_returns() -> TestResult {
    check_ok!(syscall::sync(), "sync");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn syncfs_temp_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"syncf")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"data"), "write");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn syncfs_after_truncate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"st", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 4096), "truncate");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_twice() -> TestResult {
    check_ok!(syscall::sync(), "sync1");
    check_ok!(syscall::sync(), "sync2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn syncfs_memfd() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"syncm\0", 0), "memfd");
    check_ok!(syscall::write(fd, b"x"), "write");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn syncfs_dir_fd() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(syscall::open(tmp.path(), crate::syscall::oflag::O_RDONLY | crate::syscall::oflag::O_DIRECTORY, 0), "open dir");
    check_ok!(syscall::syncfs(fd), "syncfs dir");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sync_then_syncfs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"both")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::sync(), "sync");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn syncfs_empty_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"empty")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
