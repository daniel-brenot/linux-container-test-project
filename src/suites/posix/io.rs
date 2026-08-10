//! POSIX I/O semantics tests.

use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, read_file, write_file};
use crate::syscall::{self, oflag};

#[crate::lctp_test(suite = posix)]
fn pipe_eof_after_close_writer() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::close(w), "close w");
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::read(r, &mut buf), "read");
    check_eq!(n, 0, "EOF");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn pipe_read_partial() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, b"12345"), "write");
    let mut small = [0u8; 2];
    check_eq!(check_ok!(syscall::read(r, &mut small), "read"), 2, "partial");
    check_eq!(&small, b"12", "bytes");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn read_returns_available() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"abcdef")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 3];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 3, "len");
    check_eq!(&buf, b"abc", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn append_atomic_sequence() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"log")?;
    write_file(&path, b"1")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0), "append");
    check_ok!(syscall::write(fd, b"2"), "w2");
    check_ok!(syscall::write(fd, b"3"), "w3");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 8];
    check_eq!(read_file(&path, &mut buf)?, 3, "len");
    check_eq!(&buf[..3], b"123", "content");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn write_at_offset_pread() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::pwrite(fd, b"ZZZZ", 0), "pwrite");
    check_ok!(syscall::pwrite(fd, b"AB", 1), "pwrite mid");
    let mut buf = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, b"ZABZ", "content");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn sequential_read_advances() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcd"), "write");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut one = [0u8; 1];
    check_ok!(syscall::read(fd, &mut one), "r1");
    check_eq!(one[0], b'a', "a");
    check_ok!(syscall::read(fd, &mut one), "r2");
    check_eq!(one[0], b'b', "b");
    let pos = check_ok!(syscall::lseek(fd, 0, syscall::SEEK_CUR), "cur");
    check_eq!(pos, 2, "offset");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn empty_file_read_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"empty")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 0, "EOF");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn pipe_blocking_write_read() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let msg = b"block-test";
    check_eq!(check_ok!(syscall::write(w, msg), "write"), msg.len(), "wlen");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), msg.len(), "rlen");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}
