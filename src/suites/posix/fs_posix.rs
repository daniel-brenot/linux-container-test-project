//! POSIX open/creat/access/mkdir/rmdir filesystem semantics.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, write_file};
use crate::syscall::{self, oflag, Errno, F_OK, R_OK, W_OK, X_OK};

fn mode_subset(actual: u32, requested: u32) -> bool {
    (actual & 0o777) & !(requested & 0o777) == 0
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_CREAT creates a regular file whose mode is a subset of 0644")]
fn fs_creat_via_open_644() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"c644")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_reg(), "reg");
    check!(mode_subset(st.mode_bits(), 0o644), "subset");
    check!(st.mode_bits() & 0o400 != 0, "owner r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_CREAT creates a regular file whose mode is a subset of 0666")]
fn fs_creat_via_open_666() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"c666")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o666),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(mode_subset(st.mode_bits(), 0o666), "subset");
    check!(st.mode_bits() & 0o400 != 0, "owner r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_CREAT|O_EXCL creates a regular file whose mode is a subset of 0600")]
fn fs_creat_via_open_600() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"c600")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o600),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(mode_subset(st.mode_bits(), 0o600), "subset");
    check!(st.mode_bits() & 0o400 != 0, "owner r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_CREAT|O_EXCL creates a regular file whose mode is a subset of 0755")]
fn fs_creat_via_open_755() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"c755")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o755),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(mode_subset(st.mode_bits(), 0o755), "subset");
    check!(st.mode_bits() & 0o400 != 0, "owner r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_CREAT|O_EXCL does not set permission bits beyond the requested 0640")]
fn fs_creat_mode_no_extra_bits() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"extra")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o640),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(mode_subset(st.mode_bits(), 0o640), "no extra");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(F_OK) succeeds on an existing regular file")]
fn fs_access_f_ok_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"af")?;
    check_ok!(syscall::access(&path, F_OK), "F_OK");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(R_OK) succeeds on a readable regular file")]
fn fs_access_r_ok_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ar")?;
    check_ok!(syscall::access(&path, R_OK), "R_OK");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(W_OK) succeeds on a writable regular file")]
fn fs_access_w_ok_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"aw")?;
    check_ok!(syscall::access(&path, W_OK), "W_OK");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(R_OK|W_OK) succeeds on a readable and writable regular file")]
fn fs_access_rw_ok_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"arw")?;
    check_ok!(syscall::access(&path, R_OK | W_OK), "RW");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "access(F_OK) of a path that does not exist returns ENOENT")]
fn fs_access_f_ok_missing() -> TestResult {
    check_err!(
        syscall::access(b"/tmp/lctp-no-access-xyz\0", F_OK),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "access(R_OK) of a path that does not exist returns ENOENT")]
fn fs_access_r_ok_missing() -> TestResult {
    check_err!(
        syscall::access(b"/tmp/lctp-no-access-r\0", R_OK),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(F_OK) succeeds on an existing directory")]
fn fs_access_dir_f_ok() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_ok!(syscall::access(tmp.path(), F_OK), "dir F_OK");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(X_OK) succeeds on a searchable directory")]
fn fs_access_dir_x_ok() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_ok!(syscall::access(tmp.path(), X_OK), "dir X_OK");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "mkdir() creates a directory that stat() reports as a directory")]
fn fs_mkdir_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"md")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "dir");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "mkdir() with mode 0755 creates a directory whose mode is a subset of 0755")]
fn fs_mkdir_mode_755_subset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"md755")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(mode_subset(st.mode_bits(), 0o755), "subset");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "mkdir() with mode 0700 creates a directory whose mode is a subset of 0700")]
fn fs_mkdir_mode_700_subset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"md700")?;
    check_ok!(syscall::mkdir(&dir, 0o700), "mkdir");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(mode_subset(st.mode_bits(), 0o700), "subset");
    check!(st.mode_bits() & 0o100 != 0, "owner x");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "mkdir() of a directory that already exists returns EEXIST")]
fn fs_mkdir_exists_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"ex", 0o755)?;
    check_err!(syscall::mkdir(&dir, 0o755), Errno::EEXIST, "exists");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rmdir() removes an empty directory so a later stat() returns ENOENT")]
