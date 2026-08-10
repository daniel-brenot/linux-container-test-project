//! prctl thread/process name tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self};

#[crate::lctp_test(suite = syscall)]
fn prctl_set_get_name() -> TestResult {
    let name = b"lctp-test\0";
    check_ok!(syscall::prctl_set_name(name), "set name");
    let mut buf = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "get name");
    check_eq!(&buf[..9], b"lctp-test", "name");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prctl_get_name_nul_terminated() -> TestResult {
    let mut buf = [0xFFu8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "get name");
    check!(buf.iter().any(|&b| b == 0), "nul terminated");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn prctl_set_name_roundtrip() -> TestResult {
    for &name in &[b"a\0" as &[u8], b"xy\0", b"longname12\0"] {
        check_ok!(syscall::prctl_set_name(name), "set");
        let mut buf = [0u8; 16];
        check_ok!(syscall::prctl_get_name(&mut buf), "get");
        let nlen = name.iter().position(|&c| c == 0).unwrap();
        check_eq!(&buf[..nlen], &name[..nlen], "match");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn prctl_name_in_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::prctl_set_name(b"child\0").is_ok() {
            let mut buf = [0u8; 16];
            if syscall::prctl_get_name(&mut buf).is_ok() && &buf[..5] == b"child" {
                syscall::exit(0);
            }
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "child name");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prctl_set_short_name() -> TestResult {
    check_ok!(syscall::prctl_set_name(b"z\0"), "set");
    let mut buf = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "get");
    check_eq!(buf[0], b'z', "char");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prctl_name_max_15_chars() -> TestResult {
    let name = b"123456789012345\0";
    check_ok!(syscall::prctl_set_name(name), "set max");
    let mut buf = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "get");
    check_eq!(&buf[..15], b"123456789012345", "15 chars");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn prctl_restore_name() -> TestResult {
    let mut orig = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut orig), "orig");
    check_ok!(syscall::prctl_set_name(b"tempname\0"), "temp");
    check_ok!(syscall::prctl_set_name(&orig), "restore");
    let mut buf = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "get");
    let end = orig.iter().position(|&c| c == 0).unwrap_or(16);
    check_eq!(&buf[..end], &orig[..end], "restored");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prctl_get_name_readable() -> TestResult {
    let mut buf = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "get");
    // Name should be printable ASCII or empty.
    let end = buf.iter().position(|&c| c == 0).unwrap_or(16);
    for &c in &buf[..end] {
        check!(c >= 0x20 && c <= 0x7e || c == 0, "printable");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prctl_get_set_dumpable() -> TestResult {
    let orig = check_ok!(
        syscall::prctl(syscall::PR_GET_DUMPABLE, 0, 0, 0, 0),
        "get dumpable"
    );
    check!(orig == 0 || orig == 1 || orig == 2, "dumpable range");
    check_ok!(
        syscall::prctl(syscall::PR_SET_DUMPABLE, 1, 0, 0, 0),
        "set dumpable 1"
    );
    let v = check_ok!(
        syscall::prctl(syscall::PR_GET_DUMPABLE, 0, 0, 0, 0),
        "get after set"
    );
    check_eq!(v, 1, "dumpable 1");
    check_ok!(
        syscall::prctl(syscall::PR_SET_DUMPABLE, orig as usize, 0, 0, 0),
        "restore"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn prctl_get_no_new_privs() -> TestResult {
    let v = check_ok!(
        syscall::prctl(syscall::PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0),
        "get nnp"
    );
    check!(v == 0 || v == 1, "nnp 0/1");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn prctl_set_dumpable_zero_readback() -> TestResult {
    let orig = check_ok!(
        syscall::prctl(syscall::PR_GET_DUMPABLE, 0, 0, 0, 0),
        "orig"
    );
    check_ok!(
        syscall::prctl(syscall::PR_SET_DUMPABLE, 0, 0, 0, 0),
        "set 0"
    );
    let v = check_ok!(
        syscall::prctl(syscall::PR_GET_DUMPABLE, 0, 0, 0, 0),
        "get"
    );
    check_eq!(v, 0, "dumpable 0");
    let _ = syscall::prctl(syscall::PR_SET_DUMPABLE, orig as usize, 0, 0, 0);
    Ok(())
}
