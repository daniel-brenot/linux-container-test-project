//! Deep file/fcntl/lseek/preadv coverage for unprivileged containers.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{
    self, fcntl_cmd, oflag, Errno, Flock, IoVec, FD_CLOEXEC, F_RDLCK, F_UNLCK, F_WRLCK, SEEK_CUR,
    SEEK_DATA, SEEK_END, SEEK_HOLE, SEEK_SET,
};

fn soft_einval_enosys(e: Errno) -> bool {
    matches!(e, Errno::EINVAL | Errno::ENOSYS | Errno::EOPNOTSUPP | Errno::ENOTSUP)
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD duplicates a file descriptor to a distinct fd")]
fn fcntl_dupfd_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, 0), "F_DUPFD");
    check!(d as i32 >= 0 && d as i32 != fd, "new fd");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d as i32), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD with a minimum fd returns a descriptor at or above that floor")]
fn fcntl_dupfd_min_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, 50), "F_DUPFD");
    check!(d as i32 >= 50, "min fd");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d as i32), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD_CLOEXEC duplicates an fd and sets FD_CLOEXEC on the new one")]
fn fcntl_dupfd_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD_CLOEXEC, 40), "dupfd cloexec");
    check!(d as i32 >= 40, "min");
    let flags = check_ok!(syscall::fcntl(d as i32, fcntl_cmd::F_GETFD, 0), "getfd");
    check!(flags & FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d as i32), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD shares the file offset with the original descriptor")]
fn fcntl_dupfd_shares_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcdef"), "write");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, 0), "dupfd") as i32;
    check_ok!(syscall::lseek(fd, 2, SEEK_SET), "seek");
    check_eq!(check_ok!(syscall::lseek(d, 0, SEEK_CUR), "cur"), 2, "shared");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "F_DUPFD on fd -1 returns EBADF")]
fn fcntl_dupfd_bad_fd() -> TestResult {
    check_err!(syscall::fcntl(-1, fcntl_cmd::F_DUPFD, 0), Errno::EBADF, "bad");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETFL can set O_NONBLOCK on a regular file")]
fn fcntl_setfl_nonblock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "getfl");
    check_ok!(
        syscall::fcntl(fd, fcntl_cmd::F_SETFL, (fl as i32 | oflag::O_NONBLOCK) as usize),
        "setfl"
    );
    let fl2 = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "getfl2");
    check!(fl2 as i32 & oflag::O_NONBLOCK != 0, "nonblock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETFL can clear O_NONBLOCK on a pipe")]
fn fcntl_clear_nonblock() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_NONBLOCK), "pipe");
    let fl = check_ok!(syscall::fcntl(r, fcntl_cmd::F_GETFL, 0), "getfl");
    check_ok!(
        syscall::fcntl(r, fcntl_cmd::F_SETFL, (fl as i32 & !oflag::O_NONBLOCK) as usize),
        "clear"
    );
    let fl2 = check_ok!(syscall::fcntl(r, fcntl_cmd::F_GETFL, 0), "getfl2");
    check!(fl2 as i32 & oflag::O_NONBLOCK == 0, "cleared");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETLK can take and release a write lock on a regular file")]
fn fcntl_setlk_write_unlock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"lk", 0o644), "create");
    check_ok!(syscall::write(fd, b"lockdata"), "write");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "setlk");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETLK can take and release a read lock on a byte range")]
fn fcntl_setlk_read_lock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"rlk", 0o644), "create");
    check_ok!(syscall::write(fd, b"xx"), "write");
    let mut lk = Flock {
        l_type: F_RDLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 2,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "rdlck");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlck");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_GETLK on an unlocked file reports F_UNLCK")]
fn fcntl_getlk_unlocked() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"glk", 0o644), "create");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_GETLK, &mut lk), "getlk");
    check_eq!(lk.l_type, F_UNLCK, "unlocked");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETLKW acquires an uncontended write lock immediately")]
fn fcntl_setlkw_immediate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"lkw", 0o644), "create");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 1,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLKW, &mut lk), "setlkw");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETLK can lock a mid-file byte range")]
fn fcntl_setlk_range_mid() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"mid", 0o644), "create");
    check_ok!(syscall::write(fd, b"0123456789"), "write");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 3,
        l_len: 4,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "setlk");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlck");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETLK with SEEK_CUR locks relative to the current offset")]
fn fcntl_setlk_whence_cur() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"cur", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcdefgh"), "write");
    check_ok!(syscall::lseek(fd, 2, SEEK_SET), "seek");
    let mut lk = Flock {
        l_type: F_RDLCK,
        l_whence: SEEK_CUR as i16,
        l_start: 0,
        l_len: 2,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "setlk");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlck");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETLK with SEEK_END locks a range relative to the end of the file")]
