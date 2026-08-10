//! fallocate filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, write_file};
use crate::syscall::{
    self, oflag, Errno, FALLOC_FL_KEEP_SIZE, FALLOC_FL_PUNCH_HOLE, FALLOC_FL_ZERO_RANGE,
};

#[crate::lctp_test(suite = fs)]
fn fallocate_basic_grow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 4096), "fallocate");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 4096, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_offset_extend() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"ab")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 2, 100), "fallocate");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 102, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_keep_size() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"data")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(
        syscall::fallocate(fd, FALLOC_FL_KEEP_SIZE, 0, 4096),
        "keep size"
    );
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 4, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_zero_range_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"ABCDEFGH")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    match syscall::fallocate(fd, FALLOC_FL_ZERO_RANGE, 2, 4) {
        Ok(()) => {
            check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
            let mut buf = [0u8; 8];
            check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 8, "len");
            check_eq!(buf[0], b'A', "a");
            check_eq!(buf[1], b'B', "b");
            check_eq!(buf[2], 0, "z0");
            check_eq!(buf[3], 0, "z1");
            check_eq!(buf[4], 0, "z2");
            check_eq!(buf[5], 0, "z3");
            check_eq!(buf[6], b'G', "g");
            check_eq!(buf[7], b'H', "h");
        }
        Err(Errno::EOPNOTSUPP) | Err(Errno::ENOTSUP) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("zero range errno")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_punch_hole_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"0123456789ABCDEF")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    match syscall::fallocate(fd, FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE, 4, 4) {
        Ok(()) => {
            check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
            let mut buf = [0u8; 16];
            check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 16, "len");
            check_eq!(&buf[4..8], &[0, 0, 0, 0], "hole");
        }
        Err(Errno::EOPNOTSUPP) | Err(Errno::ENOTSUP) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("punch hole errno")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_bad_fd() -> TestResult {
    check_err!(syscall::fallocate(-1, 0, 0, 1), Errno::EBADF, "ebadf");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_zero_len_einval() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_err!(syscall::fallocate(fd, 0, 0, 0), Errno::EINVAL, "zero len");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_negative_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_err!(syscall::fallocate(fd, 0, -1, 10), Errno::EINVAL, "neg off");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_rdonly_ebadf_or_einval() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    match syscall::fallocate(fd, 0, 0, 100) {
        Err(Errno::EBADF) | Err(Errno::EINVAL) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("fallocate rdonly ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("fallocate rdonly errno")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_idempotent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 2048), "first");
    check_ok!(syscall::fallocate(fd, 0, 0, 2048), "second");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 2048, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_preserves_prefix() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"HEAD")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 100), "fallocate");
    check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 4, "len");
    check_eq!(&buf, b"HEAD", "prefix");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn fallocate_large() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 1_048_576), "1MiB");
    check_eq!(
        check_ok!(syscall::fstat(fd), "fstat").st_size,
        1_048_576,
        "size"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_beyond_current() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 1000, 50), "fallocate");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 1050, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_then_stat_path() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 512), "fallocate");
    check_ok!(syscall::close(fd), "close");
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 512, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_wronly_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 256), "fallocate");
    check_ok!(syscall::close(fd), "close");
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 256, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_small_len() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 1), "fallocate");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 1, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_after_trunc() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"abcdefgh")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::ftruncate(fd, 2), "ftruncate");
    check_ok!(syscall::fallocate(fd, 0, 0, 64), "fallocate");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 64, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_keep_size_beyond_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"x"), "write");
    check_ok!(
        syscall::fallocate(fd, FALLOC_FL_KEEP_SIZE, 0, 8192),
        "keep"
    );
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 1, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_multiple_regions() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 100), "r1");
    check_ok!(syscall::fallocate(fd, 0, 100, 100), "r2");
    check_eq!(check_ok!(syscall::fstat(fd), "fstat").st_size, 200, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn fallocate_path_still_reg() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fallocate(fd, 0, 0, 32), "fallocate");
    check_ok!(syscall::close(fd), "close");
    check!(check_ok!(syscall::stat(&path), "stat").is_reg(), "reg");
    Ok(())
}
