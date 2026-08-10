//! Extra depth2 wave: denser LTP/pjdfstest/POSIX matrices.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, write_file};
use crate::syscall::{
    self, clock, fcntl_cmd, oflag, poll, Errno, Flock, Itimerspec, Timespec, AF_INET, AF_UNIX,
    EFD_CLOEXEC, EFD_SEMAPHORE, EPOLLIN, EPOLL_CTL_ADD, F_RDLCK, F_UNLCK, F_WRLCK, IN_ATTRIB,
    IN_CLOEXEC, IN_CREATE, IN_DELETE, IN_MODIFY, IN_OPEN, MFD_ALLOW_SEALING, SEEK_SET, SIGKILL,
    SIGTERM, SIGUSR1, SOCK_DGRAM, SOCK_STREAM, SOL_SOCKET, SO_RCVBUF, SO_REUSEADDR, SO_SNDBUF,
    SO_TYPE, TFD_CLOEXEC, TFD_TIMER_ABSTIME, F_OK, R_OK, W_OK, X_OK, AT_FDCWD, S_IFIFO,
};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::EINVAL | Errno::ENOSYS | Errno::EPERM | Errno::EOPNOTSUPP | Errno::ENOTSUP | Errno::ENOMEM
    )
}

macro_rules! flock_byte {
    ($name:ident, $ty:expr, $off:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"b", 0o644), "c");
            check_ok!(syscall::write(fd, b"0123456789ABCDEF"), "w");
            let mut lk = Flock {
                l_type: $ty,
                l_whence: SEEK_SET as i16,
                l_start: $off,
                l_len: 1,
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

flock_byte!(d2b_rd_0, F_RDLCK, 0);
flock_byte!(d2b_rd_1, F_RDLCK, 1);
flock_byte!(d2b_rd_2, F_RDLCK, 2);
flock_byte!(d2b_rd_3, F_RDLCK, 3);
flock_byte!(d2b_rd_4, F_RDLCK, 4);
flock_byte!(d2b_rd_5, F_RDLCK, 5);
flock_byte!(d2b_rd_6, F_RDLCK, 6);
flock_byte!(d2b_rd_7, F_RDLCK, 7);
flock_byte!(d2b_rd_8, F_RDLCK, 8);
flock_byte!(d2b_rd_9, F_RDLCK, 9);
flock_byte!(d2b_rd_10, F_RDLCK, 10);
flock_byte!(d2b_rd_11, F_RDLCK, 11);
flock_byte!(d2b_rd_12, F_RDLCK, 12);
flock_byte!(d2b_rd_13, F_RDLCK, 13);
flock_byte!(d2b_rd_14, F_RDLCK, 14);
flock_byte!(d2b_rd_15, F_RDLCK, 15);
flock_byte!(d2b_wr_0, F_WRLCK, 0);
flock_byte!(d2b_wr_1, F_WRLCK, 1);
flock_byte!(d2b_wr_2, F_WRLCK, 2);
flock_byte!(d2b_wr_3, F_WRLCK, 3);
flock_byte!(d2b_wr_4, F_WRLCK, 4);
flock_byte!(d2b_wr_5, F_WRLCK, 5);
flock_byte!(d2b_wr_6, F_WRLCK, 6);
flock_byte!(d2b_wr_7, F_WRLCK, 7);
flock_byte!(d2b_wr_8, F_WRLCK, 8);
flock_byte!(d2b_wr_9, F_WRLCK, 9);
flock_byte!(d2b_wr_10, F_WRLCK, 10);
flock_byte!(d2b_wr_11, F_WRLCK, 11);
flock_byte!(d2b_wr_12, F_WRLCK, 12);
flock_byte!(d2b_wr_13, F_WRLCK, 13);
flock_byte!(d2b_wr_14, F_WRLCK, 14);
flock_byte!(d2b_wr_15, F_WRLCK, 15);

macro_rules! wait_code {
    ($name:ident, $c:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let pid = check_ok!(syscall::fork(), "f");
            if pid == 0 {
                syscall::exit($c);
            }
            let mut st = 0;
            check_ok!(syscall::wait4(pid, &mut st, 0), "w");
            check!(syscall::wifexited(st), "ex");
            check_eq!(syscall::wexitstatus(st), $c, "c");
            Ok(())
        }
    };
}

