//! unlink filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty};
use crate::syscall::{self, Errno};

#[crate::lctp_test(suite = fs)]
fn unlink_regular_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::unlink(&path), "unlink");
    check_err!(syscall::stat(&path), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_symlink_only() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    check_ok!(syscall::unlink(&link), "unlink link");
    check_err!(syscall::lstat(&link), Errno::ENOENT, "link gone");
    check_ok!(syscall::stat(&copy_child(&mut tmp, b"t")?), "target remains");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_symlink_keeps_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    check_ok!(syscall::unlink(&link), "unlink");
    let st = check_ok!(syscall::stat(&file), "stat");
    check!(st.is_reg(), "target ok");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_directory_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    match syscall::unlink(&dir) {
        Err(Errno::EISDIR) | Err(Errno::EPERM) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("unlink dir ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("unlink dir errno")),
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_enoent() -> TestResult {
    check_err!(
        syscall::unlink(b"/tmp/lctp-fs-missing\0"),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn unlink_hardlink_decrements() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    check_ok!(syscall::unlink(&b), "unlink b");
    let st = check_ok!(syscall::stat(&a), "stat a");
    check_eq!(st.st_nlink, 1, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_last_link_gone() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"x")?;
    check_ok!(syscall::unlink(&path), "unlink");
    check_err!(syscall::stat(&path), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn unlink_open_file_still_accessible() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"data"), "write");
    check_ok!(syscall::unlink(&path), "unlink");
    let mut buf = [0u8; 4];
    check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 4, "read deleted");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
