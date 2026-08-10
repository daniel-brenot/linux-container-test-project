//! unlink filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, write_file};
use crate::syscall::{self, oflag, Errno, S_IFIFO};

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
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"data"), "write");
    check_ok!(syscall::unlink(&path), "unlink");
    let mut buf = [0u8; 4];
    check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 4, "read deleted");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_fifo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_ok!(syscall::unlink(&path), "unlink");
    check_err!(syscall::stat(&path), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_twice_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::unlink(&path), "first");
    check_err!(syscall::unlink(&path), Errno::ENOENT, "second");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_enotdir_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"x\0", &mut nested)?;
    check_err!(syscall::unlink(path), Errno::ENOTDIR, "enotdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_in_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_parent_no_write_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chmod(&dir, 0o555), "chmod");
    check_err!(syscall::unlink(child), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_parent_no_search_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chmod(&dir, 0o000), "chmod");
    check_err!(syscall::unlink(child), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlinkat_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let dirfd = check_ok!(
        syscall::open(&dir, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    let fd = check_ok!(
        syscall::openat(dirfd, b"f\0", oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlinkat(dirfd, b"f\0", 0), "unlinkat");
    check_ok!(syscall::close(dirfd), "close dir");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_nlink_three_to_two() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    let c = copy_child(&mut tmp, b"c")?;
    check_ok!(syscall::link(&a, &b), "link b");
    check_ok!(syscall::link(&a, &c), "link c");
    check_ok!(syscall::unlink(&c), "unlink c");
    check_eq!(check_ok!(syscall::stat(&a), "stat").st_nlink, 2, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_symlink_to_missing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"dangling")?;
    check_ok!(syscall::symlink(b"nowhere\0", &link), "symlink");
    check_ok!(syscall::unlink(&link), "unlink");
    check_err!(syscall::lstat(&link), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_after_rename() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::rename(&a, &b), "rename");
    check_ok!(syscall::unlink(&b), "unlink");
    check_err!(syscall::stat(&b), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_empty_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"empty")?;
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_nonzero_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"nz")?;
    write_file(&path, b"payload")?;
    check_ok!(syscall::unlink(&path), "unlink");
    check_err!(syscall::stat(&path), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_dot_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut dot = [0u8; 160];
    let path = join_path(&dir, b".\0", &mut dot)?;
    match syscall::unlink(path) {
        Err(Errno::EISDIR) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("unlink . ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("unlink . errno")),
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlinkat_enoent() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let dirfd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_err!(
        syscall::unlinkat(dirfd, b"missing\0", 0),
        Errno::ENOENT,
        "enoent"
    );
    check_ok!(syscall::close(dirfd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn unlink_many_sequential() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    for name in [b"u0\0".as_slice(), b"u1\0", b"u2\0", b"u3\0", b"u4\0"] {
        let path = copy_child(&mut tmp, name)?;
        let fd = check_ok!(
            syscall::open(&path, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
            "creat"
        );
        check_ok!(syscall::close(fd), "close");
        check_ok!(syscall::unlink(&path), "unlink");
    }
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn unlink_open_then_recreate_name() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::unlink(&path), "unlink");
    let fd2 = check_ok!(
        syscall::open(&path, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "recreat"
    );
    check_ok!(syscall::write(fd2, b"new"), "write new");
    check_ok!(syscall::close(fd2), "close2");
    check_ok!(syscall::write(fd, b"old"), "write old fd");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    let n = crate::suites::common::read_file(&path, &mut buf)?;
    check_eq!(n, 3, "len");
    check_eq!(&buf[..3], b"new", "new content");
    Ok(())
}
