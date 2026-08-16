//! statx(2) tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty, write_file};
use crate::syscall::{self, oflag, AT_SYMLINK_NOFOLLOW, STATX_BASIC_STATS, Statx};

#[crate::lctp_test(suite = syscall, expect = success, case = "statx basic stats match fstat size, inode, and mode bits")]
fn statx_basic_vs_fstat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"sx")?;
    write_file(&path, b"statx-data")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let st = check_ok!(syscall::fstat(fd), "fstat");
    let mut sx = Statx::default();
    check_ok!(
        syscall::statx(syscall::AT_FDCWD, &path, 0, STATX_BASIC_STATS, &mut sx),
        "statx"
    );
    check!(sx.is_reg(), "reg");
    check_eq!(sx.stx_size, st.st_size as u64, "size");
    check_eq!(sx.stx_ino, st.st_ino, "ino");
    check_eq!(sx.mode_bits() & 0o777, st.mode_bits() & 0o777, "mode");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statx reports size 0 for an empty regular file")]
fn statx_empty_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"empty")?;
    let mut sx = Statx::default();
    check_ok!(
        syscall::statx(syscall::AT_FDCWD, &path, 0, STATX_BASIC_STATS, &mut sx),
        "statx"
    );
    check_eq!(sx.stx_size, 0, "size");
    check!(sx.is_reg(), "reg");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statx follows a symlink by default and reports a symlink with AT_SYMLINK_NOFOLLOW")]
fn statx_symlink_nofollow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"target")?;
    write_file(&file, b"abcdef")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");

    let mut followed = Statx::default();
    check_ok!(
        syscall::statx(syscall::AT_FDCWD, &link, 0, STATX_BASIC_STATS, &mut followed),
        "statx follow"
    );
    check!(followed.is_reg(), "followed reg");
    check_eq!(followed.stx_size, 6, "followed size");

    let mut nofollow = Statx::default();
    check_ok!(
        syscall::statx(
            syscall::AT_FDCWD,
            &link,
            AT_SYMLINK_NOFOLLOW,
            STATX_BASIC_STATS,
            &mut nofollow
        ),
        "statx nofollow"
    );
    check!(nofollow.is_lnk(), "symlink type");
    check_eq!(nofollow.stx_size, 6, "link path len");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statx reports a directory mode for a directory path")]
fn statx_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let mut sx = Statx::default();
    check_ok!(
        syscall::statx(syscall::AT_FDCWD, tmp.path(), 0, STATX_BASIC_STATS, &mut sx),
        "statx dir"
    );
    check_eq!(sx.stx_mode as u32 & 0o170000, 0o040000, "dir mode");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "statx with STATX_SIZE|STATX_INO returns size and a nonzero inode")]
fn statx_mask_partial() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"m")?;
    write_file(&path, b"xy")?;
    let mut sx = Statx::default();
    check_ok!(
        syscall::statx(
            syscall::AT_FDCWD,
            &path,
            0,
            syscall::STATX_SIZE | syscall::STATX_INO,
            &mut sx
        ),
        "statx partial"
    );
    check_eq!(sx.stx_size, 2, "size");
    check!(sx.stx_ino != 0, "ino");
    Ok(())
}
