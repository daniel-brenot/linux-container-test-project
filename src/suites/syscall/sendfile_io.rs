//! sendfile, splice, and copy_file_range tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, read_file, write_file};
use crate::syscall::{self, oflag};

fn make_data_file(tmp: &mut TempDir, name: &[u8], data: &[u8]) -> Result<i32, crate::harness::AssertFail> {
    let path = copy_child(tmp, name)?;
    write_file(&path, data)?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open in");
    Ok(fd)
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendfile copies a regular file into another regular file")]
fn sendfile_file_to_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"sendfile-payload-12345";
    let in_fd = make_data_file(&mut tmp, b"in", data)?;
    let out_path = copy_child(&mut tmp, b"out")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "open out"
    );
    let mut off = 0i64;
    let n = check_ok!(syscall::sendfile(out_fd, in_fd, &mut off, data.len()), "sendfile");
    check_eq!(n, data.len(), "copied");
    check_ok!(syscall::close(in_fd), "close in");
    check_ok!(syscall::close(out_fd), "close out");
    let mut buf = [0u8; 32];
    let rn = read_file(&out_path, &mut buf)?;
    check_eq!(rn, data.len(), "out len");
    check!(&buf[..data.len()] == data, "out data");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendfile copies a regular file into a pipe")]
fn sendfile_file_to_pipe() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"pipe-via-sendfile";
    let in_fd = make_data_file(&mut tmp, b"sf", data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut off = 0i64;
    let n = check_ok!(syscall::sendfile(w, in_fd, &mut off, data.len()), "sendfile");
    check_eq!(n, data.len(), "copied");
    check_ok!(syscall::close(w), "close w");
    check_ok!(syscall::close(in_fd), "close in");
    let mut buf = [0u8; 32];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), data.len(), "rlen");
    check!(&buf[..data.len()] == data, "data");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendfile with a count of 4 copies the first four bytes of a file into a pipe")]
fn sendfile_partial() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"0123456789";
    let in_fd = make_data_file(&mut tmp, b"pin", data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut off = 0i64;
    let n = check_ok!(syscall::sendfile(w, in_fd, &mut off, 4), "sendfile");
    check_eq!(n, 4, "partial");
    check_ok!(syscall::close(w), "close w");
    check_ok!(syscall::close(in_fd), "close in");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 4, "rlen");
    check_eq!(&buf[..4], b"0123", "partial data");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendfile from a nonzero offset copies those bytes and advances the offset")]
fn sendfile_with_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"ABCDEFGH";
    let in_fd = make_data_file(&mut tmp, b"off", data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut off = 3i64;
    let n = check_ok!(syscall::sendfile(w, in_fd, &mut off, 3), "sendfile");
    check_eq!(n, 3, "n");
    check_eq!(off, 6, "off advanced");
    check_ok!(syscall::close(w), "close w");
    check_ok!(syscall::close(in_fd), "close in");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 3, "rlen");
    check_eq!(&buf[..3], b"DEF", "data");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "splice moves bytes from one pipe into another")]
fn splice_pipe_to_pipe() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "pipe1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "pipe2");
    let msg = b"splice-pipe-data";
    check_ok!(syscall::write(w1, msg), "write");
    check_ok!(syscall::close(w1), "close w1");
    let n = check_ok!(syscall::splice(r1, None, w2, None, msg.len(), 0), "splice");
    check!(n > 0, "spliced");
    check_ok!(syscall::close(w2), "close w2");
    check_ok!(syscall::close(r1), "close r1");
    let mut buf = [0u8; 32];
    let rn = check_ok!(syscall::read(r2, &mut buf), "read");
    check_eq!(&buf[..rn], &msg[..rn], "data");
    check_ok!(syscall::close(r2), "close r2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "splice copies a regular file into a pipe")]
fn splice_file_to_pipe() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"file-to-pipe-splice";
    let in_fd = make_data_file(&mut tmp, b"sp", data)?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut off = 0i64;
    let n = check_ok!(syscall::splice(in_fd, Some(&mut off), w, None, data.len(), 0), "splice");
    check!(n > 0, "spliced");
    check_ok!(syscall::close(w), "close w");
    check_ok!(syscall::close(in_fd), "close in");
    let mut buf = [0u8; 32];
    let rn = check_ok!(syscall::read(r, &mut buf), "read");
    check!(&buf[..rn] == &data[..rn], "data");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "splice copies a pipe into a regular file")]
fn splice_pipe_to_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"pipe-to-file-splice";
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, data), "write");
    check_ok!(syscall::close(w), "close w");
    let out_path = copy_child(&mut tmp, b"sout")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "open out"
    );
    let n = check_ok!(syscall::splice(r, None, out_fd, None, data.len(), 0), "splice");
    check!(n > 0, "spliced");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(out_fd), "close out");
    let mut buf = [0u8; 32];
    let rn = read_file(&out_path, &mut buf)?;
    check_eq!(&buf[..rn], data, "file data");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "copy_file_range copies a whole regular file into another file")]
