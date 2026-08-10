//! Timestamp / ctime side-effect filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{
    copy_child, create_dir, create_empty, nanosleep_secs, timespec_later, write_file,
};
use crate::syscall::{self, oflag};

#[crate::lctp_test(suite = fs, full)]
fn ts_ctime_on_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    write_file(&path, b"x")?;
    let after = check_ok!(syscall::stat(&path), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_mtime_on_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    write_file(&path, b"y")?;
    let after = check_ok!(syscall::stat(&path), "after");
    check!(
        timespec_later(
            after.st_mtime,
            after.st_mtime_nsec,
            before.st_mtime,
            before.st_mtime_nsec
        ),
        "mtime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_ctime_on_chmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    let after = check_ok!(syscall::stat(&path), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_ctime_on_fchmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fchmod(fd, 0o640), "fchmod");
    check_ok!(syscall::close(fd), "close");
    let after = check_ok!(syscall::stat(&path), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_ctime_on_link() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let before = check_ok!(syscall::stat(&a), "before");
    nanosleep_secs(1)?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let after = check_ok!(syscall::stat(&a), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_ctime_on_unlink_sibling_link() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let before = check_ok!(syscall::stat(&a), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::unlink(&b), "unlink");
    let after = check_ok!(syscall::stat(&a), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_mtime_on_truncate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"abcdef")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::truncate(&path, 2), "trunc");
    let after = check_ok!(syscall::stat(&path), "after");
    check!(
        timespec_later(
            after.st_mtime,
            after.st_mtime_nsec,
            before.st_mtime,
            before.st_mtime_nsec
        ),
        "mtime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_ctime_on_truncate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"abcdef")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::truncate(&path, 1), "trunc");
    let after = check_ok!(syscall::stat(&path), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_mtime_on_ftruncate_grow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let before = check_ok!(syscall::fstat(fd), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::ftruncate(fd, 100), "grow");
    let after = check_ok!(syscall::fstat(fd), "after");
    check!(
        timespec_later(
            after.st_mtime,
            after.st_mtime_nsec,
            before.st_mtime,
            before.st_mtime_nsec
        ),
        "mtime"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_dir_ctime_on_creat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let before = check_ok!(syscall::stat(&dir), "before");
    nanosleep_secs(1)?;
    let mut nested = [0u8; 160];
    let child = crate::suites::common::join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let after = check_ok!(syscall::stat(&dir), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_dir_mtime_on_creat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let before = check_ok!(syscall::stat(&dir), "before");
    nanosleep_secs(1)?;
    let mut nested = [0u8; 160];
    let child = crate::suites::common::join_path(&dir, b"g\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let after = check_ok!(syscall::stat(&dir), "after");
    check!(
        timespec_later(
            after.st_mtime,
            after.st_mtime_nsec,
            before.st_mtime,
            before.st_mtime_nsec
        ),
        "mtime"
    );
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_dir_ctime_on_unlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = crate::suites::common::join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let before = check_ok!(syscall::stat(&dir), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::unlink(child), "unlink");
    let after = check_ok!(syscall::stat(&dir), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_ctime_on_rename() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let before = check_ok!(syscall::stat(&a), "before");
    nanosleep_secs(1)?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::rename(&a, &b), "rename");
    let after = check_ok!(syscall::stat(&b), "after");
    // rename may update ctime depending on FS; accept equal-or-later.
    check!(
        after.st_ctime > before.st_ctime
            || (after.st_ctime == before.st_ctime
                && after.st_ctime_nsec >= before.st_ctime_nsec),
        "ctime not earlier"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_mtime_unchanged_on_chmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"z")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    let after = check_ok!(syscall::stat(&path), "after");
    check_eq!(after.st_mtime, before.st_mtime, "mtime");
    check_eq!(after.st_mtime_nsec, before.st_mtime_nsec, "mtime nsec");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_open_rdonly_no_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"data")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::close(fd), "close");
    let after = check_ok!(syscall::stat(&path), "after");
    check_eq!(after.st_mtime, before.st_mtime, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_append_updates_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"A")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0),
        "append"
    );
    check_ok!(syscall::write(fd, b"B"), "write");
    check_ok!(syscall::close(fd), "close");
    let after = check_ok!(syscall::stat(&path), "after");
    check!(
        timespec_later(
            after.st_mtime,
            after.st_mtime_nsec,
            before.st_mtime,
            before.st_mtime_nsec
        ),
        "mtime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_mkdir_sets_timestamps() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"d")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.st_mtime > 0, "mtime");
    check!(st.st_ctime > 0, "ctime");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_symlink_ctime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    let st = check_ok!(syscall::lstat(&link), "lstat");
    check!(st.st_ctime > 0, "ctime");
    check!(st.st_mtime > 0, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_ftruncate_shrink_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"123456")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let before = check_ok!(syscall::fstat(fd), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::ftruncate(fd, 2), "shrink");
    let after = check_ok!(syscall::fstat(fd), "after");
    check!(
        timespec_later(
            after.st_mtime,
            after.st_mtime_nsec,
            before.st_mtime,
            before.st_mtime_nsec
        ),
        "mtime"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn ts_chmod_dir_ctime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let before = check_ok!(syscall::stat(&dir), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::chmod(&dir, 0o700), "chmod");
    let after = check_ok!(syscall::stat(&dir), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}
