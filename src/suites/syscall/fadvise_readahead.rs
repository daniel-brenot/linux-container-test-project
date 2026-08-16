//! sync_file_range, fadvise64/posix_fadvise, and readahead tests.

use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, write_file};
use crate::syscall::{
    self, oflag, Errno, POSIX_FADV_DONTNEED, POSIX_FADV_NORMAL, POSIX_FADV_WILLNEED,
    SYNC_FILE_RANGE_WAIT_AFTER, SYNC_FILE_RANGE_WAIT_BEFORE, SYNC_FILE_RANGE_WRITE,
};

#[crate::lctp_test(suite = syscall, expect = success, case = "sync_file_range with WAIT_BEFORE, WRITE, and WAIT_AFTER succeeds after a write")]
fn sync_file_range_after_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"sfr")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"sync-range-data"), "write");
    check_ok!(
        syscall::sync_file_range(
            fd,
            0,
            0,
            SYNC_FILE_RANGE_WAIT_BEFORE | SYNC_FILE_RANGE_WRITE | SYNC_FILE_RANGE_WAIT_AFTER
        ),
        "sync_file_range"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sync_file_range of a four-byte range with SYNC_FILE_RANGE_WRITE succeeds")]
fn sync_file_range_partial() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"sfr2")?;
    write_file(&path, b"0123456789")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(
        syscall::sync_file_range(fd, 0, 4, SYNC_FILE_RANGE_WRITE),
        "partial"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "sync_file_range on fd -1 returns EBADF")]
fn sync_file_range_ebadf() -> TestResult {
    check_err!(
        syscall::sync_file_range(-1, 0, 0, SYNC_FILE_RANGE_WRITE),
        Errno::EBADF,
        "bad fd"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "fadvise64 with POSIX_FADV_NORMAL on a regular file succeeds")]
fn fadvise_normal() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fa")?;
    write_file(&path, b"advise")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::fadvise64(fd, 0, 0, POSIX_FADV_NORMAL), "NORMAL");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "posix_fadvise WILLNEED then DONTNEED on a regular file both succeed")]
fn fadvise_willneed_dontneed() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fa2")?;
    write_file(&path, &[b'x'; 256])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(
        syscall::posix_fadvise(fd, 0, 256, POSIX_FADV_WILLNEED),
        "WILLNEED"
    );
    check_ok!(
        syscall::posix_fadvise(fd, 0, 256, POSIX_FADV_DONTNEED),
        "DONTNEED"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "fadvise64 WILLNEED on a byte range of a regular file succeeds")]
fn fadvise_range() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fa3")?;
    write_file(&path, &[b'y'; 64])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::fadvise64(fd, 8, 16, POSIX_FADV_WILLNEED), "range");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "fadvise64 on fd -1 returns EBADF")]
fn fadvise_ebadf() -> TestResult {
    check_err!(
        syscall::fadvise64(-1, 0, 0, POSIX_FADV_NORMAL),
        Errno::EBADF,
        "bad fd"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "readahead of a regular file from offset 0 succeeds")]
fn readahead_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ra")?;
    write_file(&path, &[b'z'; 128])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::readahead(fd, 0, 128), "readahead");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "readahead of a regular file at a nonzero offset succeeds")]
fn readahead_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ra2")?;
    write_file(&path, &[b'a'; 64])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::readahead(fd, 16, 32), "readahead off");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "readahead on fd -1 returns EBADF")]
fn readahead_ebadf() -> TestResult {
    check_err!(syscall::readahead(-1, 0, 1), Errno::EBADF, "bad fd");
    Ok(())
}