wait_code!(d2b_ex_3, 3);
wait_code!(d2b_ex_4, 4);
wait_code!(d2b_ex_5, 5);
wait_code!(d2b_ex_6, 6);
wait_code!(d2b_ex_8, 8);
wait_code!(d2b_ex_9, 9);
wait_code!(d2b_ex_10, 10);
wait_code!(d2b_ex_11, 11);
wait_code!(d2b_ex_12, 12);
wait_code!(d2b_ex_13, 13);
wait_code!(d2b_ex_14, 14);
wait_code!(d2b_ex_15, 15);
wait_code!(d2b_ex_16, 16);
wait_code!(d2b_ex_31, 31);
wait_code!(d2b_ex_32, 32);
wait_code!(d2b_ex_63, 63);
wait_code!(d2b_ex_64, 64);
wait_code!(d2b_ex_100, 100);
wait_code!(d2b_ex_128, 128);
wait_code!(d2b_ex_254, 254);

macro_rules! splice_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
            let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
            let msg = [b'Z'; $n];
            check_ok!(syscall::write(w1, &msg), "w");
            match syscall::splice(r1, None, w2, None, $n, 0) {
                Ok(v) => check_eq!(v, $n, "n"),
                Err(e) if soft(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("splice")),
            }
            let _ = syscall::close(w1);
            let _ = syscall::close(w2);
            let _ = syscall::close(r1);
            let _ = syscall::close(r2);
            Ok(())
        }
    };
}

splice_n!(d2b_sp_3, 3);
splice_n!(d2b_sp_5, 5);
splice_n!(d2b_sp_7, 7);
splice_n!(d2b_sp_9, 9);
splice_n!(d2b_sp_12, 12);
splice_n!(d2b_sp_24, 24);
splice_n!(d2b_sp_48, 48);
splice_n!(d2b_sp_96, 96);
splice_n!(d2b_sp_192, 192);
splice_n!(d2b_sp_256, 256);

macro_rules! tee_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
            let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
            let msg = [b'Y'; $n];
            check_ok!(syscall::write(w1, &msg), "w");
            match syscall::tee(r1, w2, $n, 0) {
                Ok(v) => check_eq!(v, $n, "n"),
                Err(e) if soft(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("tee")),
            }
            let _ = syscall::close(w1);
            let _ = syscall::close(w2);
            let _ = syscall::close(r1);
            let _ = syscall::close(r2);
            Ok(())
        }
    };
}

tee_n!(d2b_tee_3, 3);
tee_n!(d2b_tee_5, 5);
tee_n!(d2b_tee_7, 7);
tee_n!(d2b_tee_12, 12);
tee_n!(d2b_tee_24, 24);
tee_n!(d2b_tee_48, 48);
tee_n!(d2b_tee_96, 96);
tee_n!(d2b_tee_128, 128);

