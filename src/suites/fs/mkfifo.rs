//! mkfifo (FIFO) filesystem tests via mknodat.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::copy_child;
use crate::syscall::{self, oflag, Errno, S_IFIFO};

#[crate::lctp_test(suite = fs)]
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

#[crate::lctp_test(suite = fs)]
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

#[crate::lctp_test(suite = fs, full)]
fn mkfifo_open_nonblock_enxio() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mknodat"
    );
    // O_WRONLY|O_NONBLOCK with no reader → ENXIO on Linux.
    check_err!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_NONBLOCK, 0),
        Errno::ENXIO,
        "expected ENXIO"
    );
    // O_RDWR|O_NONBLOCK opens without a peer.
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_NONBLOCK, 0),
        "open rdwr"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn mkfifo_mode_bits() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o620, 0),
        "mknodat"
    );
    // umask may clear bits at creation; set exact mode with chmod.
    check_ok!(syscall::chmod(&path, 0o620), "chmod");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.mode_bits() & 0o777, 0o620, "mode");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}
