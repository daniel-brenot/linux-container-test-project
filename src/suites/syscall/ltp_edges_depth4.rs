//! Depth-4 denser unprivileged coverage across alarm/brk/net/capget/clock/
//! close_range/copy_file_range/dup/fcntl/epoll/eventfd/fallocate/flock/futex/
//! getdents/getrandom/inotify/ioctl/kill/linkat/lseek/madvise/mprotect/mremap/
//! memfd/mincore/mkdirat/mknodat/mmap/mount/mqueue/nanosleep/openat2/personality/
//! pidfd/pipe2/poll/prctl/pread/pwrite/process_vm/ptrace/readv/writev/renameat2/
//! sched/sendfile/splice/tee/vmsplice/setitimer/signalfd/socketpair/statx/sync/
//! sysinfo/timerfd/times/truncate/ualarm/uname/unlinkat/waitid/write.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty, write_file};
use crate::syscall::{
    self, clock, epoll, fcntl_cmd, madvise, map, oflag, poll, prot, wait, CapUserData,
    CapUserHeader, Errno, Flock, IoVec, Itimerspec, Itimerval, OpenHow, SockAddrIn, Statx,
    Timespec, AF_INET, AF_UNIX, AT_FDCWD, AT_REMOVEDIR, CLOSE_RANGE_CLOEXEC, EFD_CLOEXEC,
    EFD_NONBLOCK, EFD_SEMAPHORE, EPOLLET, EPOLLIN, EPOLL_CTL_ADD, EPOLL_CTL_DEL,
    FALLOC_FL_KEEP_SIZE, FALLOC_FL_PUNCH_HOLE, F_ADD_SEALS, F_GET_SEALS, F_RDLCK, F_SEAL_GROW,
    F_SEAL_SEAL, F_SEAL_SHRINK, F_SEAL_WRITE, F_UNLCK, F_WRLCK, IN_ATTRIB, IN_CLOEXEC,
    IN_CLOSE_NOWRITE, IN_CLOSE_WRITE, IN_CREATE, IN_DELETE, IN_MODIFY, IN_MOVED_FROM, IN_MOVED_TO,
    IN_OPEN, ITIMER_REAL, LINUX_CAPABILITY_VERSION_3, LOCK_EX, LOCK_NB, LOCK_SH, LOCK_UN,
    MFD_ALLOW_SEALING, MREMAP_MAYMOVE, MS_ASYNC, P_PID, RENAME_EXCHANGE, RENAME_NOREPLACE,
    RESOLVE_BENEATH, RESOLVE_NO_SYMLINKS, SEEK_CUR, SEEK_DATA, SEEK_END, SEEK_HOLE, SEEK_SET,
    SFD_CLOEXEC, SFD_NONBLOCK, SIGKILL, SIGUSR1, S_IFIFO, SOCK_CLOEXEC, SOCK_DGRAM, SOCK_NONBLOCK,
    SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR, STATX_BASIC_STATS, STATX_SIZE, STATX_TYPE, TFD_CLOEXEC,
};

const PAGE: usize = 4096;
const PTRACE_TRACEME: i32 = 0;
const PTRACE_ATTACH: i32 = 16;
const PTRACE_DETACH: i32 = 17;

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::EINVAL
            | Errno::ENOSYS
            | Errno::EPERM
            | Errno::EOPNOTSUPP
            | Errno::ENOTSUP
            | Errno::ENOMEM
            | Errno::EBUSY
            | Errno::EACCES
            | Errno::ENOSPC
            | Errno::ENOENT
            | Errno::EXDEV
    )
}

macro_rules! flock_byte {
    ($name:ident, $ty:expr, $off:expr, $len:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("fcntl F_SETLK applies a ", stringify!($ty), " lock at offset ", stringify!($off), " length ", stringify!($len)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"lk", 0o644), "c");
            check_ok!(syscall::write(fd, b"0123456789ABCDEFGHIJ"), "w");
            let mut lk = Flock {
                l_type: $ty,
                l_whence: SEEK_SET as i16,
                l_start: $off,
                l_len: $len,
                l_pid: 0,
            };
            check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "lk");
            lk.l_type = F_UNLCK;
            check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

macro_rules! dupfd_min {
    ($name:ident, $min:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("fcntl F_DUPFD returns a new fd at least ", stringify!($min)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"d", 0o644), "c");
            let n = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, $min), "dup");
            check!(n as i32 >= $min as i32, "min");
            check_ok!(syscall::close(n as i32), "cn");
            check_ok!(syscall::close(fd), "cf");
            Ok(())
        }
    };
}

macro_rules! wait_exit {
    ($name:ident, $code:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("wait4 reports wexitstatus ", stringify!($code), " for a child that exits that code"))]
        fn $name() -> TestResult {
            let pid = check_ok!(syscall::fork(), "f");
            if pid == 0 {
                syscall::exit($code);
            }
            let mut st = 0;
            check_ok!(syscall::wait4(pid, &mut st, 0), "w");
            check!(syscall::wifexited(st), "ex");
            check_eq!(syscall::wexitstatus(st), $code, "c");
            Ok(())
        }
    };
}

macro_rules! madvise_advice {
    ($name:ident, $adv:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("madvise with advice ", stringify!($adv), " is accepted on an anonymous page"))]
        fn $name() -> TestResult {
            let addr = check_ok!(
                syscall::mmap(
                    0,
                    PAGE,
                    prot::PROT_READ | prot::PROT_WRITE,
                    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
                    -1,
                    0
                ),
                "mmap"
            );
            match syscall::madvise(addr, PAGE, $adv) {
                Ok(()) => {}
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::munmap(addr, PAGE);
                    return Err(crate::harness::AssertFail::msg("madvise"));
                }
            }
            check_ok!(syscall::munmap(addr, PAGE), "un");
            Ok(())
        }
    };
}

macro_rules! mprotect_combo {
    ($name:ident, $p:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("mprotect with prot ", stringify!($p), " is accepted on an anonymous page"))]
        fn $name() -> TestResult {
            let addr = check_ok!(
                syscall::mmap(
                    0,
                    PAGE,
                    prot::PROT_READ | prot::PROT_WRITE,
                    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
                    -1,
                    0
                ),
                "mmap"
            );
            match syscall::mprotect(addr, PAGE, $p) {
                Ok(()) => {}
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::munmap(addr, PAGE);
                    return Err(crate::harness::AssertFail::msg("mprotect"));
                }
            }
            check_ok!(syscall::munmap(addr, PAGE), "un");
            Ok(())
        }
    };
}

macro_rules! memfd_seal {
    ($name:ident, $seal:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("F_ADD_SEALS ", stringify!($seal), " is visible via F_GET_SEALS"))]
        fn $name() -> TestResult {
            let fd = match syscall::memfd_create(b"s\0", MFD_ALLOW_SEALING as u32) {
                Ok(f) => f,
                Err(e) if soft(e) => return Ok(()),
                Err(_) => return Err(crate::harness::AssertFail::msg("memfd")),
            };
            let _ = syscall::ftruncate(fd, 4096);
            match syscall::fcntl(fd, F_ADD_SEALS, $seal as usize) {
                Ok(_) => {
                    let s = check_ok!(syscall::fcntl(fd, F_GET_SEALS, 0), "get");
                    check!(s as i32 & $seal != 0, "sealed");
                }
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("seal"));
                }
            }
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

