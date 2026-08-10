//! Misc LTP-ish depth: uname, getrandom, ioctl, dup, close_range, kcmp, etc.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, cstr_prefix};
use crate::syscall::{
    self, oflag, Errno, CapUserData, CapUserHeader, IoVec, Winsize, CLOSE_RANGE_CLOEXEC,
    FD_CLOEXEC, GRND_NONBLOCK, GRND_RANDOM, KCMP_FILE, LINUX_CAPABILITY_VERSION_3, LOCK_EX,
    LOCK_NB, LOCK_SH, LOCK_UN, MEMBARRIER_CMD_QUERY, TIOCGWINSZ,
};

#[crate::lctp_test(suite = syscall)]
fn misc_uname_sysname() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname");
    check!(cstr_prefix(&u.sysname).starts_with(b"Linux"), "sys");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_uname_nodename() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname");
    check!(!cstr_prefix(&u.nodename).is_empty(), "node");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_uname_release() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname");
    check!(!cstr_prefix(&u.release).is_empty(), "rel");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_uname_version() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname");
    check!(!cstr_prefix(&u.version).is_empty(), "ver");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_uname_machine() -> TestResult {
    let u = check_ok!(syscall::uname(), "uname");
    let m = cstr_prefix(&u.machine);
    check!(!m.is_empty(), "mach");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_getrandom_basic() -> TestResult {
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::getrandom(&mut buf, 0), "gr"), 16, "n");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_getrandom_nonblock_soft() -> TestResult {
    let mut buf = [0u8; 8];
    match syscall::getrandom(&mut buf, GRND_NONBLOCK) {
        Ok(n) => check!(n > 0, "n"),
        Err(Errno::EAGAIN) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("grnd nonblock")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_getrandom_random_soft() -> TestResult {
    let mut buf = [0u8; 8];
    match syscall::getrandom(&mut buf, GRND_RANDOM) {
        Ok(n) => check!(n > 0, "n"),
        Err(Errno::EAGAIN) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("grnd random")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_ioctl_tiocgwinsz_enotty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"tty", 0o644), "create");
    let mut ws = Winsize::default();
    match syscall::ioctl(fd, TIOCGWINSZ, &mut ws as *mut _ as usize) {
        Err(Errno::ENOTTY) | Err(Errno::EINVAL) => {}
        Ok(_) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("tiocgwinsz"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_dup_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"d", 0o644), "create");
    let d = check_ok!(syscall::dup(fd), "dup");
    check!(d != fd, "diff");
    check_ok!(syscall::close(fd), "c");
    check_ok!(syscall::close(d), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_dup2_to_high() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"d2", 0o644), "create");
    let d = check_ok!(syscall::dup2(fd, 80), "dup2");
    check_eq!(d, 80, "fd");
    check_ok!(syscall::close(fd), "c");
    check_ok!(syscall::close(d), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_dup3_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"d3", 0o644), "create");
    let d = check_ok!(syscall::dup3(fd, 81, oflag::O_CLOEXEC), "dup3");
    let flags = check_ok!(syscall::fcntl(d, syscall::fcntl_cmd::F_GETFD, 0), "fd");
    check!(flags & FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(fd), "c");
    check_ok!(syscall::close(d), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_dup_ebadf() -> TestResult {
    check_err!(syscall::dup(-1), Errno::EBADF, "dup");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_dup2_ebadf() -> TestResult {
    check_err!(syscall::dup2(-1, 10), Errno::EBADF, "dup2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_close_range_cloexec_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"cr", 0o644), "create");
    let d = check_ok!(syscall::dup(fd), "dup");
    match syscall::close_range(d as u32, d as u32, CLOSE_RANGE_CLOEXEC) {
        Ok(()) => {
            let flags = check_ok!(syscall::fcntl(d, syscall::fcntl_cmd::F_GETFD, 0), "fd");
            check!(flags & FD_CLOEXEC as usize != 0, "cloexec");
            check_ok!(syscall::close(d), "cd");
        }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {
            check_ok!(syscall::close(d), "cd");
        }
        Err(_) => {
            let _ = syscall::close(d);
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("close_range"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_close_range_close_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"crc", 0o644), "create");
    let d = check_ok!(syscall::fcntl(fd, syscall::fcntl_cmd::F_DUPFD, 90), "dup") as i32;
    match syscall::close_range(d as u32, d as u32, 0) {
        Ok(()) => {
            check_err!(syscall::close(d), Errno::EBADF, "closed");
        }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {
            check_ok!(syscall::close(d), "cd");
        }
        Err(_) => {
            let _ = syscall::close(d);
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("cr close"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_kcmp_same() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"k")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let pid = syscall::getpid();
    match syscall::kcmp(pid, pid, KCMP_FILE, fd as u64, fd as u64) {
        Ok(0) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("kcmp ne")),
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("kcmp"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_kcmp_distinct_files() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = check_ok!(tmp.create_file(b"a", 0o644), "a");
    let b = check_ok!(tmp.create_file(b"b", 0o644), "b");
    let pid = syscall::getpid();
    match syscall::kcmp(pid, pid, KCMP_FILE, a as u64, b as u64) {
        Ok(v) => check!(v != 0, "diff"),
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => {
            let _ = syscall::close(a);
            let _ = syscall::close(b);
            return Err(crate::harness::AssertFail::msg("kcmp2"));
        }
    }
    check_ok!(syscall::close(a), "ca");
    check_ok!(syscall::close(b), "cb");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_process_vm_readv_self() -> TestResult {
    let mut src = [1u8, 2, 3, 4];
    let mut dst = [0u8; 4];
    let mut local = [IoVec {
        iov_base: dst.as_mut_ptr(),
        iov_len: 4,
    }];
    let remote = [IoVec {
        iov_base: src.as_mut_ptr(),
        iov_len: 4,
    }];
    match syscall::process_vm_readv(syscall::getpid(), &mut local, &remote, 0) {
        Ok(n) => {
            check_eq!(n, 4, "n");
            check_eq!(&dst, &[1, 2, 3, 4], "d");
        }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("vm_readv")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_process_vm_writev_self() -> TestResult {
    let mut dst = [0u8; 4];
    let src = [9u8, 8, 7, 6];
    let mut local = [IoVec {
        iov_base: src.as_ptr() as *mut u8,
        iov_len: 4,
    }];
    let remote = [IoVec {
        iov_base: dst.as_mut_ptr(),
        iov_len: 4,
    }];
    match syscall::process_vm_writev(syscall::getpid(), &local, &remote, 0) {
        Ok(n) => {
            check_eq!(n, 4, "n");
            check_eq!(&dst, &[9, 8, 7, 6], "d");
        }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("vm_writev")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_membarrier_query() -> TestResult {
    let m = check_ok!(syscall::membarrier(MEMBARRIER_CMD_QUERY, 0), "q");
    let _ = m;
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_personality_query() -> TestResult {
    let p = check_ok!(syscall::personality(0xffff_ffff), "p");
    let p2 = check_ok!(syscall::personality(0xffff_ffff), "p2");
    check_eq!(p, p2, "stable");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_capget_v3() -> TestResult {
    let mut hdr = CapUserHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let mut data = [CapUserData::default(); 2];
    check_ok!(syscall::capget(&mut hdr, &mut data), "capget");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_getcpu_soft() -> TestResult {
    let mut cpu = 0u32;
    let mut node = 0u32;
    match syscall::getcpu(Some(&mut cpu), Some(&mut node)) {
        Ok(()) => {
            check!(cpu < 4096, "cpu");
            let _ = node;
        }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("getcpu")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_sched_yield_many() -> TestResult {
    for _ in 0..8 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_sched_getaffinity() -> TestResult {
    let mut mask = [0u8; 128];
    match syscall::sched_getaffinity(0, &mut mask) {
        Ok(()) => check!(mask.iter().any(|&b| b != 0), "mask"),
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("affinity")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_sched_setaffinity_roundtrip_soft() -> TestResult {
    let mut mask = [0u8; 128];
    if syscall::sched_getaffinity(0, &mut mask).is_err() {
        return Ok(());
    }
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ENOSYS) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("setaff")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_flock_ex_un() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fl", 0o644), "create");
    check_ok!(syscall::flock(fd, LOCK_EX), "ex");
    check_ok!(syscall::flock(fd, LOCK_UN), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_flock_sh_un() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fls", 0o644), "create");
    check_ok!(syscall::flock(fd, LOCK_SH), "sh");
    check_ok!(syscall::flock(fd, LOCK_UN), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_flock_ex_nb() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fln", 0o644), "create");
    check_ok!(syscall::flock(fd, LOCK_EX | LOCK_NB), "exnb");
    check_ok!(syscall::flock(fd, LOCK_UN), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_flock_conflict_nb() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"flc")?;
    let a = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "a");
    let b = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "b");
    check_ok!(syscall::flock(a, LOCK_EX), "ex");
    check_err!(syscall::flock(b, LOCK_EX | LOCK_NB), Errno::EWOULDBLOCK, "busy");
    check_ok!(syscall::flock(a, LOCK_UN), "un");
    check_ok!(syscall::close(a), "ca");
    check_ok!(syscall::close(b), "cb");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_sched_getscheduler() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, syscall::SCHED_OTHER, "other");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_getpriority() -> TestResult {
    let p = check_ok!(syscall::getpriority(syscall::PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "range");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn misc_getrandom_64() -> TestResult {
    let mut buf = [0u8; 64];
    check_eq!(check_ok!(syscall::getrandom(&mut buf, 0), "gr"), 64, "n");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_dup2_self() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"ds", 0o644), "create");
    check_eq!(check_ok!(syscall::dup2(fd, fd), "self"), fd, "same");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_uname_stable() -> TestResult {
    let a = check_ok!(syscall::uname(), "a");
    let b = check_ok!(syscall::uname(), "b");
    check_eq!(cstr_prefix(&a.sysname), cstr_prefix(&b.sysname), "sys");
    check_eq!(cstr_prefix(&a.release), cstr_prefix(&b.release), "rel");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_membarrier_twice() -> TestResult {
    let a = check_ok!(syscall::membarrier(MEMBARRIER_CMD_QUERY, 0), "a");
    let b = check_ok!(syscall::membarrier(MEMBARRIER_CMD_QUERY, 0), "b");
    check_eq!(a, b, "stable");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_capget_self_pid() -> TestResult {
    let mut hdr = CapUserHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: syscall::getpid(),
    };
    let mut data = [CapUserData::default(); 2];
    check_ok!(syscall::capget(&mut hdr, &mut data), "capget");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_close_range_empty_soft() -> TestResult {
    match syscall::close_range(1000, 999, 0) {
        Ok(()) | Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("cr empty")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn misc_flock_sh_two_fds() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"fl2")?;
    let a = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "a");
    let b = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "b");
    check_ok!(syscall::flock(a, LOCK_SH), "a");
    check_ok!(syscall::flock(b, LOCK_SH), "b");
    check_ok!(syscall::flock(a, LOCK_UN), "ua");
    check_ok!(syscall::flock(b, LOCK_UN), "ub");
    check_ok!(syscall::close(a), "ca");
    check_ok!(syscall::close(b), "cb");
    Ok(())
}
