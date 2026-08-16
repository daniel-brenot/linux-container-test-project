//! sync / syncfs filesystem tests.

use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{self, oflag};

#[crate::lctp_test(suite = fs, expect = success, case = "sync returns success")]
fn fs_sync_returns() -> TestResult {
    check_ok!(syscall::sync(), "sync");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "syncfs on a file fd after write succeeds")]
fn fs_syncfs_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"s")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"sync-data"), "write");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "syncfs after ftruncate succeeds")]
fn fs_syncfs_after_ftruncate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"st", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 8192), "truncate");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "syncfs on an empty file fd succeeds")]
fn fs_syncfs_empty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"e")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "sync then syncfs on a written file fd succeeds")]
fn fs_sync_then_syncfs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"b", 0o644), "create");
    check_ok!(syscall::write(fd, b"x"), "write");
    check_ok!(syscall::sync(), "sync");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "syncfs on a directory fd succeeds")]
fn fs_syncfs_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0), "open");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "syncfs after a multi-block write succeeds")]
fn fs_syncfs_large_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"big", 0o644), "create");
    let chunk = [0xABu8; 4096];
    for _ in 0..4 {
        check_ok!(syscall::write(fd, &chunk), "write");
    }
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "calling sync twice succeeds")]
fn fs_sync_twice() -> TestResult {
    check_ok!(syscall::sync(), "sync1");
    check_ok!(syscall::sync(), "sync2");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "syncfs after fsync succeeds")]
fn fs_syncfs_after_fsync() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fs", 0o644), "create");
    check_ok!(syscall::write(fd, b"data"), "write");
    check_ok!(syscall::fsync(fd), "fsync");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "syncfs on a read-only file fd succeeds")]
fn fs_syncfs_rdonly() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ro")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::syncfs(fd), "syncfs");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
