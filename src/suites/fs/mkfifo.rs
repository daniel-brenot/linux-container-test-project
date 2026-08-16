//! mkfifo (FIFO) filesystem tests via mknodat.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path};
use crate::syscall::{self, oflag, Errno, S_IFIFO};

macro_rules! mkfifo_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = concat!("mknodat creates a FIFO with mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let path = copy_child(&mut tmp, b"fifo")?;
            check_ok!(
                syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | ($mode & 0o777), 0),
                "mknodat"
            );
            check_ok!(syscall::chmod(&path, $mode & 0o777), "chmod");
            let st = check_ok!(syscall::stat(&path), "stat");
            check!(st.is_fifo(), "fifo");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            check_ok!(syscall::unlink(&path), "unlink");
            Ok(())
        }
    };
}

#[crate::lctp_test(suite = fs, expect = success, case = "mknodat with S_IFIFO creates a FIFO")]
fn mkfifo_create() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mknodat fifo"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_fifo(), "is fifo");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "lstat of a newly created FIFO reports a FIFO type")]
fn mkfifo_is_fifo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o600, 0),
        "mknodat"
    );
    let st = check_ok!(syscall::lstat(&path), "lstat");
    check!(st.is_fifo(), "fifo type");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = failure, case = "open of a FIFO O_WRONLY|O_NONBLOCK with no reader returns ENXIO")]
fn mkfifo_open_nonblock_enxio() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mknodat"
    );
    check_err!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_NONBLOCK, 0),
        Errno::ENXIO,
        "expected ENXIO"
    );
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_NONBLOCK, 0),
        "open rdwr"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a FIFO sets mode 0620")]
fn mkfifo_mode_bits() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o620, 0),
        "mknodat"
    );
    check_ok!(syscall::chmod(&path, 0o620), "chmod");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.mode_bits() & 0o777, 0o620, "mode");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

mkfifo_mode!(mkfifo_mode_000, 0o000);
mkfifo_mode!(mkfifo_mode_111, 0o111);
mkfifo_mode!(mkfifo_mode_222, 0o222);
mkfifo_mode!(mkfifo_mode_444, 0o444);
mkfifo_mode!(mkfifo_mode_555, 0o555);
mkfifo_mode!(mkfifo_mode_666, 0o666);
mkfifo_mode!(mkfifo_mode_700, 0o700);
mkfifo_mode!(mkfifo_mode_755, 0o755);
mkfifo_mode!(mkfifo_mode_777, 0o777);
mkfifo_mode!(mkfifo_mode_640, 0o640);
mkfifo_mode!(mkfifo_mode_400, 0o400);
mkfifo_mode!(mkfifo_mode_200, 0o200);
mkfifo_mode!(mkfifo_mode_100, 0o100);
mkfifo_mode!(mkfifo_mode_001, 0o001);
mkfifo_mode!(mkfifo_mode_010, 0o010);

#[crate::lctp_test(suite = fs, expect = failure, case = "mknodat of an existing FIFO returns EEXIST")]
fn mkfifo_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "first"
    );
    check_err!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        Errno::EEXIST,
        "eexist"
    );
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "mknodat FIFO onto an existing regular file returns EEXIST")]
fn mkfifo_eexist_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        Errno::EEXIST,
        "eexist"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "mknodat FIFO onto an existing directory returns EEXIST")]
fn mkfifo_eexist_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(
        syscall::mknodat(syscall::AT_FDCWD, &dir, S_IFIFO | 0o644, 0),
        Errno::EEXIST,
        "eexist"
    );
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "mknodat creates a FIFO inside a subdirectory")]
fn mkfifo_in_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let path = join_path(&dir, b"fifo\0", &mut nested)?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, path, S_IFIFO | 0o644, 0),
        "mknodat"
    );
    check!(check_ok!(syscall::stat(path), "stat").is_fifo(), "fifo");
    check_ok!(syscall::unlink(path), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "mknodat FIFO with a missing parent returns ENOENT")]
fn mkfifo_parent_missing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let mut path = [0u8; 160];
    let base = tmp.path();
    let blen = base.iter().position(|&c| c == 0).unwrap();
    path[..blen].copy_from_slice(&base[..blen]);
    path[blen..blen + 10].copy_from_slice(b"/nope/fifo");
    path[blen + 10] = 0;
    check_err!(
        syscall::mknodat(
            syscall::AT_FDCWD,
            crate::suites::common::truncate_cstr(&path),
            S_IFIFO | 0o644,
            0
        ),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "mknodat FIFO in a directory without write permission returns EACCES")]
fn mkfifo_parent_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o555), "chmod");
    let mut nested = [0u8; 160];
    let path = join_path(&dir, b"fifo\0", &mut nested)?;
    check_err!(
        syscall::mknodat(syscall::AT_FDCWD, path, S_IFIFO | 0o644, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "write and read through a FIFO opened O_RDWR|O_NONBLOCK succeed")]
fn mkfifo_rdwr_write_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mknodat"
    );
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_NONBLOCK, 0),
        "open"
    );
    check_ok!(syscall::write(fd, b"P"), "write");
    let mut buf = [0u8; 1];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 1, "len");
    check_eq!(buf[0], b'P', "data");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "stat of a FIFO reports it is not a regular file, directory, or symlink")]
fn mkfifo_not_reg_not_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mknodat"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(!st.is_reg(), "not reg");
    check!(!st.is_dir(), "not dir");
    check!(!st.is_lnk(), "not lnk");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "mknodat FIFO through a non-directory path component returns ENOTDIR")]
fn mkfifo_enotdir_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"fifo\0", &mut nested)?;
    check_err!(
        syscall::mknodat(syscall::AT_FDCWD, path, S_IFIFO | 0o644, 0),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}