fn copy_file_range_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"copy_file_range_payload";
    let in_fd = make_data_file(&mut tmp, b"cfr_in", data)?;
    let out_path = copy_child(&mut tmp, b"cfr_out")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "open out"
    );
    let mut off_in = 0i64;
    let mut off_out = 0i64;
    let n = check_ok!(
        syscall::copy_file_range(in_fd, Some(&mut off_in), out_fd, Some(&mut off_out), data.len(), 0),
        "cfr"
    );
    check_eq!(n, data.len(), "copied");
    check_ok!(syscall::close(in_fd), "close in");
    check_ok!(syscall::close(out_fd), "close out");
    let mut buf = [0u8; 32];
    let rn = read_file(&out_path, &mut buf)?;
    check_eq!(&buf[..rn], data, "out");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "copy_file_range from a nonzero input offset copies a six-byte slice")]
fn copy_file_range_partial() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"0123456789ABCDEF";
    let in_fd = make_data_file(&mut tmp, b"cfr_p", data)?;
    let out_path = copy_child(&mut tmp, b"cfr_po")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_WRONLY | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "open out"
    );
    let mut off_in = 4i64;
    let mut off_out = 0i64;
    let n = check_ok!(
        syscall::copy_file_range(in_fd, Some(&mut off_in), out_fd, Some(&mut off_out), 6, 0),
        "cfr"
    );
    check_eq!(n, 6, "n");
    check_ok!(syscall::close(in_fd), "close in");
    check_ok!(syscall::close(out_fd), "close out");
    let mut buf = [0u8; 16];
    let rn = read_file(&out_path, &mut buf)?;
    check_eq!(&buf[..rn], b"456789", "partial");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "copy_file_range with NULL offsets copies between two fds on the same filesystem")]
fn copy_file_range_fd_to_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let data = b"fd-to-fd-copy";
    let in_fd = make_data_file(&mut tmp, b"cfr2_in", data)?;
    // Same filesystem for both ends: cross-FS copy_file_range often returns EXDEV.
    let out_path = copy_child(&mut tmp, b"cfr2_out")?;
    let out_fd = check_ok!(
        syscall::open(&out_path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC, 0o644),
        "open out"
    );
    let n = check_ok!(
        syscall::copy_file_range(in_fd, None, out_fd, None, data.len(), 0),
        "cfr"
    );
    check_eq!(n, data.len(), "n");
    check_ok!(syscall::close(in_fd), "close in");
    check_ok!(syscall::lseek(out_fd, 0, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 32];
    check_eq!(check_ok!(syscall::read(out_fd, &mut buf), "read"), data.len(), "rlen");
    check!(&buf[..data.len()] == data, "data");
    check_ok!(syscall::close(out_fd), "close out");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendfile with a count of zero copies zero bytes")]
fn sendfile_zero_count() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let in_fd = make_data_file(&mut tmp, b"z", b"x")?;
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut off = 0i64;
    let n = check_ok!(syscall::sendfile(w, in_fd, &mut off, 0), "sendfile");
    check_eq!(n, 0, "zero");
    check_ok!(syscall::close(w), "close w");
    check_ok!(syscall::close(in_fd), "close in");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "tee copies pipe data into another pipe without consuming the input")]
fn tee_pipe_to_pipe() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "pipe1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "pipe2");
    let msg = b"tee-payload";
    check_ok!(syscall::write(w1, msg), "write");
    let n = check_ok!(syscall::tee(r1, w2, msg.len(), 0), "tee");
    check_eq!(n, msg.len(), "teed");
    // tee copies without consuming input; both pipes should have data.
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    check_eq!(check_ok!(syscall::read(r1, &mut a), "read r1"), msg.len(), "r1");
    check_eq!(check_ok!(syscall::read(r2, &mut b), "read r2"), msg.len(), "r2");
    check!(&a[..msg.len()] == msg, "r1 data");
    check!(&b[..msg.len()] == msg, "r2 data");
    check_ok!(syscall::close(r1), "close r1");
    check_ok!(syscall::close(w1), "close w1");
    check_ok!(syscall::close(r2), "close r2");
    check_ok!(syscall::close(w2), "close w2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "tee of four bytes copies the prefix and leaves the input pipe intact")]
fn tee_partial() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "pipe1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w1, b"ABCDEFGH"), "write");
    let n = check_ok!(syscall::tee(r1, w2, 4, 0), "tee");
    check_eq!(n, 4, "partial");
    let mut out = [0u8; 8];
    check_eq!(check_ok!(syscall::read(r2, &mut out), "read"), 4, "rlen");
    check_eq!(&out[..4], b"ABCD", "prefix");
    // Input still fully present.
    check_eq!(check_ok!(syscall::read(r1, &mut out), "read in"), 8, "full in");
    check_eq!(&out[..8], b"ABCDEFGH", "input intact");
    check_ok!(syscall::close(r1), "close r1");
    check_ok!(syscall::close(w1), "close w1");
    check_ok!(syscall::close(r2), "close r2");
    check_ok!(syscall::close(w2), "close w2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "tee with a length of zero copies zero bytes")]
fn tee_zero_len() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "pipe1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w1, b"x"), "write");
    let n = check_ok!(syscall::tee(r1, w2, 0, 0), "tee");
    check_eq!(n, 0, "zero");
    check_ok!(syscall::close(r1), "close r1");
    check_ok!(syscall::close(w1), "close w1");
    check_ok!(syscall::close(r2), "close r2");
    check_ok!(syscall::close(w2), "close w2");
    Ok(())
}
