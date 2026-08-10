//! hard link filesystem tests.

use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty};
use crate::syscall::{self, Errno};

#[crate::lctp_test(suite = fs)]
fn link_same_inode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let sa = check_ok!(syscall::stat(&a), "stat a");
    let sb = check_ok!(syscall::stat(&b), "stat b");
    check_eq!(sa.st_ino, sb.st_ino, "inode");
    check_eq!(sa.st_dev, sb.st_dev, "dev");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn link_nlink_two() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let st = check_ok!(syscall::stat(&a), "stat");
    check_eq!(st.st_nlink, 2, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn link_missing_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dst = copy_child(&mut tmp, b"dst")?;
    check_err!(
        syscall::link(b"/tmp/lctp-no-src\0", &dst),
        Errno::ENOENT,
        "missing src"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn link_to_directory_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"dir", 0o755)?;
    let link = copy_child(&mut tmp, b"link")?;
    match syscall::link(&dir, &link) {
        Err(Errno::EPERM) | Err(Errno::EACCES) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("link dir ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("link dir errno")),
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn link_existing_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    check_err!(syscall::link(&a, &b), Errno::EEXIST, "link over existing");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn link_share_content() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"a", 0o644), "create");
    check_ok!(syscall::write(fd, b"XY"), "write");
    check_ok!(syscall::close(fd), "close");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let fd = check_ok!(syscall::open(&b, crate::syscall::oflag::O_RDONLY, 0), "open b");
    let mut buf = [0u8; 2];
    check_ok!(syscall::read(fd, &mut buf), "read");
    check_eq!(&buf, b"XY", "shared");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn link_unlink_one_remaining() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    check_ok!(syscall::unlink(&a), "unlink a");
    check_ok!(syscall::stat(&b), "b remains");
    check_ok!(syscall::unlink(&b), "unlink b");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn link_multiple_hardlinks() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    let c = copy_child(&mut tmp, b"c")?;
    check_ok!(syscall::link(&a, &b), "link b");
    check_ok!(syscall::link(&a, &c), "link c");
    let st = check_ok!(syscall::stat(&a), "stat");
    check_eq!(st.st_nlink, 3, "nlink 3");
    Ok(())
}
