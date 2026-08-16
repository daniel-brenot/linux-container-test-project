//! rmdir filesystem tests.

use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, truncate_cstr};
use crate::syscall::{self, oflag, Errno, S_IFIFO};

#[crate::lctp_test(suite = fs, expect = success, case = "rmdir of an empty directory succeeds and the path is gone")]
fn rmdir_empty_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::rmdir(&dir), "rmdir");
    check_err!(syscall::stat(&dir), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of a directory that contains a file returns ENOTEMPTY")]
fn rmdir_notempty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"file\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "create"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(syscall::rmdir(&dir), Errno::ENOTEMPTY, "notempty");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of a regular file returns ENOTDIR")]
fn rmdir_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::close(fd), "close");
    let file = copy_child(&mut tmp, b"f")?;
    check_err!(syscall::rmdir(&file), Errno::ENOTDIR, "rmdir file");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of '.' returns EINVAL")]
fn rmdir_dot_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut dot = [0u8; 160];
    let path = join_path(&dir, b".\0", &mut dot)?;
    check_err!(syscall::rmdir(path), Errno::EINVAL, "rmdir .");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of '..' returns ENOTEMPTY, EINVAL, or ENOTDIR")]
fn rmdir_dotdot_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut dotdot = [0u8; 160];
    let path = join_path(&dir, b"..\0", &mut dotdot)?;
    match syscall::rmdir(path) {
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

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of a missing path returns ENOENT")]
fn rmdir_enoent() -> TestResult {
    check_err!(
        syscall::rmdir(b"/tmp/lctp-no-dir\0"),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rmdir of a nested empty directory then its parent succeeds")]
fn rmdir_nested_empty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let outer = create_dir(&mut tmp, b"outer", 0o755)?;
    let mut inner = [0u8; 160];
    let path = join_path(&outer, b"inner\0", &mut inner)?;
    check_ok!(syscall::mkdir(path, 0o755), "mkdir inner");
    check_ok!(syscall::rmdir(path), "rmdir inner");
    check_ok!(syscall::rmdir(&outer), "rmdir outer");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "rmdir succeeds after unlinking the directory contents")]
fn rmdir_after_unlink_contents() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut f = [0u8; 160];
    let child = join_path(&dir, b"file\0", &mut f)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR, 0o644),
        "create"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of a directory that contains a subdirectory returns ENOTEMPTY")]
fn rmdir_notempty_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let outer = create_dir(&mut tmp, b"o", 0o755)?;
    let mut inner = [0u8; 160];
    let path = join_path(&outer, b"i\0", &mut inner)?;
    check_ok!(syscall::mkdir(path, 0o755), "mkdir");
    check_err!(syscall::rmdir(&outer), Errno::ENOTEMPTY, "notempty");
    check_ok!(syscall::rmdir(path), "rmdir inner");
    check_ok!(syscall::rmdir(&outer), "rmdir outer");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of a symlink to a directory returns ENOTDIR")]
fn rmdir_symlink_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"d\0", &link), "symlink");
    check_err!(syscall::rmdir(&link), Errno::ENOTDIR, "symlink");
    check_ok!(syscall::unlink(&link), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of a FIFO returns ENOTDIR")]
fn rmdir_fifo_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_err!(syscall::rmdir(&path), Errno::ENOTDIR, "fifo");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of an already-removed directory returns ENOENT")]
fn rmdir_twice_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::rmdir(&dir), "first");
    check_err!(syscall::rmdir(&dir), Errno::ENOENT, "second");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir in a parent without write permission returns EACCES")]
fn rmdir_parent_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let outer = create_dir(&mut tmp, b"o", 0o755)?;
    let mut inner = [0u8; 160];
    let path = join_path(&outer, b"i\0", &mut inner)?;
    check_ok!(syscall::mkdir(path, 0o755), "mkdir");
    check_ok!(syscall::chmod(&outer, 0o555), "chmod");
    check_err!(syscall::rmdir(path), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&outer, 0o755), "restore");
    check_ok!(syscall::rmdir(path), "rmdir");
    check_ok!(syscall::rmdir(&outer), "rmdir outer");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "unlinkat with AT_REMOVEDIR removes an empty directory")]
fn rmdir_unlinkat_removedir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let outer = create_dir(&mut tmp, b"o", 0o755)?;
    let dirfd = check_ok!(
        syscall::open(&outer, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_ok!(syscall::mkdirat(dirfd, b"c\0", 0o755), "mkdirat");
    check_ok!(
        syscall::unlinkat(dirfd, b"c\0", crate::syscall::AT_REMOVEDIR),
        "unlinkat REMOVEDIR"
    );
    check_ok!(syscall::close(dirfd), "close");
    check_ok!(syscall::rmdir(&outer), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rmdir of a mode 0000 directory succeeds when the parent is writable")]
fn rmdir_mode_000_still_ok_if_writable_parent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o000), "chmod");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir through a non-directory path component returns ENOTDIR")]
fn rmdir_enotdir_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"x\0", &mut nested)?;
    check_err!(syscall::rmdir(path), Errno::ENOTDIR, "enotdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir of a directory that contains a symlink returns ENOTEMPTY")]
fn rmdir_notempty_with_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut link = [0u8; 160];
    let path = join_path(&dir, b"l\0", &mut link)?;
    check_ok!(syscall::symlink(b"nowhere\0", path), "symlink");
    check_err!(syscall::rmdir(&dir), Errno::ENOTEMPTY, "notempty");
    check_ok!(syscall::unlink(path), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rmdir of a three-level empty directory chain succeeds from the inside out")]
fn rmdir_chain_three() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_dir(&mut tmp, b"a", 0o755)?;
    let mut b = [0u8; 160];
    let bp = join_path(&a, b"b\0", &mut b)?;
    check_ok!(syscall::mkdir(bp, 0o755), "mkdir b");
    let mut c = [0u8; 160];
    let cp = join_path(bp, b"c\0", &mut c)?;
    check_ok!(syscall::mkdir(cp, 0o755), "mkdir c");
    check_ok!(syscall::rmdir(cp), "rmdir c");
    check_ok!(syscall::rmdir(bp), "rmdir b");
    check_ok!(syscall::rmdir(&a), "rmdir a");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rmdir with a missing parent component returns ENOENT")]
fn rmdir_missing_parent_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let mut path = [0u8; 160];
    let base = tmp.path();
    let blen = base.iter().position(|&c| c == 0).unwrap();
    let suffix = b"/nope/dir";
    path[..blen].copy_from_slice(&base[..blen]);
    path[blen..blen + suffix.len()].copy_from_slice(suffix);
    path[blen + suffix.len()] = 0;
    check_err!(
        syscall::rmdir(truncate_cstr(&path)),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}