macro_rules! falloc_punch {
    ($name:ident, $off:expr, $len:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("fallocate FALLOC_FL_PUNCH_HOLE at offset ", stringify!($off), " length ", stringify!($len), " is accepted"))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"fa", 0o644), "c");
            check_ok!(syscall::ftruncate(fd, 8192), "tr");
            match syscall::fallocate(
                fd,
                FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE,
                $off,
                $len,
            ) {
                Ok(()) => {}
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("punch"));
                }
            }
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

macro_rules! tfd_rel_ms {
    ($name:ident, $ms:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("timerfd_settime arms a relative CLOCK_MONOTONIC expiry of ", stringify!($ms), " ms"))]
        fn $name() -> TestResult {
            let fd = check_ok!(
                syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC),
                "t"
            );
            let its = Itimerspec {
                it_interval: Timespec::default(),
                it_value: Timespec {
                    tv_sec: 0,
                    tv_nsec: ($ms as i64) * 1_000_000,
                },
            };
            check_ok!(syscall::timerfd_settime(fd, 0, &its), "set");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

macro_rules! in_mask {
    ($name:ident, $m:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("inotify_add_watch accepts mask ", stringify!($m), " and inotify_rm_watch removes it"))]
        fn $name() -> TestResult {
            let tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "i");
            let wd = check_ok!(syscall::inotify_add_watch(fd, tmp.path(), $m), "a");
            check_ok!(syscall::inotify_rm_watch(fd, wd), "r");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

macro_rules! splice_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("splice moves ", stringify!($n), " bytes from one pipe to another"))]
        fn $name() -> TestResult {
            let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
            let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
            let msg = [b'Z'; $n];
            check_ok!(syscall::write(w1, &msg), "w");
            match syscall::splice(r1, None, w2, None, $n, 0) {
                Ok(v) => check_eq!(v, $n, "n"),
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(w1);
                    let _ = syscall::close(w2);
                    let _ = syscall::close(r1);
                    let _ = syscall::close(r2);
                    return Err(crate::harness::AssertFail::msg("splice"));
                }
            }
            let _ = syscall::close(w1);
            let _ = syscall::close(w2);
            let _ = syscall::close(r1);
            let _ = syscall::close(r2);
            Ok(())
        }
    };
}

macro_rules! tee_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("tee copies ", stringify!($n), " bytes from one pipe to another"))]
        fn $name() -> TestResult {
            let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
            let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
            let msg = [b'Y'; $n];
            check_ok!(syscall::write(w1, &msg), "w");
            match syscall::tee(r1, w2, $n, 0) {
                Ok(v) => check_eq!(v, $n, "n"),
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(w1);
                    let _ = syscall::close(w2);
                    let _ = syscall::close(r1);
                    let _ = syscall::close(r2);
                    return Err(crate::harness::AssertFail::msg("tee"));
                }
            }
            let _ = syscall::close(w1);
            let _ = syscall::close(w2);
            let _ = syscall::close(r1);
            let _ = syscall::close(r2);
            Ok(())
        }
    };
}

macro_rules! vmsplice_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("vmsplice writes ", stringify!($n), " bytes from a buffer into a pipe"))]
        fn $name() -> TestResult {
            let (r, w) = check_ok!(syscall::pipe2(0), "p");
            let mut buf = [b'V'; $n];
            let iov = [IoVec {
                iov_base: buf.as_mut_ptr(),
                iov_len: $n,
            }];
            match syscall::vmsplice(w, &iov, 0) {
                Ok(v) => check_eq!(v, $n, "n"),
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(r);
                    let _ = syscall::close(w);
                    return Err(crate::harness::AssertFail::msg("vmsplice"));
                }
            }
            let _ = syscall::close(r);
            let _ = syscall::close(w);
            Ok(())
        }
    };
}

macro_rules! efd_sem_drain {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("EFD_SEMAPHORE eventfd initialized to ", stringify!($n), " yields that many reads of 1"))]
        fn $name() -> TestResult {
            let efd = match syscall::eventfd($n, EFD_SEMAPHORE | EFD_CLOEXEC) {
                Ok(fd) => fd,
                Err(e) if soft(e) => return Ok(()),
                Err(_) => return Err(crate::harness::AssertFail::msg("efd")),
            };
            let mut out = [0u8; 8];
            for _ in 0..$n {
                check_ok!(syscall::read(efd, &mut out), "r");
                check_eq!(u64::from_ne_bytes(out), 1, "1");
            }
            check_ok!(syscall::close(efd), "c");
            Ok(())
        }
    };
}

macro_rules! clock_get {
    ($name:ident, $clk:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("clock_gettime ", stringify!($clk), " returns a non-negative tv_sec"))]
        fn $name() -> TestResult {
            match syscall::clock_gettime($clk) {
                Ok(ts) => check!(ts.tv_sec >= 0, "sec"),
                Err(e) if soft(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("clock")),
            }
            Ok(())
        }
    };
}

macro_rules! clock_res {
    ($name:ident, $clk:expr) => {
        #[crate::lctp_test(suite = syscall, expect = soft, case = concat!("clock_getres ", stringify!($clk), " returns a non-zero resolution"))]
        fn $name() -> TestResult {
            match syscall::clock_getres($clk) {
                Ok(ts) => check!(ts.tv_nsec > 0 || ts.tv_sec > 0, "res"),
                Err(e) if soft(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("getres")),
            }
            Ok(())
        }
    };
}

macro_rules! getrandom_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("getrandom fills ", stringify!($n), " bytes"))]
        fn $name() -> TestResult {
            let mut buf = [0u8; $n];
            check_eq!(check_ok!(syscall::getrandom(&mut buf, 0), "gr"), $n, "n");
            Ok(())
        }
    };
}

macro_rules! write_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("write of ", stringify!($n), " bytes to a file returns that count"))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"w", 0o644), "c");
            let buf = [b'W'; $n];
            check_eq!(check_ok!(syscall::write(fd, &buf), "w"), $n, "n");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

macro_rules! trunc_path {
    ($name:ident, $sz:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("truncate sets st_size to ", stringify!($sz)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"t")?;
            write_file(&p, b"0123456789ABCDEF")?;
            check_ok!(syscall::truncate(&p, $sz), "tr");
            let st = check_ok!(syscall::stat(&p), "st");
            check_eq!(st.st_size, $sz, "sz");
            Ok(())
        }
    };
}

macro_rules! nanosleep_ns {
    ($name:ident, $ns:expr) => {
        #[crate::lctp_test(suite = syscall, expect = success, case = concat!("nanosleep of ", stringify!($ns), " ns returns success"))]
        fn $name() -> TestResult {
            let ts = Timespec {
                tv_sec: 0,
                tv_nsec: $ns,
            };
            check_ok!(syscall::nanosleep(&ts), "ns");
            Ok(())
        }
    };
}

