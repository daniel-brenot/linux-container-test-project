//! flock filesystem tests.

use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{self, oflag, LOCK_EX, LOCK_NB, LOCK_SH, LOCK_UN};

#[crate::lctp_test(suite = fs)]
fn fs_flock_exclusive() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fl")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_flock_shared() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fls")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_SH), "lock");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_flock_nb_success() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flnb")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX | LOCK_NB), "lock nb");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn fs_flock_two_fds_same_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fl2")?;
    let fd1 = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open1");
    let fd2 = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open2");
    check_ok!(syscall::flock(fd1, LOCK_SH), "sh1");
    check_ok!(syscall::flock(fd2, LOCK_SH), "sh2");
    check_ok!(syscall::flock(fd1, LOCK_UN), "un1");
    check_ok!(syscall::flock(fd2, LOCK_UN), "un2");
    check_ok!(syscall::close(fd1), "close1");
    check_ok!(syscall::close(fd2), "close2");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_flock_relock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flr")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::flock(fd, LOCK_EX), "relock");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock2");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn fs_flock_with_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flw")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock");
    check_ok!(syscall::write(fd, b"locked-write"), "write");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_flock_rdonly_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flro")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_SH), "lock sh");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_flock_upgrade() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flu")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_SH), "sh");
    check_ok!(syscall::flock(fd, LOCK_EX), "ex");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn fs_flock_close_auto_unlock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flc")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock");
    check_ok!(syscall::close(fd), "close unlocks");
    let fd2 = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "reopen");
    check_ok!(syscall::flock(fd2, LOCK_EX | LOCK_NB), "relock");
    check_ok!(syscall::flock(fd2, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd2), "close2");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_flock_wronly_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flwo")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
