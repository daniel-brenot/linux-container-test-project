//! POSIX errno semantics tests.

use crate::check;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = posix)]
fn errno_enoent_open() -> TestResult {
    check_err!(
        syscall::open(b"/tmp/lctp-missing-open\0", oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "open missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_stat() -> TestResult {
    check_err!(
        syscall::stat(b"/tmp/lctp-missing-stat\0"),
        Errno::ENOENT,
        "stat missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_unlink() -> TestResult {
    check_err!(
        syscall::unlink(b"/tmp/lctp-missing-unlink\0"),
        Errno::ENOENT,
        "unlink missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eisdir_open_write() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_err!(
        syscall::open(tmp.path(), oflag::O_RDWR, 0),
        Errno::EISDIR,
        "O_RDWR dir"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eisdir_unlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    match syscall::unlink(&dir) {
        Err(Errno::EISDIR) | Err(Errno::EPERM) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("unlink dir succeeded")),
        Err(_) => return Err(crate::harness::AssertFail::msg("unlink dir errno")),
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_read() -> TestResult {
    check_err!(syscall::read(-1, &mut [0u8; 1]), Errno::EBADF, "read bad fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_write() -> TestResult {
    check_err!(syscall::write(-1, b"x"), Errno::EBADF, "write bad fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_close() -> TestResult {
    check_err!(syscall::close(-1), Errno::EBADF, "close bad fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_einval_lseek() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_err!(
        syscall::lseek(fd, 0, 999),
        Errno::EINVAL,
        "invalid whence"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotdir_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut bad = [0u8; 160];
    let end = file.iter().position(|&c| c == 0).unwrap();
    // Append "/x" so a regular file is used as a directory component.
    check!(end + 3 < bad.len(), "path too long");
    bad[..end].copy_from_slice(&file[..end]);
    bad[end..end + 2].copy_from_slice(b"/x");
    bad[end + 2] = 0;
    check_err!(
        syscall::open(crate::suites::common::truncate_cstr(&bad), oflag::O_RDONLY, 0),
        Errno::ENOTDIR,
        "file as dir component"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eexist_mkdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"d")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    check_err!(syscall::mkdir(&dir, 0o755), Errno::EEXIST, "mkdir again");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotempty_rmdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    check!(dlen + 6 < nested.len(), "path too long");
    nested[..dlen].copy_from_slice(&dir[..dlen]);
    nested[dlen..dlen + 5].copy_from_slice(b"/file");
    nested[dlen + 5] = 0;
    let path = crate::suites::common::truncate_cstr(&nested);
    let fd = check_ok!(
        syscall::open(path, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "create nested"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(syscall::rmdir(&dir), Errno::ENOTEMPTY, "rmdir nonempty");
    check_ok!(syscall::unlink(path), "unlink nested");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_chmod() -> TestResult {
    check_err!(
        syscall::chmod(b"/tmp/lctp-missing-chmod\0", 0o644),
        Errno::ENOENT,
        "chmod"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_access() -> TestResult {
    check_err!(
        syscall::access(b"/tmp/lctp-missing-access\0", syscall::F_OK),
        Errno::ENOENT,
        "access"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_chdir() -> TestResult {
    check_err!(
        syscall::chdir(b"/tmp/lctp-missing-chdir\0"),
        Errno::ENOENT,
        "chdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_mkdir_parent() -> TestResult {
    check_err!(
        syscall::mkdir(b"/tmp/lctp-no-parent-x/child\0", 0o755),
        Errno::ENOENT,
        "mkdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_rmdir() -> TestResult {
    check_err!(
        syscall::rmdir(b"/tmp/lctp-missing-rmdir\0"),
        Errno::ENOENT,
        "rmdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_link() -> TestResult {
    check_err!(
        syscall::link(b"/tmp/lctp-missing-link-src\0", b"/tmp/lctp-missing-link-dst\0"),
        Errno::ENOENT,
        "link"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_rename() -> TestResult {
    check_err!(
        syscall::rename(b"/tmp/lctp-missing-ren-src\0", b"/tmp/lctp-missing-ren-dst\0"),
        Errno::ENOENT,
        "rename"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_readlink() -> TestResult {
    let mut buf = [0u8; 64];
    check_err!(
        syscall::readlink(b"/tmp/lctp-missing-readlink\0", &mut buf),
        Errno::ENOENT,
        "readlink"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_truncate() -> TestResult {
    check_err!(
        syscall::truncate(b"/tmp/lctp-missing-trunc\0", 0),
        Errno::ENOENT,
        "truncate"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_fstat() -> TestResult {
    check_err!(syscall::fstat(-1), Errno::EBADF, "fstat");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_ftruncate() -> TestResult {
    check_err!(syscall::ftruncate(-1, 0), Errno::EBADF, "ftruncate");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_fsync() -> TestResult {
    check_err!(syscall::fsync(-1), Errno::EBADF, "fsync");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_fdatasync() -> TestResult {
    check_err!(syscall::fdatasync(-1), Errno::EBADF, "fdatasync");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_fchmod() -> TestResult {
    check_err!(syscall::fchmod(-1, 0o644), Errno::EBADF, "fchmod");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_lseek() -> TestResult {
    check_err!(syscall::lseek(-1, 0, syscall::SEEK_SET), Errno::EBADF, "lseek");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_dup() -> TestResult {
    check_err!(syscall::dup(-1), Errno::EBADF, "dup");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_fcntl_getfl() -> TestResult {
    check_err!(
        syscall::fcntl(-1, syscall::fcntl_cmd::F_GETFL, 0),
        Errno::EBADF,
        "fcntl"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_einval_lseek_neg_whence() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_err!(syscall::lseek(fd, 0, -1), Errno::EINVAL, "whence");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eisdir_open_wronly() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_err!(
        syscall::open(tmp.path(), oflag::O_WRONLY, 0),
        Errno::EISDIR,
        "wronly dir"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotdir_chdir_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    check_err!(syscall::chdir(&file), Errno::ENOTDIR, "chdir file");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotdir_mkdir_through_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut bad = [0u8; 160];
    let end = file.iter().position(|&c| c == 0).unwrap();
    check!(end + 4 < bad.len(), "path");
    bad[..end].copy_from_slice(&file[..end]);
    bad[end] = b'/';
    bad[end + 1] = b'x';
    bad[end + 2] = 0;
    check_err!(
        syscall::mkdir(crate::suites::common::truncate_cstr(&bad), 0o755),
        Errno::ENOTDIR,
        "mkdir through file"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotdir_open_slash_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut with_slash = [0u8; 160];
    let end = file.iter().position(|&c| c == 0).unwrap();
    with_slash[..end].copy_from_slice(&file[..end]);
    with_slash[end] = b'/';
    with_slash[end + 1] = 0;
    check_err!(
        syscall::open(crate::suites::common::truncate_cstr(&with_slash), oflag::O_RDONLY, 0),
        Errno::ENOTDIR,
        "slash"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_eisdir_truncate_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    match syscall::truncate(tmp.path(), 0) {
        Err(Errno::EISDIR) | Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("truncate dir ok")),
        Err(_) => Err(crate::harness::AssertFail::msg("truncate dir errno")),
    }
}

#[crate::lctp_test(suite = posix)]
fn errno_eexist_open_excl() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ex")?;
    check_err!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "excl"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_symlink_target_open() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"dangling")?;
    check_ok!(syscall::symlink(b"no-such-target\0", &link), "symlink");
    check_err!(
        syscall::open(&link, oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "dangling"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_pread() -> TestResult {
    let mut buf = [0u8; 4];
    check_err!(syscall::pread(-1, &mut buf, 0), Errno::EBADF, "pread");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_pwrite() -> TestResult {
    check_err!(syscall::pwrite(-1, b"x", 0), Errno::EBADF, "pwrite");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_einval_ftruncate_neg() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_err!(syscall::ftruncate(fd, -1), Errno::EINVAL, "neg");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_lstat() -> TestResult {
    check_err!(
        syscall::lstat(b"/tmp/lctp-missing-lstat\0"),
        Errno::ENOENT,
        "lstat"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_readv() -> TestResult {
    check_err!(syscall::readv(-1, &mut []), Errno::EBADF, "readv");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_writev() -> TestResult {
    check_err!(syscall::writev(-1, &mut []), Errno::EBADF, "writev");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enotdir_unlinkat_dir_as_file_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut bad = [0u8; 160];
    let end = file.iter().position(|&c| c == 0).unwrap();
    bad[..end].copy_from_slice(&file[..end]);
    bad[end] = b'/';
    bad[end + 1] = b'z';
    bad[end + 2] = 0;
    check_err!(
        syscall::unlink(crate::suites::common::truncate_cstr(&bad)),
        Errno::ENOTDIR,
        "unlink through file"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_enoent_openat_cwd() -> TestResult {
    check_err!(
        syscall::openat(syscall::AT_FDCWD, b"/tmp/lctp-no-openat\0", oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "openat"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn errno_ebadf_openat_dirfd() -> TestResult {
    check_err!(
        syscall::openat(-1, b"x\0", oflag::O_RDONLY, 0),
        Errno::EBADF,
        "openat dirfd"
    );
    Ok(())
}
