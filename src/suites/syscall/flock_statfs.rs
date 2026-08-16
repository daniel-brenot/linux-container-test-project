//! flock and statfs syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{self, Errno, LOCK_EX, LOCK_NB, LOCK_SH, LOCK_UN};

#[crate::lctp_test(suite = syscall, expect = success, case = "flock LOCK_EX then LOCK_UN on a regular file both succeed")]
fn flock_exclusive_unlock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fl")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock ex");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "flock LOCK_SH then LOCK_UN on a regular file both succeed")]
fn flock_shared_unlock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fls")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_SH), "lock sh");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "flock can upgrade LOCK_SH to LOCK_EX, downgrade back to LOCK_SH, then unlock")]
fn flock_upgrade_downgrade() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flu")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_SH), "sh");
    check_ok!(syscall::flock(fd, LOCK_EX), "ex");
    check_ok!(syscall::flock(fd, LOCK_SH), "sh again");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = failure, case = "flock LOCK_EX|LOCK_NB on a file already exclusively locked by the parent returns EWOULDBLOCK")]
fn flock_nb_contention_fork() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flnb")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "parent lock");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let cfd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open child");
        match syscall::flock(cfd, LOCK_EX | LOCK_NB) {
            Err(Errno::EWOULDBLOCK) => syscall::exit(0),
            Ok(()) => syscall::exit(1),
            Err(_) => syscall::exit(2),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "child EWOULDBLOCK");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statfs of /tmp reports a positive block size and block count")]
fn statfs_tmp() -> TestResult {
    let st = check_ok!(syscall::statfs(b"/tmp\0"), "statfs /tmp");
    check!(st.f_bsize > 0, "bsize");
    check!(st.f_blocks > 0, "blocks");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statfs of / reports a positive block size")]
fn statfs_root() -> TestResult {
    let st = check_ok!(syscall::statfs(b"/\0"), "statfs /");
    check!(st.f_bsize > 0, "bsize");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "fstatfs of a temporary file reports a positive block size")]
fn fstatfs_temp_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"sf")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    let st = check_ok!(syscall::fstatfs(fd), "fstatfs");
    check!(st.f_bsize > 0, "bsize");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statfs of a temporary directory reports positive bsize and namelen")]
fn statfs_temp_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let st = check_ok!(syscall::statfs(tmp.path()), "statfs temp");
    check!(st.f_bsize > 0, "bsize");
    check!(st.f_namelen > 0, "namelen");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "flock LOCK_EX succeeds again after LOCK_UN on the same fd")]
fn flock_relock_after_unlock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flr")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock1");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::flock(fd, LOCK_EX), "lock2");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock2");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statfs of /tmp reports f_bavail less than or equal to f_bfree")]
fn statfs_bavail_le_bfree() -> TestResult {
    let st = check_ok!(syscall::statfs(b"/tmp\0"), "statfs");
    check!(st.f_bavail <= st.f_bfree, "bavail");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statfs of a temp directory and fstatfs of a file in it report the same f_type and f_bsize")]
fn fstatfs_matches_statfs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"cmp")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDONLY, 0), "open");
    let a = check_ok!(syscall::statfs(tmp.path()), "statfs dir");
    let b = check_ok!(syscall::fstatfs(fd), "fstatfs fd");
    check_eq!(a.f_type, b.f_type, "type");
    check_eq!(a.f_bsize, b.f_bsize, "bsize");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "flock LOCK_EX|LOCK_NB succeeds on an unlocked file")]
fn flock_nb_success() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flnbok")?;
    let fd = check_ok!(syscall::open(&path, crate::syscall::oflag::O_RDWR, 0), "open");
    check_ok!(syscall::flock(fd, LOCK_EX | LOCK_NB), "lock nb");
    check_ok!(syscall::flock(fd, LOCK_UN), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
