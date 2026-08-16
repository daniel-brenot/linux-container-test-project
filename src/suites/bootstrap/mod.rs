//! Bootstrap tests — behaviours required to run the rest of the suite.
//!
//! If these fail, later suites are skipped. All must work in an unprivileged
//! container with a writable `/tmp`.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::cstr_prefix;
use crate::syscall::{self, map, oflag, prot, Errno};

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "write() of a zero-length buffer to stdout succeeds and returns 0"
)]
fn write_stdout() -> TestResult {
    // If we got here via the harness printer, write already works; still
    // exercise the syscall directly and check a non-zero return.
    let n = check_ok!(
        syscall::write(syscall::STDOUT_FILENO, b""),
        "write(stdout, empty) failed"
    );
    check_eq!(n, 0, "write(stdout, empty) should return 0");
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "creating a file, writing, seeking, reading the same bytes, and closing all succeed"
)]
fn open_close_read_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "TempDir::create failed");
    let fd = check_ok!(tmp.create_file(b"file", 0o644), "create_file failed");
    let msg = b"bootstrap-rw";
    let n = check_ok!(syscall::write(fd, msg), "write failed");
    check_eq!(n, msg.len(), "short write");
    check_ok!(
        syscall::lseek(fd, 0, syscall::SEEK_SET),
        "lseek failed"
    );
    let mut buf = [0u8; 32];
    let n = check_ok!(syscall::read(fd, &mut buf), "read failed");
    check_eq!(n, msg.len(), "short read");
    check!(buf[..n] == msg[..], "read data mismatch");
    check_ok!(syscall::close(fd), "close failed");
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "mkdir creates a directory and rmdir/unlink remove it so later stat() fails with ENOENT"
)]
fn mkdir_rmdir_unlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "TempDir::create failed");
    let dir = check_ok!(crate::suites::common::copy_child(&mut tmp, b"subdir"), "child path failed");
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir failed");
    let st = check_ok!(syscall::stat(&dir), "stat mkdir failed");
    check!(st.is_dir(), "mkdir did not create a directory");
    check_ok!(syscall::rmdir(&dir), "rmdir failed");
    check_err!(
        syscall::stat(&dir),
        Errno::ENOENT,
        "directory still exists after rmdir"
    );

    let fd = check_ok!(tmp.create_file(b"a", 0o644), "create_file failed");
    check_ok!(syscall::close(fd), "close failed");
    let file = check_ok!(crate::suites::common::copy_child(&mut tmp, b"a"), "child a failed");
    check_ok!(syscall::unlink(&file), "unlink failed");
    check_err!(
        syscall::stat(&file),
        Errno::ENOENT,
        "file still exists after unlink"
    );
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "chdir() to a temporary directory is reflected by getcwd(), then the original directory is restored"
)]
fn getcwd_chdir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "TempDir::create failed");
    let mut cwd = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut cwd), "getcwd failed");
    check!(n > 1, "getcwd returned empty");
    check!(cwd[n - 1] == 0, "getcwd not NUL-terminated");

    let root = tmp.path();
    check_ok!(syscall::chdir(root), "chdir to temp failed");
    let mut cwd2 = [0u8; 256];
    let n2 = check_ok!(syscall::getcwd(&mut cwd2), "getcwd after chdir failed");
    let root_bytes = &root[..root.len() - 1];
    check!(
        &cwd2[..n2 - 1] == root_bytes,
        "getcwd does not match chdir target"
    );

    // Restore previous cwd so later tests are not surprised.
    check_ok!(syscall::chdir(&cwd[..n]), "chdir restore failed");
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "an anonymous writable mapping can be stored to and is released by munmap()"
)]
fn mmap_munmap() -> TestResult {
    let len = 4096usize;
    let addr = check_ok!(
        syscall::mmap(
            0,
            len,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0,
        ),
        "mmap failed"
    );
    check!(addr != 0 && addr != usize::MAX, "mmap returned invalid addr");
    // Safety: anonymous private mapping of `len` bytes.
    unsafe {
        let slice = core::slice::from_raw_parts_mut(addr as *mut u8, len);
        slice[0] = 0xA5;
        slice[len - 1] = 0x5A;
        check_eq!(slice[0], 0xA5, "mmap write/read mismatch at start");
        check_eq!(slice[len - 1], 0x5A, "mmap write/read mismatch at end");
    }
    check_ok!(syscall::munmap(addr, len), "munmap failed");
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "pipe2() creates a pipe and bytes written to the write end are read from the read end"
)]
fn pipe2_roundtrip() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_CLOEXEC), "pipe2 failed");
    let msg = b"pipe-ok";
    let n = check_ok!(syscall::write(w, msg), "pipe write failed");
    check_eq!(n, msg.len(), "pipe short write");
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::read(r, &mut buf), "pipe read failed");
    check_eq!(n, msg.len(), "pipe short read");
    check!(&buf[..n] == msg, "pipe data mismatch");
    check_ok!(syscall::close(r), "close read end failed");
    check_ok!(syscall::close(w), "close write end failed");
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "fork() creates a child that exits 42 and wait4() reports that status"
)]
fn fork_wait_exit() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork failed");
    if pid == 0 {
        syscall::exit(42);
    }
    let mut status = 0i32;
    let waited = check_ok!(syscall::wait4(pid, &mut status, 0), "wait4 failed");
    check_eq!(waited, pid, "wait4 returned unexpected pid");
    check!(syscall::wifexited(status), "child did not exit normally");
    check_eq!(
        syscall::wexitstatus(status),
        42,
        "child exit status mismatch"
    );
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "CLOCK_MONOTONIC returns a valid timespec and does not go backwards"
)]
fn clock_gettime_monotonic() -> TestResult {
    let t1 = check_ok!(
        syscall::clock_gettime(syscall::clock::CLOCK_MONOTONIC),
        "clock_gettime failed"
    );
    check!(t1.tv_sec >= 0, "negative monotonic seconds");
    check!(
        t1.tv_nsec >= 0 && t1.tv_nsec < 1_000_000_000,
        "invalid nsec"
    );
    let t2 = check_ok!(
        syscall::clock_gettime(syscall::clock::CLOCK_MONOTONIC),
        "clock_gettime 2 failed"
    );
    check!(
        t2.tv_sec > t1.tv_sec || (t2.tv_sec == t1.tv_sec && t2.tv_nsec >= t1.tv_nsec),
        "monotonic clock went backwards"
    );
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "getpid() is positive and stable, and equals gettid() in a single-threaded process"
)]
fn getpid_identity() -> TestResult {
    let a = syscall::getpid();
    let b = syscall::getpid();
    check!(a > 0, "getpid <= 0");
    check_eq!(a, b, "getpid not stable");
    check_eq!(syscall::gettid(), a, "gettid should equal getpid in single-thread");
    Ok(())
}

#[crate::lctp_test(
    suite = bootstrap,
    expect = success,
    case = "uname() succeeds and reports a Linux kernel"
)]
fn uname_linux() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname failed");
    let sys = cstr_prefix(&u.sysname);
    check!(sys.starts_with(b"Linux"), "sysname is not Linux");
    Ok(())
}
