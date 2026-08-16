//! truncate/ftruncate filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, write_file};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = fs, expect = success, case = "ftruncate shrinks a regular file to the requested size")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "ftruncate grows a regular file to the requested size")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "ftruncate to 0 sets the file size to 0")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "truncate shrinks a regular file to the requested size")]
fn truncate_path_shrink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"0123456789")?;
    check_ok!(syscall::truncate(&path, 3), "truncate");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 3, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "truncate grows a regular file to the requested size")]
fn truncate_path_grow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"ab")?;
    check_ok!(syscall::truncate(&path, 500), "truncate grow");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 500, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "ftruncate to 1000000 sets the logical file size")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "ftruncate to the current size leaves the size unchanged")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "read after truncate returns the remaining prefix")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "truncate that grows a file fills the extension with zeros")]
fn truncate_grows_zeros() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"AB")?;
    check_ok!(syscall::truncate(&path, 6), "grow");
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 6, "len");
    check_eq!(&buf[..2], b"AB", "prefix");
    check_eq!(&buf[2..6], &[0, 0, 0, 0], "zeros");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "ftruncate that grows a file fills the extension with zeros")]
fn ftruncate_grows_zeros() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"X"), "write");
    check_ok!(syscall::ftruncate(fd, 4), "grow");
    check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 4, "len");
    check_eq!(buf[0], b'X', "first");
    check_eq!(buf[1], 0, "z1");
    check_eq!(buf[2], 0, "z2");
    check_eq!(buf[3], 0, "z3");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "truncate of a missing path returns ENOENT")]
fn truncate_enoent() -> TestResult {
    check_err!(
        syscall::truncate(b"/tmp/lctp-trunc-missing\0", 0),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "truncate of a directory returns EISDIR")]
fn truncate_dir_eisdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::truncate(&dir, 0), Errno::EISDIR, "eisdir");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "ftruncate on fd -1 returns EBADF")]
fn ftruncate_bad_fd() -> TestResult {
    check_err!(syscall::ftruncate(-1, 0), Errno::EBADF, "ebadf");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "ftruncate on a read-only fd returns EINVAL or EBADF")]
fn ftruncate_rdonly_einval() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"data")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "rdonly");
    match syscall::ftruncate(fd, 1) {
        Err(Errno::EINVAL) | Err(Errno::EBADF) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("ftruncate rdonly ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("ftruncate rdonly errno")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "truncate to the current size leaves the size unchanged")]
fn truncate_to_same_size() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"abcd")?;
    check_ok!(syscall::truncate(&path, 4), "same");
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 4, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "truncate to 1 byte leaves the first byte")]
fn truncate_one_byte() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"YZ")?;
    check_ok!(syscall::truncate(&path, 1), "trunc");
    let mut buf = [0u8; 2];
    check_eq!(crate::suites::common::read_file(&path, &mut buf)?, 1, "len");
    check_eq!(buf[0], b'Y', "data");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "truncate of a symlink follows it and shrinks the target")]
fn truncate_via_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    write_file(&file, b"123456")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"f\0", &link), "symlink");
    check_ok!(syscall::truncate(&link, 2), "truncate link");
    check_eq!(check_ok!(syscall::stat(&file), "stat").st_size, 2, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "truncate through a non-directory path component returns ENOTDIR")]
fn truncate_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"x\0", &mut nested)?;
    check_err!(syscall::truncate(path, 0), Errno::ENOTDIR, "enotdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "ftruncate grows a file to 10000 bytes")]
fn ftruncate_large_grow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 10_000), "grow");
    let path = copy_child(&mut tmp, b"f")?;
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 10_000, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "append after truncate writes after the truncated prefix")]
fn truncate_then_append() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"ABCDEF")?;
    check_ok!(syscall::truncate(&path, 3), "trunc");
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0),
        "append"
    );
    check_ok!(syscall::write(fd, b"Z"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 8];
    check_eq!(crate::suites::common::read_file(&path, &mut buf)?, 4, "len");
    check_eq!(&buf[..4], b"ABCZ", "data");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "ftruncate that shrinks a file preserves the remaining prefix")]
fn ftruncate_shrink_preserves_prefix() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"HELLO WORLD"), "write");
    check_ok!(syscall::ftruncate(fd, 5), "shrink");
    check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 5, "len");
    check_eq!(&buf[..5], b"HELLO", "prefix");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "truncate of an empty file to 8 bytes reads as zeros")]
fn truncate_empty_file_grow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::truncate(&path, 8), "grow");
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 8, "len");
    check!(&buf.iter().all(|&b| b == 0), "all zeros");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = soft, case = "truncate of a mode 0000 file succeeds or returns EACCES")]
fn truncate_chmod0_still_ok() -> TestResult {
    // truncate uses path; owner can still truncate on Linux even if mode is 000.
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"data")?;
    check_ok!(syscall::chmod(&path, 0o000), "chmod");
    match syscall::truncate(&path, 1) {
        Ok(()) => {}
        Err(Errno::EACCES) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("truncate mode0 errno")),
    }
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}
