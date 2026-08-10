//! stat/lstat/fstat filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, write_file};
use crate::syscall::{self, oflag, Errno, S_IFIFO};

#[crate::lctp_test(suite = fs)]
fn stat_regular_type() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_reg(), "reg");
    check!(!st.is_dir(), "not dir");
    check!(!st.is_lnk(), "not lnk");
    check!(!st.is_fifo(), "not fifo");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_dir_type() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "dir");
    check!(!st.is_reg(), "not reg");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_fifo_type() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_fifo(), "fifo");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_symlink_type_follow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    check!(check_ok!(syscall::stat(&link), "stat").is_reg(), "follow");
    check!(check_ok!(syscall::lstat(&link), "lstat").is_lnk(), "lnk");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_size_empty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_size_written() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"hello!")?;
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 6, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_nlink_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_nlink, 1, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_nlink_hardlinks() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    check_eq!(check_ok!(syscall::stat(&a), "stat").st_nlink, 2, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_nlink_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    // empty dir: typically nlink >= 2 (. and ..)
    check!(check_ok!(syscall::stat(&dir), "stat").st_nlink >= 2, "nlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fstat_matches_stat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"xyz")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let fs = check_ok!(syscall::fstat(fd), "fstat");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(fs.st_ino, st.st_ino, "ino");
    check_eq!(fs.st_size, st.st_size, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_enoent() -> TestResult {
    check_err!(
        syscall::stat(b"/tmp/lctp-stat-missing\0"),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn lstat_enoent() -> TestResult {
    check_err!(
        syscall::lstat(b"/tmp/lctp-lstat-missing\0"),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fstat_bad_fd() -> TestResult {
    check_err!(syscall::fstat(-1), Errno::EBADF, "ebadf");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_uid_gid_self() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_uid, syscall::getuid(), "uid");
    check_eq!(st.st_gid, syscall::getgid(), "gid");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_mode_bits_644() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o644), "chmod");
    check_eq!(
        check_ok!(syscall::stat(&path), "stat").mode_bits() & 0o777,
        0o644,
        "mode"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_blksize_positive() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check!(check_ok!(syscall::stat(&path), "stat").st_blksize > 0, "blksize");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_dev_nonzero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check!(check_ok!(syscall::stat(&path), "stat").st_dev != 0, "dev");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_ino_nonzero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check!(check_ok!(syscall::stat(&path), "stat").st_ino != 0, "ino");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_two_files_different_ino() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    check!(
        check_ok!(syscall::stat(&a), "a").st_ino != check_ok!(syscall::stat(&b), "b").st_ino,
        "distinct"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn lstat_symlink_size_is_target_len() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"abcdef\0", &link), "symlink");
    check_eq!(check_ok!(syscall::lstat(&link), "lstat").st_size, 6, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_after_truncate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"0123456789")?;
    check_ok!(syscall::truncate(&path, 4), "trunc");
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 4, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fstatat_cwd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let st = check_ok!(syscall::fstatat(syscall::AT_FDCWD, &path, 0), "fstatat");
    check!(st.is_reg(), "reg");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fstatat_nofollow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    let st = check_ok!(
        syscall::fstatat(syscall::AT_FDCWD, &link, crate::syscall::AT_SYMLINK_NOFOLLOW),
        "fstatat"
    );
    check!(st.is_lnk(), "lnk");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_timestamps_nonneg() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.st_atime >= 0, "atime");
    check!(st.st_mtime >= 0, "mtime");
    check!(st.st_ctime >= 0, "ctime");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_dir_mode_bits() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o700)?;
    check_eq!(
        check_ok!(syscall::stat(&dir), "stat").mode_bits() & 0o777,
        0o700,
        "mode"
    );
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_same_dev_tmpdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    check_eq!(
        check_ok!(syscall::stat(&a), "a").st_dev,
        check_ok!(syscall::stat(&b), "b").st_dev,
        "same dev"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn lstat_dangling_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"dangle")?;
    check_ok!(syscall::symlink(b"nope\0", &link), "symlink");
    check!(check_ok!(syscall::lstat(&link), "lstat").is_lnk(), "lnk");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_blocks_nonneg() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"data")?;
    check!(check_ok!(syscall::stat(&path), "stat").st_blocks >= 0, "blocks");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fstat_after_write_size() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"abcd"), "write");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 4, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_loop_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::symlink(b"b\0", &a), "a");
    check_ok!(syscall::symlink(b"a\0", &b), "b");
    check_err!(syscall::stat(&a), Errno::ELOOP, "eloop");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn stat_size_large_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "open");
    let chunk = [b'Z'; 64];
    for _ in 0..16 {
        check_ok!(syscall::write(fd, &chunk), "write");
    }
    check_ok!(syscall::close(fd), "close");
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 1024, "size");
    Ok(())
}
