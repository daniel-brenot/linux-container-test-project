//! chmod/fchmod filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty};
use crate::syscall::{self, oflag, Errno, AT_SYMLINK_NOFOLLOW};

fn assert_mode(path: &[u8], mode: u32) -> TestResult {
    let st = check_ok!(syscall::stat(path), "stat");
    check_eq!(st.mode_bits() & 0o777, mode, "mode");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn chmod_file_644() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o644), "chmod");
    assert_mode(&path, 0o644)
}

#[crate::lctp_test(suite = fs)]
fn chmod_file_600() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    assert_mode(&path, 0o600)
}

#[crate::lctp_test(suite = fs)]
fn chmod_file_755() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o755), "chmod");
    assert_mode(&path, 0o755)
}

#[crate::lctp_test(suite = fs)]
fn chmod_file_444() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o444), "chmod");
    assert_mode(&path, 0o444)
}

#[crate::lctp_test(suite = fs, full)]
fn chmod_file_777() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o777), "chmod");
    assert_mode(&path, 0o777)
}

#[crate::lctp_test(suite = fs)]
fn chmod_dir_700() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o700), "chmod");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "dir");
    check_eq!(st.mode_bits() & 0o777, 0o700, "mode");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn chmod_dir_755() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o700)?;
    check_ok!(syscall::chmod(&dir, 0o755), "chmod");
    assert_mode(&dir, 0o755)?;
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fchmod_matches_chmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fchmod(fd, 0o640), "fchmod");
    check_ok!(syscall::close(fd), "close");
    assert_mode(&path, 0o640)
}

#[crate::lctp_test(suite = fs, full)]
fn chmod_symlink_follow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    check_ok!(syscall::chmod(&link, 0o644), "chmod link");
    // Linux chmod follows symlinks — target mode changes.
    let st = check_ok!(syscall::stat(&file), "stat target");
    check_eq!(st.mode_bits() & 0o777, 0o644, "target mode");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn fchmodat_symlink_nofollow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _file = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    // Linux does not implement fchmodat(AT_SYMLINK_NOFOLLOW); expect ENOTSUP.
    match syscall::fchmodat(syscall::AT_FDCWD, &link, 0o600, AT_SYMLINK_NOFOLLOW) {
        Err(Errno::EOPNOTSUPP) | Err(Errno::ENOSYS) => Ok(()),
        Err(e) if e.0 == 95 => Ok(()), // EOPNOTSUPP alternate
        Ok(()) => Ok(()),               // some FS may allow it
        Err(_) => Err(crate::harness::AssertFail::msg(
            "unexpected fchmodat(AT_SYMLINK_NOFOLLOW) errno",
        )),
    }
}

#[crate::lctp_test(suite = fs)]
fn chmod_clear_group_other() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o777), "chmod wide");
    check_ok!(syscall::chmod(&path, 0o600), "chmod narrow");
    assert_mode(&path, 0o600)
}

#[crate::lctp_test(suite = fs)]
fn chmod_set_executable() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o755), "chmod +x");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.mode_bits() & 0o111 != 0, "execute bits");
    Ok(())
}
