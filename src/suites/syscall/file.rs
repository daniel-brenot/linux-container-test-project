//! File descriptor and path syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty, read_file, write_file};
use crate::syscall::{self, fcntl_cmd, oflag, Errno, FD_CLOEXEC, IoVec};

#[crate::lctp_test(suite = syscall, expect = success, case = "open/read/write/close of a regular file succeed")]
fn open_read_write_close() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let msg = b"hello-file";
    check_eq!(check_ok!(syscall::write(fd, msg), "write"), msg.len(), "short write");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "lseek");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), msg.len(), "short read");
    check!(&buf[..msg.len()] == msg, "data mismatch");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "write on O_RDONLY and read on O_WRONLY return EBADF")]
fn open_rdonly_wronly() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let ro = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "O_RDONLY");
    check_err!(syscall::write(ro, b"x"), Errno::EBADF, "write on O_RDONLY");
    check_ok!(syscall::close(ro), "close ro");
    let wo = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "O_WRONLY");
    check_ok!(syscall::write(wo, b"x"), "write wo");
    check_err!(syscall::read(wo, &mut [0u8; 1]), Errno::EBADF, "read on O_WRONLY");
    check_ok!(syscall::close(wo), "close wo");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "write of an empty buffer and read at EOF both return 0")]
fn read_write_zero_len() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_eq!(check_ok!(syscall::write(fd, b""), "write0"), 0, "write empty");
    let mut b = [1u8; 1];
    check_eq!(check_ok!(syscall::read(fd, &mut b), "read0"), 0, "read at eof");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "close of fd -1 returns EBADF")]
fn close_ebadf() -> TestResult {
    check_err!(syscall::close(-1), Errno::EBADF, "close(-1)");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "closing a file descriptor twice returns EBADF")]
fn close_twice_ebadf() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::close(fd), "close1");
    check_err!(syscall::close(fd), Errno::EBADF, "close twice");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "lseek with SEEK_SET, SEEK_CUR, and SEEK_END returns the expected offsets")]
fn lseek_set_cur_end() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"abcdef"), "write");
    check_eq!(check_ok!(syscall::lseek(fd, 0, syscall::SEEK_END), "SEEK_END"), 6, "end");
    check_eq!(check_ok!(syscall::lseek(fd, 2, syscall::SEEK_SET), "SEEK_SET"), 2, "set");
    check_eq!(check_ok!(syscall::lseek(fd, 1, syscall::SEEK_CUR), "SEEK_CUR"), 3, "cur");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "lseek with a negative offset from SEEK_END lands at the expected position")]
fn lseek_negative_from_end() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"12345"), "write");
    let pos = check_ok!(syscall::lseek(fd, -2, syscall::SEEK_END), "SEEK_END -2");
    check_eq!(pos, 3, "pos from end");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwrite and pread at explicit offsets assemble the expected file contents")]
fn pread_pwrite_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::pwrite(fd, b"XXXX", 0), "pwrite0");
    check_ok!(syscall::pwrite(fd, b"YZ", 2), "pwrite2");
    let mut buf = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, b"XXYZ", "contents");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pread past EOF returns 0")]
fn pread_beyond_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"ab"), "write");
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::pread(fd, &mut buf, 10), "pread past eof");
    check_eq!(n, 0, "pread past eof returns 0");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwrite does not change the file offset")]
fn pwrite_no_offset_change() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::lseek(fd, 5, syscall::SEEK_SET), "seek");
    check_ok!(syscall::pwrite(fd, b"Z", 0), "pwrite");
    check_eq!(check_ok!(syscall::lseek(fd, 0, syscall::SEEK_CUR), "cur"), 5, "offset unchanged");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "writev and readv scatter-gather the expected bytes")]
fn readv_writev() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let a = b"AB";
    let b = b"CD";
    let mut iov = [
        IoVec { iov_base: a.as_ptr() as *mut u8, iov_len: a.len() },
        IoVec { iov_base: b.as_ptr() as *mut u8, iov_len: b.len() },
    ];
    let n = check_ok!(syscall::writev(fd, &mut iov), "writev");
    check_eq!(n, 4, "writev len");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut out1 = [0u8; 2];
    let mut out2 = [0u8; 2];
    let mut riov = [
        IoVec { iov_base: out1.as_mut_ptr(), iov_len: 2 },
        IoVec { iov_base: out2.as_mut_ptr(), iov_len: 2 },
    ];
    let rn = check_ok!(syscall::readv(fd, &mut riov), "readv");
    check_eq!(rn, 4, "readv len");
    check_eq!(&out1, b"AB", "readv part1");
    check_eq!(&out2, b"CD", "readv part2");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "dup returns a new fd and dup3 with O_CLOEXEC sets FD_CLOEXEC")]
