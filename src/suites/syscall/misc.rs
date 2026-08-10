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

#[crate::lctp_test(suite = syscall)]
fn membarrier_query() -> TestResult {
    let mask = check_ok!(
        syscall::membarrier(syscall::MEMBARRIER_CMD_QUERY, 0),
        "query"
    );
    // Supported command bitmask; may be zero on unusual kernels but QUERY itself succeeds.
    let _ = mask;
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn personality_query() -> TestResult {
    // 0xffffffff asks for the current personality without changing it.
    let p = check_ok!(syscall::personality(0xffff_ffff), "personality");
    // Query again; value should be stable.
    let p2 = check_ok!(syscall::personality(0xffff_ffff), "personality2");
    check_eq!(p, p2, "stable");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn capget_v3() -> TestResult {
    let mut hdr = syscall::CapUserHeader {
        version: syscall::LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let mut data = [syscall::CapUserData::default(); 2];
    check_ok!(syscall::capget(&mut hdr, &mut data), "capget");
    // Unprivileged process: effective caps are typically empty, but data is filled.
    let _ = data[0].effective | data[0].permitted | data[1].effective;
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn membarrier_query_nonzero_or_zero() -> TestResult {
    let mask = check_ok!(syscall::membarrier(syscall::MEMBARRIER_CMD_QUERY, 0), "q");
    // Just ensure we can call it twice with a consistent result.
    let mask2 = check_ok!(syscall::membarrier(syscall::MEMBARRIER_CMD_QUERY, 0), "q2");
    check_eq!(mask, mask2, "stable mask");
    Ok(())
}