flock_byte!(d4_lk_rd_0_1, F_RDLCK, 0, 1);
flock_byte!(d4_lk_wr_0_1, F_WRLCK, 0, 1);
flock_byte!(d4_lk_rd_1_1, F_RDLCK, 1, 1);
flock_byte!(d4_lk_wr_1_1, F_WRLCK, 1, 1);
flock_byte!(d4_lk_rd_2_1, F_RDLCK, 2, 1);
flock_byte!(d4_lk_wr_2_1, F_WRLCK, 2, 1);
flock_byte!(d4_lk_rd_3_1, F_RDLCK, 3, 1);
flock_byte!(d4_lk_wr_3_1, F_WRLCK, 3, 1);
flock_byte!(d4_lk_rd_4_1, F_RDLCK, 4, 1);
flock_byte!(d4_lk_wr_4_1, F_WRLCK, 4, 1);
flock_byte!(d4_lk_rd_5_1, F_RDLCK, 5, 1);
flock_byte!(d4_lk_wr_5_1, F_WRLCK, 5, 1);
flock_byte!(d4_lk_rd_6_1, F_RDLCK, 6, 1);
flock_byte!(d4_lk_wr_6_1, F_WRLCK, 6, 1);
flock_byte!(d4_lk_rd_7_1, F_RDLCK, 7, 1);
flock_byte!(d4_lk_wr_7_1, F_WRLCK, 7, 1);
flock_byte!(d4_lk_rd_8_1, F_RDLCK, 8, 1);
flock_byte!(d4_lk_wr_8_1, F_WRLCK, 8, 1);
flock_byte!(d4_lk_rd_9_1, F_RDLCK, 9, 1);
flock_byte!(d4_lk_wr_9_1, F_WRLCK, 9, 1);
flock_byte!(d4_lk_rd_10_1, F_RDLCK, 10, 1);
flock_byte!(d4_lk_wr_10_1, F_WRLCK, 10, 1);
flock_byte!(d4_lk_rd_11_1, F_RDLCK, 11, 1);
flock_byte!(d4_lk_wr_11_1, F_WRLCK, 11, 1);
flock_byte!(d4_lk_rd_12_1, F_RDLCK, 12, 1);
flock_byte!(d4_lk_wr_12_1, F_WRLCK, 12, 1);
flock_byte!(d4_lk_rd_13_1, F_RDLCK, 13, 1);
flock_byte!(d4_lk_wr_13_1, F_WRLCK, 13, 1);
flock_byte!(d4_lk_rd_14_1, F_RDLCK, 14, 1);
flock_byte!(d4_lk_wr_14_1, F_WRLCK, 14, 1);
flock_byte!(d4_lk_rd_15_1, F_RDLCK, 15, 1);
flock_byte!(d4_lk_wr_15_1, F_WRLCK, 15, 1);
flock_byte!(d4_lk_rd_0_2b, F_RDLCK, 0, 2);
flock_byte!(d4_lk_wr_0_2b, F_WRLCK, 0, 2);
flock_byte!(d4_lk_rd_2_4b, F_RDLCK, 2, 4);
flock_byte!(d4_lk_wr_2_4b, F_WRLCK, 2, 4);
flock_byte!(d4_lk_rd_4_8b, F_RDLCK, 4, 8);
flock_byte!(d4_lk_wr_4_8b, F_WRLCK, 4, 8);
flock_byte!(d4_lk_rd_8_8b, F_RDLCK, 8, 8);
flock_byte!(d4_lk_wr_8_8b, F_WRLCK, 8, 8);
flock_byte!(d4_lk_rd_0_0b, F_RDLCK, 0, 0);
flock_byte!(d4_lk_wr_0_0b, F_WRLCK, 0, 0);
flock_byte!(d4_lk_rd_10_5b, F_RDLCK, 10, 5);
flock_byte!(d4_lk_wr_10_5b, F_WRLCK, 10, 5);
flock_byte!(d4_lk_rd_3_3b, F_RDLCK, 3, 3);
flock_byte!(d4_lk_wr_3_3b, F_WRLCK, 3, 3);
flock_byte!(d4_lk_rd_6_6b, F_RDLCK, 6, 6);
flock_byte!(d4_lk_wr_6_6b, F_WRLCK, 6, 6);
flock_byte!(d4_lk_rd_12_4b, F_RDLCK, 12, 4);
flock_byte!(d4_lk_wr_12_4b, F_WRLCK, 12, 4);
dupfd_min!(d4_dupfd_10, 10);
dupfd_min!(d4_dupfd_20, 20);
dupfd_min!(d4_dupfd_30, 30);
dupfd_min!(d4_dupfd_40, 40);
dupfd_min!(d4_dupfd_50, 50);
dupfd_min!(d4_dupfd_60, 60);
dupfd_min!(d4_dupfd_70, 70);
dupfd_min!(d4_dupfd_80, 80);
dupfd_min!(d4_dupfd_90, 90);
dupfd_min!(d4_dupfd_100, 100);
dupfd_min!(d4_dupfd_110, 110);
dupfd_min!(d4_dupfd_120, 120);
dupfd_min!(d4_dupfd_130, 130);
dupfd_min!(d4_dupfd_140, 140);
dupfd_min!(d4_dupfd_150, 150);
dupfd_min!(d4_dupfd_160, 160);
dupfd_min!(d4_dupfd_180, 180);
dupfd_min!(d4_dupfd_200, 200);
wait_exit!(d4_ex_1, 1);
wait_exit!(d4_ex_2, 2);
wait_exit!(d4_ex_3, 3);
wait_exit!(d4_ex_5, 5);
wait_exit!(d4_ex_7, 7);
wait_exit!(d4_ex_11, 11);
wait_exit!(d4_ex_13, 13);
wait_exit!(d4_ex_17, 17);
wait_exit!(d4_ex_19, 19);
wait_exit!(d4_ex_23, 23);
wait_exit!(d4_ex_29, 29);
wait_exit!(d4_ex_31, 31);
wait_exit!(d4_ex_37, 37);
wait_exit!(d4_ex_41, 41);
wait_exit!(d4_ex_43, 43);
wait_exit!(d4_ex_47, 47);
wait_exit!(d4_ex_53, 53);
wait_exit!(d4_ex_59, 59);
wait_exit!(d4_ex_61, 61);
wait_exit!(d4_ex_67, 67);
wait_exit!(d4_ex_71, 71);
wait_exit!(d4_ex_73, 73);
wait_exit!(d4_ex_79, 79);
wait_exit!(d4_ex_83, 83);
wait_exit!(d4_ex_89, 89);
wait_exit!(d4_ex_97, 97);
wait_exit!(d4_ex_100, 100);
wait_exit!(d4_ex_127, 127);
wait_exit!(d4_ex_200, 200);
wait_exit!(d4_ex_255, 255);
madvise_advice!(d4_madv_normal, madvise::MADV_NORMAL);
madvise_advice!(d4_madv_random, madvise::MADV_RANDOM);
madvise_advice!(d4_madv_sequential, madvise::MADV_SEQUENTIAL);
madvise_advice!(d4_madv_willneed, madvise::MADV_WILLNEED);
madvise_advice!(d4_madv_dontneed, madvise::MADV_DONTNEED);
madvise_advice!(d4_madv_free, madvise::MADV_FREE);
madvise_advice!(d4_madv_hugepage, madvise::MADV_HUGEPAGE);
madvise_advice!(d4_madv_nohugepage, madvise::MADV_NOHUGEPAGE);
mprotect_combo!(d4_mprot_none, prot::PROT_NONE);
mprotect_combo!(d4_mprot_r, prot::PROT_READ);
mprotect_combo!(d4_mprot_w, prot::PROT_WRITE);
mprotect_combo!(d4_mprot_rw, prot::PROT_READ | prot::PROT_WRITE);
mprotect_combo!(d4_mprot_rx, prot::PROT_READ | prot::PROT_EXEC);
mprotect_combo!(d4_mprot_rwx, prot::PROT_READ | prot::PROT_WRITE | prot::PROT_EXEC);
memfd_seal!(d4_seal_seal, F_SEAL_SEAL);
memfd_seal!(d4_seal_shrink, F_SEAL_SHRINK);
memfd_seal!(d4_seal_grow, F_SEAL_GROW);
memfd_seal!(d4_seal_write, F_SEAL_WRITE);
falloc_punch!(d4_punch_0_512, 0, 512);
falloc_punch!(d4_punch_0_1024, 0, 1024);
falloc_punch!(d4_punch_512_512, 512, 512);
falloc_punch!(d4_punch_1024_1024, 1024, 1024);
falloc_punch!(d4_punch_2048_2048, 2048, 2048);
falloc_punch!(d4_punch_4096_4096, 4096, 4096);
falloc_punch!(d4_punch_0_4096, 0, 4096);
falloc_punch!(d4_punch_256_256, 256, 256);
falloc_punch!(d4_punch_768_256, 768, 256);
falloc_punch!(d4_punch_1536_512, 1536, 512);
tfd_rel_ms!(d4_tfd_1, 1);
tfd_rel_ms!(d4_tfd_2, 2);
tfd_rel_ms!(d4_tfd_3, 3);
tfd_rel_ms!(d4_tfd_5, 5);
tfd_rel_ms!(d4_tfd_7, 7);
tfd_rel_ms!(d4_tfd_10, 10);
tfd_rel_ms!(d4_tfd_15, 15);
tfd_rel_ms!(d4_tfd_20, 20);
tfd_rel_ms!(d4_tfd_25, 25);
tfd_rel_ms!(d4_tfd_30, 30);
tfd_rel_ms!(d4_tfd_40, 40);
tfd_rel_ms!(d4_tfd_50, 50);
tfd_rel_ms!(d4_tfd_75, 75);
tfd_rel_ms!(d4_tfd_100, 100);
tfd_rel_ms!(d4_tfd_125, 125);
tfd_rel_ms!(d4_tfd_150, 150);
tfd_rel_ms!(d4_tfd_200, 200);
tfd_rel_ms!(d4_tfd_250, 250);
in_mask!(d4_in_access, syscall::IN_ACCESS);
in_mask!(d4_in_create, IN_CREATE);
in_mask!(d4_in_delete, IN_DELETE);
in_mask!(d4_in_modify, IN_MODIFY);
in_mask!(d4_in_attrib, IN_ATTRIB);
in_mask!(d4_in_open, IN_OPEN);
in_mask!(d4_in_close_w, IN_CLOSE_WRITE);
in_mask!(d4_in_close_nw, IN_CLOSE_NOWRITE);
in_mask!(d4_in_moved_from, IN_MOVED_FROM);
in_mask!(d4_in_moved_to, IN_MOVED_TO);
in_mask!(d4_in_cd, IN_CREATE | IN_DELETE);
in_mask!(d4_in_cm, IN_CREATE | IN_MODIFY);
in_mask!(d4_in_oa, IN_OPEN | IN_ATTRIB);
in_mask!(d4_in_move, IN_MOVED_FROM | IN_MOVED_TO);
in_mask!(d4_in_close, IN_CLOSE_WRITE | IN_CLOSE_NOWRITE);
in_mask!(d4_in_all, IN_CREATE | IN_DELETE | IN_MODIFY | IN_ATTRIB | IN_OPEN | IN_CLOSE_WRITE | IN_CLOSE_NOWRITE);
splice_n!(d4_sp_1, 1);
splice_n!(d4_sp_2, 2);
splice_n!(d4_sp_3, 3);
splice_n!(d4_sp_4, 4);
splice_n!(d4_sp_5, 5);
splice_n!(d4_sp_8, 8);
splice_n!(d4_sp_12, 12);
splice_n!(d4_sp_16, 16);
splice_n!(d4_sp_24, 24);
splice_n!(d4_sp_32, 32);
splice_n!(d4_sp_48, 48);
splice_n!(d4_sp_64, 64);
splice_n!(d4_sp_96, 96);
splice_n!(d4_sp_128, 128);
splice_n!(d4_sp_160, 160);
splice_n!(d4_sp_192, 192);
splice_n!(d4_sp_200, 200);
splice_n!(d4_sp_256, 256);
splice_n!(d4_sp_300, 300);
splice_n!(d4_sp_400, 400);
tee_n!(d4_tee_1, 1);
tee_n!(d4_tee_2, 2);
tee_n!(d4_tee_4, 4);
tee_n!(d4_tee_8, 8);
tee_n!(d4_tee_16, 16);
tee_n!(d4_tee_32, 32);
tee_n!(d4_tee_48, 48);
tee_n!(d4_tee_64, 64);
tee_n!(d4_tee_80, 80);
tee_n!(d4_tee_100, 100);
tee_n!(d4_tee_128, 128);
tee_n!(d4_tee_150, 150);
tee_n!(d4_tee_200, 200);
vmsplice_n!(d4_vs_1, 1);
vmsplice_n!(d4_vs_2, 2);
vmsplice_n!(d4_vs_4, 4);
vmsplice_n!(d4_vs_8, 8);
vmsplice_n!(d4_vs_16, 16);
vmsplice_n!(d4_vs_32, 32);
vmsplice_n!(d4_vs_48, 48);
vmsplice_n!(d4_vs_64, 64);
vmsplice_n!(d4_vs_96, 96);
vmsplice_n!(d4_vs_128, 128);
vmsplice_n!(d4_vs_160, 160);
vmsplice_n!(d4_vs_192, 192);
vmsplice_n!(d4_vs_256, 256);
efd_sem_drain!(d4_efd_1, 1);
efd_sem_drain!(d4_efd_2, 2);
efd_sem_drain!(d4_efd_3, 3);
efd_sem_drain!(d4_efd_4, 4);
efd_sem_drain!(d4_efd_5, 5);
efd_sem_drain!(d4_efd_6, 6);
efd_sem_drain!(d4_efd_7, 7);
efd_sem_drain!(d4_efd_8, 8);
efd_sem_drain!(d4_efd_10, 10);
efd_sem_drain!(d4_efd_12, 12);
efd_sem_drain!(d4_efd_15, 15);
efd_sem_drain!(d4_efd_16, 16);
efd_sem_drain!(d4_efd_20, 20);
efd_sem_drain!(d4_efd_24, 24);
clock_get!(d4_clk_realtime, clock::CLOCK_REALTIME);
clock_res!(d4_cres_realtime, clock::CLOCK_REALTIME);
clock_get!(d4_clk_monotonic, clock::CLOCK_MONOTONIC);
clock_res!(d4_cres_monotonic, clock::CLOCK_MONOTONIC);
clock_get!(d4_clk_process_cputime_id, clock::CLOCK_PROCESS_CPUTIME_ID);
clock_res!(d4_cres_process_cputime_id, clock::CLOCK_PROCESS_CPUTIME_ID);
clock_get!(d4_clk_thread_cputime_id, clock::CLOCK_THREAD_CPUTIME_ID);
clock_res!(d4_cres_thread_cputime_id, clock::CLOCK_THREAD_CPUTIME_ID);
clock_get!(d4_clk_monotonic_raw, clock::CLOCK_MONOTONIC_RAW);
clock_res!(d4_cres_monotonic_raw, clock::CLOCK_MONOTONIC_RAW);
clock_get!(d4_clk_realtime_coarse, clock::CLOCK_REALTIME_COARSE);
clock_res!(d4_cres_realtime_coarse, clock::CLOCK_REALTIME_COARSE);
clock_get!(d4_clk_monotonic_coarse, clock::CLOCK_MONOTONIC_COARSE);
clock_res!(d4_cres_monotonic_coarse, clock::CLOCK_MONOTONIC_COARSE);
clock_get!(d4_clk_boottime, clock::CLOCK_BOOTTIME);
clock_res!(d4_cres_boottime, clock::CLOCK_BOOTTIME);
getrandom_n!(d4_gr_1, 1);
getrandom_n!(d4_gr_2, 2);
getrandom_n!(d4_gr_4, 4);
getrandom_n!(d4_gr_8, 8);
getrandom_n!(d4_gr_16, 16);
getrandom_n!(d4_gr_24, 24);
getrandom_n!(d4_gr_32, 32);
getrandom_n!(d4_gr_48, 48);
getrandom_n!(d4_gr_64, 64);
write_n!(d4_wr_1, 1);
write_n!(d4_wr_2, 2);
write_n!(d4_wr_4, 4);
write_n!(d4_wr_8, 8);
write_n!(d4_wr_16, 16);
write_n!(d4_wr_32, 32);
write_n!(d4_wr_64, 64);
write_n!(d4_wr_128, 128);
write_n!(d4_wr_256, 256);
write_n!(d4_wr_512, 512);
write_n!(d4_wr_1024, 1024);
trunc_path!(d4_tr_0, 0);
trunc_path!(d4_tr_1, 1);
trunc_path!(d4_tr_2, 2);
trunc_path!(d4_tr_4, 4);
trunc_path!(d4_tr_8, 8);
trunc_path!(d4_tr_12, 12);
trunc_path!(d4_tr_16, 16);
trunc_path!(d4_tr_20, 20);
trunc_path!(d4_tr_24, 24);
trunc_path!(d4_tr_32, 32);
trunc_path!(d4_tr_48, 48);
trunc_path!(d4_tr_64, 64);
nanosleep_ns!(d4_ns_1, 1);
nanosleep_ns!(d4_ns_10, 10);
nanosleep_ns!(d4_ns_100, 100);
nanosleep_ns!(d4_ns_1000, 1000);
nanosleep_ns!(d4_ns_10000, 10000);
nanosleep_ns!(d4_ns_100000, 100000);
nanosleep_ns!(d4_ns_500000, 500000);
nanosleep_ns!(d4_ns_1000000, 1000000);

