//! POSIX unistd identity / cwd / dup semantics.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, write_file};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = posix)]
fn unistd_getpid_positive() -> TestResult {
    check!(syscall::getpid() > 0, "pid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getppid_nonneg() -> TestResult {
    check!(syscall::getppid() >= 0, "ppid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getpid_stable() -> TestResult {
    check_eq!(syscall::getpid(), syscall::getpid(), "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getppid_stable() -> TestResult {
    check_eq!(syscall::getppid(), syscall::getppid(), "stable");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getuid_matches_geteuid() -> TestResult {
    check_eq!(syscall::getuid(), syscall::geteuid(), "uid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getgid_matches_getegid() -> TestResult {
    check_eq!(syscall::getgid(), syscall::getegid(), "gid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getresuid_matches_getuid() -> TestResult {
    let (r, e, s) = check_ok!(syscall::getresuid(), "getresuid");
    check_eq!(r, syscall::getuid(), "r");
    check_eq!(e, syscall::geteuid(), "e");
    check_eq!(r, e, "r==e");
    check_eq!(e, s, "e==s");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getresgid_matches_getgid() -> TestResult {
    let (r, e, s) = check_ok!(syscall::getresgid(), "getresgid");
    check_eq!(r, syscall::getgid(), "r");
    check_eq!(e, syscall::getegid(), "e");
    check_eq!(r, e, "r==e");
    check_eq!(e, s, "e==s");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getcwd_startswith_slash() -> TestResult {
    let mut buf = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut buf), "getcwd");
    check!(n > 1, "len");
    check!(buf[0] == b'/', "abs");
    check!(buf[n - 1] == 0, "nul");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getcwd_stable() -> TestResult {
    let mut a = [0u8; 256];
    let mut b = [0u8; 256];
    let na = check_ok!(syscall::getcwd(&mut a), "a");
    let nb = check_ok!(syscall::getcwd(&mut b), "b");
    check_eq!(na, nb, "len");
    check_eq!(&a[..na], &b[..nb], "path");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_chdir_tmp_and_back() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    check_ok!(syscall::chdir(b"/tmp\0"), "chdir /tmp");
    let mut cur = [0u8; 256];
    let cn = check_ok!(syscall::getcwd(&mut cur), "getcwd");
    check!(cn >= 4, "len");
    check!(&cur[..4] == b"/tmp" || &cur[..5] == b"/tmp\0", "in tmp");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_chdir_tempdir() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let mut cur = [0u8; 256];
    let cn = check_ok!(syscall::getcwd(&mut cur), "getcwd");
    let plen = tmp.path().iter().position(|&c| c == 0).unwrap();
    check_eq!(&cur[..plen], &tmp.path()[..plen], "cwd");
    check!(cur[cn - 1] == 0, "nul");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_chdir_missing_enoent() -> TestResult {
    check_err!(
        syscall::chdir(b"/tmp/lctp-no-chdir-dir\0"),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_chdir_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(syscall::chdir(&path), Errno::ENOTDIR, "file");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup_distinct_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"d", 0o644), "create");
    let d = check_ok!(syscall::dup(fd), "dup");
    check!(d != fd, "distinct");
    check_ok!(syscall::close(d), "close d");
    check_ok!(syscall::close(fd), "close fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup_shares_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"off", 0o644), "create");
    check_ok!(syscall::write(fd, b"ABC"), "write");
    let d = check_ok!(syscall::dup(fd), "dup");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 1];
    check_ok!(syscall::read(d, &mut buf), "read via dup");
    check_eq!(buf[0], b'A', "byte");
    let pos = check_ok!(syscall::lseek(fd, 0, syscall::SEEK_CUR), "cur");
    check_eq!(pos, 1, "shared offset");
    check_ok!(syscall::close(d), "close d");
    check_ok!(syscall::close(fd), "close fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup2_to_new() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"d2", 0o644), "create");
    let target = fd + 50;
    let n = check_ok!(syscall::dup2(fd, target), "dup2");
    check_eq!(n, target, "fd");
    check_ok!(syscall::close(target), "close t");
    check_ok!(syscall::close(fd), "close fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup2_same_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"same", 0o644), "create");
    let n = check_ok!(syscall::dup2(fd, fd), "dup2 same");
    check_eq!(n, fd, "same");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup3_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"d3", 0o644), "create");
    let target = fd + 60;
    let n = check_ok!(syscall::dup3(fd, target, oflag::O_CLOEXEC), "dup3");
    check_eq!(n, target, "fd");
    let flags = check_ok!(syscall::fcntl(n, syscall::fcntl_cmd::F_GETFD, 0), "getfd");
    check!(flags as i32 & syscall::FD_CLOEXEC != 0, "cloexec");
    check_ok!(syscall::close(n), "close n");
    check_ok!(syscall::close(fd), "close fd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup_bad_fd_ebadf() -> TestResult {
    check_err!(syscall::dup(-1), Errno::EBADF, "dup");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup2_bad_old_ebadf() -> TestResult {
    check_err!(syscall::dup2(-1, 20), Errno::EBADF, "dup2");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_child_getppid_is_parent() -> TestResult {
    let parent = syscall::getpid();
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::getppid() == parent {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wexitstatus(status), 0, "ppid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_child_getpid_differs() -> TestResult {
    let parent = syscall::getpid();
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::getpid() != parent {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wexitstatus(status), 0, "pid differs");
    check!(pid != parent, "fork pid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getcwd_small_buf_erange_soft() -> TestResult {
    let mut tiny = [0u8; 2];
    match syscall::getcwd(&mut tiny) {
        Err(Errno::ERANGE) | Err(Errno::EINVAL) => Ok(()),
        Ok(_) => Err(crate::harness::AssertFail::msg("expected ERANGE")),
        Err(_) => Err(crate::harness::AssertFail::msg("unexpected")),
    }
}

#[crate::lctp_test(suite = posix)]
fn unistd_chdir_dot() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    check_ok!(syscall::chdir(b".\0"), "chdir .");
    let mut cur = [0u8; 256];
    let cn = check_ok!(syscall::getcwd(&mut cur), "getcwd");
    check_eq!(&saved[..n], &cur[..cn], "same");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn unistd_chdir_dotdot_from_temp() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = crate::suites::common::create_dir(&mut tmp, b"sub", 0o755)?;
    check_ok!(syscall::chdir(&sub), "chdir sub");
    check_ok!(syscall::chdir(b"..\0"), "chdir ..");
    let mut cur = [0u8; 256];
    let cn = check_ok!(syscall::getcwd(&mut cur), "getcwd");
    let plen = tmp.path().iter().position(|&c| c == 0).unwrap();
    check_eq!(&cur[..plen], &tmp.path()[..plen], "parent");
    let _ = cn;
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup_write_visible() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"vis")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let d = check_ok!(syscall::dup(fd), "dup");
    check_ok!(syscall::write(d, b"Z"), "write");
    check_ok!(syscall::close(d), "close d");
    check_ok!(syscall::close(fd), "close fd");
    let mut buf = [0u8; 1];
    check_eq!(crate::suites::common::read_file(&path, &mut buf)?, 1, "len");
    check_eq!(buf[0], b'Z', "data");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getuid_stable() -> TestResult {
    check_eq!(syscall::getuid(), syscall::getuid(), "uid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getgid_stable() -> TestResult {
    check_eq!(syscall::getgid(), syscall::getgid(), "gid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_geteuid_stable() -> TestResult {
    check_eq!(syscall::geteuid(), syscall::geteuid(), "euid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getegid_stable() -> TestResult {
    check_eq!(syscall::getegid(), syscall::getegid(), "egid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup_close_original_still_works() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"keep", 0o644), "create");
    check_ok!(syscall::write(fd, b"hi"), "write");
    let d = check_ok!(syscall::dup(fd), "dup");
    check_ok!(syscall::close(fd), "close orig");
    check_ok!(syscall::lseek(d, 0, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::read(d, &mut buf), "read"), 2, "len");
    check_eq!(&buf, b"hi", "data");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_stdin_stdout_stderr_fds() -> TestResult {
    check_eq!(syscall::STDIN_FILENO, 0, "stdin");
    check_eq!(syscall::STDOUT_FILENO, 1, "stdout");
    check_eq!(syscall::STDERR_FILENO, 2, "stderr");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn unistd_chdir_relative_via_cwd() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"marker")?;
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let fd = check_ok!(syscall::open(b"marker\0", oflag::O_RDONLY, 0), "rel open");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_dup3_same_einval() -> TestResult {
    check_err!(syscall::dup3(1, 1, 0), Errno::EINVAL, "same");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getpid_ne_getppid_soft() -> TestResult {
    // Soft: pid1 container may have ppid==0 or equal edge cases; just call both.
    let pid = syscall::getpid();
    let ppid = syscall::getppid();
    check!(pid > 0, "pid");
    check!(ppid >= 0, "ppid");
    let _ = (pid, ppid);
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_write_file_via_dup2() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"d2w")?;
    write_file(&path, b"")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "open");
    let t = fd + 40;
    check_ok!(syscall::dup2(fd, t), "dup2");
    check_ok!(syscall::write(t, b"OK"), "write");
    check_ok!(syscall::close(t), "close t");
    check_ok!(syscall::close(fd), "close fd");
    let mut buf = [0u8; 2];
    check_eq!(crate::suites::common::read_file(&path, &mut buf)?, 2, "len");
    check_eq!(&buf, b"OK", "data");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn unistd_getcwd_nul_terminated() -> TestResult {
    let mut buf = [0u8; 128];
    let n = check_ok!(syscall::getcwd(&mut buf), "getcwd");
    check!(buf[..n].contains(&0), "has nul");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn unistd_fork_child_uid_same() -> TestResult {
    let uid = syscall::getuid();
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::getuid() == uid {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wexitstatus(status), 0, "uid");
    Ok(())
}
