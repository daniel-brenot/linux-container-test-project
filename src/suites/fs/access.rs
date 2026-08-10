//! access/faccessat filesystem tests.

use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, write_file};
use crate::syscall::{self, oflag, Errno, F_OK, R_OK, W_OK, X_OK};

#[crate::lctp_test(suite = fs)]
fn access_f_ok_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::access(&path, F_OK), "F_OK");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_f_ok_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::access(&dir, F_OK), "F_OK");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_f_ok_missing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"missing")?;
    check_err!(syscall::access(&path, F_OK), Errno::ENOENT, "enoent");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_r_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o400), "chmod");
    check_ok!(syscall::access(&path, R_OK), "R_OK");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_w_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o200), "chmod");
    check_ok!(syscall::access(&path, W_OK), "W_OK");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_x_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o100), "chmod");
    check_ok!(syscall::access(&path, X_OK), "X_OK");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_rw_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    check_ok!(syscall::access(&path, R_OK | W_OK), "RW");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_rx_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o500), "chmod");
    check_ok!(syscall::access(&path, R_OK | X_OK), "RX");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_wx_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o300), "chmod");
    check_ok!(syscall::access(&path, W_OK | X_OK), "WX");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_rwx_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o700), "chmod");
    check_ok!(syscall::access(&path, R_OK | W_OK | X_OK), "RWX");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_r_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o200), "chmod");
    check_err!(syscall::access(&path, R_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_w_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o400), "chmod");
    check_err!(syscall::access(&path, W_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_x_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    check_err!(syscall::access(&path, X_OK), Errno::EACCES, "eacces");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_zero_mode_eacces_all() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o000), "chmod");
    check_err!(syscall::access(&path, R_OK), Errno::EACCES, "R");
    check_err!(syscall::access(&path, W_OK), Errno::EACCES, "W");
    check_err!(syscall::access(&path, X_OK), Errno::EACCES, "X");
    check_ok!(syscall::access(&path, F_OK), "F_OK still");
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_dir_x_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::access(&dir, X_OK), "search");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_dir_no_search() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o600), "chmod");
    check_err!(syscall::access(&dir, X_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_follows_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&file, 0o400), "chmod");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"f\0", &link), "symlink");
    check_ok!(syscall::access(&link, R_OK), "R_OK follow");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_dangling_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"missing\0", &link), "symlink");
    check_err!(syscall::access(&link, F_OK), Errno::ENOENT, "enoent");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn faccessat_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let dirfd = check_ok!(
        syscall::open(&dir, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_ok!(syscall::faccessat(dirfd, b"f\0", F_OK, 0), "faccessat");
    check_ok!(syscall::close(dirfd), "close");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn faccessat_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o000), "chmod");
    check_err!(
        syscall::faccessat(syscall::AT_FDCWD, &path, R_OK, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_parent_no_search() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"f\0", &mut nested)?;
    write_file(child, b"x")?;
    check_ok!(syscall::chmod(&dir, 0o000), "chmod");
    check_err!(syscall::access(child, F_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"x\0", &mut nested)?;
    check_err!(syscall::access(path, F_OK), Errno::ENOTDIR, "enotdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_loop_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::symlink(b"b\0", &a), "a");
    check_ok!(syscall::symlink(b"a\0", &b), "b");
    check_err!(syscall::access(&a, F_OK), Errno::ELOOP, "eloop");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_dir_r_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::access(&dir, R_OK), "R_OK");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_dir_w_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::access(&dir, W_OK), "W_OK");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_dir_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o555), "chmod");
    check_err!(syscall::access(&dir, W_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_fifo_f_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, crate::syscall::S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_ok!(syscall::access(&path, F_OK), "F_OK");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_after_unlink_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::unlink(&path), "unlink");
    check_err!(syscall::access(&path, F_OK), Errno::ENOENT, "enoent");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_rw_partial_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o400), "chmod");
    check_err!(syscall::access(&path, R_OK | W_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_tmp_path_f_ok() -> TestResult {
    check_ok!(syscall::access(b"/tmp\0", F_OK), "tmp");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_tmp_path_x_ok() -> TestResult {
    check_ok!(syscall::access(b"/tmp\0", X_OK), "tmp x");
    Ok(())
}

macro_rules! access_mode_ok {
    ($name:ident, $mode_chmod:expr, $mode_access:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $mode_chmod), "chmod");
            check_ok!(syscall::access(&path, $mode_access), "access");
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

macro_rules! access_mode_eacces {
    ($name:ident, $mode_chmod:expr, $mode_access:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $mode_chmod), "chmod");
            check_err!(syscall::access(&path, $mode_access), Errno::EACCES, "eacces");
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

access_mode_ok!(access_ok_700_r, 0o700, R_OK);
access_mode_ok!(access_ok_700_w, 0o700, W_OK);
access_mode_ok!(access_ok_700_x, 0o700, X_OK);
access_mode_ok!(access_ok_600_r, 0o600, R_OK);
access_mode_ok!(access_ok_600_w, 0o600, W_OK);
access_mode_ok!(access_ok_500_r, 0o500, R_OK);
access_mode_ok!(access_ok_500_x, 0o500, X_OK);
access_mode_ok!(access_ok_300_w, 0o300, W_OK);
access_mode_ok!(access_ok_300_x, 0o300, X_OK);
access_mode_ok!(access_ok_755_r, 0o755, R_OK);
access_mode_ok!(access_ok_755_w, 0o755, W_OK);
access_mode_ok!(access_ok_755_x, 0o755, X_OK);
access_mode_ok!(access_ok_644_r, 0o644, R_OK);
access_mode_ok!(access_ok_644_w, 0o644, W_OK);
access_mode_ok!(access_ok_555_r, 0o555, R_OK);
access_mode_ok!(access_ok_555_x, 0o555, X_OK);
access_mode_ok!(access_ok_111_x, 0o111, X_OK);
access_mode_ok!(access_ok_222_w, 0o222, W_OK);
access_mode_ok!(access_ok_444_r, 0o444, R_OK);
access_mode_ok!(access_ok_711_x, 0o711, X_OK);

access_mode_eacces!(access_deny_200_r, 0o200, R_OK);
access_mode_eacces!(access_deny_200_x, 0o200, X_OK);
access_mode_eacces!(access_deny_400_w, 0o400, W_OK);
access_mode_eacces!(access_deny_400_x, 0o400, X_OK);
access_mode_eacces!(access_deny_100_r, 0o100, R_OK);
access_mode_eacces!(access_deny_100_w, 0o100, W_OK);
access_mode_eacces!(access_deny_000_r, 0o000, R_OK);
access_mode_eacces!(access_deny_000_w, 0o000, W_OK);
access_mode_eacces!(access_deny_000_x, 0o000, X_OK);
access_mode_eacces!(access_deny_444_w, 0o444, W_OK);
access_mode_eacces!(access_deny_444_x, 0o444, X_OK);
access_mode_eacces!(access_deny_222_r, 0o222, R_OK);
access_mode_eacces!(access_deny_222_x, 0o222, X_OK);
access_mode_eacces!(access_deny_111_r, 0o111, R_OK);
access_mode_eacces!(access_deny_111_w, 0o111, W_OK);
access_mode_eacces!(access_deny_600_x, 0o600, X_OK);
access_mode_eacces!(access_deny_640_x, 0o640, X_OK);
access_mode_eacces!(access_deny_620_x, 0o620, X_OK);
access_mode_eacces!(access_deny_460_w, 0o460, W_OK);
access_mode_eacces!(access_deny_240_r, 0o240, R_OK);

#[crate::lctp_test(suite = fs)]
fn access_dir_555_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o555), "chmod");
    check_ok!(syscall::access(&dir, R_OK | X_OK), "rx");
    check_err!(syscall::access(&dir, W_OK), Errno::EACCES, "w");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_dir_111_search_only() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o111), "chmod");
    check_ok!(syscall::access(&dir, X_OK), "x");
    check_err!(syscall::access(&dir, R_OK), Errno::EACCES, "r");
    check_err!(syscall::access(&dir, W_OK), Errno::EACCES, "w");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn faccessat_w_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o200), "chmod");
    check_ok!(
        syscall::faccessat(syscall::AT_FDCWD, &path, W_OK, 0),
        "W_OK"
    );
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn faccessat_x_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    check_err!(
        syscall::faccessat(syscall::AT_FDCWD, &path, X_OK, 0),
        Errno::EACCES,
        "eacces"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn access_symlink_target_no_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&file, 0o000), "chmod");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"f\0", &link), "symlink");
    check_err!(syscall::access(&link, R_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&file, 0o644), "restore");
    Ok(())
}