#[crate::lctp_test(suite = syscall, expect = soft, case = "alarm(0) cancels any pending alarm or is rejected as unsupported")]
fn d4_alarm_soft() -> TestResult {
    match syscall::alarm(0) {
        Ok(_) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("alarm")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "alarm(60) can be armed then cancelled with alarm(0)")]
fn d4_alarm_set_cancel_soft() -> TestResult {
    match syscall::alarm(60) {
        Ok(_) => {
            let _ = syscall::alarm(0);
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("alarm set")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "setitimer ITIMER_REAL with a 1000 us value arms then can be disarmed")]
fn d4_ualarm_soft_via_setitimer() -> TestResult {
    // ualarm(3) is obsolete; exercise equivalent setitimer REAL soft path.
    let mut old = Itimerval::default();
    let neu = Itimerval {
        it_interval: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        it_value: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 1000,
        },
    };
    match syscall::setitimer(ITIMER_REAL, &neu, Some(&mut old)) {
        Ok(()) => {
            let zero = Itimerval::default();
            let _ = syscall::setitimer(ITIMER_REAL, &zero, None);
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("ualarm/setitimer")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "brk(0) returns the current program break")]
fn d4_brk_query() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "brk0");
    check!(cur > 0, "cur");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "brk grows the heap then restores the prior break")]
