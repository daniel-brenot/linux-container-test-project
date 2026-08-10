//! statfs / fstatfs filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{self, oflag};

#[crate::lctp_test(suite = fs)]
fn fs_statfs_temp_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let st = check_ok!(syscall::statfs(tmp.path()), "statfs");
    check!(st.f_bsize > 0, "bsize");
    check!(st.f_blocks > 0, "blocks");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_statfs_tmp() -> TestResult {
    let st = check_ok!(syscall::statfs(b"/tmp\0"), "statfs /tmp");
    check!(st.f_bsize > 0, "bsize");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_fstatfs_open_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"sf")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let st = check_ok!(syscall::fstatfs(fd), "fstatfs");
    check!(st.f_bsize > 0, "bsize");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_statfs_namelen() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let st = check_ok!(syscall::statfs(tmp.path()), "statfs");
    check!(st.f_namelen >= 255, "namelen");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_statfs_bavail_le_bfree() -> TestResult {
    let st = check_ok!(syscall::statfs(b"/tmp\0"), "statfs");
    check!(st.f_bavail <= st.f_bfree, "bavail");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn fs_fstatfs_dir_fd() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0), "open dir");
    let st = check_ok!(syscall::fstatfs(fd), "fstatfs");
    check!(st.f_blocks > 0, "blocks");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_statfs_files_nonzero() -> TestResult {
    let st = check_ok!(syscall::statfs(b"/\0"), "statfs /");
    check!(st.f_files > 0, "files");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_statfs_frsize() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let st = check_ok!(syscall::statfs(tmp.path()), "statfs");
    check!(st.f_frsize > 0, "frsize");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn fs_statfs_fstatfs_consistent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"c")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let a = check_ok!(syscall::statfs(tmp.path()), "statfs");
    let b = check_ok!(syscall::fstatfs(fd), "fstatfs");
    check_eq!(a.f_type, b.f_type, "type");
    check_eq!(a.f_bsize, b.f_bsize, "bsize");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_statfs_root() -> TestResult {
    let st = check_ok!(syscall::statfs(b"/\0"), "statfs");
    check!(st.f_bsize > 0, "bsize");
    check!(st.f_blocks > 0, "blocks");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fs_fstatfs_after_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"w", 0o644), "create");
    check_ok!(syscall::write(fd, b"data"), "write");
    let st = check_ok!(syscall::fstatfs(fd), "fstatfs");
    check!(st.f_bsize > 0, "bsize");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
