//! Shared helpers for test suites.

use crate::check_ok;
use crate::harness::{AssertFail, TempDir};
use crate::syscall::{self, oflag};

pub fn copy_child(tmp: &mut TempDir, name: &[u8]) -> Result<[u8; 128], AssertFail> {
    let p = check_ok!(tmp.child(name), "child path");
    let mut b = [0u8; 128];
    b[..p.len()].copy_from_slice(p);
    Ok(b)
}

/// Return a NUL-terminated slice covering the C string in `buf`.
pub fn truncate_cstr(buf: &[u8]) -> &[u8] {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    &buf[..end + 1]
}

/// Join `base` and `name` into `out`, NUL-terminated.
pub fn join_path<'a>(
    base: &[u8],
    name: &[u8],
    out: &'a mut [u8; 160],
) -> Result<&'a [u8], AssertFail> {
    let blen = base.iter().position(|&c| c == 0).unwrap_or(base.len());
    let nlen = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    let need = blen + 1 + nlen + 1;
    if need > out.len() {
        return Err(AssertFail::msg("join_path buffer too small"));
    }
    out[..blen].copy_from_slice(&base[..blen]);
    out[blen] = b'/';
    out[blen + 1..blen + 1 + nlen].copy_from_slice(&name[..nlen]);
    out[blen + 1 + nlen] = 0;
    Ok(truncate_cstr(out))
}

pub fn write_file(path: &[u8], data: &[u8]) -> Result<(), AssertFail> {
    let fd = check_ok!(
        syscall::open(path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "write_file open"
    );
    if !data.is_empty() {
        check_ok!(syscall::write(fd, data), "write_file write");
    }
    check_ok!(syscall::close(fd), "write_file close");
    Ok(())
}

pub fn read_file(path: &[u8], buf: &mut [u8]) -> Result<usize, AssertFail> {
    let fd = check_ok!(syscall::open(path, oflag::O_RDONLY, 0), "read_file open");
    let n = check_ok!(syscall::read(fd, buf), "read_file read");
    check_ok!(syscall::close(fd), "read_file close");
    Ok(n)
}

pub fn create_empty(tmp: &mut TempDir, name: &[u8]) -> Result<[u8; 128], AssertFail> {
    let path = copy_child(tmp, name)?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "create_empty"
    );
    check_ok!(syscall::close(fd), "create_empty close");
    Ok(path)
}

pub fn create_dir(tmp: &mut TempDir, name: &[u8], mode: u32) -> Result<[u8; 128], AssertFail> {
    let path = copy_child(tmp, name)?;
    check_ok!(syscall::mkdir(&path, mode), "create_dir mkdir");
    Ok(path)
}

pub fn cstr_prefix(buf: &[u8]) -> &[u8] {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    &buf[..end]
}

pub fn chmod_path(path: &[u8], mode: u32) -> Result<(), AssertFail> {
    check_ok!(syscall::chmod(path, mode), "chmod_path");
    Ok(())
}

/// Sleep for `secs` whole seconds via `nanosleep(2)`.
pub fn nanosleep_secs(secs: i64) -> Result<(), AssertFail> {
    let req = syscall::Timespec {
        tv_sec: secs,
        tv_nsec: 0,
    };
    check_ok!(syscall::nanosleep(&req), "nanosleep");
    Ok(())
}

/// True if `(a_sec, a_nsec)` is strictly later than `(b_sec, b_nsec)`.
pub fn timespec_later(a_sec: i64, a_nsec: i64, b_sec: i64, b_nsec: i64) -> bool {
    a_sec > b_sec || (a_sec == b_sec && a_nsec > b_nsec)
}