fn dup_dup3_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d = check_ok!(syscall::dup(fd), "dup");
    check!(d != fd, "same fd");
    let d3 = check_ok!(syscall::dup3(fd, d + 100, oflag::O_CLOEXEC), "dup3");
    check_eq!(d3, d + 100, "dup3 fd");
    let flags = check_ok!(syscall::fcntl(d3, fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(flags & FD_CLOEXEC as usize != 0, "CLOEXEC");
    check_ok!(syscall::close(fd), "close fd");
    check_ok!(syscall::close(d), "close d");
    check_ok!(syscall::close(d3), "close d3");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "dup2 to a new fd copies the file and dup2 onto itself is a no-op")]
fn dup2_same_and_new() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"dup2"), "write");
    // dup2 onto a fresh high fd.
    let target = 70;
    let d = check_ok!(syscall::dup2(fd, target), "dup2");
    check_eq!(d, target, "target fd");
    check_ok!(syscall::lseek(d, 0, syscall::SEEK_SET), "lseek");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(d, &mut buf), "read"), 4, "len");
    check_eq!(&buf, b"dup2", "data");
    // dup2 onto itself is a no-op success.
    check_eq!(check_ok!(syscall::dup2(d, d), "dup2 self"), d, "self");
    check_ok!(syscall::close(fd), "close fd");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "dup2 onto an open target replaces that descriptor with the source file")]
fn dup2_overwrites_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = check_ok!(tmp.create_file(b"a", 0o644), "a");
    let b = check_ok!(tmp.create_file(b"b", 0o644), "b");
    check_ok!(syscall::write(a, b"AAAA"), "write a");
    check_ok!(syscall::write(b, b"BBBB"), "write b");
    let d = check_ok!(syscall::dup2(a, b), "dup2 overwrite");
    check_eq!(d, b, "returns newfd");
    check_ok!(syscall::lseek(d, 0, syscall::SEEK_SET), "lseek");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::read(d, &mut buf), "read"), 4, "len");
    check_eq!(&buf, b"AAAA", "now a");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_GETFL reports O_RDWR and F_SETFL can add O_APPEND")]
fn fcntl_getfl_setfl() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "F_GETFL");
    check!(fl & 3 == oflag::O_RDWR as usize, "O_RDWR");
    check_ok!(
        syscall::fcntl(fd, fcntl_cmd::F_SETFL, (fl as i32 | oflag::O_APPEND) as usize),
        "F_SETFL"
    );
    let fl2 = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "F_GETFL2");
    check!(fl2 as i32 & oflag::O_APPEND != 0, "O_APPEND");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETFD and F_GETFD round-trip FD_CLOEXEC")]
fn fcntl_getfd_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::fcntl(fd, fcntl_cmd::F_SETFD, FD_CLOEXEC as usize), "F_SETFD");
    let flags = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(flags & FD_CLOEXEC as usize != 0, "CLOEXEC set");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "fsync on a regular file after write succeeds")]
fn fsync_regular_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"sync"), "write");
    check_ok!(syscall::fsync(fd), "fsync");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "fdatasync on a regular file after write succeeds")]
fn fdatasync_regular_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"data"), "write");
    check_ok!(syscall::fdatasync(fd), "fdatasync");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "ftruncate grows and then shrinks a regular file to the requested sizes")]
fn ftruncate_grow_shrink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"1234567890"), "write");
    check_ok!(syscall::ftruncate(fd, 100), "grow");
    let path = copy_child(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 100, "grown size");
    check_ok!(syscall::ftruncate(fd, 3), "shrink");
    let st = check_ok!(syscall::stat(&path), "stat2");
    check_eq!(st.st_size, 3, "shrunk size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "truncate by path shrinks a file to 5 bytes and then to 0")]
fn truncate_path() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"0123456789")?;
    check_ok!(syscall::truncate(&path, 5), "truncate");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 5, "truncated size");
    check_ok!(syscall::truncate(&path, 0), "truncate zero");
    let st = check_ok!(syscall::stat(&path), "stat2");
    check_eq!(st.st_size, 0, "zero size");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "getdents64 on a directory with a created file returns a nonempty listing")]
