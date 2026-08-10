//! mkdir filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir};
use crate::syscall::{self, Errno};

#[crate::lctp_test(suite = fs)]
fn mkdir_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"dir")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "is dir");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_mode_755() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let st = check_ok!(syscall::stat(&dir), "stat");
    check_eq!(st.mode_bits() & 0o777, 0o755, "mode");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_mode_700() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o700)?;
    let st = check_ok!(syscall::stat(&dir), "stat");
    check_eq!(st.mode_bits() & 0o777, 0o700, "mode");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::mkdir(&dir, 0o755), Errno::EEXIST, "eexist");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_parent_missing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let base = copy_child(&mut tmp, b"base")?;
    let mut nested = [0u8; 160];
    let suffix = b"/a/b/c";
    let blen = base.iter().position(|&c| c == 0).unwrap();
    check!(blen + suffix.len() + 1 < nested.len(), "path too long");
    nested[..blen].copy_from_slice(&base[..blen]);
    nested[blen..blen + suffix.len()].copy_from_slice(suffix);
    nested[blen + suffix.len()] = 0;
    check_err!(
        syscall::mkdir(crate::suites::common::truncate_cstr(&nested), 0o755),
        Errno::ENOENT,
        "no parent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_nested() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let parent = create_dir(&mut tmp, b"p", 0o755)?;
    let mut child = [0u8; 160];
    let plen = parent.iter().position(|&c| c == 0).unwrap();
    child[..plen].copy_from_slice(&parent[..plen]);
    child[plen..plen + 4].copy_from_slice(b"/sub");
    child[plen + 4] = 0;
    check_ok!(syscall::mkdir(&child, 0o755), "mkdir sub");
    check_ok!(syscall::rmdir(&child), "rmdir sub");
    check_ok!(syscall::rmdir(&parent), "rmdir parent");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn mkdir_dot_entries() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let fd = check_ok!(
        syscall::open(&dir, crate::syscall::oflag::O_RDONLY | crate::syscall::oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_is_directory() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "directory");
    check!(!st.is_reg(), "not file");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}
