//! POSIX I/O semantics tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, read_file, write_file};
use crate::syscall::{self, oflag, Errno, IoVec};

#[crate::lctp_test(suite = posix, expect = success, case = "read() on a pipe returns 0 after the write end is closed")]
fn pipe_eof_after_close_writer() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::close(w), "close w");
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::read(r, &mut buf), "read");
    check_eq!(n, 0, "EOF");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() from a pipe with a smaller buffer returns a partial prefix of the written bytes")]
fn pipe_read_partial() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    check_ok!(syscall::write(w, b"12345"), "write");
    let mut small = [0u8; 2];
    check_eq!(check_ok!(syscall::read(r, &mut small), "read"), 2, "partial");
    check_eq!(&small, b"12", "bytes");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() of a regular file with a smaller buffer returns the requested number of bytes")]
fn read_returns_available() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"abcdef")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 3];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 3, "len");
    check_eq!(&buf, b"abc", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "sequential write() calls on a file opened O_APPEND concatenate in order")]
fn append_atomic_sequence() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"log")?;
    write_file(&path, b"1")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0), "append");
    check_ok!(syscall::write(fd, b"2"), "w2");
    check_ok!(syscall::write(fd, b"3"), "w3");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 8];
    check_eq!(read_file(&path, &mut buf)?, 3, "len");
    check_eq!(&buf[..3], b"123", "content");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "pwrite() at an offset updates those bytes and pread() from 0 reads the combined contents")]
fn write_at_offset_pread() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::pwrite(fd, b"ZZZZ", 0), "pwrite");
    check_ok!(syscall::pwrite(fd, b"AB", 1), "pwrite mid");
    let mut buf = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, b"ZABZ", "content");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "sequential read() calls on a regular file advance the file offset")]
fn sequential_read_advances() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcd"), "write");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut one = [0u8; 1];
    check_ok!(syscall::read(fd, &mut one), "r1");
    check_eq!(one[0], b'a', "a");
    check_ok!(syscall::read(fd, &mut one), "r2");
    check_eq!(one[0], b'b', "b");
    let pos = check_ok!(syscall::lseek(fd, 0, syscall::SEEK_CUR), "cur");
    check_eq!(pos, 2, "offset");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() of an empty regular file returns 0")]
fn empty_file_read_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"empty")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 0, "EOF");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "write() to a pipe followed by read() transfers the same bytes")]
fn pipe_blocking_write_read() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let msg = b"block-test";
    check_eq!(check_ok!(syscall::write(w, msg), "write"), msg.len(), "wlen");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), msg.len(), "rlen");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() on a pipe with a closed write end returns 0 on two successive calls")]
fn pipe_eof_twice() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::close(w), "close w");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "r1"), 0, "eof1");
    check_eq!(check_ok!(syscall::read(r, &mut buf), "r2"), 0, "eof2");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() on a pipe returns the written bytes and then 0 after the write end is closed")]
fn pipe_data_then_eof() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::write(w, b"xy"), "write");
    check_ok!(syscall::close(w), "close w");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 2, "data");
    check_eq!(check_ok!(syscall::read(r, &mut buf), "eof"), 0, "eof");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two read() calls on a pipe return the written bytes in order without loss")]
fn pipe_partial_then_rest() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::write(w, b"ABCDEF"), "write");
    let mut a = [0u8; 2];
    let mut b = [0u8; 4];
    check_eq!(check_ok!(syscall::read(r, &mut a), "r1"), 2, "2");
    check_eq!(&a, b"AB", "ab");
    check_eq!(check_ok!(syscall::read(r, &mut b), "r2"), 4, "4");
    check_eq!(&b, b"CDEF", "cdef");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "writev() then readv() on a regular file round-trip concatenated iovec bytes")]
fn io_writev_readv_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"iov", 0o644), "create");
    let a = b"AB";
    let b = b"CD";
    let mut iov = [
        IoVec {
            iov_base: a.as_ptr() as *mut u8,
            iov_len: a.len(),
        },
        IoVec {
            iov_base: b.as_ptr() as *mut u8,
            iov_len: b.len(),
        },
    ];
    check_eq!(check_ok!(syscall::writev(fd, &mut iov), "writev"), 4, "wlen");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut o1 = [0u8; 2];
    let mut o2 = [0u8; 2];
    let mut riov = [
        IoVec {
            iov_base: o1.as_mut_ptr(),
            iov_len: 2,
        },
        IoVec {
            iov_base: o2.as_mut_ptr(),
            iov_len: 2,
        },
    ];
    check_eq!(check_ok!(syscall::readv(fd, &mut riov), "readv"), 4, "rlen");
    check_eq!(&o1, b"AB", "p1");
    check_eq!(&o2, b"CD", "p2");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "writev() to a pipe concatenates iovec bytes that read() returns in order")]