fn getdents64_lists_entries() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = check_ok!(tmp.create_file(b"entry", 0o644), "create");
    let fd = check_ok!(syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0), "opendir");
    let mut buf = [0u8; 1024];
    let n = check_ok!(syscall::getdents64(fd, &mut buf), "getdents64");
    check!(n > 0, "empty listing");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "rename moves a file so the old path is gone and the new path exists")]
fn rename_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    let dst = copy_child(&mut tmp, b"dst")?;
    check_ok!(syscall::rename(&src, &dst), "rename");
    check_err!(syscall::stat(&src), Errno::ENOENT, "src gone");
    check_ok!(syscall::stat(&dst), "dst exists");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "link raises nlink to 2 and unlink of the new name restores nlink to 1")]
fn link_nlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let st = check_ok!(syscall::stat(&a), "stat");
    check_eq!(st.st_nlink, 2, "nlink");
    check_ok!(syscall::unlink(&b), "unlink b");
    let st = check_ok!(syscall::stat(&a), "stat2");
    check_eq!(st.st_nlink, 1, "nlink after unlink");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "link of a missing source path returns ENOENT")]
fn link_missing_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dst = copy_child(&mut tmp, b"dst")?;
    check_err!(
        syscall::link(b"/tmp/lctp-no-such-src\0", &dst),
        Errno::ENOENT,
        "link missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "symlink and readlink round-trip the target name")]
fn symlink_readlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    let mut buf = [0u8; 64];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(&buf[..n], b"target", "target");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "access with F_OK, R_OK, and W_OK succeeds on a writable file")]
fn access_f_ok_r_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::access(&path, syscall::F_OK), "F_OK");
    check_ok!(syscall::access(&path, syscall::R_OK), "R_OK");
    check_ok!(syscall::access(&path, syscall::W_OK), "W_OK");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "stat reports a regular file of the written size with nlink 1")]
fn stat_regular_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"xy")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_reg(), "regular");
    check_eq!(st.st_size, 2, "size");
    check_eq!(st.st_nlink, 1, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "lstat reports a symlink while stat follows it to a regular file")]
fn lstat_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    let lst = check_ok!(syscall::lstat(&link), "lstat");
    check!(lst.is_lnk(), "symlink");
    let st = check_ok!(syscall::stat(&link), "stat follow");
    check!(st.is_reg(), "followed");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "openat with a directory fd creates a relative file that can be written")]
fn openat_relative() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dirfd = check_ok!(syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0), "opendir");
    let fd = check_ok!(syscall::openat(dirfd, b"rel\0", oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644), "openat");
    check_ok!(syscall::write(fd, b"r"), "write");
    check_ok!(syscall::close(fd), "close file");
    check_ok!(syscall::close(dirfd), "close dir");
    let path = copy_child(&mut tmp, b"rel")?;
    let mut buf = [0u8; 1];
    check_eq!(read_file(&path, &mut buf)?, 1, "read len");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "openat with an invalid dirfd returns EBADF")]
fn openat_bad_fd() -> TestResult {
    check_err!(
        syscall::openat(0x7fff_fffe_u32 as i32, b"x\0", oflag::O_RDONLY, 0),
        Errno::EBADF,
        "bad dirfd"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "read into a buffer shorter than the file returns a partial result")]
fn read_short_buffer() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"1234567890"), "write");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut small = [0u8; 3];
    let n = check_ok!(syscall::read(fd, &mut small), "read");
    check_eq!(n, 3, "short read len");
    check_eq!(&small, b"123", "short read data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "F_SETFL O_APPEND makes a subsequent write append rather than overwrite")]
fn write_append_via_fcntl() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"AB")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "getfl");
    check_ok!(
        syscall::fcntl(fd, fcntl_cmd::F_SETFL, (fl as i32 | oflag::O_APPEND) as usize),
        "setfl"
    );
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    check_ok!(syscall::write(fd, b"CD"), "append write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 8];
    check_eq!(read_file(&path, &mut buf)?, 4, "len");
    check_eq!(&buf[..4], b"ABCD", "append data");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "unlink of a missing path returns ENOENT")]