fn fcntl_setlk_whence_end() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"end", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcdefgh"), "write");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_END as i16,
        l_start: -2,
        l_len: 2,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "setlk");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlck");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_GETLK of the caller's own lock reports F_UNLCK")]
fn fcntl_getlk_after_own_lock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"own", 0o644), "create");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "setlk");
    // F_GETLK of own lock reports F_UNLCK (not conflicting).
    let mut probe = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_GETLK, &mut probe), "getlk");
    check_eq!(probe.l_type, F_UNLCK, "own lock");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlck");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "lseek SEEK_DATA finds data or is rejected with EINVAL/ENOSYS/EOPNOTSUPP")]
fn lseek_seek_data_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sd", 0o644), "create");
    check_ok!(syscall::write(fd, b"data"), "write");
    match syscall::lseek(fd, 0, SEEK_DATA) {
        Ok(pos) => check!(pos >= 0, "pos"),
        Err(e) if soft_einval_enosys(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("SEEK_DATA errno")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "lseek SEEK_HOLE finds a hole or is rejected with EINVAL/ENOSYS/EOPNOTSUPP")]
fn lseek_seek_hole_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sh", 0o644), "create");
    check_ok!(syscall::write(fd, b"hole?"), "write");
    match syscall::lseek(fd, 0, SEEK_HOLE) {
        Ok(pos) => check!(pos >= 0, "pos"),
        Err(e) if soft_einval_enosys(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("SEEK_HOLE errno")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "lseek SEEK_DATA past EOF returns ENXIO or is unsupported")]
fn lseek_seek_data_past_eof_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sde", 0o644), "create");
    check_ok!(syscall::write(fd, b"ab"), "write");
    match syscall::lseek(fd, 100, SEEK_DATA) {
        Ok(_) => {}
        Err(Errno::ENXIO) | Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("SEEK_DATA past eof")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "lseek SEEK_HOLE after sparse data succeeds or is unsupported")]
fn lseek_seek_hole_after_data_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"shd", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 8192), "trunc");
    check_ok!(syscall::pwrite(fd, b"XXXX", 0), "pwrite");
    match syscall::lseek(fd, 0, SEEK_HOLE) {
        Ok(pos) => check!(pos >= 0, "hole pos"),
        Err(e) if soft_einval_enosys(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("SEEK_HOLE")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "preadv into three iovecs reconstructs the file contents")]
fn preadv_three_iov() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"pv3", 0o644), "create");
    check_ok!(syscall::write(fd, b"ABCDEF"), "write");
    let mut a = [0u8; 2];
    let mut b = [0u8; 2];
    let mut c = [0u8; 2];
    let mut iov = [
        IoVec { iov_base: a.as_mut_ptr(), iov_len: 2 },
        IoVec { iov_base: b.as_mut_ptr(), iov_len: 2 },
        IoVec { iov_base: c.as_mut_ptr(), iov_len: 2 },
    ];
    let n = check_ok!(syscall::preadv(fd, &mut iov, 0), "preadv");
    check_eq!(n, 6, "n");
    check_eq!(&a, b"AB", "a");
    check_eq!(&b, b"CD", "b");
    check_eq!(&c, b"EF", "c");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwritev from three iovecs writes the concatenated bytes")]
fn pwritev_three_iov() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"pw3", 0o644), "create");
    let a = b"12";
    let b = b"34";
    let c = b"56";
    let mut iov = [
        IoVec { iov_base: a.as_ptr() as *mut u8, iov_len: 2 },
        IoVec { iov_base: b.as_ptr() as *mut u8, iov_len: 2 },
        IoVec { iov_base: c.as_ptr() as *mut u8, iov_len: 2 },
    ];
    check_eq!(check_ok!(syscall::pwritev(fd, &mut iov, 0), "pwritev"), 6, "n");
    let mut out = [0u8; 6];
    check_ok!(syscall::pread(fd, &mut out, 0), "pread");
    check_eq!(&out, b"123456", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "preadv with an empty iovec array returns 0")]
fn preadv_empty_iov_zero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"pve", 0o644), "create");
    check_ok!(syscall::write(fd, b"x"), "write");
    let mut iov: [IoVec; 0] = [];
    let n = check_ok!(syscall::preadv(fd, &mut iov, 0), "preadv");
    check_eq!(n, 0, "empty");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwritev with a zero-length iovec returns 0")]