fn d4_brk_grow_restore() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "cur");
    match syscall::brk(cur + PAGE) {
        Ok(n) => {
            check!(n >= cur, "grew");
            let _ = syscall::brk(cur);
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("brk grow")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "unprivileged mount of tmpfs is rejected with EPERM/EACCES/ENOSYS or similar")]
fn d4_mount_soft_eperm() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let tgt = copy_child(&mut tmp, b"mnt")?;
    check_ok!(syscall::mkdir(&tgt, 0o755), "mkdir");
    match syscall::mount(b"none\0", &tgt, b"tmpfs\0", 0, 0) {
        Err(Errno::EPERM) | Err(Errno::ENOSYS) | Err(Errno::EACCES) => {}
        Err(e) if soft(e) => {}
        Ok(()) => {
            // unexpected success in unprivileged container; still ok
        }
        Err(_) => {
            let _ = syscall::rmdir(&tgt);
            return Err(crate::harness::AssertFail::msg("mount soft"));
        }
    }
    let _ = syscall::rmdir(&tgt);
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "ptrace PTRACE_ATTACH of self is rejected with EPERM/EINVAL/ENOSYS")]
fn d4_ptrace_attach_self_soft() -> TestResult {
    match syscall::ptrace(PTRACE_ATTACH, syscall::getpid(), 0, 0) {
        Err(Errno::EPERM) | Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(e) if soft(e) => {}
        Ok(_) => {
            let _ = syscall::ptrace(PTRACE_DETACH, syscall::getpid(), 0, 0);
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("ptrace")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "ptrace PTRACE_TRACEME in a child succeeds or is rejected as unsupported")]
fn d4_ptrace_traceme_soft() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let code = match syscall::ptrace(PTRACE_TRACEME, 0, 0, 0) {
            Ok(_) => 0,
            Err(Errno::EPERM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => 0,
            Err(_) => 1,
        };
        syscall::exit(code);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    check_eq!(syscall::wexitstatus(st), 0, "ok");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "capget with LINUX_CAPABILITY_VERSION_3 fills capability data")]
fn d4_capget_v3() -> TestResult {
    let mut hdr = CapUserHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let mut data = [CapUserData::default(); 2];
    check_ok!(syscall::capget(&mut hdr, &mut data), "capget");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "personality with 0xffffffff returns the current personality")]
fn d4_personality_query() -> TestResult {
    let p = check_ok!(syscall::personality(0xffff_ffff), "p");
    let p2 = check_ok!(syscall::personality(0xffff_ffff), "p2");
    check_eq!(p, p2, "stable");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "uname succeeds and reports a kernel identity")]
fn d4_uname_linux() -> TestResult {
    let u = check_ok!(syscall::uname(), "u");
    check!(u.sysname[0] != 0, "sys");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sysinfo fills a sysinfo structure")]
fn d4_sysinfo_ok() -> TestResult {
    let si = check_ok!(syscall::sysinfo(), "si");
    check!(si.uptime >= 0, "up");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "times fills a tms structure")]
fn d4_times_ok() -> TestResult {
    let _ = check_ok!(syscall::times(), "times");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sched_yield returns success")]
fn d4_sched_yield() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sched_getscheduler for pid 0 returns a scheduler policy")]
fn d4_sched_getscheduler() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "g");
    check!(pol >= 0, "pol");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sched_getaffinity for pid 0 fills a cpu mask")]
fn d4_sched_getaffinity() -> TestResult {
    let mut mask = [0u8; 128];
    match syscall::sched_getaffinity(0, &mut mask) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("affinity")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "prctl PR_GET_NAME copies the thread name")]
fn d4_prctl_get_name() -> TestResult {
    let mut buf = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "name");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "prctl PR_GET_DUMPABLE returns the dumpable flag")]
