//! kcmp(2) tests.
//!
//! Default container seccomp allows `kcmp` only with `CAP_SYS_PTRACE`, so
//! unprivileged runs must accept EPERM/ENOSYS.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{self, oflag, Errno, KCMP_FILE};

fn kcmp_denied(e: Errno) -> bool {
    matches!(e, Errno::EPERM | Errno::ENOSYS | Errno::EACCES | Errno::EINVAL)
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "kcmp KCMP_FILE of the same fd against itself returns 0, or is rejected as unprivileged")]
fn kcmp_same_fd_equal() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"k")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let pid = syscall::getpid();
    match syscall::kcmp(pid, pid, KCMP_FILE, fd as u64, fd as u64) {
        Ok(cmp) => check_eq!(cmp, 0, "same fd"),
        Err(e) if kcmp_denied(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("kcmp same"));
        }
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "kcmp KCMP_FILE of a fd and its dup returns 0, or is rejected as unprivileged")]
fn kcmp_dup_fds_equal() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"kd")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let dup = check_ok!(syscall::dup(fd), "dup");
    let pid = syscall::getpid();
    match syscall::kcmp(pid, pid, KCMP_FILE, fd as u64, dup as u64) {
        Ok(cmp) => check_eq!(cmp, 0, "dup shares file"),
        Err(e) if kcmp_denied(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            let _ = syscall::close(dup);
            return Err(crate::harness::AssertFail::msg("kcmp dup"));
        }
    }
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(dup), "close dup");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "kcmp KCMP_FILE of two distinct files returns nonzero, or is rejected as unprivileged")]
fn kcmp_distinct_files_unequal() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    let fda = check_ok!(syscall::open(&a, oflag::O_RDONLY, 0), "open a");
    let fdb = check_ok!(syscall::open(&b, oflag::O_RDONLY, 0), "open b");
    let pid = syscall::getpid();
    match syscall::kcmp(pid, pid, KCMP_FILE, fda as u64, fdb as u64) {
        Ok(cmp) => check!(cmp != 0, "distinct files differ"),
        Err(e) if kcmp_denied(e) => {}
        Err(_) => {
            let _ = syscall::close(fda);
            let _ = syscall::close(fdb);
            return Err(crate::harness::AssertFail::msg("kcmp distinct"));
        }
    }
    check_ok!(syscall::close(fda), "close a");
    check_ok!(syscall::close(fdb), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "two kcmp KCMP_FILE comparisons of the same distinct fds return the same nonzero order, or kcmp is rejected as unprivileged")]
fn kcmp_order_stable() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    let fda = check_ok!(syscall::open(&a, oflag::O_RDONLY, 0), "open a");
    let fdb = check_ok!(syscall::open(&b, oflag::O_RDONLY, 0), "open b");
    let pid = syscall::getpid();
    let ab1 = match syscall::kcmp(pid, pid, KCMP_FILE, fda as u64, fdb as u64) {
        Ok(v) => v,
        Err(e) if kcmp_denied(e) => {
            check_ok!(syscall::close(fda), "close a");
            check_ok!(syscall::close(fdb), "close b");
            return Ok(());
        }
        Err(_) => {
            let _ = syscall::close(fda);
            let _ = syscall::close(fdb);
            return Err(crate::harness::AssertFail::msg("a vs b"));
        }
    };
    let ab2 = check_ok!(
        syscall::kcmp(pid, pid, KCMP_FILE, fda as u64, fdb as u64),
        "a vs b again"
    );
    check!(ab1 != 0, "unequal");
    check_eq!(ab1, ab2, "stable order");
    check_ok!(syscall::close(fda), "close a");
    check_ok!(syscall::close(fdb), "close b");
    Ok(())
}
