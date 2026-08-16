//! Process group, session, and real/effective ID tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self};

#[crate::lctp_test(suite = syscall, expect = success, case = "getpgid(0) returns a positive process group id")]
fn getpgid_zero() -> TestResult {
    let pgid = check_ok!(syscall::getpgid(0), "getpgid 0");
    check!(pgid > 0, "pgid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getpgid of the calling pid returns a positive process group id")]
fn getpgid_self() -> TestResult {
    let pgid = check_ok!(syscall::getpgid(syscall::getpid()), "getpgid self");
    check!(pgid > 0, "pgid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getpgid(0) equals getpgid of the calling pid")]
fn getpgid_zero_equals_self() -> TestResult {
    let a = check_ok!(syscall::getpgid(0), "pgid 0");
    let b = check_ok!(syscall::getpgid(syscall::getpid()), "pgid pid");
    check_eq!(a, b, "match");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsid(0) returns a positive session id")]
fn getsid_zero() -> TestResult {
    let sid = check_ok!(syscall::getsid(0), "getsid 0");
    check!(sid > 0, "sid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsid of the calling pid returns a positive session id")]
fn getsid_self() -> TestResult {
    let sid = check_ok!(syscall::getsid(syscall::getpid()), "getsid self");
    check!(sid > 0, "sid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getresuid real, effective, and saved uids match getuid/geteuid")]
fn getresuid_matches() -> TestResult {
    let (r, e, s) = check_ok!(syscall::getresuid(), "getresuid");
    check_eq!(r, syscall::getuid(), "ruid");
    check_eq!(e, syscall::geteuid(), "euid");
    check_eq!(r, e, "real==eff");
    check_eq!(e, s, "eff==saved");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getresgid real, effective, and saved gids match getgid/getegid")]
fn getresgid_matches() -> TestResult {
    let (r, e, s) = check_ok!(syscall::getresgid(), "getresgid");
    check_eq!(r, syscall::getgid(), "rgid");
    check_eq!(e, syscall::getegid(), "egid");
    check_eq!(r, e, "real==eff");
    check_eq!(e, s, "eff==saved");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a child that is not a process-group leader can setsid and become session leader")]
fn setsid_in_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let sid = match syscall::setsid() {
            Ok(s) => s,
            Err(_) => syscall::exit(1),
        };
        if sid == syscall::getpid() {
            syscall::exit(0);
        }
        syscall::exit(2);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "setsid child");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a child can setpgid itself to its own pid")]
fn setpgid_self() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let pgid = syscall::getpid();
        if syscall::setpgid(0, pgid).is_ok() && syscall::getpgid(0).ok() == Some(pgid) {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "setpgid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsid(0) equals getsid of the calling pid")]
fn getsid_zero_equals_self() -> TestResult {
    let a = check_ok!(syscall::getsid(0), "sid 0");
    let b = check_ok!(syscall::getsid(syscall::getpid()), "sid pid");
    check_eq!(a, b, "match");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a child inherits the parent's process group")]
fn child_getpgid_parent_group() -> TestResult {
    let parent_pgid = check_ok!(syscall::getpgid(0), "parent pgid");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let pgid = syscall::getpgid(0).unwrap_or(-1);
        if pgid == parent_pgid {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "same pgid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getresuid reports a positive real or effective uid")]
fn getresuid_nonzero() -> TestResult {
    let (r, e, _) = check_ok!(syscall::getresuid(), "getresuid");
    check!(r > 0 || e > 0, "uid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getresgid reports a positive real or effective gid")]
fn getresgid_nonzero() -> TestResult {
    let (r, e, _) = check_ok!(syscall::getresgid(), "getresgid");
    check!(r > 0 || e > 0, "gid");
    Ok(())
}