fn fs_rmdir_empty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"empty", 0o755)?;
    check_ok!(syscall::rmdir(&dir), "rmdir");
    check_err!(syscall::stat(&dir), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "rmdir() of a directory that still contains a file returns ENOTEMPTY")]
fn fs_rmdir_nonempty_enotempty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"ne", 0o755)?;
    let mut nested = [0u8; 160];
    let dlen = dir.iter().position(|&c| c == 0).unwrap();
    nested[..dlen].copy_from_slice(&dir[..dlen]);
    nested[dlen..dlen + 2].copy_from_slice(b"/f");
    nested[dlen + 2] = 0;
    let path = crate::suites::common::truncate_cstr(&nested);
    let fd = check_ok!(
        syscall::open(path, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "file"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(syscall::rmdir(&dir), Errno::ENOTEMPTY, "nonempty");
    check_ok!(syscall::unlink(path), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "rmdir() of a regular file returns ENOTDIR")]
fn fs_rmdir_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"notdir")?;
    check_err!(syscall::rmdir(&path), Errno::ENOTDIR, "file");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "rmdir() of a path that does not exist returns ENOENT")]
fn fs_rmdir_missing_enoent() -> TestResult {
    check_err!(
        syscall::rmdir(b"/tmp/lctp-no-rmdir\0"),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "mkdir() whose parent directory does not exist returns ENOENT")]
fn fs_mkdir_missing_parent_enoent() -> TestResult {
    check_err!(
        syscall::mkdir(b"/tmp/lctp-no-parent/child\0", 0o755),
        Errno::ENOENT,
        "no parent"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_CREAT|O_EXCL creates a new file and a second call returns EEXIST")]
fn fs_open_creat_excl_twice() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"excl")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "first"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "second"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_TRUNC of an existing file sets the file size to 0")]
fn fs_open_trunc_existing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"tr")?;
    write_file(&path, b"abcdef")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_APPEND writes after existing file contents")]
fn fs_open_append_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ap")?;
    write_file(&path, b"A")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0), "append");
    check_ok!(syscall::write(fd, b"B"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    let n = crate::suites::common::read_file(&path, &mut buf)?;
    check_eq!(n, 2, "len");
    check_eq!(&buf[..2], b"AB", "data");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "mkdir() creates a nested subdirectory that rmdir() can remove")]
fn fs_mkdir_nested() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_dir(&mut tmp, b"a", 0o755)?;
    let mut b = [0u8; 160];
    crate::suites::common::join_path(&a, b"b", &mut b)?;
    let bp = crate::suites::common::truncate_cstr(&b);
    check_ok!(syscall::mkdir(bp, 0o755), "mkdir b");
    check_ok!(syscall::rmdir(bp), "rmdir b");
    check_ok!(syscall::rmdir(&a), "rmdir a");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "access(F_OK) after unlink() of the path returns ENOENT")]
fn fs_access_after_unlink_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"gone")?;
    check_ok!(syscall::unlink(&path), "unlink");
    check_err!(syscall::access(&path, F_OK), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "chmod() to 0400 leaves access(R_OK) successful")]
fn fs_chmod_then_access_r() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ch")?;
    check_ok!(syscall::chmod(&path, 0o400), "chmod");
    check_ok!(syscall::access(&path, R_OK), "R_OK");
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "open() with O_CREAT|O_EXCL creates a file whose mode is a subset of 0444")]
fn fs_creat_mode_444_subset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"c444")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_CREAT | oflag::O_EXCL, 0o444),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(mode_subset(st.mode_bits(), 0o444), "subset");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "mkdirat() creates a subdirectory relative to a directory fd")]
fn fs_mkdirat_via_openat_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let dirfd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "dirfd"
    );
    check_ok!(syscall::mkdirat(dirfd, b"sub\0", 0o755), "mkdirat");
    check_ok!(syscall::unlinkat(dirfd, b"sub\0", syscall::AT_REMOVEDIR), "rmdirat");
    check_ok!(syscall::close(dirfd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "openat() with O_CREAT|O_EXCL creates a file relative to a directory fd")]