fn pwritev_zero_len_vecs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"pwz", 0o644), "create");
    let mut iov = [IoVec {
        iov_base: core::ptr::null_mut(),
        iov_len: 0,
    }];
    let n = check_ok!(syscall::pwritev(fd, &mut iov, 0), "pwritev");
    check_eq!(n, 0, "zero");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "preadv into oversized iovecs returns the available byte count")]
fn preadv_partial_fill() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"ppf", 0o644), "create");
    check_ok!(syscall::write(fd, b"XY"), "write");
    let mut a = [0u8; 4];
    let mut b = [0u8; 4];
    let mut iov = [
        IoVec { iov_base: a.as_mut_ptr(), iov_len: 4 },
        IoVec { iov_base: b.as_mut_ptr(), iov_len: 4 },
    ];
    let n = check_ok!(syscall::preadv(fd, &mut iov, 0), "preadv");
    check_eq!(n, 2, "short");
    check_eq!(&a[..2], b"XY", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwritev at offset 10 stores bytes that pread can read back")]
fn pwritev_at_offset_ten() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"po10", 0o644), "create");
    let data = b"ZZ";
    let mut iov = [IoVec {
        iov_base: data.as_ptr() as *mut u8,
        iov_len: 2,
    }];
    check_ok!(syscall::pwritev(fd, &mut iov, 10), "pwritev");
    let mut out = [0u8; 2];
    check_ok!(syscall::pread(fd, &mut out, 10), "pread");
    check_eq!(&out, b"ZZ", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "readv into four buffers reconstructs an eight-byte file")]
fn readv_four_bufs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"rv4", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcdefgh"), "write");
    check_ok!(syscall::lseek(fd, 0, SEEK_SET), "seek");
    let mut b0 = [0u8; 2];
    let mut b1 = [0u8; 2];
    let mut b2 = [0u8; 2];
    let mut b3 = [0u8; 2];
    let mut iov = [
        IoVec { iov_base: b0.as_mut_ptr(), iov_len: 2 },
        IoVec { iov_base: b1.as_mut_ptr(), iov_len: 2 },
        IoVec { iov_base: b2.as_mut_ptr(), iov_len: 2 },
        IoVec { iov_base: b3.as_mut_ptr(), iov_len: 2 },
    ];
    check_eq!(check_ok!(syscall::readv(fd, &mut iov), "readv"), 8, "n");
    check_eq!(&b0, b"ab", "0");
    check_eq!(&b3, b"gh", "3");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "writev from four buffers concatenates them on disk")]
fn writev_four_bufs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"wv4", 0o644), "create");
    let p = [b"aa", b"bb", b"cc", b"dd"];
    let mut iov = [
        IoVec { iov_base: p[0].as_ptr() as *mut u8, iov_len: 2 },
        IoVec { iov_base: p[1].as_ptr() as *mut u8, iov_len: 2 },
        IoVec { iov_base: p[2].as_ptr() as *mut u8, iov_len: 2 },
        IoVec { iov_base: p[3].as_ptr() as *mut u8, iov_len: 2 },
    ];
    check_eq!(check_ok!(syscall::writev(fd, &mut iov), "writev"), 8, "n");
    let mut out = [0u8; 8];
    check_ok!(syscall::pread(fd, &mut out, 0), "pread");
    check_eq!(&out, b"aabbccdd", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_GETFL reports O_APPEND after opening with that flag")]
fn fcntl_getfl_after_append_open() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ap")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0), "open");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & oflag::O_APPEND != 0, "append");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD_CLOEXEC with a high minimum returns an fd at or above that floor")]
fn fcntl_dupfd_cloexec_high() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"hi", 0o644), "create");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD_CLOEXEC, 100), "dup") as i32;
    check!(d >= 100, "high");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETFD can clear FD_CLOEXEC that was set at open")]
fn fcntl_setfd_clear_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(
            check_ok!(tmp.child(b"c"), "child"),
            oflag::O_RDWR | oflag::O_CREAT | oflag::O_CLOEXEC,
            0o644
        ),
        "open"
    );
    let flags = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "getfd");
    check!(flags & FD_CLOEXEC as usize != 0, "set");
    check_ok!(syscall::fcntl(fd, fcntl_cmd::F_SETFD, 0), "clear");
    let flags2 = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "getfd2");
    check!(flags2 & FD_CLOEXEC as usize == 0, "cleared");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "lseek SEEK_SET to 0 returns offset 0")]
fn lseek_seek_set_zero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"z", 0o644), "create");
    check_ok!(syscall::write(fd, b"abc"), "write");
    check_eq!(check_ok!(syscall::lseek(fd, 0, SEEK_SET), "seek"), 0, "zero");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "lseek past EOF succeeds and returns the requested offset")]