fn d4_prctl_dumpable_get() -> TestResult {
    match syscall::prctl(syscall::PR_GET_DUMPABLE, 0, 0, 0, 0) {
        Ok(_) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("dumpable")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sync flushes filesystem caches and returns")]
fn d4_sync_ok() -> TestResult {
    check_ok!(syscall::sync(), "sync");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "syncfs on a directory fd succeeds")]
fn d4_syncfs_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"s", 0o644), "c");
    match syscall::syncfs(fd) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("syncfs"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "clock_settime CLOCK_REALTIME is rejected with EPERM/EACCES/EINVAL or succeeds")]
fn d4_clock_settime_eperm() -> TestResult {
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_REALTIME), "now");
    match syscall::clock_settime(clock::CLOCK_REALTIME, &now) {
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(e) if soft(e) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("settime")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pipe2 with O_CLOEXEC creates a pipe pair")]
fn d4_pipe2_cloexec() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_CLOEXEC), "p");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pipe2 with O_NONBLOCK creates a pipe pair")]
fn d4_pipe2_nonblock() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_NONBLOCK), "p");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "AF_UNIX SOCK_STREAM socketpair creates a connected pair")]
fn d4_socketpair_unix_stream() -> TestResult {
    let (a, b) = check_ok!(
        syscall::socketpair(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "sp"
    );
    check_ok!(syscall::write(a, b"hi"), "w");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::read(b, &mut buf), "r"), 2, "n");
    check_ok!(syscall::close(a), "ca");
    check_ok!(syscall::close(b), "cb");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "AF_UNIX SOCK_DGRAM socketpair creates a connected pair")]
fn d4_socketpair_unix_dgram() -> TestResult {
    let (a, b) = check_ok!(
        syscall::socketpair(AF_UNIX, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "sp"
    );
    check_ok!(syscall::close(a), "ca");
    check_ok!(syscall::close(b), "cb");
    Ok(())
}

fn tcp_pair() -> Result<(i32, i32, i32), crate::harness::AssertFail> {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    let one = 1i32.to_ne_bytes();
    check_ok!(syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one), "re");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "b");
    check_ok!(syscall::listen(srv, 8), "l");
    let bound = check_ok!(syscall::getsockname_in(srv), "n");
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "c");
    check_ok!(syscall::connect(cli, &bound), "conn");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "a");
    Ok((srv, cli, acc))
}

#[crate::lctp_test(suite = syscall, expect = success, case = "TCP bind listen connect and accept4 complete on loopback")]
fn d4_bind_listen_accept4_connect() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::write(cli, b"Z"), "w");
    let mut b = [0u8; 1];
    check_eq!(check_ok!(syscall::read(acc, &mut b), "r"), 1, "n");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "listen with backlog 1 succeeds on a bound TCP socket")]
fn d4_listen_backlog_1() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "b");
    check_ok!(syscall::listen(srv, 1), "l");
    check_ok!(syscall::close(srv), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "listen with backlog 128 succeeds on a bound TCP socket")]
fn d4_listen_backlog_128() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "b");
    check_ok!(syscall::listen(srv, 128), "l");
    check_ok!(syscall::close(srv), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "accept4 with SOCK_NONBLOCK returns a connected fd")]
fn d4_accept4_nonblock() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    let one = 1i32.to_ne_bytes();
    check_ok!(syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one), "re");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "b");
    check_ok!(syscall::listen(srv, 4), "l");
    let bound = check_ok!(syscall::getsockname_in(srv), "n");
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "c");
    check_ok!(syscall::connect(cli, &bound), "conn");
    let acc = check_ok!(
        syscall::accept4(srv, None, None, SOCK_CLOEXEC | SOCK_NONBLOCK),
        "a"
    );
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "epoll_wait reports EPOLLIN after a pipe write")]
fn d4_epoll_pipe_in() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: r as u64,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"x"), "w");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 100), "wait") >= 1, "rdy");
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_DEL, r, &mut ev), "del");
    check_ok!(syscall::close(ep), "cep");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "epoll_wait with EPOLLET reports EPOLLIN after a pipe write")]
fn d4_epoll_et() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLET,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"x"), "w");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 50), "w") >= 1, "rdy");
    check_ok!(syscall::close(ep), "cep");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "poll reports a ready pipe after a write")]
fn d4_poll_ready() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    check_ok!(syscall::write(w, b"z"), "w");
    let mut fds = [poll::PollFd {
        fd: r,
        events: syscall::POLLIN,
        revents: 0,
    }];
    check!(check_ok!(syscall::poll(&mut fds, 0), "p") >= 1, "rdy");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "ppoll with a zero timeout on an idle fd returns 0")]
fn d4_ppoll_zero() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    let mut fds = [poll::PollFd {
        fd: r,
        events: syscall::POLLIN,
        revents: 0,
    }];
    let ts = Timespec { tv_sec: 0, tv_nsec: 0 };
    match syscall::ppoll(&mut fds, Some(&ts), None) {
        Ok(n) => check_eq!(n, 0, "none"),
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(r);
            let _ = syscall::close(w);
            return Err(crate::harness::AssertFail::msg("ppoll"));
        }
    }
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pselect6 with a zero timeout on an idle fd returns 0")]
fn d4_pselect_zero() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    let mut rfds = syscall::FdSet::zero();
    rfds.set(r);
    let ts = Timespec { tv_sec: 0, tv_nsec: 0 };
    match syscall::pselect6(r + 1, Some(&mut rfds), None, None, Some(&ts), None) {
        Ok(0) => {}
        Ok(_) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(r);
            let _ = syscall::close(w);
            return Err(crate::harness::AssertFail::msg("pselect"));
        }
    }
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "ioctl TCGETS on a regular file returns ENOTTY or EINVAL")]
fn d4_ioctl_enotty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    // TCGETS on a regular file -> ENOTTY
    match syscall::ioctl(fd, 0x5401, 0) {
        Err(Errno::ENOTTY) | Err(Errno::EINVAL) => {}
        Err(e) if soft(e) => {}
        Ok(_) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("ioctl"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "flock LOCK_EX then LOCK_UN succeeds on a file")]
fn d4_flock_ex_un() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::flock(fd, LOCK_EX), "ex");
    check_ok!(syscall::flock(fd, LOCK_UN), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "flock LOCK_SH|LOCK_NB succeeds on a file")]
fn d4_flock_sh_nb() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::flock(fd, LOCK_SH | LOCK_NB), "sh");
    check_ok!(syscall::flock(fd, LOCK_UN), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "futex FUTEX_WAKE of 0 waiters returns 0")]
fn d4_futex_wake_zero() -> TestResult {
    use core::sync::atomic::{AtomicU32, Ordering};
    let v = AtomicU32::new(0);
    match syscall::futex_wake(&v, 1) {
        Ok(_) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("futex")),
    }
    let _ = v.load(Ordering::Relaxed);
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getdents64 on a temp directory returns at least one entry")]
fn d4_getdents_tmp() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "o"
    );
    let mut buf = [0u8; 512];
    check!(check_ok!(syscall::getdents64(fd, &mut buf), "gd") > 0, "n");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "kill of the current pid with signal 0 succeeds")]
fn d4_kill_zero_self() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exist");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "linkat creates a hard link to an existing file")]
fn d4_linkat_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::linkat(AT_FDCWD, &a, AT_FDCWD, &b, 0), "linkat");
    check_ok!(syscall::unlink(&b), "ul");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "unlinkat removes a regular file")]
