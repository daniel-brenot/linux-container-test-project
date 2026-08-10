//! symlink filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty, write_file};
use crate::syscall::{self, oflag};

#[crate::lctp_test(suite = fs)]
fn symlink_create_readlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    let mut buf = [0u8; 64];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(&buf[..n], b"target", "target");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_lstat_vs_stat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let lst = check_ok!(syscall::lstat(&link), "lstat");
    check!(lst.is_lnk(), "symlink");
    let st = check_ok!(syscall::stat(&link), "stat");
    check!(st.is_reg(), "followed reg");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_stat_follows_size() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    write_file(&file, b"12345")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let st = check_ok!(syscall::stat(&link), "stat");
    check_eq!(st.st_size, 5, "followed size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_relative_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"./file\0", &link), "symlink rel");
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(&buf[..n], b"./file", "rel target");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_open_follows() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    write_file(&file, b"Z")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let fd = check_ok!(syscall::open(&link, oflag::O_RDONLY, 0), "open link");
    let mut b = [0u8; 1];
    check_ok!(syscall::read(fd, &mut b), "read");
    check_eq!(b[0], b'Z', "content");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_readlink_buffer() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"abcdefghij\0", &link), "symlink");
    let mut small = [0u8; 4];
    let n = check_ok!(syscall::readlink(&link, &mut small), "readlink partial");
    check_eq!(n, 4, "truncated readlink len");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn symlink_chain() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    let l1 = copy_child(&mut tmp, b"l1")?;
    let l2 = copy_child(&mut tmp, b"l2")?;
    check_ok!(syscall::symlink(b"file\0", &l1), "l1");
    check_ok!(syscall::symlink(b"l1\0", &l2), "l2");
    let st = check_ok!(syscall::stat(&l2), "stat chain");
    let ft = check_ok!(syscall::stat(&file), "stat file");
    check_eq!(st.st_ino, ft.st_ino, "same inode");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn symlink_empty_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"link")?;
    // Empty symlink targets are rejected on Linux (EINVAL/ENOENT).
    match syscall::symlink(b"\0", &link) {
        Err(crate::syscall::Errno::EINVAL) | Err(crate::syscall::Errno::ENOENT) => Ok(()),
        Ok(()) => {
            let mut buf = [0u8; 4];
            let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
            check_eq!(n, 0, "empty target");
            Ok(())
        }
        Err(_) => Err(crate::harness::AssertFail::msg(
            "symlink empty unexpected errno",
        )),
    }
}