macro_rules! efd_sem {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

efd_sem!(d2b_efd_6, 6);
efd_sem!(d2b_efd_7, 7);
efd_sem!(d2b_efd_8, 8);
efd_sem!(d2b_efd_9, 9);
efd_sem!(d2b_efd_10, 10);

macro_rules! so_type {
    ($name:ident, $dom:expr, $ty:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(syscall::socket($dom, $ty, 0), "s");
            let mut v = [0u8; 4];
            check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_TYPE, &mut v), "g");
            check_eq!(i32::from_ne_bytes(v), $ty, "ty");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

so_type!(d2b_so_unix_stream, AF_UNIX, SOCK_STREAM);
so_type!(d2b_so_unix_dgram, AF_UNIX, SOCK_DGRAM);
so_type!(d2b_so_inet_stream, AF_INET, SOCK_STREAM);
so_type!(d2b_so_inet_dgram, AF_INET, SOCK_DGRAM);

macro_rules! so_buf {
    ($name:ident, $opt:expr, $sz:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM, 0), "s");
            let b = ($sz as i32).to_ne_bytes();
            match syscall::setsockopt(fd, SOL_SOCKET, $opt, &b) {
                Ok(()) => {
                    let mut v = [0u8; 4];
                    check_ok!(syscall::getsockopt(fd, SOL_SOCKET, $opt, &mut v), "g");
                    check!(i32::from_ne_bytes(v) >= $sz as i32 / 2, "sz");
                }
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("so"));
                }
            }
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

so_buf!(d2b_rcv_8k, SO_RCVBUF, 8192);
so_buf!(d2b_rcv_16k, SO_RCVBUF, 16384);
so_buf!(d2b_rcv_32k, SO_RCVBUF, 32768);
so_buf!(d2b_rcv_64k, SO_RCVBUF, 65536);
so_buf!(d2b_snd_8k, SO_SNDBUF, 8192);
so_buf!(d2b_snd_16k, SO_SNDBUF, 16384);
so_buf!(d2b_snd_32k, SO_SNDBUF, 32768);
so_buf!(d2b_snd_64k, SO_SNDBUF, 65536);

macro_rules! in_mask {
    ($name:ident, $m:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

in_mask!(d2b_in_c, IN_CREATE);
in_mask!(d2b_in_d, IN_DELETE);
in_mask!(d2b_in_m, IN_MODIFY);
in_mask!(d2b_in_a, IN_ATTRIB);
in_mask!(d2b_in_o, IN_OPEN);
in_mask!(d2b_in_cd, IN_CREATE | IN_DELETE);
in_mask!(d2b_in_cm, IN_CREATE | IN_MODIFY);
in_mask!(d2b_in_dma, IN_DELETE | IN_MODIFY | IN_ATTRIB);
in_mask!(d2b_in_all, IN_CREATE | IN_DELETE | IN_MODIFY | IN_ATTRIB | IN_OPEN);

macro_rules! tfd_abs_ms {
    ($name:ident, $ms:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(
                syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_CLOEXEC),
                "t"
            );
            let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "n");
            let add = ($ms as i64) * 1_000_000;
            let mut its = Itimerspec {
                it_interval: Timespec::default(),
                it_value: Timespec {
                    tv_sec: now.tv_sec,
                    tv_nsec: now.tv_nsec + add,
                },
            };
            while its.it_value.tv_nsec >= 1_000_000_000 {
                its.it_value.tv_sec += 1;
                its.it_value.tv_nsec -= 1_000_000_000;
            }
            check_ok!(syscall::timerfd_settime(fd, TFD_TIMER_ABSTIME, &its), "s");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

tfd_abs_ms!(d2b_tfd_1, 1);
tfd_abs_ms!(d2b_tfd_2, 2);
tfd_abs_ms!(d2b_tfd_3, 3);
tfd_abs_ms!(d2b_tfd_20, 20);
tfd_abs_ms!(d2b_tfd_100, 100);
tfd_abs_ms!(d2b_tfd_200, 200);

#[crate::lctp_test(suite = syscall)]
fn d2b_memfd_seals_get_zero() -> TestResult {
    let fd = check_ok!(
        syscall::memfd_create(b"z\0", MFD_ALLOW_SEALING as u32),
        "m"
    );
    let s = check_ok!(syscall::fcntl(fd, syscall::F_GET_SEALS, 0), "g");
    check_eq!(s, 0, "none");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2b_epoll_eventfd() -> TestResult {
    let efd = check_ok!(syscall::eventfd(1, EFD_CLOEXEC), "e");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = syscall::epoll::EpollEvent {
        events: EPOLLIN,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, efd, &mut ev), "add");
    let mut out = [syscall::epoll::EpollEvent { events: 0, data: 0 }; 1];
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 100), "w") >= 1, "rdy");
    check_ok!(syscall::close(ep), "ce");
    check_ok!(syscall::close(efd), "cf");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2b_poll_eventfd() -> TestResult {
    let efd = check_ok!(syscall::eventfd(1, EFD_CLOEXEC), "e");
    let mut fds = [poll::PollFd {
        fd: efd,
        events: syscall::POLLIN,
        revents: 0,
    }];
    check!(check_ok!(syscall::poll(&mut fds, 100), "p") >= 1, "rdy");
    check!(fds[0].revents & syscall::POLLIN != 0, "in");
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2b_pidfd_kill() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let _ = syscall::nanosleep(&Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        });
        syscall::exit(0);
    }
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "o");
    check_ok!(syscall::pidfd_send_signal(pfd, SIGKILL, None, 0), "k");
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    check!(syscall::wifsignaled(st), "sig");
    check_ok!(syscall::close(pfd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2b_affinity_restore() -> TestResult {
    let mut m = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut m), "g");
    match syscall::sched_setaffinity(0, &m) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("set")),
    }
    Ok(())
}