fn lseek_beyond_eof_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"be", 0o644), "create");
    check_ok!(syscall::write(fd, b"x"), "write");
    check_eq!(check_ok!(syscall::lseek(fd, 1000, SEEK_SET), "seek"), 1000, "pos");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "lseek on a pipe returns ESPIPE")]
fn lseek_pipe_espipe() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_err!(syscall::lseek(r, 0, SEEK_SET), Errno::ESPIPE, "pipe seek");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "pread on fd -1 returns EBADF")]
fn pread_ebadf() -> TestResult {
    let mut buf = [0u8; 4];
    check_err!(syscall::pread(-1, &mut buf, 0), Errno::EBADF, "pread");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "pwrite on fd -1 returns EBADF")]
fn pwrite_ebadf() -> TestResult {
    check_err!(syscall::pwrite(-1, b"x", 0), Errno::EBADF, "pwrite");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "F_SETLK on fd -1 returns EBADF")]
fn fcntl_setlk_bad_fd() -> TestResult {
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 1,
        l_pid: 0,
    };
    check_err!(
        syscall::fcntl_flock(-1, fcntl_cmd::F_SETLK, &mut lk),
        Errno::EBADF,
        "bad"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD preserves O_RDONLY access mode on the new fd")]
fn fcntl_dupfd_preserves_mode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"m")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, 0), "dup") as i32;
    let fl = check_ok!(syscall::fcntl(d, fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & 3 == oflag::O_RDONLY, "rdonly");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "F_SETLK can hold two non-overlapping write locks on one file")]
fn fcntl_setlk_two_ranges() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"2r", 0o644), "create");
    check_ok!(syscall::write(fd, b"0123456789"), "write");
    let mut a = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 2,
        l_pid: 0,
    };
    let mut b = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 5,
        l_len: 2,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut a), "a");
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut b), "b");
    a.l_type = F_UNLCK;
    b.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut a), "ua");
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut b), "ub");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_GETFL reports O_WRONLY after opening write-only")]
fn fcntl_getfl_wronly() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"wo")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "open");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & 3 == oflag::O_WRONLY, "wronly");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "preadv at a mid-file offset returns the expected slice")]
fn preadv_offset_mid() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"pom", 0o644), "create");
    check_ok!(syscall::write(fd, b"0123456789"), "write");
    let mut out = [0u8; 3];
    let mut iov = [IoVec {
        iov_base: out.as_mut_ptr(),
        iov_len: 3,
    }];
    check_eq!(check_ok!(syscall::preadv(fd, &mut iov, 4), "preadv"), 3, "n");
    check_eq!(&out, b"456", "mid");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "writev of a single buffer is readable back as the same bytes")]
fn writev_single_then_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"ws", 0o644), "create");
    let msg = b"solo";
    let mut iov = [IoVec {
        iov_base: msg.as_ptr() as *mut u8,
        iov_len: msg.len(),
    }];
    check_ok!(syscall::writev(fd, &mut iov), "writev");
    check_ok!(syscall::lseek(fd, 0, SEEK_SET), "seek");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 4, "n");
    check_eq!(&buf, b"solo", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "write through an F_DUPFD descriptor is visible on the original fd")]
fn fcntl_dupfd_then_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"dw", 0o644), "create");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, 0), "dup") as i32;
    check_ok!(syscall::write(d, b"via-dup"), "write");
    let mut buf = [0u8; 7];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, b"via-dup", "data");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETFL O_APPEND makes a write append after seeking to the start")]
fn fcntl_setfl_append_and_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"sa")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"A"), "w1");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "getfl");
    check_ok!(
        syscall::fcntl(fd, fcntl_cmd::F_SETFL, (fl as i32 | oflag::O_APPEND) as usize),
        "setfl"
    );
    check_ok!(syscall::lseek(fd, 0, SEEK_SET), "seek");
    check_ok!(syscall::write(fd, b"B"), "w2");
    let mut buf = [0u8; 2];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, b"AB", "append");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "preadv into five one-byte iovecs scatters each file byte")]
