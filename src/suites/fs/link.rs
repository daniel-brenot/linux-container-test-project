//! hard link filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, write_file};
use crate::syscall::{self, oflag, Errno, S_IFIFO};

#[crate::lctp_test(suite = fs, expect = success, case = "link creates a second name that shares inode and device")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "link raises the hard-link count to 2")]
fn link_nlink_two() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let st = check_ok!(syscall::stat(&a), "stat");
    check_eq!(st.st_nlink, 2, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "link with a missing source path returns ENOENT")]
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

#[crate::lctp_test(suite = fs, expect = failure, case = "link of a directory returns EPERM or EACCES")]
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

#[crate::lctp_test(suite = fs, expect = failure, case = "link onto an existing path returns EEXIST")]
fn link_existing_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    check_err!(syscall::link(&a, &b), Errno::EEXIST, "link over existing");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "reading a hard link returns the same bytes as the original name")]
fn link_share_content() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"a", 0o644), "create");
    check_ok!(syscall::write(fd, b"XY"), "write");
    check_ok!(syscall::close(fd), "close");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let fd = check_ok!(syscall::open(&b, oflag::O_RDONLY, 0), "open b");
    let mut buf = [0u8; 2];
    check_ok!(syscall::read(fd, &mut buf), "read");
    check_eq!(&buf, b"XY", "shared");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "unlink of one hard-link name leaves the other name intact")]
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

#[crate::lctp_test(suite = fs, full, expect = success, case = "two extra hard links raise nlink to 3")]
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

#[crate::lctp_test(suite = fs, expect = soft, case = "linkat with AT_EMPTY_PATH creates a hard link when the interface is supported")]
fn linkat_empty_path() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let dst = copy_child(&mut tmp, b"via_fd")?;
    let fd = check_ok!(syscall::open(&a, oflag::O_RDONLY, 0), "open");
    match syscall::linkat(fd, b"\0", syscall::AT_FDCWD, &dst, syscall::AT_EMPTY_PATH) {
        Ok(()) => {
            let sa = check_ok!(syscall::stat(&a), "stat a");
            let sb = check_ok!(syscall::stat(&dst), "stat dst");
            check_eq!(sa.st_ino, sb.st_ino, "same inode");
        }
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) | Err(Errno::ENOENT) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("linkat EMPTY_PATH")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "linkat with AT_SYMLINK_FOLLOW hard-links the symlink target")]
fn linkat_symlink_follow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let target = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"slink")?;
    let dst = copy_child(&mut tmp, b"hard")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    check_ok!(
        syscall::linkat(
            syscall::AT_FDCWD,
            &link,
            syscall::AT_FDCWD,
            &dst,
            syscall::AT_SYMLINK_FOLLOW
        ),
        "linkat FOLLOW"
    );
    let st_t = check_ok!(syscall::stat(&target), "stat target");
    let st_d = check_ok!(syscall::stat(&dst), "stat hard");
    check_eq!(st_t.st_ino, st_d.st_ino, "followed to target");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod through one hard-link name changes the mode seen via the other")]
fn link_then_chmod_shared() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    check_ok!(syscall::chmod(&a, 0o640), "chmod a");
    let sb = check_ok!(syscall::stat(&b), "stat b");
    check_eq!(sb.mode_bits() & 0o777, 0o640, "shared mode");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "link of '.' returns EPERM, EACCES, EISDIR, or EINVAL")]
fn link_dot_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dst = copy_child(&mut tmp, b"dotlink")?;
    match syscall::link(b".\0", &dst) {
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EISDIR) | Err(Errno::EINVAL) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("link . ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("link . errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "unlink of one hard-link name restores nlink to 1")]
fn link_nlink_after_unlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    check_ok!(syscall::unlink(&b), "unlink b");
    let st = check_ok!(syscall::stat(&a), "stat");
    check_eq!(st.st_nlink, 1, "nlink back to 1");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "link of a FIFO creates a second FIFO name with nlink 2")]
fn link_fifo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    let dst = copy_child(&mut tmp, b"fifo2")?;
    check_ok!(syscall::link(&path, &dst), "link fifo");
    let st = check_ok!(syscall::stat(&dst), "stat");
    check!(st.is_fifo(), "fifo");
    check_eq!(st.st_nlink, 2, "nlink");
    check_ok!(syscall::unlink(&dst), "unlink dst");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "link into a subdirectory raises nlink to 2")]
fn link_into_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let dst = join_path(&dir, b"b\0", &mut nested)?;
    check_ok!(syscall::link(&a, dst), "link");
    check_eq!(check_ok!(syscall::stat(&a), "stat").st_nlink, 2, "nlink");
    check_ok!(syscall::unlink(dst), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "link with a non-directory destination component returns ENOTDIR")]
fn link_enotdir_dst() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let dst = join_path(&file, b"x\0", &mut nested)?;
    check_err!(syscall::link(&a, dst), Errno::ENOTDIR, "enotdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "link into a directory without write permission returns EACCES")]
fn link_parent_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o555), "chmod");
    let mut nested = [0u8; 160];
    let dst = join_path(&dir, b"b\0", &mut nested)?;
    check_err!(syscall::link(&a, dst), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "stat through a hard link reports the same file size")]
fn link_share_size() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    write_file(&a, b"hello")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    check_eq!(check_ok!(syscall::stat(&b), "stat").st_size, 5, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "write through a hard-link name is visible via the original name")]
fn link_write_via_second() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    write_file(&b, b"ZZ")?;
    let mut buf = [0u8; 2];
    check_eq!(crate::suites::common::read_file(&a, &mut buf)?, 2, "len");
    check_eq!(&buf, b"ZZ", "shared write");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "link onto an existing directory returns EEXIST")]
fn link_eexist_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::link(&a, &dir), Errno::EEXIST, "eexist");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "link of a symlink follows it and hard-links the target file")]
fn link_to_symlink_nofollow_creates_link_to_target() -> TestResult {
    // link(2) follows symlinks by default on Linux for oldpath.
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let target = create_empty(&mut tmp, b"t")?;
    let sl = copy_child(&mut tmp, b"sl")?;
    let hl = copy_child(&mut tmp, b"hl")?;
    check_ok!(syscall::symlink(b"t\0", &sl), "symlink");
    check_ok!(syscall::link(&sl, &hl), "link");
    let st = check_ok!(syscall::stat(&hl), "stat");
    check!(st.is_reg(), "reg");
    check_eq!(st.st_ino, check_ok!(syscall::stat(&target), "t").st_ino, "ino");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "link into a missing destination directory returns ENOENT")]
fn link_missing_dst_parent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let mut dest = [0u8; 160];
    let base = tmp.path();
    let blen = base.iter().position(|&c| c == 0).unwrap();
    dest[..blen].copy_from_slice(&base[..blen]);
    dest[blen..blen + 10].copy_from_slice(b"/nope/link");
    dest[blen + 10] = 0;
    check_err!(
        syscall::link(&a, crate::suites::common::truncate_cstr(&dest)),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "three extra hard links raise nlink to 4")]
fn link_four_names() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    for name in [b"b\0".as_slice(), b"c\0", b"d\0"] {
        let p = copy_child(&mut tmp, name)?;
        check_ok!(syscall::link(&a, &p), "link");
    }
    check_eq!(check_ok!(syscall::stat(&a), "stat").st_nlink, 4, "nlink");
    Ok(())
}