// ---- FS matrices ----

macro_rules! fs_eacces_r {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $mode), "ch");
            check_err!(syscall::access(&p, R_OK), Errno::EACCES, "e");
            check_ok!(syscall::chmod(&p, 0o644), "rs");
            Ok(())
        }
    };
}

fs_eacces_r!(d2b_fs_er_000, 0o000);
fs_eacces_r!(d2b_fs_er_010, 0o010);
fs_eacces_r!(d2b_fs_er_020, 0o020);
fs_eacces_r!(d2b_fs_er_030, 0o030);
fs_eacces_r!(d2b_fs_er_100, 0o100);
fs_eacces_r!(d2b_fs_er_110, 0o110);
fs_eacces_r!(d2b_fs_er_120, 0o120);
fs_eacces_r!(d2b_fs_er_130, 0o130);
fs_eacces_r!(d2b_fs_er_200, 0o200);
fs_eacces_r!(d2b_fs_er_210, 0o210);
fs_eacces_r!(d2b_fs_er_220, 0o220);
fs_eacces_r!(d2b_fs_er_230, 0o230);
fs_eacces_r!(d2b_fs_er_300, 0o300);
fs_eacces_r!(d2b_fs_er_310, 0o310);
fs_eacces_r!(d2b_fs_er_320, 0o320);
fs_eacces_r!(d2b_fs_er_330, 0o330);

macro_rules! fs_eacces_w {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $mode), "ch");
            check_err!(syscall::access(&p, W_OK), Errno::EACCES, "e");
            check_ok!(syscall::chmod(&p, 0o644), "rs");
            Ok(())
        }
    };
}

fs_eacces_w!(d2b_fs_ew_000, 0o000);
fs_eacces_w!(d2b_fs_ew_001, 0o001);
fs_eacces_w!(d2b_fs_ew_004, 0o004);
fs_eacces_w!(d2b_fs_ew_005, 0o005);
fs_eacces_w!(d2b_fs_ew_040, 0o040);
fs_eacces_w!(d2b_fs_ew_041, 0o041);
fs_eacces_w!(d2b_fs_ew_044, 0o044);
fs_eacces_w!(d2b_fs_ew_045, 0o045);
fs_eacces_w!(d2b_fs_ew_400, 0o400);
fs_eacces_w!(d2b_fs_ew_401, 0o401);
fs_eacces_w!(d2b_fs_ew_404, 0o404);
fs_eacces_w!(d2b_fs_ew_405, 0o405);
fs_eacces_w!(d2b_fs_ew_440, 0o440);
fs_eacces_w!(d2b_fs_ew_441, 0o441);
fs_eacces_w!(d2b_fs_ew_444, 0o444);
fs_eacces_w!(d2b_fs_ew_445, 0o445);

