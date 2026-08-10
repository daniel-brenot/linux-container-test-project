//! POSIX errno semantics tests.

use crate::check;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = posix)]
fn errno_enoent_open() -> TestResult {
    check_err!(
        syscall::open(b"/tmp/lctp-missing-open\0", oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "open missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_stat() -> TestResult {
    check_err!(
        syscall::stat(b"/tmp/lctp-missing-stat\0"),
        Errno::ENOENT,
        "stat missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_unlink() -> TestResult {
    check_err!(
        syscall::unlink(b"/tmp/lctp-missing-unlink\0"),
        Errno::ENOENT,
        "unlink missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eisdir_open_write() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_err!(
        syscall::open(tmp.path(), oflag::O_RDWR, 0),
        Errno::EISDIR,
        "O_RDWR dir"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eisdir_unlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    match syscall::unlink(&dir) {
        Err(Errno::EISDIR) | Err(Errno::EPERM) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("unlink dir succeeded")),
        Err(_) => return Err(crate::harness::AssertFail::msg("unlink dir errno")),
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_read() -> TestResult {
    check_err!(syscall::read(-1, &mut [0u8; 1]), Errno::EBADF, "read bad fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_write() -> TestResult {
    check_err!(syscall::write(-1, b"x"), Errno::EBADF, "write bad fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_close() -> TestResult {
    check_err!(syscall::close(-1), Errno::EBADF, "close bad fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_einval_lseek() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_err!(
        syscall::lseek(fd, 0, 999),
        Errno::EINVAL,
        "invalid whence"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotdir_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut bad = [0u8; 160];
    let end = file.iter().position(|&c| c == 0).unwrap();
    // Append "/x" so a regular file is used as a directory component.
    check!(end + 3 < bad.len(), "path too long");
    bad[..end].copy_from_slice(&file[..end]);
    bad[end..end + 2].copy_from_slice(b"/x");
    bad[end + 2] = 0;
    check_err!(
        syscall::open(crate::suites::common::truncate_cstr(&bad), oflag::O_RDONLY, 0),
        Errno::ENOTDIR,
        "file as dir component"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eexist_mkdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"d")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    check_err!(syscall::mkdir(&dir, 0o755), Errno::EEXIST, "mkdir again");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotempty_rmdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    check!(dlen + 6 < nested.len(), "path too long");
    nested[..dlen].copy_from_slice(&dir[..dlen]);
    nested[dlen..dlen + 5].copy_from_slice(b"/file");
    nested[dlen + 5] = 0;
    let path = crate::suites::common::truncate_cstr(&nested);
    let fd = check_ok!(
        syscall::open(path, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "create nested"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(syscall::rmdir(&dir), Errno::ENOTEMPTY, "rmdir nonempty");
    check_ok!(syscall::unlink(path), "unlink nested");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}