fn preadv_five_iov_scatter() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"5i", 0o644), "create");
    check_ok!(syscall::write(fd, b"12345"), "write");
    let mut c = [[0u8; 1]; 5];
    let mut iov = [
        IoVec { iov_base: c[0].as_mut_ptr(), iov_len: 1 },
        IoVec { iov_base: c[1].as_mut_ptr(), iov_len: 1 },
        IoVec { iov_base: c[2].as_mut_ptr(), iov_len: 1 },
        IoVec { iov_base: c[3].as_mut_ptr(), iov_len: 1 },
        IoVec { iov_base: c[4].as_mut_ptr(), iov_len: 1 },
    ];
    check_eq!(check_ok!(syscall::preadv(fd, &mut iov, 0), "preadv"), 5, "n");
    check_eq!(c[0][0], b'1', "1");
    check_eq!(c[4][0], b'5', "5");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "F_GETLK with an invalid whence returns EINVAL or succeeds")]
fn fcntl_getlk_bad_whence_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"bw", 0o644), "create");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: 99,
        l_start: 0,
        l_len: 1,
        l_pid: 0,
    };
    match syscall::fcntl_flock(fd, fcntl_cmd::F_GETLK, &mut lk) {
        Err(Errno::EINVAL) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("getlk whence")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD_CLOEXEC sets FD_CLOEXEC on the new fd only")]
fn fcntl_dupfd_cloexec_not_on_orig() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"orig", 0o644), "create");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD_CLOEXEC, 0), "dup") as i32;
    let oflags = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "orig fd");
    check!(oflags & FD_CLOEXEC as usize == 0, "orig no cloexec");
    let dflags = check_ok!(syscall::fcntl(d, fcntl_cmd::F_GETFD, 0), "dup fd");
    check!(dflags & FD_CLOEXEC as usize != 0, "dup cloexec");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "lseek SEEK_CUR before offset 0 returns EINVAL")]
fn lseek_cur_negative_clamp_einval() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"neg", 0o644), "create");
    check_ok!(syscall::write(fd, b"x"), "write");
    check_ok!(syscall::lseek(fd, 0, SEEK_SET), "seek0");
    check_err!(syscall::lseek(fd, -1, SEEK_CUR), Errno::EINVAL, "neg");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwritev updates the file size reported by fstat")]
fn pwritev_then_stat_size() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"psz", 0o644), "create");
    let data = b"SIZE";
    let mut iov = [IoVec {
        iov_base: data.as_ptr() as *mut u8,
        iov_len: data.len(),
    }];
    check_ok!(syscall::pwritev(fd, &mut iov, 0), "pwritev");
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check_eq!(st.st_size, 4, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETLK with length 0 locks from the start offset through EOF")]
fn fcntl_setlk_len_zero_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"zeof", 0o644), "create");
    check_ok!(syscall::write(fd, b"abc"), "write");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 1,
        l_len: 0, // to EOF
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "setlk");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "unlck");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "nonblocking read of an empty pipe returns EAGAIN")]
fn fcntl_nonblock_pipe_eagain() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let fl = check_ok!(syscall::fcntl(r, fcntl_cmd::F_GETFL, 0), "getfl");
    check_ok!(
        syscall::fcntl(r, fcntl_cmd::F_SETFL, (fl as i32 | oflag::O_NONBLOCK) as usize),
        "setfl"
    );
    let mut buf = [0u8; 1];
    check_err!(syscall::read(r, &mut buf), Errno::EAGAIN, "eagain");
    check_ok!(syscall::close(r), "close r");
    check_ok!(syscall::close(w), "close w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "lseek SEEK_DATA after a hole finds data or is unsupported")]
fn lseek_seek_data_after_hole_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"dah", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 16384), "trunc");
    check_ok!(syscall::pwrite(fd, b"DDDD", 8192), "pwrite");
    match syscall::lseek(fd, 0, SEEK_DATA) {
        Ok(pos) => check!(pos >= 0, "data"),
        Err(e) if soft_einval_enosys(e) || e == Errno::ENXIO => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("SEEK_DATA hole")),
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "readv with an empty iovec array returns 0")]
fn readv_empty_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"rve", 0o644), "create");
    check_ok!(syscall::write(fd, b"x"), "write");
    check_ok!(syscall::lseek(fd, 0, SEEK_SET), "seek");
    let mut iov: [IoVec; 0] = [];
    check_eq!(check_ok!(syscall::readv(fd, &mut iov), "readv"), 0, "empty");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "writev with an empty iovec array returns 0")]
fn writev_empty_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"wve", 0o644), "create");
    let mut iov: [IoVec; 0] = [];
    check_eq!(check_ok!(syscall::writev(fd, &mut iov), "writev"), 0, "empty");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_DUPFD honors several minimum-fd floors")]
fn fcntl_dupfd_min_equals_result_floor() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fl", 0o644), "create");
    for min in [0usize, 10, 20, 30] {
        let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, min), "dup") as i32;
        check!(d as usize >= min, "floor");
        check_ok!(syscall::close(d), "close d");
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