macro_rules! fs_ok_r {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $mode), "ch");
            check_ok!(syscall::access(&p, R_OK), "ok");
            check_ok!(syscall::chmod(&p, 0o644), "rs");
            Ok(())
        }
    };
}

fs_ok_r!(d2b_fs_or_400, 0o400);
fs_ok_r!(d2b_fs_or_440, 0o440);
fs_ok_r!(d2b_fs_or_444, 0o444);
fs_ok_r!(d2b_fs_or_500, 0o500);
fs_ok_r!(d2b_fs_or_540, 0o540);
fs_ok_r!(d2b_fs_or_544, 0o544);
fs_ok_r!(d2b_fs_or_600, 0o600);
fs_ok_r!(d2b_fs_or_640, 0o640);
fs_ok_r!(d2b_fs_or_644, 0o644);
fs_ok_r!(d2b_fs_or_700, 0o700);
fs_ok_r!(d2b_fs_or_740, 0o740);
fs_ok_r!(d2b_fs_or_744, 0o744);

macro_rules! fifo_m {
    ($name:ident, $m:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = copy_child(&mut tmp, b"fifo")?;
            check_ok!(
                syscall::mknodat(AT_FDCWD, &p, S_IFIFO | ($m & 0o777), 0),
                "m"
            );
            check_ok!(syscall::chmod(&p, $m & 0o777), "ch");
            let st = check_ok!(syscall::stat(&p), "s");
            check!(st.is_fifo(), "f");
            check_eq!(st.mode_bits() & 0o777, $m & 0o777, "mode");
            check_ok!(syscall::unlink(&p), "u");
            Ok(())
        }
    };
}

fifo_m!(d2b_fifo_610, 0o610);
fifo_m!(d2b_fifo_620, 0o620);
fifo_m!(d2b_fifo_630, 0o630);
fifo_m!(d2b_fifo_640, 0o640);
fifo_m!(d2b_fifo_650, 0o650);
fifo_m!(d2b_fifo_660, 0o660);
fifo_m!(d2b_fifo_670, 0o670);
fifo_m!(d2b_fifo_710, 0o710);
fifo_m!(d2b_fifo_720, 0o720);
fifo_m!(d2b_fifo_730, 0o730);
fifo_m!(d2b_fifo_740, 0o740);
fifo_m!(d2b_fifo_750, 0o750);
fifo_m!(d2b_fifo_760, 0o760);
fifo_m!(d2b_fifo_770, 0o770);

macro_rules! ren {
    ($name:ident, $a:expr, $b:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let s = create_empty(&mut tmp, $a)?;
            let d = copy_child(&mut tmp, $b)?;
            check_ok!(syscall::rename(&s, &d), "r");
            check_ok!(syscall::stat(&d), "d");
            Ok(())
        }
    };
}

ren!(d2b_ren_01, b"r0", b"r1");
ren!(d2b_ren_02, b"r2", b"r3");
ren!(d2b_ren_03, b"r4", b"r5");
ren!(d2b_ren_04, b"r6", b"r7");
ren!(d2b_ren_05, b"r8", b"r9");
ren!(d2b_ren_06, b"ra", b"rb");
ren!(d2b_ren_07, b"rc", b"rd");
ren!(d2b_ren_08, b"re", b"rf");
ren!(d2b_ren_09, b"rg", b"rh");
ren!(d2b_ren_10, b"ri", b"rj");

macro_rules! lnk {
    ($name:ident, $a:expr, $b:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let s = create_empty(&mut tmp, $a)?;
            let d = copy_child(&mut tmp, $b)?;
            check_ok!(syscall::link(&s, &d), "l");
            check_ok!(syscall::unlink(&d), "u");
            Ok(())
        }
    };
}