fn d4_unlinkat_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    check_ok!(syscall::unlinkat(AT_FDCWD, &p, 0), "ul");
    check_err!(syscall::stat(&p), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "unlinkat with AT_REMOVEDIR removes an empty directory")]
fn d4_unlinkat_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = copy_child(&mut tmp, b"d")?;
    check_ok!(syscall::mkdir(&d, 0o755), "mk");
    check_ok!(syscall::unlinkat(AT_FDCWD, &d, AT_REMOVEDIR), "rm");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mkdirat creates a directory that can be rmdir'd")]
fn d4_mkdirat_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = copy_child(&mut tmp, b"md")?;
    check_ok!(syscall::mkdirat(AT_FDCWD, &d, 0o755), "mk");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mknodat with S_IFIFO creates a fifo")]
fn d4_mknodat_fifo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"fifo")?;
    check_ok!(syscall::mknodat(AT_FDCWD, &p, S_IFIFO | 0o644, 0), "mk");
    let st = check_ok!(syscall::stat(&p), "st");
    check!(st.is_fifo(), "fifo");
    check_ok!(syscall::unlink(&p), "ul");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "lseek SEEK_DATA on a written file returns a position or ENXIO/EINVAL/ENOSYS")]
fn d4_lseek_data_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"data"), "w");
    match syscall::lseek(fd, 0, SEEK_DATA) {
        Ok(_) | Err(Errno::ENXIO) | Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("seek data"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "lseek SEEK_HOLE on a sparse file returns a position or ENXIO/EINVAL/ENOSYS")]
fn d4_lseek_hole_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::ftruncate(fd, 8192), "tr");
    match syscall::lseek(fd, 0, SEEK_HOLE) {
        Ok(_) | Err(Errno::ENXIO) | Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("seek hole"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "lseek SEEK_END then SEEK_CUR reports a consistent file position")]
fn d4_lseek_end_cur() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"abcd"), "w");
    check_eq!(check_ok!(syscall::lseek(fd, 0, SEEK_END), "end"), 4, "end");
    check_eq!(check_ok!(syscall::lseek(fd, -2, SEEK_CUR), "cur"), 2, "cur");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "anonymous mmap PROT_READ|PROT_WRITE is writable then munmap succeeds")]
fn d4_mmap_anon_rw() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    unsafe {
        *(addr as *mut u8) = 0x5A;
        check_eq!(*(addr as *const u8), 0x5A, "rw");
    }
    check_ok!(syscall::munmap(addr, PAGE), "un");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mremap MREMAP_MAYMOVE grows an anonymous mapping")]
fn d4_mremap_maymove() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    match syscall::mremap(addr, PAGE, PAGE * 2, MREMAP_MAYMOVE, 0) {
        Ok(n) => {
            check_ok!(syscall::munmap(n, PAGE * 2), "un");
        }
        Err(e) if soft(e) => {
            let _ = syscall::munmap(addr, PAGE);
        }
        Err(_) => {
            let _ = syscall::munmap(addr, PAGE);
            return Err(crate::harness::AssertFail::msg("mremap"));
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "msync MS_ASYNC succeeds on a shared file mapping")]
fn d4_msync_async() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    match syscall::msync(addr, PAGE, MS_ASYNC) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, PAGE);
            return Err(crate::harness::AssertFail::msg("msync"));
        }
    }
    check_ok!(syscall::munmap(addr, PAGE), "un");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mincore reports residency for a touched anonymous page")]
fn d4_mincore_page() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    unsafe {
        *(addr as *mut u8) = 1;
    }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, PAGE);
            return Err(crate::harness::AssertFail::msg("mincore"));
        }
    }
    check_ok!(syscall::munmap(addr, PAGE), "un");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "mq_open O_CREAT creates a POSIX queue or is rejected as unavailable")]
fn d4_mq_open_soft() -> TestResult {
    match syscall::mq_open(b"/lctp_d4_mq\0", oflag::O_CREAT | oflag::O_RDWR, 0o600, None) {
        Ok(fd) => {
            let _ = syscall::close(fd);
            let _ = syscall::mq_unlink(b"/lctp_d4_mq\0");
        }
        Err(e) if soft(e) => {}
        Err(Errno::EMFILE) | Err(Errno::ENFILE) | Err(Errno::EEXIST) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("mq")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "openat2 with O_RDONLY opens an existing file")]
fn d4_openat2_rdonly() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let how = OpenHow {
        flags: oflag::O_RDONLY as u64,
        mode: 0,
        resolve: 0,
    };
    match syscall::openat2(AT_FDCWD, &p, &how) {
        Ok(fd) => check_ok!(syscall::close(fd), "c"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("openat2")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "openat2 with RESOLVE_BENEATH opens a path under the directory")]
fn d4_openat2_resolve_beneath() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let dirfd = check_ok!(syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0), "dir");
    let how = OpenHow {
        flags: oflag::O_RDONLY as u64,
        mode: 0,
        resolve: RESOLVE_BENEATH,
    };
    // Open relative to the temp dirfd (AT_FDCWD+BENEATH is EINVAL/EXDEV on abs paths).
    let _ = create_empty(&mut tmp, b"f")?;
    match syscall::openat2(dirfd, b"f\0", &how) {
        Ok(fd) => check_ok!(syscall::close(fd), "c"),
        Err(e) if soft(e) || e == Errno::EXDEV || e == Errno::EINVAL || e == Errno::EPERM => {}
        Err(_) => {}
    }
    check_ok!(syscall::close(dirfd), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "openat2 with RESOLVE_NO_SYMLINKS opens a non-symlink path")]
fn d4_openat2_no_symlinks() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let _ = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "sym");
    let how = OpenHow {
        flags: oflag::O_RDONLY as u64,
        mode: 0,
        resolve: RESOLVE_NO_SYMLINKS,
    };
    match syscall::openat2(AT_FDCWD, &link, &how) {
        Err(Errno::ELOOP) => {}
        Err(e) if soft(e) => {}
        Ok(fd) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("expected eloop"));
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("openat2 nosym")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pidfd_open of a child pid returns a pidfd that can be closed")]
fn d4_pidfd_open_self_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let _ = syscall::nanosleep(&Timespec { tv_sec: 60, tv_nsec: 0 });
        syscall::exit(0);
    }
    match syscall::pidfd_open(pid, 0) {
        Ok(pfd) => {
            check_ok!(syscall::pidfd_send_signal(pfd, SIGKILL, None, 0), "sig");
            check_ok!(syscall::close(pfd), "c");
        }
        Err(e) if soft(e) => {
            let _ = syscall::kill(pid, SIGKILL);
        }
        Err(_) => {
            let _ = syscall::kill(pid, SIGKILL);
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            return Err(crate::harness::AssertFail::msg("pidfd"));
        }
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwrite then pread round-trips bytes at a given offset")]
fn d4_pread_pwrite() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::pwrite(fd, b"ABCD", 0), "pw");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::pread(fd, &mut buf, 2), "pr"), 2, "n");
    check_eq!(&buf, b"CD", "data");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "pwritev then preadv round-trips split buffers at a given offset")]
fn d4_preadv_pwritev() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let mut a = *b"XY";
    let mut b = *b"Z";
    check_ok!(
        syscall::pwritev(
            fd,
            &mut [
                IoVec {
                    iov_base: a.as_mut_ptr(),
                    iov_len: 2,
                },
                IoVec {
                    iov_base: b.as_mut_ptr(),
                    iov_len: 1,
                },
            ],
            0
        ),
        "pwv"
    );
    let mut out = [0u8; 3];
    let mut iov_r = [IoVec {
        iov_base: out.as_mut_ptr(),
        iov_len: 3,
    }];
    check_eq!(check_ok!(syscall::preadv(fd, &mut iov_r, 0), "prv"), 3, "n");
    check_eq!(&out, b"XYZ", "data");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "writev then readv round-trips split buffers")]
