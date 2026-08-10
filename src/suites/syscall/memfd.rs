//! memfd_create and anonymous file tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, fcntl_cmd, F_ADD_SEALS, F_GET_SEALS, F_SEAL_WRITE, FD_CLOEXEC, MFD_ALLOW_SEALING, MFD_CLOEXEC};

#[crate::lctp_test(suite = syscall)]
fn memfd_create_basic() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"lctp\0", 0), "create");
    check!(fd >= 0, "bad fd");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn memfd_create_cloexec() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"x\0", MFD_CLOEXEC as u32), "create");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(fl & FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn memfd_write_read() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"wr\0", 0), "create");
    let msg = b"memfd-data";
    check_eq!(check_ok!(syscall::write(fd, msg), "write"), msg.len(), "wlen");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), msg.len(), "rlen");
    check!(&buf[..msg.len()] == msg, "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn memfd_ftruncate_grow() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"grow\0", 0), "create");
    check_ok!(syscall::ftruncate(fd, 4096), "ftruncate");
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check_eq!(st.st_size, 4096, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn memfd_ftruncate_shrink() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"shrink\0", 0), "create");
    check_ok!(syscall::write(fd, b"0123456789"), "write");
    check_ok!(syscall::ftruncate(fd, 4), "ftruncate");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::read(fd, &mut buf), "read");
    check_eq!(n, 4, "len");
    check_eq!(&buf[..4], b"0123", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn memfd_pwrite_pread() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"pp\0", 0), "create");
    check_ok!(syscall::pwrite(fd, b"ABCD", 100), "pwrite");
    let mut buf = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut buf, 100), "pread");
    check_eq!(&buf, b"ABCD", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn memfd_seal_write() -> TestResult {
    let fd = check_ok!(
        syscall::memfd_create(b"seal\0", MFD_ALLOW_SEALING as u32),
        "create"
    );
    check_ok!(syscall::write(fd, b"x"), "write");
    check_ok!(syscall::fcntl(fd, F_ADD_SEALS, F_SEAL_WRITE as usize), "add seal");
    let seals = check_ok!(syscall::fcntl(fd, F_GET_SEALS, 0), "get seals");
    check!(seals & F_SEAL_WRITE as usize != 0, "sealed");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn memfd_empty_name() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"\0", 0), "create empty name");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn memfd_size_after_write() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"sz\0", 0), "create");
    let data = [0xCDu8; 512];
    check_ok!(syscall::write(fd, &data), "write");
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check_eq!(st.st_size, 512, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn memfd_grow_then_write() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"gw\0", 0), "create");
    check_ok!(syscall::ftruncate(fd, 8192), "grow");
    check_ok!(syscall::pwrite(fd, b"Z", 8191), "pwrite end");
    let mut b = [0u8; 1];
    check_ok!(syscall::pread(fd, &mut b, 8191), "pread");
    check_eq!(b[0], b'Z', "byte");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn memfd_cloexec_and_write() -> TestResult {
    let fd = check_ok!(syscall::memfd_create(b"ce\0", MFD_CLOEXEC as u32), "create");
    check_ok!(syscall::write(fd, b"ok"), "write");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
