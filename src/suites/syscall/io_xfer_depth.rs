//! sendfile/splice/tee/fallocate/sync_file_range/copy_file_range depth.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, read_file, write_file};
use crate::syscall::{
    self, oflag, Errno, FALLOC_FL_KEEP_SIZE, FALLOC_FL_PUNCH_HOLE, FALLOC_FL_ZERO_RANGE,
    SYNC_FILE_RANGE_WAIT_AFTER, SYNC_FILE_RANGE_WAIT_BEFORE, SYNC_FILE_RANGE_WRITE,
};

fn make_in(tmp: &mut TempDir, name: &[u8], data: &[u8]) -> Result<i32, crate::harness::AssertFail> {
    let path = copy_child(tmp, name)?;
    write_file(&path, data)?;
    Ok(check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open in"))
}

fn soft_falloc(e: Errno) -> bool {
    matches!(
        e,
        Errno::EOPNOTSUPP | Errno::ENOTSUP | Errno::EINVAL | Errno::ENOSYS | Errno::EPERM | Errno::ENOSPC
    )
}

#[crate::lctp_test(suite = syscall)]
fn sendfile_sizes_1() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"1";
    let in_fd = make_in(&mut tmp, b"s1", data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 0i64;
    check_eq!(check_ok!(syscall::sendfile(w, in_fd, &mut off, 1), "sf"), 1, "n");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 1];
    check_eq!(check_ok!(syscall::read(r, &mut b), "r"), 1, "r");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sendfile_sizes_16() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"0123456789abcdef";
    let in_fd = make_in(&mut tmp, b"s16", data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 0i64;
    check_eq!(check_ok!(syscall::sendfile(w, in_fd, &mut off, 16), "sf"), 16, "n");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 16];
    check_eq!(check_ok!(syscall::read(r, &mut b), "r"), 16, "r");
    check_eq!(&b, data, "d");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sendfile_sizes_256() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let mut data = [b'A'; 256];
    for (i, b) in data.iter_mut().enumerate() {
        *b = b'A' + (i % 26) as u8;
    }
    let in_fd = make_in(&mut tmp, b"s256", &data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 0i64;
    let n = check_ok!(syscall::sendfile(w, in_fd, &mut off, 256), "sf");
    check_eq!(n, 256, "n");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 256];
    check_eq!(check_ok!(syscall::read(r, &mut b), "r"), 256, "r");
    check!(&b == &data, "d");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sendfile_sizes_1024() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = [0x5Au8; 1024];
    let in_fd = make_in(&mut tmp, b"s1k", &data)?;
    let out_path = copy_child(&mut tmp, b"o1k")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "out"
    );
    let mut off = 0i64;
    check_eq!(check_ok!(syscall::sendfile(out_fd, in_fd, &mut off, 1024), "sf"), 1024, "n");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(out_fd), "co");
    let mut buf = [0u8; 1024];
    check_eq!(read_file(&out_path, &mut buf)?, 1024, "len");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sendfile_offset_advance() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"soa", b"ABCDEFGH")?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 2i64;
    check_eq!(check_ok!(syscall::sendfile(w, in_fd, &mut off, 3), "sf"), 3, "n");
    check_eq!(off, 5, "off");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 3];
    check_ok!(syscall::read(r, &mut b), "r");
    check_eq!(&b, b"CDE", "d");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn splice_sizes_8() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    check_ok!(syscall::write(w1, b"12345678"), "w");
    check_ok!(syscall::close(w1), "cw1");
    let n = check_ok!(syscall::splice(r1, None, w2, None, 8, 0), "sp");
    check_eq!(n, 8, "n");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    let mut b = [0u8; 8];
    check_eq!(check_ok!(syscall::read(r2, &mut b), "r"), 8, "r");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn splice_sizes_1() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    check_ok!(syscall::write(w1, b"Z"), "w");
    let n = check_ok!(syscall::splice(r1, None, w2, None, 1, 0), "sp");
    check_eq!(n, 1, "n");
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn splice_sizes_64() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    let msg = [b'x'; 64];
    check_ok!(syscall::write(w1, &msg), "w");
    check_ok!(syscall::close(w1), "cw1");
    let n = check_ok!(syscall::splice(r1, None, w2, None, 64, 0), "sp");
    check_eq!(n, 64, "n");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    let mut b = [0u8; 64];
    check_eq!(check_ok!(syscall::read(r2, &mut b), "r"), 64, "r");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tee_sizes_1() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    check_ok!(syscall::write(w1, b"Q"), "w");
    check_eq!(check_ok!(syscall::tee(r1, w2, 1, 0), "tee"), 1, "n");
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tee_sizes_16() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    let msg = b"0123456789abcdef";
    check_ok!(syscall::write(w1, msg), "w");
    check_eq!(check_ok!(syscall::tee(r1, w2, 16, 0), "tee"), 16, "n");
    let mut a = [0u8; 16];
    let mut b = [0u8; 16];
    check_ok!(syscall::read(r1, &mut a), "a");
    check_ok!(syscall::read(r2, &mut b), "b");
    check_eq!(&a, msg, "a");
    check_eq!(&b, msg, "b");
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn tee_sizes_32() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    let msg = [b'T'; 32];
    check_ok!(syscall::write(w1, &msg), "w");
    check_eq!(check_ok!(syscall::tee(r1, w2, 32, 0), "tee"), 32, "n");
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sfr", 0o644), "create");
    check_ok!(syscall::write(fd, b"sync-range"), "w");
    check_ok!(
        syscall::sync_file_range(fd, 0, 10, SYNC_FILE_RANGE_WRITE),
        "sfr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_wait_before() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sfb", 0o644), "create");
    check_ok!(syscall::write(fd, b"abc"), "w");
    check_ok!(
        syscall::sync_file_range(fd, 0, 3, SYNC_FILE_RANGE_WAIT_BEFORE),
        "sfr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_wait_after() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sfa", 0o644), "create");
    check_ok!(syscall::write(fd, b"abc"), "w");
    check_ok!(
        syscall::sync_file_range(fd, 0, 3, SYNC_FILE_RANGE_WAIT_AFTER),
        "sfr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_all_flags() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sfall", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcdef"), "w");
    let flags = SYNC_FILE_RANGE_WAIT_BEFORE | SYNC_FILE_RANGE_WRITE | SYNC_FILE_RANGE_WAIT_AFTER;
    check_ok!(syscall::sync_file_range(fd, 0, 6, flags), "sfr");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_zero_len() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sfz", 0o644), "create");
    check_ok!(syscall::write(fd, b"x"), "w");
    // nbytes==0 means "to EOF" on Linux.
    check_ok!(
        syscall::sync_file_range(fd, 0, 0, SYNC_FILE_RANGE_WRITE),
        "sfr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sfo", 0o644), "create");
    check_ok!(syscall::write(fd, b"0123456789"), "w");
    check_ok!(
        syscall::sync_file_range(fd, 4, 3, SYNC_FILE_RANGE_WRITE),
        "sfr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn fallocate_allocate_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fa", 0o644), "create");
    match syscall::fallocate(fd, 0, 0, 4096) {
        Ok(()) => {
            let st = check_ok!(syscall::fstat(fd), "stat");
            check_eq!(st.st_size, 4096, "size");
        }
        Err(e) if soft_falloc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("fallocate")),
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn fallocate_keep_size_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fks", 0o644), "create");
    check_ok!(syscall::write(fd, b"hi"), "w");
    match syscall::fallocate(fd, FALLOC_FL_KEEP_SIZE, 0, 8192) {
        Ok(()) => {
            let st = check_ok!(syscall::fstat(fd), "stat");
            check_eq!(st.st_size, 2, "size kept");
        }
        Err(e) if soft_falloc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("keep_size")),
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn fallocate_punch_hole_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fph", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 8192), "trunc");
    check_ok!(syscall::pwrite(fd, b"XXXX", 0), "pw");
    match syscall::fallocate(fd, FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE, 0, 4) {
        Ok(()) => {}
        Err(e) if soft_falloc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("punch")),
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn fallocate_zero_range_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fzr", 0o644), "create");
    check_ok!(syscall::write(fd, b"ABCDEFGH"), "w");
    match syscall::fallocate(fd, FALLOC_FL_ZERO_RANGE, 2, 4) {
        Ok(()) => {
            let mut b = [0u8; 8];
            check_ok!(syscall::pread(fd, &mut b, 0), "pread");
            check_eq!(&b[2..6], &[0, 0, 0, 0], "zeroed");
        }
        Err(e) if soft_falloc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("zero_range")),
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn fallocate_offset_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fo", 0o644), "create");
    match syscall::fallocate(fd, 0, 1024, 1024) {
        Ok(()) => {
            let st = check_ok!(syscall::fstat(fd), "stat");
            check!(st.st_size >= 2048, "size");
        }
        Err(e) if soft_falloc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("falloc off")),
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn copy_file_range_zero_len() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"cz", b"data")?;
    let out_path = copy_child(&mut tmp, b"czo")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "out"
    );
    let mut oi = 0i64;
    let mut oo = 0i64;
    match syscall::copy_file_range(in_fd, Some(&mut oi), out_fd, Some(&mut oo), 0, 0) {
        Ok(0) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("zero len nonzero")),
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) | Err(Errno::EXDEV) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("cfr zero")),
    }
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(out_fd), "co");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn copy_file_range_offsets() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"co", b"0123456789")?;
    let out_path = copy_child(&mut tmp, b"coo")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "out"
    );
    let mut oi = 2i64;
    let mut oo = 0i64;
    let n = check_ok!(
        syscall::copy_file_range(in_fd, Some(&mut oi), out_fd, Some(&mut oo), 4, 0),
        "cfr"
    );
    check_eq!(n, 4, "n");
    check_eq!(oi, 6, "oi");
    check_eq!(oo, 4, "oo");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(out_fd), "co");
    let mut buf = [0u8; 8];
    check_eq!(read_file(&out_path, &mut buf)?, 4, "len");
    check_eq!(&buf[..4], b"2345", "data");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn copy_file_range_null_offs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"cn", b"abcdef")?;
    let out_path = copy_child(&mut tmp, b"cno")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "out"
    );
    let n = check_ok!(syscall::copy_file_range(in_fd, None, out_fd, None, 3, 0), "cfr");
    check_eq!(n, 3, "n");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(out_fd), "co");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn copy_file_range_large_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = [0xABu8; 4096];
    let in_fd = make_in(&mut tmp, b"cl", &data)?;
    let out_path = copy_child(&mut tmp, b"clo")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "out"
    );
    let mut oi = 0i64;
    let mut oo = 0i64;
    let n = check_ok!(
        syscall::copy_file_range(in_fd, Some(&mut oi), out_fd, Some(&mut oo), 4096, 0),
        "cfr"
    );
    check_eq!(n, 4096, "n");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(out_fd), "co");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sendfile_past_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"pe", b"xy")?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 10i64;
    let n = check_ok!(syscall::sendfile(w, in_fd, &mut off, 8), "sf");
    check_eq!(n, 0, "past eof");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn splice_partial_consume() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    check_ok!(syscall::write(w1, b"ABCDEF"), "w");
    let n = check_ok!(syscall::splice(r1, None, w2, None, 3, 0), "sp");
    check_eq!(n, 3, "n");
    let mut left = [0u8; 8];
    check_eq!(check_ok!(syscall::read(r1, &mut left), "left"), 3, "left");
    check_eq!(&left[..3], b"DEF", "rest");
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tee_does_not_consume() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    check_ok!(syscall::write(w1, b"KEEP"), "w");
    check_ok!(syscall::tee(r1, w2, 4, 0), "tee");
    let mut a = [0u8; 4];
    check_eq!(check_ok!(syscall::read(r1, &mut a), "r1"), 4, "still");
    check_eq!(&a, b"KEEP", "data");
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sendfile_chunked() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"ABCDEFGHIJKLMNOP";
    let in_fd = make_in(&mut tmp, b"ch", data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 0i64;
    let mut total = 0usize;
    while total < data.len() {
        let n = check_ok!(syscall::sendfile(w, in_fd, &mut off, 4), "sf");
        if n == 0 {
            break;
        }
        total += n;
    }
    check_eq!(total, data.len(), "total");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 16];
    check_eq!(check_ok!(syscall::read(r, &mut b), "r"), 16, "r");
    check_eq!(&b, data, "d");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn fallocate_bad_fd() -> TestResult {
    match syscall::fallocate(-1, 0, 0, 1) {
        Err(Errno::EBADF) => {}
        Err(e) if soft_falloc(e) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("falloc bad ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("falloc bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_bad_fd() -> TestResult {
    match syscall::sync_file_range(-1, 0, 1, SYNC_FILE_RANGE_WRITE) {
        Err(Errno::EBADF) => {}
        Err(Errno::EINVAL) | Err(Errno::ESPIPE) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("sfr bad ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("sfr bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn copy_file_range_same_off_progress() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"sp", b"hello-world")?;
    let out_path = copy_child(&mut tmp, b"spo")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "out"
    );
    let mut oi = 0i64;
    let mut oo = 0i64;
    let n1 = check_ok!(
        syscall::copy_file_range(in_fd, Some(&mut oi), out_fd, Some(&mut oo), 5, 0),
        "1"
    );
    check_eq!(n1, 5, "n1");
    let n2 = check_ok!(
        syscall::copy_file_range(in_fd, Some(&mut oi), out_fd, Some(&mut oo), 6, 0),
        "2"
    );
    check_eq!(n2, 6, "n2");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(out_fd), "co");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn splice_file_offset_advance() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"sfa2", b"01234567")?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 1i64;
    let n = check_ok!(syscall::splice(in_fd, Some(&mut off), w, None, 3, 0), "sp");
    check_eq!(n, 3, "n");
    check_eq!(off, 4, "off");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 3];
    check_ok!(syscall::read(r, &mut b), "r");
    check_eq!(&b, b"123", "d");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn fallocate_then_punch_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fp2", 0o644), "create");
    match syscall::fallocate(fd, 0, 0, 4096) {
        Ok(()) => {
            match syscall::fallocate(fd, FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE, 100, 100) {
                Ok(()) => {}
                Err(e) if soft_falloc(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("punch2")),
            }
        }
        Err(e) if soft_falloc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("alloc")),
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sendfile_null_like_via_pipe_capacity() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_in(&mut tmp, b"cap", b"abcd")?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut off = 0i64;
    check_eq!(check_ok!(syscall::sendfile(w, in_fd, &mut off, 2), "sf"), 2, "n");
    check_eq!(check_ok!(syscall::sendfile(w, in_fd, &mut off, 2), "sf2"), 2, "n2");
    check_ok!(syscall::close(w), "cw");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 4];
    check_eq!(check_ok!(syscall::read(r, &mut b), "r"), 4, "r");
    check_eq!(&b, b"abcd", "d");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sync_file_range_whole_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sfw", 0o644), "create");
    check_ok!(syscall::write(fd, b"whole-file-sync"), "w");
    let flags = SYNC_FILE_RANGE_WRITE | SYNC_FILE_RANGE_WAIT_AFTER;
    check_ok!(syscall::sync_file_range(fd, 0, 0, flags), "sfr");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}
