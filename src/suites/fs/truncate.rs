//! truncate/ftruncate filesystem tests.

use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty, write_file};
use crate::syscall::{self, oflag};

#[crate::lctp_test(suite = fs)]
fn ftruncate_shrink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"1234567890"), "write");
    check_ok!(syscall::ftruncate(fd, 4), "ftruncate");
    let path = copy_child(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 4, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn ftruncate_grow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"ab"), "write");
    check_ok!(syscall::ftruncate(fd, 1000), "grow");
    let path = copy_child(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 1000, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn ftruncate_zero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"data"), "write");
    check_ok!(syscall::ftruncate(fd, 0), "zero");
    let path = copy_child(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 0, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn truncate_path_shrink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"0123456789")?;
    check_ok!(syscall::truncate(&path, 3), "truncate");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 3, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn truncate_path_grow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"ab")?;
    check_ok!(syscall::truncate(&path, 500), "truncate grow");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 500, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn truncate_sparseness() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sparse", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 1_000_000), "sparse");
    let path = copy_child(&mut tmp, b"sparse")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 1_000_000, "logical size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn ftruncate_idempotent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"12345"), "write");
    check_ok!(syscall::ftruncate(fd, 5), "same size");
    check_ok!(syscall::ftruncate(fd, 5), "again");
    let path = copy_child(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 5, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn truncate_then_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"ABCDEF")?;
    check_ok!(syscall::truncate(&path, 2), "truncate");
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 2, "len");
    check_eq!(&buf[..2], b"AB", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