fn fs_openat_creat_relative() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let dirfd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "dirfd"
    );
    let fd = check_ok!(
        syscall::openat(dirfd, b"rel\0", oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "openat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlinkat(dirfd, b"rel\0", 0), "unlink");
    check_ok!(syscall::close(dirfd), "close dir");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "stat() after creating and writing a file reports a regular file of the written size")]
fn fs_stat_regular_after_creat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"st")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::write(fd, b"xy"), "write");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_reg(), "reg");
    check_eq!(st.st_size, 2, "size");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "open() of a path after unlink() returns ENOENT")]
fn fs_unlink_then_open_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"u")?;
    check_ok!(syscall::unlink(&path), "unlink");
    check_err!(
        syscall::open(&path, oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "open"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "access(X_OK) on a file created with mode 0755 succeeds or returns EACCES")]
fn fs_access_x_ok_on_755_file_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"xfile")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o755),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    // Soft: umask may clear +x; accept success or EACCES.
    match syscall::access(&path, X_OK) {
        Ok(()) | Err(Errno::EACCES) => Ok(()),
        Err(_) => Err(crate::harness::AssertFail::msg("X_OK soft")),
    }
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "repeated mkdir() and rmdir() of distinct empty directories succeed")]
fn fs_mkdir_rmdir_many() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    for i in 0u8..8 {
        let mut name = *b"d0";
        name[1] = b'0' + i;
        let dir = copy_child(&mut tmp, &name)?;
        check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
        check_ok!(syscall::rmdir(&dir), "rmdir");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "write() on a file opened O_RDONLY returns EBADF")]
fn fs_open_rdonly_write_ebadf() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ro")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_err!(syscall::write(fd, b"x"), Errno::EBADF, "write");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "read() on a file opened O_WRONLY returns EBADF")]
fn fs_open_wronly_read_ebadf() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"wo")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "open");
    let mut buf = [0u8; 1];
    check_err!(syscall::read(fd, &mut buf), Errno::EBADF, "read");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "stat() of a newly created regular file reports a link count of at least 1")]
fn fs_creat_then_stat_nlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"nl")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.st_nlink >= 1, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "mkdir() of the . entry in a directory returns EEXIST, EINVAL, or ENOENT")]
fn fs_mkdir_dot_einval_or_eexist() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let mut dot = [0u8; 160];
    let end = tmp.path().iter().position(|&c| c == 0).unwrap();
    dot[..end].copy_from_slice(&tmp.path()[..end]);
    dot[end..end + 2].copy_from_slice(b"/.");
    dot[end + 2] = 0;
    match syscall::mkdir(crate::suites::common::truncate_cstr(&dot), 0o755) {
        Err(Errno::EEXIST) | Err(Errno::EINVAL) | Err(Errno::ENOENT) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("mkdir . succeeded")),
        Err(_) => Err(crate::harness::AssertFail::msg("mkdir . errno")),
    }
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(F_OK) succeeds on /tmp")]
fn fs_access_f_ok_tmp() -> TestResult {
    check_ok!(syscall::access(b"/tmp\0", F_OK), "/tmp");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "open() with O_CREAT|O_EXCL and mode 0777 creates a file whose mode is a subset of 0777")]
fn fs_creat_umask_soft_777() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"u777")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o777),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(mode_subset(st.mode_bits(), 0o777), "subset of 777");
    check!(st.mode_bits() & 0o400 != 0, "owner r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rmdir() removes an empty subdirectory")]
fn fs_rmdir_cwd_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"leaf", 0o755)?;
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "open() with O_CLOEXEC sets FD_CLOEXEC on the new file descriptor")]
fn fs_open_creat_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"cx")?;
    let fd = check_ok!(
        syscall::open(
            &path,
            oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL | oflag::O_CLOEXEC,
            0o644
        ),
        "creat"
    );
    let flags = check_ok!(syscall::fcntl(fd, syscall::fcntl_cmd::F_GETFD, 0), "getfd");
    check!(flags as i32 & syscall::FD_CLOEXEC != 0, "cloexec");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
