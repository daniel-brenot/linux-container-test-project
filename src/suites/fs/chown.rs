//! chown filesystem tests (unprivileged-only).
//!
//! Requires `syscall::chown` wrapper from the parent agent.

use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{self, Errno};

#[crate::lctp_test(suite = fs, expect = failure, case = "chown to another uid returns EPERM for an unprivileged caller")]
fn chown_other_uid_eperm() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let other = syscall::getuid().wrapping_add(1000);
    check_err!(
        syscall::chown(&path, other, syscall::getgid()),
        Errno::EPERM,
        "chown other uid"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "chown to uid 0 returns EPERM for an unprivileged caller")]
fn chown_root_uid_eperm() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::chown(&path, 0, 0),
        Errno::EPERM,
        "chown root"
    );
    Ok(())
}
