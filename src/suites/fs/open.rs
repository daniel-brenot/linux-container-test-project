//! open/creat filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, write_file};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = fs)]
fn open_creat_new() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"new")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_reg(), "regular");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_excl_fails_if_exists() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "excl"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_trunc_zeroes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"long data here")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_append_mode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"X")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0), "append");
    check_ok!(syscall::write(fd, b"Y"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "read");
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 2, "len");
    check_eq!(&buf[..2], b"XY", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_directory_on_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "O_DIRECTORY"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_directory_on_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        Errno::ENOTDIR,
        "ENOTDIR"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_readonly_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "rdonly");
    check_err!(syscall::write(fd, b"x"), Errno::EBADF, "no write");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_wronly_no_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "wronly");
    check_err!(syscall::read(fd, &mut [0u8; 1]), Errno::EBADF, "no read");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn creat_in_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"sub", 0o755)?;
    let mut nested = [0u8; 160];
    let suffix = b"/x";
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    check!(dlen + suffix.len() + 1 < nested.len(), "path too long");
    nested[..dlen].copy_from_slice(&dir[..dlen]);
    nested[dlen..dlen + suffix.len()].copy_from_slice(suffix);
    nested[dlen + suffix.len()] = 0;
    let path = crate::suites::common::truncate_cstr(&nested);
    let fd = check_ok!(
        syscall::open(path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(path), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_existing_no_trunc() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"keep")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 8];
    check_eq!(crate::suites::common::read_file(&path, &mut buf)?, 4, "len");
    check_eq!(&buf[..4], b"keep", "preserved");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_opath_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"opath")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_PATH, 0), "O_PATH");
    // O_PATH fds reject normal read/write with EBADF.
    check_err!(syscall::read(fd, &mut [0u8; 1]), Errno::EBADF, "no read");
    check_err!(syscall::write(fd, b"x"), Errno::EBADF, "no write");
    // fstat still works on O_PATH.
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check!(st.is_reg(), "reg");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn open_opath_directory() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(tmp.path(), oflag::O_PATH | oflag::O_DIRECTORY, 0),
        "O_PATH dir"
    );
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check!(st.is_dir(), "dir");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn open_opath_symlink_nofollow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"tgt")?;
    let link = copy_child(&mut tmp, b"lnk")?;
    check_ok!(syscall::symlink(b"tgt\0", &link), "symlink");
    let fd = check_ok!(
        syscall::open(&link, oflag::O_PATH | oflag::O_NOFOLLOW, 0),
        "O_PATH|O_NOFOLLOW"
    );
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check!(st.is_lnk(), "symlink");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