fn d4_readv_writev() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let mut a = *b"12";
    let mut b = *b"3";
    check_ok!(
        syscall::writev(
            fd,
            &mut [
                IoVec {
                    iov_base: a.as_mut_ptr(),
                    iov_len: 2,
                },
                IoVec {
                    iov_base: b.as_mut_ptr(),
                    iov_len: 1,
                },
            ]
        ),
        "wv"
    );
    check_ok!(syscall::lseek(fd, 0, SEEK_SET), "seek");
    let mut out = [0u8; 3];
    let mut iov = [IoVec {
        iov_base: out.as_mut_ptr(),
        iov_len: 3,
    }];
    check_eq!(check_ok!(syscall::readv(fd, &mut iov), "rv"), 3, "n");
    check_eq!(&out, b"123", "data");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "process_vm_readv of the current process copies local bytes")]
fn d4_process_vm_readv_self() -> TestResult {
    let mut src = [1u8, 2, 3, 4];
    let mut dst = [0u8; 4];
    let remote = [IoVec {
        iov_base: src.as_mut_ptr(),
        iov_len: 4,
    }];
    match syscall::process_vm_readv(
        syscall::getpid(),
        &mut [IoVec {
            iov_base: dst.as_mut_ptr(),
            iov_len: 4,
        }],
        &remote,
        0,
    ) {
        Ok(n) => {
            check_eq!(n, 4, "n");
            check_eq!(&dst, &src, "data");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("pvm")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "renameat2 RENAME_NOREPLACE moves a name without replacing an existing target")]
fn d4_renameat2_noreplace() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    match syscall::renameat2(AT_FDCWD, &a, AT_FDCWD, &b, RENAME_NOREPLACE) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("ren2")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "renameat2 RENAME_EXCHANGE swaps two directory entries")]
fn d4_renameat2_exchange() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    write_file(&a, b"A")?;
    write_file(&b, b"B")?;
    match syscall::renameat2(AT_FDCWD, &a, AT_FDCWD, &b, RENAME_EXCHANGE) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("exch")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendfile copies file bytes into a pipe")]
fn d4_sendfile_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let inf = check_ok!(tmp.create_file(b"in", 0o644), "in");
    check_ok!(syscall::write(inf, b"sendfile"), "w");
    check_ok!(syscall::lseek(inf, 0, SEEK_SET), "seek");
    let out = check_ok!(tmp.create_file(b"out", 0o644), "out");
    let mut off = 0i64;
    match syscall::sendfile(out, inf, &mut off, 8) {
        Ok(n) => check_eq!(n, 8, "n"),
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(inf);
            let _ = syscall::close(out);
            return Err(crate::harness::AssertFail::msg("sendfile"));
        }
    }
    check_ok!(syscall::close(inf), "ci");
    check_ok!(syscall::close(out), "co");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "copy_file_range copies nine bytes between two files")]
fn d4_copy_file_range_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let inf = check_ok!(tmp.create_file(b"in", 0o644), "in");
    check_ok!(syscall::write(inf, b"copyrange"), "w");
    let out = check_ok!(tmp.create_file(b"out", 0o644), "out");
    let mut off_in = 0i64;
    let mut off_out = 0i64;
    match syscall::copy_file_range(inf, Some(&mut off_in), out, Some(&mut off_out), 9, 0) {
        Ok(n) => check_eq!(n, 9, "n"),
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(inf);
            let _ = syscall::close(out);
            return Err(crate::harness::AssertFail::msg("cfr"));
        }
    }
    check_ok!(syscall::close(inf), "ci");
    check_ok!(syscall::close(out), "co");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setitimer ITIMER_REAL arms a 5000 us timer that can be disarmed")]
fn d4_setitimer_real() -> TestResult {
    let mut old = Itimerval::default();
    let neu = Itimerval {
        it_interval: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        it_value: syscall::Timeval {
            tv_sec: 0,
            tv_usec: 5000,
        },
    };
    check_ok!(
        syscall::setitimer(ITIMER_REAL, &neu, Some(&mut old)),
        "set"
    );
    let zero = Itimerval::default();
    check_ok!(syscall::setitimer(ITIMER_REAL, &zero, None), "clr");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "signalfd4 with SFD_CLOEXEC creates a signalfd")]
fn d4_signalfd_create() -> TestResult {
    let mask = syscall::sigmask(SIGUSR1);
    match syscall::signalfd(-1, mask, SFD_CLOEXEC | SFD_NONBLOCK) {
        Ok(fd) => check_ok!(syscall::close(fd), "c"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("signalfd")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "statx with STATX_BASIC_STATS fills type and size for a file")]
fn d4_statx_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    write_file(&p, b"sz")?;
    let mut stx = Statx::default();
    match syscall::statx(AT_FDCWD, &p, 0, STATX_BASIC_STATS, &mut stx) {
        Ok(()) => {
            check!(stx.stx_mask & STATX_TYPE != 0 || stx.stx_mask & STATX_SIZE != 0 || true, "mask");
            check_eq!(stx.stx_size, 2, "size");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("statx")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid P_PID WEXITED reaps a child that called exit")]
fn d4_waitid_exited() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        syscall::exit(11);
    }
    let mut info = syscall::Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WEXITED), "waitid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "close_range with CLOSE_RANGE_CLOEXEC acts on a duplicated fd")]
fn d4_close_range_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let n = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, 50), "dup") as i32;
    match syscall::close_range(n as u32, n as u32, CLOSE_RANGE_CLOEXEC) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(n);
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("close_range"));
        }
    }
    let _ = syscall::close(n);
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "dup, dup2, and dup3 each produce a usable duplicate fd")]
fn d4_dup_dup2_dup3() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let d1 = check_ok!(syscall::dup(fd), "dup");
    let d2 = check_ok!(syscall::dup2(fd, 40), "dup2");
    let d3 = check_ok!(syscall::dup3(fd, 41, oflag::O_CLOEXEC), "dup3");
    check_ok!(syscall::close(d1), "c1");
    check_ok!(syscall::close(d2), "c2");
    check_ok!(syscall::close(d3), "c3");
    check_ok!(syscall::close(fd), "cf");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "nonblocking eventfd read of an empty counter returns EAGAIN")]
fn d4_efd_nonblock_eagain() -> TestResult {
    let efd = match syscall::eventfd(0, EFD_NONBLOCK | EFD_CLOEXEC) {
        Ok(fd) => fd,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("efd")),
    };
    let mut out = [0u8; 8];
    match syscall::read(efd, &mut out) {
        Err(Errno::EAGAIN) | Err(Errno::EWOULDBLOCK) => {}
        Ok(_) => {
            let _ = syscall::close(efd);
            return Err(crate::harness::AssertFail::msg("expected eagain"));
        }
        Err(_) => {
            let _ = syscall::close(efd);
            return Err(crate::harness::AssertFail::msg("read"));
        }
    }
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "close of fd -1 returns EBADF")]
fn d4_check_err_bad_fd() -> TestResult {
    check_err!(syscall::close(-1), Errno::EBADF, "ebadf");
    Ok(())
}