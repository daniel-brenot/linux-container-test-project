//! mkdir filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, truncate_cstr};
use crate::syscall::{self, oflag, Errno};

macro_rules! mkdir_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let dir = copy_child(&mut tmp, b"d")?;
            check_ok!(syscall::mkdir(&dir, $mode), "mkdir");
            // umask may clear bits; force exact mode.
            check_ok!(syscall::chmod(&dir, $mode & 0o777), "chmod");
            let st = check_ok!(syscall::stat(&dir), "stat");
            check!(st.is_dir(), "dir");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            check_ok!(syscall::rmdir(&dir), "rmdir");
            Ok(())
        }
    };
}

#[crate::lctp_test(suite = fs)]
fn mkdir_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"dir")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "is dir");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_mode_755() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let st = check_ok!(syscall::stat(&dir), "stat");
    check_eq!(st.mode_bits() & 0o777, 0o755, "mode");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_mode_700() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o700)?;
    let st = check_ok!(syscall::stat(&dir), "stat");
    check_eq!(st.mode_bits() & 0o777, 0o700, "mode");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::mkdir(&dir, 0o755), Errno::EEXIST, "eexist");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_parent_missing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let base = copy_child(&mut tmp, b"base")?;
    let mut nested = [0u8; 160];
    let suffix = b"/a/b/c";
    let blen = base.iter().position(|&c| c == 0).unwrap();
    check!(blen + suffix.len() + 1 < nested.len(), "path too long");
    nested[..blen].copy_from_slice(&base[..blen]);
    nested[blen..blen + suffix.len()].copy_from_slice(suffix);
    nested[blen + suffix.len()] = 0;
    check_err!(
        syscall::mkdir(truncate_cstr(&nested), 0o755),
        Errno::ENOENT,
        "no parent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_nested() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let parent = create_dir(&mut tmp, b"p", 0o755)?;
    let mut child = [0u8; 160];
    let path = join_path(&parent, b"sub\0", &mut child)?;
    check_ok!(syscall::mkdir(path, 0o755), "mkdir sub");
    check_ok!(syscall::rmdir(path), "rmdir sub");
    check_ok!(syscall::rmdir(&parent), "rmdir parent");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn mkdir_dot_entries() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let fd = check_ok!(
        syscall::open(&dir, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_is_directory() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "directory");
    check!(!st.is_reg(), "not file");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdirat_relative_dirfd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let parent = create_dir(&mut tmp, b"p", 0o755)?;
    let dirfd = check_ok!(
        syscall::open(&parent, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_ok!(syscall::mkdirat(dirfd, b"child\0", 0o755), "mkdirat");
    let mut child = [0u8; 160];
    let path = join_path(&parent, b"child\0", &mut child)?;
    let st = check_ok!(syscall::stat(path), "stat child");
    check!(st.is_dir(), "is dir");
    check_ok!(syscall::rmdir(path), "rmdir");
    check_ok!(syscall::close(dirfd), "close");
    check_ok!(syscall::rmdir(&parent), "rmdir parent");
    Ok(())
}

mkdir_mode!(mkdir_mode_000, 0o000);
mkdir_mode!(mkdir_mode_001, 0o001);
mkdir_mode!(mkdir_mode_010, 0o010);
mkdir_mode!(mkdir_mode_100, 0o100);
mkdir_mode!(mkdir_mode_111, 0o111);
mkdir_mode!(mkdir_mode_222, 0o222);
mkdir_mode!(mkdir_mode_444, 0o444);
mkdir_mode!(mkdir_mode_555, 0o555);
mkdir_mode!(mkdir_mode_666, 0o666);
mkdir_mode!(mkdir_mode_711, 0o711);
mkdir_mode!(mkdir_mode_750, 0o750);
mkdir_mode!(mkdir_mode_764, 0o764);
mkdir_mode!(mkdir_mode_775, 0o775);
mkdir_mode!(mkdir_mode_777, 0o777);
mkdir_mode!(mkdir_mode_731, 0o731);
mkdir_mode!(mkdir_mode_517, 0o517);
mkdir_mode!(mkdir_mode_070, 0o070);
mkdir_mode!(mkdir_mode_007, 0o007);
mkdir_mode!(mkdir_mode_505, 0o505);
mkdir_mode!(mkdir_mode_303, 0o303);

#[crate::lctp_test(suite = fs)]
fn mkdir_eexist_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    check_err!(syscall::mkdir(&file, 0o755), Errno::EEXIST, "eexist file");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_enotdir_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"d\0", &mut nested)?;
    check_err!(syscall::mkdir(path, 0o755), Errno::ENOTDIR, "enotdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_parent_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let parent = create_dir(&mut tmp, b"p", 0o755)?;
    check_ok!(syscall::chmod(&parent, 0o555), "chmod");
    let mut child = [0u8; 160];
    let path = join_path(&parent, b"c\0", &mut child)?;
    check_err!(syscall::mkdir(path, 0o755), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&parent, 0o755), "restore");
    check_ok!(syscall::rmdir(&parent), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_sticky_bit_request() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"d")?;
    check_ok!(syscall::mkdir(&dir, 0o1755), "mkdir sticky");
    check_ok!(syscall::chmod(&dir, 0o1755), "chmod");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check_eq!(st.mode_bits() & 0o7777, 0o1755, "sticky");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_setgid_request() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"d")?;
    check_ok!(syscall::mkdir(&dir, 0o2755), "mkdir setgid");
    check_ok!(syscall::chmod(&dir, 0o2755), "chmod");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check_eq!(st.mode_bits() & 0o7777, 0o2755, "setgid");
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_eexist_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    check_err!(syscall::mkdir(&link, 0o755), Errno::EEXIST, "eexist");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdirat_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let parent = create_dir(&mut tmp, b"p", 0o755)?;
    let dirfd = check_ok!(
        syscall::open(&parent, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_ok!(syscall::mkdirat(dirfd, b"c\0", 0o755), "mkdirat");
    check_err!(
        syscall::mkdirat(dirfd, b"c\0", 0o755),
        Errno::EEXIST,
        "eexist"
    );
    check_ok!(syscall::unlinkat(dirfd, b"c\0", crate::syscall::AT_REMOVEDIR), "rm");
    check_ok!(syscall::close(dirfd), "close");
    check_ok!(syscall::rmdir(&parent), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_nlink_parent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let parent = create_dir(&mut tmp, b"p", 0o755)?;
    let before = check_ok!(syscall::stat(&parent), "stat").st_nlink;
    let mut child = [0u8; 160];
    let path = join_path(&parent, b"c\0", &mut child)?;
    check_ok!(syscall::mkdir(path, 0o755), "mkdir");
    let after = check_ok!(syscall::stat(&parent), "stat2").st_nlink;
    check_eq!(after, before + 1, "nlink +1");
    check_ok!(syscall::rmdir(path), "rmdir");
    check_ok!(syscall::rmdir(&parent), "rmdir p");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkdir_then_creat_inside() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut f = [0u8; 160];
    let path = join_path(&dir, b"f\0", &mut f)?;
    let fd = check_ok!(
        syscall::open(path, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(path), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}