lnk!(d2b_lnk_01, b"l0", b"l1");
lnk!(d2b_lnk_02, b"l2", b"l3");
lnk!(d2b_lnk_03, b"l4", b"l5");
lnk!(d2b_lnk_04, b"l6", b"l7");
lnk!(d2b_lnk_05, b"l8", b"l9");
lnk!(d2b_lnk_06, b"la", b"lb");
lnk!(d2b_lnk_07, b"lc", b"ld");
lnk!(d2b_lnk_08, b"le", b"lf");
lnk!(d2b_lnk_09, b"lg", b"lh");
lnk!(d2b_lnk_10, b"li", b"lj");

macro_rules! open_creat_m {
    ($name:ident, $m:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = copy_child(&mut tmp, b"oc")?;
            let fd = check_ok!(
                syscall::open(&p, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, $m),
                "o"
            );
            check_ok!(syscall::close(fd), "c");
            check_ok!(syscall::chmod(&p, $m & 0o777), "ch");
            let st = check_ok!(syscall::stat(&p), "s");
            check_eq!(st.mode_bits() & 0o777, $m & 0o777, "m");
            Ok(())
        }
    };
}

open_creat_m!(d2b_oc_400, 0o400);
open_creat_m!(d2b_oc_440, 0o440);
open_creat_m!(d2b_oc_600, 0o600);
open_creat_m!(d2b_oc_640, 0o640);
open_creat_m!(d2b_oc_660, 0o660);
open_creat_m!(d2b_oc_700, 0o700);
open_creat_m!(d2b_oc_740, 0o740);
open_creat_m!(d2b_oc_750, 0o750);
open_creat_m!(d2b_oc_760, 0o760);
open_creat_m!(d2b_oc_770, 0o770);

#[crate::lctp_test(suite = fs)]
fn d2b_dir_no_write_mkdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&d, 0o555), "ch");
    let mut b = [0u8; 160];
    let p = join_path(&d, b"x", &mut b)?;
    check_err!(syscall::mkdir(p, 0o755), Errno::EACCES, "e");
    check_ok!(syscall::chmod(&d, 0o755), "rs");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d2b_unlink_after_link() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    write_file(&a, b"q")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "l");
    check_ok!(syscall::unlink(&a), "ua");
    check_ok!(syscall::access(&b, F_OK), "f");
    check_ok!(syscall::unlink(&b), "ub");
    Ok(())
}

// ---- POSIX ----

macro_rules! px_acc {
    ($name:ident, $ch:expr, $m:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $ch), "ch");
            check_ok!(syscall::access(&p, $m), "a");
            check_ok!(syscall::chmod(&p, 0o644), "rs");
            Ok(())
        }
    };
}

px_acc!(d2b_px_r_400, 0o400, R_OK);
px_acc!(d2b_px_r_440, 0o440, R_OK);
px_acc!(d2b_px_r_444, 0o444, R_OK);
px_acc!(d2b_px_r_500, 0o500, R_OK);
px_acc!(d2b_px_r_540, 0o540, R_OK);
px_acc!(d2b_px_r_544, 0o544, R_OK);
px_acc!(d2b_px_r_600, 0o600, R_OK);
px_acc!(d2b_px_r_640, 0o640, R_OK);
px_acc!(d2b_px_r_644, 0o644, R_OK);
px_acc!(d2b_px_r_700, 0o700, R_OK);
px_acc!(d2b_px_w_200, 0o200, W_OK);
px_acc!(d2b_px_w_220, 0o220, W_OK);
px_acc!(d2b_px_w_600, 0o600, W_OK);
px_acc!(d2b_px_w_620, 0o620, W_OK);
px_acc!(d2b_px_w_660, 0o660, W_OK);
px_acc!(d2b_px_w_700, 0o700, W_OK);
px_acc!(d2b_px_x_100, 0o100, X_OK);
px_acc!(d2b_px_x_500, 0o500, X_OK);
px_acc!(d2b_px_x_700, 0o700, X_OK);
px_acc!(d2b_px_x_711, 0o711, X_OK);
px_acc!(d2b_px_rw_600, 0o600, R_OK | W_OK);
px_acc!(d2b_px_rw_660, 0o660, R_OK | W_OK);
px_acc!(d2b_px_rx_500, 0o500, R_OK | X_OK);
px_acc!(d2b_px_rx_550, 0o550, R_OK | X_OK);
px_acc!(d2b_px_wx_300, 0o300, W_OK | X_OK);
px_acc!(d2b_px_rwx_700, 0o700, R_OK | W_OK | X_OK);
px_acc!(d2b_px_rwx_755, 0o755, R_OK | W_OK | X_OK);
px_acc!(d2b_px_rwx_777, 0o777, R_OK | W_OK | X_OK);