fn io_writev_readv_pipe() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let a = b"hi";
    let b = b"yo";
    let mut iov = [
        IoVec {
            iov_base: a.as_ptr() as *mut u8,
            iov_len: a.len(),
        },
        IoVec {
            iov_base: b.as_ptr() as *mut u8,
            iov_len: b.len(),
        },
    ];
    check_eq!(check_ok!(syscall::writev(w, &mut iov), "writev"), 4, "w");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 4, "r");
    check_eq!(&buf, b"hiyo", "data");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "readv() from a pipe fills iovecs in order with a prefix of the written bytes")]
fn io_readv_partial_pipe() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::write(w, b"ABCDEFGH"), "write");
    let mut o1 = [0u8; 3];
    let mut o2 = [0u8; 3];
    let mut riov = [
        IoVec {
            iov_base: o1.as_mut_ptr(),
            iov_len: 3,
        },
        IoVec {
            iov_base: o2.as_mut_ptr(),
            iov_len: 3,
        },
    ];
    check_eq!(check_ok!(syscall::readv(r, &mut riov), "readv"), 6, "6");
    check_eq!(&o1, b"ABC", "1");
    check_eq!(&o2, b"DEF", "2");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "read() on an empty non-blocking pipe returns EAGAIN")]
fn io_nonblock_pipe_eagain_read() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_NONBLOCK), "pipe");
    let mut buf = [0u8; 1];
    check_err!(syscall::read(r, &mut buf), Errno::EAGAIN, "EAGAIN");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "read() on a pipe set O_NONBLOCK with fcntl(F_SETFL) returns EAGAIN when empty")]
fn io_nonblock_setfl_eagain() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(
        syscall::fcntl(r, syscall::fcntl_cmd::F_SETFL, oflag::O_NONBLOCK as usize),
        "setfl"
    );
    let mut buf = [0u8; 1];
    check_err!(syscall::read(r, &mut buf), Errno::EAGAIN, "EAGAIN");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "write() of 4096 bytes to a pipe succeeds and read() consumes the same total")]
fn io_pipe_buf_atomic_soft() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let msg = [b'P'; 4096];
    check_eq!(
        check_ok!(syscall::write(w, &msg), "write PIPE_BUF"),
        4096,
        "atomic soft len"
    );
    let mut got = 0usize;
    let mut buf = [0u8; 1024];
    while got < 4096 {
        let n = check_ok!(syscall::read(r, &mut buf), "read");
        check!(n > 0, "progress");
        got += n;
    }
    check_eq!(got, 4096, "total");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "write() of 512 bytes to a pipe is returned unchanged by read()")]
fn io_pipe_buf_under_limit() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let msg = [b'x'; 512];
    check_eq!(check_ok!(syscall::write(w, &msg), "write"), 512, "len");
    let mut buf = [0u8; 512];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 512, "rlen");
    check_eq!(&buf[..], &msg[..], "data");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "writev() with a single iovec writes that buffer to a regular file")]
fn io_writev_single_iovec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"one", 0o644), "create");
    let data = b"solo";
    let mut iov = [IoVec {
        iov_base: data.as_ptr() as *mut u8,
        iov_len: data.len(),
    }];
    check_eq!(check_ok!(syscall::writev(fd, &mut iov), "writev"), 4, "len");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "readv() of an empty regular file returns 0")]
fn io_readv_eof_empty_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"e", 0o644), "create");
    let mut o = [0u8; 4];
    let mut riov = [IoVec {
        iov_base: o.as_mut_ptr(),
        iov_len: 4,
    }];
    check_eq!(check_ok!(syscall::readv(fd, &mut riov), "readv"), 0, "eof");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "pread() reads at an explicit offset without changing the file offset")]
fn io_pread_does_not_move_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"p", 0o644), "create");
    check_ok!(syscall::write(fd, b"012345"), "write");
    check_ok!(syscall::lseek(fd, 2, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 2];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, b"01", "data");
    let pos = check_ok!(syscall::lseek(fd, 0, syscall::SEEK_CUR), "cur");
    check_eq!(pos, 2, "offset");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "pwrite() writes at an explicit offset without changing the file offset")]
