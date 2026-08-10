//! Miscellaneous syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::suites::common::cstr_prefix;
use crate::syscall::{self, RLIMIT_NOFILE, Rlimit};

#[crate::lctp_test(suite = syscall)]
fn uname_linux() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname");
    let sys = cstr_prefix(&u.sysname);
    check!(sys.starts_with(b"Linux"), "not Linux");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn uname_fields_nonempty() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname");
    check!(!cstr_prefix(&u.release).is_empty(), "release empty");
    check!(!cstr_prefix(&u.machine).is_empty(), "machine empty");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getrandom_nonzero() -> TestResult {
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    check_eq!(check_ok!(syscall::getrandom(&mut a, 0), "getrandom a"), 32, "len");
    check_ok!(syscall::getrandom(&mut b, 0), "getrandom b");
    check!(a != b, "identical random");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn getrandom_partial() -> TestResult {
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::getrandom(&mut buf, 0), "getrandom"), 16, "len");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sched_yield() -> TestResult {
    check_ok!(syscall::sched_yield(), "sched_yield");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prlimit_nofile_get() -> TestResult {
    let mut lim = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, None, Some(&mut lim)),
        "prlimit get"
    );
    check!(lim.rlim_cur > 0, "soft limit zero");
    check!(lim.rlim_max > 0, "hard limit zero");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prlimit_nofile_positive() -> TestResult {
    let mut old = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, None, Some(&mut old)),
        "prlimit"
    );
    check!(old.rlim_cur >= 32, "nofile soft too small");
    Ok(())
}
