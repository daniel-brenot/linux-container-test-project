//! rmdir filesystem tests.

use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, truncate_cstr};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = fs)]
fn rmdir_empty_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::rmdir(&dir), "rmdir");
    check_err!(syscall::stat(&dir), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rmdir_notempty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    nested[..dlen].copy_from_slice(&dir[..dlen]);
    nested[dlen..dlen + 5].copy_from_slice(b"/file");
    nested[dlen + 5] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&nested), oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "create"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(syscall::rmdir(&dir), Errno::ENOTEMPTY, "notempty");
    check_ok!(syscall::unlink(truncate_cstr(&nested)), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rmdir_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::close(fd), "close");
    let file = copy_child(&mut tmp, b"f")?;
    check_err!(syscall::rmdir(&file), Errno::ENOTDIR, "rmdir file");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rmdir_dot_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut dot = [0u8; 160];
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    dot[..dlen].copy_from_slice(&dir[..dlen]);
    dot[dlen..dlen + 2].copy_from_slice(b"/.");
    dot[dlen + 2] = 0;
    check_err!(syscall::rmdir(truncate_cstr(&dot)), Errno::EINVAL, "rmdir .");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rmdir_dotdot_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut dotdot = [0u8; 160];
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    dotdot[..dlen].copy_from_slice(&dir[..dlen]);
    dotdot[dlen..dlen + 3].copy_from_slice(b"/..");
    dotdot[dlen + 3] = 0;
    // Linux typically returns ENOTEMPTY for rmdir("dir/..").
    match syscall::rmdir(truncate_cstr(&dotdot)) {
        Err(Errno::ENOTEMPTY) | Err(Errno::EINVAL) | Err(Errno::ENOTDIR) => {}
        Ok(()) => {
            return Err(crate::harness::AssertFail::msg(
                "rmdir(dir/..) unexpectedly succeeded",
            ))
        }
        Err(_) => {
            return Err(crate::harness::AssertFail::msg(
                "rmdir(dir/..) unexpected errno",
            ))
        }
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rmdir_enoent() -> TestResult {
    check_err!(
        syscall::rmdir(b"/tmp/lctp-no-dir\0"),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rmdir_nested_empty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let outer = create_dir(&mut tmp, b"outer", 0o755)?;
    let mut inner = [0u8; 160];
    let olen = outer.iter().position(|&c| c == 0).unwrap();
    inner[..olen].copy_from_slice(&outer[..olen]);
    inner[olen..olen + 6].copy_from_slice(b"/inner");
    inner[olen + 6] = 0;
    check_ok!(syscall::mkdir(truncate_cstr(&inner), 0o755), "mkdir inner");
    check_ok!(syscall::rmdir(truncate_cstr(&inner)), "rmdir inner");
    check_ok!(syscall::rmdir(&outer), "rmdir outer");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn rmdir_after_unlink_contents() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut f = [0u8; 160];
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    f[..dlen].copy_from_slice(&dir[..dlen]);
    f[dlen..dlen + 5].copy_from_slice(b"/file");
    f[dlen + 5] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&f), oflag::O_CREAT | oflag::O_RDWR, 0o644),
        "create"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(truncate_cstr(&f)), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}
