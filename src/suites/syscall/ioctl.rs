//! ioctl tests (unprivileged).

use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::syscall::{self, oflag, Errno, Winsize, TIOCGWINSZ};

#[crate::lctp_test(suite = syscall, expect = failure, case = "TIOCGWINSZ on a pipe returns ENOTTY")]
fn ioctl_tiocgwinsz_on_pipe_enotty() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut ws = Winsize::default();
    check_err!(
        syscall::ioctl(r, TIOCGWINSZ, &mut ws as *mut Winsize as usize),
        Errno::ENOTTY,
        "pipe TIOCGWINSZ"
    );
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "TIOCGWINSZ on a regular file returns ENOTTY")]
fn ioctl_tiocgwinsz_on_file_enotty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let mut ws = Winsize::default();
    check_err!(
        syscall::ioctl(fd, TIOCGWINSZ, &mut ws as *mut Winsize as usize),
        Errno::ENOTTY,
        "file TIOCGWINSZ"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "TIOCGWINSZ on a unix socket returns ENOTTY")]
fn ioctl_tiocgwinsz_on_socket_enotty() -> TestResult {
    let (a, b) = check_ok!(
        syscall::socketpair(syscall::AF_UNIX, syscall::SOCK_STREAM, 0),
        "socketpair"
    );
    let mut ws = Winsize::default();
    check_err!(
        syscall::ioctl(a, TIOCGWINSZ, &mut ws as *mut Winsize as usize),
        Errno::ENOTTY,
        "socket TIOCGWINSZ"
    );
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "TIOCGWINSZ on fd -1 returns EBADF")]
fn ioctl_ebadf() -> TestResult {
    let mut ws = Winsize::default();
    check_err!(
        syscall::ioctl(-1, TIOCGWINSZ, &mut ws as *mut Winsize as usize),
        Errno::EBADF,
        "bad fd"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "TIOCGWINSZ on a read-only regular file returns ENOTTY")]
fn ioctl_tiocgwinsz_rdonly_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = crate::suites::common::create_empty(&mut tmp, b"ro")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut ws = Winsize::default();
    check_err!(
        syscall::ioctl(fd, TIOCGWINSZ, &mut ws as *mut Winsize as usize),
        Errno::ENOTTY,
        "rdonly"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