fn io_pwrite_does_not_move_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"pw", 0o644), "create");
    check_ok!(syscall::write(fd, b"AAAA"), "write");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    check_ok!(syscall::pwrite(fd, b"BB", 2), "pwrite");
    let pos = check_ok!(syscall::lseek(fd, 0, syscall::SEEK_CUR), "cur");
    check_eq!(pos, 0, "offset");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "write() then read() on a non-blocking pipe transfers the written bytes")]
fn io_nonblock_write_then_read() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_NONBLOCK), "pipe");
    check_ok!(syscall::write(w, b"nb"), "write");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 2, "len");
    check_eq!(&buf, b"nb", "data");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "lseek(SEEK_END) then write() appends bytes to a regular file")]
fn io_lseek_end_and_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"e")?;
    write_file(&path, b"AB")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_END), "end");
    check_ok!(syscall::write(fd, b"C"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    check_eq!(read_file(&path, &mut buf)?, 3, "len");
    check_eq!(&buf[..3], b"ABC", "data");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "writev() with three iovecs concatenates their bytes on a regular file")]
fn io_writev_three_vectors() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"3", 0o644), "create");
    let a = b"A";
    let b = b"B";
    let c = b"C";
    let mut iov = [
        IoVec {
            iov_base: a.as_ptr() as *mut u8,
            iov_len: 1,
        },
        IoVec {
            iov_base: b.as_ptr() as *mut u8,
            iov_len: 1,
        },
        IoVec {
            iov_base: c.as_ptr() as *mut u8,
            iov_len: 1,
        },
    ];
    check_eq!(check_ok!(syscall::writev(fd, &mut iov), "writev"), 3, "3");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 3];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 3, "r");
    check_eq!(&buf, b"ABC", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = success, case = "write() of 2048 bytes to a pipe is fully consumed by successive read() calls")]
fn io_large_pipe_write_under_buf() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let msg = [b'L'; 2048];
    check_eq!(check_ok!(syscall::write(w, &msg), "write"), 2048, "w");
    let mut total = 0usize;
    let mut buf = [0u8; 512];
    while total < 2048 {
        total += check_ok!(syscall::read(r, &mut buf), "read");
    }
    check_eq!(total, 2048, "total");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() after lseek(SEEK_SET) returns bytes from that offset")]
fn io_read_after_seek_cur() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"s", 0o644), "create");
    check_ok!(syscall::write(fd, b"0123456789"), "write");
    check_ok!(syscall::lseek(fd, 5, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 2, "len");
    check_eq!(&buf, b"56", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "write() of a zero-length buffer to a regular file succeeds and returns 0")]
fn io_zero_length_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"z", 0o644), "create");
    check_eq!(check_ok!(syscall::write(fd, b""), "write0"), 0, "0");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() into a zero-length buffer of a non-empty file succeeds and returns 0")]
fn io_zero_length_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"zr")?;
    write_file(&path, b"data")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 0];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read0"), 0, "0");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "pipe2(O_CLOEXEC) sets FD_CLOEXEC on both pipe file descriptors")]
fn io_pipe_cloexec() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_CLOEXEC), "pipe");
    let fr = check_ok!(syscall::fcntl(r, syscall::fcntl_cmd::F_GETFD, 0), "r");
    let fw = check_ok!(syscall::fcntl(w, syscall::fcntl_cmd::F_GETFD, 0), "w");
    check!(fr as i32 & syscall::FD_CLOEXEC != 0, "r cloexec");
    check!(fw as i32 & syscall::FD_CLOEXEC != 0, "w cloexec");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "fcntl(F_GETFL) on a pipe created with O_NONBLOCK reports O_NONBLOCK")]
fn io_nonblock_flag_getfl() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_NONBLOCK), "pipe");
    let fl = check_ok!(syscall::fcntl(r, syscall::fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & oflag::O_NONBLOCK != 0, "NONBLOCK");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "writev() with zero iovecs returns 0 or EINVAL")]
fn io_writev_empty_vectors_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"ev", 0o644), "create");
    let mut iov: [IoVec; 0] = [];
    match syscall::writev(fd, &mut iov) {
        Ok(0) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("nonzero")),
        Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("writev empty")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "read() past the last byte of a regular file returns 0")]
fn io_file_read_past_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"pe")?;
    write_file(&path, b"X")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "r1"), 1, "1");
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "r2"), 0, "eof");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