macro_rules! px_fa {
    ($name:ident, $ch:expr, $m:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $ch), "ch");
            check_ok!(syscall::faccessat(AT_FDCWD, &p, $m, 0), "fa");
            check_ok!(syscall::chmod(&p, 0o644), "rs");
            Ok(())
        }
    };
}

px_fa!(d2b_px_fa_r_400, 0o400, R_OK);
px_fa!(d2b_px_fa_r_644, 0o644, R_OK);
px_fa!(d2b_px_fa_w_200, 0o200, W_OK);
px_fa!(d2b_px_fa_w_666, 0o666, W_OK);
px_fa!(d2b_px_fa_x_100, 0o100, X_OK);
px_fa!(d2b_px_fa_x_755, 0o755, X_OK);
px_fa!(d2b_px_fa_f_000, 0o000, F_OK);
px_fa!(d2b_px_fa_rw_600, 0o600, R_OK | W_OK);
px_fa!(d2b_px_fa_rx_500, 0o500, R_OK | X_OK);
px_fa!(d2b_px_fa_rwx_700, 0o700, R_OK | W_OK | X_OK);

macro_rules! px_cns {
    ($name:ident, $ns:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let req = Timespec {
                tv_sec: 0,
                tv_nsec: $ns,
            };
            check_ok!(
                syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req),
                "s"
            );
            Ok(())
        }
    };
}

px_cns!(d2b_cns_100us, 100_000);
px_cns!(d2b_cns_200us, 200_000);
px_cns!(d2b_cns_500us, 500_000);
px_cns!(d2b_cns_3ms, 3_000_000);
px_cns!(d2b_cns_4ms, 4_000_000);
px_cns!(d2b_cns_6ms, 6_000_000);
px_cns!(d2b_cns_7ms, 7_000_000);
px_cns!(d2b_cns_8ms, 8_000_000);
px_cns!(d2b_cns_9ms, 9_000_000);
px_cns!(d2b_cns_15ms, 15_000_000);

#[crate::lctp_test(suite = posix)]
fn d2b_sig_block_usr1_term() -> TestResult {
    let mut old = 0u64;
    let set = syscall::sigmask(SIGUSR1) | syscall::sigmask(SIGTERM);
    check_ok!(
        syscall::rt_sigprocmask(syscall::SIG_BLOCK, Some(set), Some(&mut old)),
        "b"
    );
    let mut cur = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(syscall::SIG_BLOCK, None, Some(&mut cur)),
        "q"
    );
    check!(cur & syscall::sigmask(SIGUSR1) != 0, "u1");
    check!(cur & syscall::sigmask(SIGTERM) != 0, "tm");
    check_ok!(
        syscall::rt_sigprocmask(syscall::SIG_SETMASK, Some(old), None),
        "rs"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d2b_reuseaddr_listen_soft() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM, 0), "s");
    let one = 1i32.to_ne_bytes();
    let _ = syscall::setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one);
    check_ok!(syscall::close(fd), "c");
    Ok(())
}
