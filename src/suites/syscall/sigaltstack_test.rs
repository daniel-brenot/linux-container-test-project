//! sigaltstack(2) tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, map, prot, Stack, SS_DISABLE};

const ALT_SIZE: usize = 16 * 1024;

#[crate::lctp_test(suite = syscall)]
fn sigaltstack_query_default() -> TestResult {
    let mut old = Stack::default();
    check_ok!(syscall::sigaltstack(None, Some(&mut old)), "query");
    // No alternate stack configured initially in a fresh process — often SS_DISABLE.
    // After other tests may have set one; accept disable or a valid size.
    if old.ss_flags & SS_DISABLE != 0 {
        Ok(())
    } else {
        check!(old.ss_size >= 2048, "configured size");
        Ok(())
    }
}

#[crate::lctp_test(suite = syscall)]
fn sigaltstack_set_and_query() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            ALT_SIZE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    let ss = Stack {
        ss_sp: addr as *mut u8,
        ss_flags: 0,
        ss_size: ALT_SIZE,
    };
    check_ok!(syscall::sigaltstack(Some(&ss), None), "set");
    let mut cur = Stack::default();
    check_ok!(syscall::sigaltstack(None, Some(&mut cur)), "get");
    check!(cur.ss_flags & SS_DISABLE == 0, "enabled");
    check_eq!(cur.ss_size, ALT_SIZE, "size");
    check_eq!(cur.ss_sp as usize, addr, "sp");

    // Disable and restore a clean state for later tests.
    let dis = Stack {
        ss_sp: core::ptr::null_mut(),
        ss_flags: SS_DISABLE,
        ss_size: 0,
    };
    check_ok!(syscall::sigaltstack(Some(&dis), None), "disable");
    let mut after = Stack::default();
    check_ok!(syscall::sigaltstack(None, Some(&mut after)), "query disabled");
    check!(after.ss_flags & SS_DISABLE != 0, "SS_DISABLE");
    check_ok!(syscall::munmap(addr, ALT_SIZE), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sigaltstack_replace_returns_old() -> TestResult {
    let a1 = check_ok!(
        syscall::mmap(
            0,
            ALT_SIZE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap1"
    );
    let a2 = check_ok!(
        syscall::mmap(
            0,
            ALT_SIZE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap2"
    );
    let s1 = Stack {
        ss_sp: a1 as *mut u8,
        ss_flags: 0,
        ss_size: ALT_SIZE,
    };
    check_ok!(syscall::sigaltstack(Some(&s1), None), "set1");
    let s2 = Stack {
        ss_sp: a2 as *mut u8,
        ss_flags: 0,
        ss_size: ALT_SIZE,
    };
    let mut old = Stack::default();
    check_ok!(syscall::sigaltstack(Some(&s2), Some(&mut old)), "replace");
    check_eq!(old.ss_sp as usize, a1, "old sp");
    check_eq!(old.ss_size, ALT_SIZE, "old size");

    let dis = Stack {
        ss_sp: core::ptr::null_mut(),
        ss_flags: SS_DISABLE,
        ss_size: 0,
    };
    check_ok!(syscall::sigaltstack(Some(&dis), None), "disable");
    check_ok!(syscall::munmap(a1, ALT_SIZE), "munmap1");
    check_ok!(syscall::munmap(a2, ALT_SIZE), "munmap2");
    Ok(())
}