fn unlink_enoent() -> TestResult {
    check_err!(
        syscall::unlink(b"/tmp/lctp-missing-unlink\0"),
        Errno::ENOENT,
        "unlink missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "open with O_TRUNC zeros an existing regular file")]
fn open_creat_trunc() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"long content")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR | oflag::O_TRUNC, 0), "trunc open");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 0, "truncated");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwritev and preadv at an offset round-trip scatter-gather data")]
fn pwritev_preadv_offset() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
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
    let n = check_ok!(syscall::pwritev(fd, &mut iov, 2), "pwritev");
    check_eq!(n, 4, "wrote");
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
    let rn = check_ok!(syscall::preadv(fd, &mut riov, 2), "preadv");
    check_eq!(rn, 4, "read");
    check_eq!(&o1, b"AB", "part1");
    check_eq!(&o2, b"CD", "part2");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwritev does not change the file offset")]
fn pwritev_no_offset_change() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::lseek(fd, 7, syscall::SEEK_SET), "seek");
    let data = b"Z";
    let mut iov = [IoVec {
        iov_base: data.as_ptr() as *mut u8,
        iov_len: 1,
    }];
    check_ok!(syscall::pwritev(fd, &mut iov, 0), "pwritev");
    check_eq!(
        check_ok!(syscall::lseek(fd, 0, syscall::SEEK_CUR), "cur"),
        7,
        "offset unchanged"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "preadv past EOF returns 0")]
fn preadv_beyond_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    check_ok!(syscall::write(fd, b"ab"), "write");
    let mut out = [0u8; 8];
    let mut iov = [IoVec {
        iov_base: out.as_mut_ptr(),
        iov_len: out.len(),
    }];
    let n = check_ok!(syscall::preadv(fd, &mut iov, 10), "preadv");
    check_eq!(n, 0, "past eof");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "pwritev at a nonzero offset leaves a hole that preadv can read")]
fn pwritev_gap_then_preadv() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let data = b"XYZ";
    let mut iov = [IoVec {
        iov_base: data.as_ptr() as *mut u8,
        iov_len: data.len(),
    }];
    check_ok!(syscall::pwritev(fd, &mut iov, 4), "pwritev");
    let mut out = [0u8; 8];
    let mut riov = [IoVec {
        iov_base: out.as_mut_ptr(),
        iov_len: out.len(),
    }];
    let n = check_ok!(syscall::preadv(fd, &mut riov, 0), "preadv");
    check_eq!(n, 7, "len with hole");
    check_eq!(&out[4..7], b"XYZ", "tail");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "dup3 with oldfd equal to newfd returns EINVAL")]
fn dup3_same_fd_einval() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    // dup3(old, old, ...) is EINVAL (unlike dup2).
    check_err!(syscall::dup3(fd, fd, 0), Errno::EINVAL, "dup3 same");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "dup3 without O_CLOEXEC leaves FD_CLOEXEC clear")]
fn dup3_no_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "create");
    let d3 = check_ok!(syscall::dup3(fd, 75, 0), "dup3");
    check_eq!(d3, 75, "fd");
    let flags = check_ok!(syscall::fcntl(d3, fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(flags & FD_CLOEXEC as usize == 0, "no cloexec");
    check_ok!(syscall::close(fd), "close fd");
    check_ok!(syscall::close(d3), "close d3");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "dup3 onto an open target replaces that descriptor with the source file")]
fn dup3_overwrite_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = check_ok!(tmp.create_file(b"a", 0o644), "a");
    let b = check_ok!(tmp.create_file(b"b", 0o644), "b");
    check_ok!(syscall::write(a, b"AAA"), "write a");
    check_ok!(syscall::write(b, b"BBB"), "write b");
    let d = check_ok!(syscall::dup3(a, b, 0), "dup3 overwrite");
    check_eq!(d, b, "newfd");
    check_ok!(syscall::lseek(d, 0, syscall::SEEK_SET), "lseek");
    let mut buf = [0u8; 3];
    check_eq!(check_ok!(syscall::read(d, &mut buf), "read"), 3, "len");
    check_eq!(&buf, b"AAA", "now a");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(d), "close d");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "dup3 with oldfd -1 returns EBADF")]
fn dup3_bad_old_ebadf() -> TestResult {
    check_err!(syscall::dup3(-1, 10, 0), Errno::EBADF, "bad old");
    Ok(())
}
