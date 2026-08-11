//! Device and `/proc` entropy surfaces used by userspace RNG initialization.
//!
//! Guests that only implement `open("/dev/urandom")` without directory
//! visibility, `stat` metadata, or `/proc/sys/kernel/random/*` break real
//! runtimes even when `getrandom(2)` alone appears to work.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, oflag, Errno, F_OK, R_OK};

fn read_all_small(path: &[u8], buf: &mut [u8]) -> Result<usize, crate::harness::AssertFail> {
    let fd = check_ok!(syscall::open(path, oflag::O_RDONLY, 0), "open");
    let n = match syscall::read(fd, buf) {
        Ok(n) => n,
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("read"));
        }
    };
    check_ok!(syscall::close(fd), "close");
    Ok(n)
}

fn is_ascii_digit(b: u8) -> bool {
    b.is_ascii_digit()
}

#[crate::lctp_test(suite = syscall)]
fn urandom_stat_is_chr() -> TestResult {
    let st = check_ok!(syscall::stat(b"/dev/urandom\0"), "stat /dev/urandom");
    check!(st.is_chr(), "not character device");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn urandom_lstat_is_chr() -> TestResult {
    let st = check_ok!(syscall::lstat(b"/dev/urandom\0"), "lstat /dev/urandom");
    check!(st.is_chr(), "not character device");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn urandom_access_rw() -> TestResult {
    check_ok!(syscall::access(b"/dev/urandom\0", F_OK), "F_OK");
    check_ok!(syscall::access(b"/dev/urandom\0", R_OK), "R_OK");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn urandom_open_fstat_read() -> TestResult {
    let fd = check_ok!(
        syscall::open(b"/dev/urandom\0", oflag::O_RDONLY, 0),
        "open"
    );
    let st = match syscall::fstat(fd) {
        Ok(st) => st,
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("fstat"));
        }
    };
    check!(st.is_chr(), "fstat not chr");
    let mut buf = [0u8; 32];
    let n = match syscall::read(fd, &mut buf) {
        Ok(n) => n,
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("read"));
        }
    };
    check_eq!(n, 32, "read len");
    check_ok!(syscall::close(fd), "close");
    // Extremely unlikely all-zero from a working RNG; still accept if a
    // synthetic source fills zeros but the length must be full.
    let _ = buf;
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn urandom_two_reads_differ() -> TestResult {
    let mut a = [0u8; 64];
    let mut b = [0u8; 64];
    let n1 = read_all_small(b"/dev/urandom\0", &mut a)?;
    let n2 = read_all_small(b"/dev/urandom\0", &mut b)?;
    check_eq!(n1, 64, "n1");
    check_eq!(n2, 64, "n2");
    check!(a != b, "identical urandom reads");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn random_stat_is_chr_soft() -> TestResult {
    match syscall::stat(b"/dev/random\0") {
        Ok(st) => check!(st.is_chr(), "not character device"),
        Err(Errno::ENOENT) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("stat /dev/random")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn random_open_read_soft() -> TestResult {
    match syscall::open(b"/dev/random\0", oflag::O_RDONLY | oflag::O_NONBLOCK, 0) {
        Ok(fd) => {
            let mut buf = [0u8; 16];
            match syscall::read(fd, &mut buf) {
                Ok(n) => check!(n > 0, "empty random read"),
                // Non-blocking can legitimately return EAGAIN when the pool is empty.
                Err(Errno::EAGAIN) | Err(Errno::EWOULDBLOCK) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("read /dev/random"));
                }
            }
            check_ok!(syscall::close(fd), "close");
        }
        Err(Errno::ENOENT) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("open /dev/random")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn dev_dir_lists_urandom() -> TestResult {
    let fd = check_ok!(
        syscall::open(b"/dev\0", oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir /dev"
    );
    let mut buf = [0u8; 4096];
    let mut found = false;
    loop {
        let n = match syscall::getdents64(fd, &mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => {
                let _ = syscall::close(fd);
                return Err(crate::harness::AssertFail::msg("getdents64 /dev"));
            }
        };
        let mut off = 0usize;
        while off + 19 <= n {
            let reclen = u16::from_le_bytes([buf[off + 16], buf[off + 17]]) as usize;
            if reclen == 0 || off + reclen > n {
                break;
            }
            let name_start = off + 19;
            let name_end = (name_start..off + reclen)
                .find(|&i| buf[i] == 0)
                .unwrap_or(off + reclen);
            let name = &buf[name_start..name_end];
            if name == b"urandom" {
                found = true;
            }
            off += reclen;
        }
        if found {
            break;
        }
    }
    check_ok!(syscall::close(fd), "close");
    check!(found, "/dev listing missing urandom");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn proc_random_entropy_avail() -> TestResult {
    let mut buf = [0u8; 64];
    let n = read_all_small(b"/proc/sys/kernel/random/entropy_avail\0", &mut buf)?;
    check!(n > 0, "empty entropy_avail");
    // Trim trailing newline; require at least one digit.
    let end = buf[..n].iter().position(|&b| b == b'\n').unwrap_or(n);
    check!(end > 0, "no digits");
    check!(buf[..end].iter().all(|&b| is_ascii_digit(b)), "non-digit");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn proc_random_uuid() -> TestResult {
    let mut buf = [0u8; 64];
    let n = read_all_small(b"/proc/sys/kernel/random/uuid\0", &mut buf)?;
    check!(n >= 36, "uuid too short");
    // 8-4-4-4-12 hex with dashes (newline optional).
    let line_end = buf[..n].iter().position(|&b| b == b'\n').unwrap_or(n);
    check_eq!(line_end, 36, "uuid len");
    let u = &buf[..36];
    for (i, &b) in u.iter().enumerate() {
        let ok = match i {
            8 | 13 | 18 | 23 => b == b'-',
            _ => matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'),
        };
        check!(ok, "uuid format");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn proc_random_poolsize() -> TestResult {
    let mut buf = [0u8; 64];
    let n = read_all_small(b"/proc/sys/kernel/random/poolsize\0", &mut buf)?;
    check!(n > 0, "empty poolsize");
    let end = buf[..n].iter().position(|&b| b == b'\n').unwrap_or(n);
    check!(end > 0 && buf[..end].iter().all(|&b| is_ascii_digit(b)), "digits");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn proc_random_boot_id_soft() -> TestResult {
    match syscall::access(b"/proc/sys/kernel/random/boot_id\0", R_OK) {
        Ok(()) => {
            let mut buf = [0u8; 64];
            let n = read_all_small(b"/proc/sys/kernel/random/boot_id\0", &mut buf)?;
            check!(n >= 36, "boot_id short");
        }
        Err(Errno::ENOENT) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("access boot_id")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getrandom_matches_urandom_surface() -> TestResult {
    // Both entropy paths must produce full buffers.
    let mut g = [0u8; 32];
    let mut u = [0u8; 32];
    check_eq!(check_ok!(syscall::getrandom(&mut g, 0), "getrandom"), 32, "gr");
    let n = read_all_small(b"/dev/urandom\0", &mut u)?;
    check_eq!(n, 32, "ur");
    Ok(())
}
