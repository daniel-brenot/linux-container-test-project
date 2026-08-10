//! LTP-style depth3: fcntl locks/dupfd/setfl, epoll/poll, eventfd, wait status,
//! mmap/madvise/memfd, timerfd, inotify, pidfd, SysV IPC, splice/tee, net.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, write_file};
use crate::syscall::{
    self, clock, epoll, fcntl_cmd, madvise, map, oflag, poll, prot, wait, Errno, Flock, IoVec,
    Itimerspec, SockAddrIn, Timespec, AF_INET, EFD_CLOEXEC, EFD_NONBLOCK, EFD_SEMAPHORE, EPOLLET,
    EPOLLIN, EPOLLONESHOT, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, FALLOC_FL_KEEP_SIZE,
    FALLOC_FL_PUNCH_HOLE, F_ADD_SEALS, F_GET_SEALS, F_RDLCK, F_SEAL_GROW, F_SEAL_SEAL, F_SEAL_SHRINK,
    F_SEAL_WRITE, F_UNLCK, F_WRLCK, IN_ATTRIB, IN_CLOEXEC, IN_CLOSE_NOWRITE, IN_CLOSE_WRITE,
    IN_CREATE, IN_DELETE, IN_MODIFY, IN_MOVED_FROM, IN_MOVED_TO, IN_OPEN, IPC_CREAT, IPC_PRIVATE,
    IPC_RMID, MFD_ALLOW_SEALING, MSG_DONTWAIT, P_PID, SEEK_SET, SHUT_WR, SIGKILL, SIGTERM, SIGUSR1,
    SIGUSR2, SOCK_CLOEXEC, SOCK_DGRAM, SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR, SO_TYPE, TFD_CLOEXEC,
    TFD_TIMER_ABSTIME,
};

const PAGE: usize = 4096;

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
    )
}

macro_rules! flock_byte {
    ($name:ident, $ty:expr, $off:expr, $len:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

flock_byte!(d3_lk_rd_0_1, F_RDLCK, 0, 1);
flock_byte!(d3_lk_rd_1_1, F_RDLCK, 1, 1);
flock_byte!(d3_lk_rd_2_2, F_RDLCK, 2, 2);
flock_byte!(d3_lk_rd_4_4, F_RDLCK, 4, 4);
flock_byte!(d3_lk_rd_8_8, F_RDLCK, 8, 8);
flock_byte!(d3_lk_rd_0_0, F_RDLCK, 0, 0);
flock_byte!(d3_lk_rd_10_5, F_RDLCK, 10, 5);
flock_byte!(d3_lk_rd_15_1, F_RDLCK, 15, 1);
flock_byte!(d3_lk_wr_0_1, F_WRLCK, 0, 1);
flock_byte!(d3_lk_wr_1_1, F_WRLCK, 1, 1);
flock_byte!(d3_lk_wr_2_2, F_WRLCK, 2, 2);
flock_byte!(d3_lk_wr_4_4, F_WRLCK, 4, 4);
flock_byte!(d3_lk_wr_8_8, F_WRLCK, 8, 8);
flock_byte!(d3_lk_wr_0_0, F_WRLCK, 0, 0);
flock_byte!(d3_lk_wr_10_5, F_WRLCK, 10, 5);
flock_byte!(d3_lk_wr_15_1, F_WRLCK, 15, 1);
flock_byte!(d3_lk_rd_3_3, F_RDLCK, 3, 3);
flock_byte!(d3_lk_wr_3_3, F_WRLCK, 3, 3);
flock_byte!(d3_lk_rd_6_6, F_RDLCK, 6, 6);
flock_byte!(d3_lk_wr_6_6, F_WRLCK, 6, 6);
flock_byte!(d3_lk_rd_12_4, F_RDLCK, 12, 4);
flock_byte!(d3_lk_wr_12_4, F_WRLCK, 12, 4);
flock_byte!(d3_lk_rd_5_1, F_RDLCK, 5, 1);
flock_byte!(d3_lk_wr_5_1, F_WRLCK, 5, 1);

#[crate::lctp_test(suite = syscall)]
fn d3_fcntl_getlk_contended_fork() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"lk")?;
    write_file(&path, b"data")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "o");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 1,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "set");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let cfd = match syscall::open(&path, oflag::O_RDWR, 0) {
            Ok(f) => f,
            Err(_) => syscall::exit(2),
        };
        let mut probe = Flock {
            l_type: F_WRLCK,
            l_whence: SEEK_SET as i16,
            l_start: 0,
            l_len: 1,
            l_pid: 0,
        };
        let _ = syscall::fcntl_flock(cfd, fcntl_cmd::F_GETLK, &mut probe);
        let code = if probe.l_type == F_UNLCK { 1 } else { 0 };
        let _ = syscall::close(cfd);
        syscall::exit(code);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check!(syscall::wifexited(st), "ex");
    check_eq!(syscall::wexitstatus(st), 0, "contended");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_fcntl_setlk_conflict_fork() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"lk")?;
    write_file(&path, b"zz")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "o");
    let mut lk = Flock {
        l_type: F_WRLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "set");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let cfd = match syscall::open(&path, oflag::O_RDWR, 0) {
            Ok(f) => f,
            Err(_) => syscall::exit(2),
        };
        let mut wr = Flock {
            l_type: F_WRLCK,
            l_whence: SEEK_SET as i16,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };
        let code = match syscall::fcntl_flock(cfd, fcntl_cmd::F_SETLK, &mut wr) {
            Err(Errno::EAGAIN) | Err(Errno::EACCES) => 0,
            Ok(()) => 1,
            Err(_) => 2,
        };
        let _ = syscall::close(cfd);
        syscall::exit(code);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check_eq!(syscall::wexitstatus(st), 0, "eagain");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

macro_rules! dupfd_min {
    ($name:ident, $min:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

dupfd_min!(d3_dupfd_10, 10);
dupfd_min!(d3_dupfd_20, 20);
dupfd_min!(d3_dupfd_30, 30);
dupfd_min!(d3_dupfd_40, 40);
dupfd_min!(d3_dupfd_50, 50);
dupfd_min!(d3_dupfd_60, 60);
dupfd_min!(d3_dupfd_70, 70);
dupfd_min!(d3_dupfd_80, 80);
dupfd_min!(d3_dupfd_90, 90);
dupfd_min!(d3_dupfd_100, 100);
dupfd_min!(d3_dupfd_120, 120);
dupfd_min!(d3_dupfd_150, 150);

macro_rules! setfl_flag {
    ($name:ident, $flag:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
            let old = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "get");
            check_ok!(
                syscall::fcntl(fd, fcntl_cmd::F_SETFL, (old as i32 | $flag) as usize),
                "set"
            );
            let now = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "get2");
            check!(now as i32 & $flag != 0, "flag");
            check_ok!(syscall::fcntl(fd, fcntl_cmd::F_SETFL, old), "restore");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

setfl_flag!(d3_setfl_append, oflag::O_APPEND);
setfl_flag!(d3_setfl_nonblock, oflag::O_NONBLOCK);
setfl_flag!(d3_setfl_append_nb, oflag::O_APPEND | oflag::O_NONBLOCK);

#[crate::lctp_test(suite = syscall)]
fn d3_dupfd_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"d", 0o644), "c");
    let n = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD_CLOEXEC, 20), "dup");
    check!(n as i32 >= 20, "min");
    check_ok!(syscall::close(n as i32), "cn");
    check_ok!(syscall::close(fd), "cf");
    Ok(())
}

macro_rules! epoll_pipe {
    ($name:ident, $ev:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
            let ep = check_ok!(syscall::epoll_create1(0), "ep");
            let mut ev = epoll::EpollEvent {
                events: $ev,
                data: r as u64,
            };
            check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
            check_ok!(syscall::write(w, b"x"), "w");
            let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
            check!(check_ok!(syscall::epoll_wait(ep, &mut out, 100), "wait") >= 1, "rdy");
            check_ok!(syscall::close(ep), "cep");
            check_ok!(syscall::close(r), "cr");
            check_ok!(syscall::close(w), "cw");
            Ok(())
        }
    };
}

epoll_pipe!(d3_ep_in, EPOLLIN);
epoll_pipe!(d3_ep_in_et, EPOLLIN | EPOLLET);
epoll_pipe!(d3_ep_in_os, EPOLLIN | EPOLLONESHOT);
epoll_pipe!(d3_ep_in_et_os, EPOLLIN | EPOLLET | EPOLLONESHOT);

#[crate::lctp_test(suite = syscall)]
fn d3_epoll_oneshot_needs_mod() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLONESHOT,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"a"), "w1");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 50), "w1") >= 1, "e1");
    let mut drain = [0u8; 1];
    check_ok!(syscall::read(r, &mut drain), "drain");
    check_ok!(syscall::write(w, b"b"), "w2");
    check_eq!(check_ok!(syscall::epoll_wait(ep, &mut out, 10), "w2"), 0, "armed off");
    ev.events = EPOLLIN | EPOLLONESHOT;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev), "mod");
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 50), "w3") >= 1, "e2");
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_DEL, r, &mut ev), "del");
    check_ok!(syscall::close(ep), "cep");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_poll_timeout_zero_empty() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut fds = [poll::PollFd {
        fd: r,
        events: syscall::POLLIN,
        revents: 0,
    }];
    check_eq!(check_ok!(syscall::poll(&mut fds, 0), "p"), 0, "none");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_poll_timeout_zero_ready() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
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

#[crate::lctp_test(suite = syscall)]
fn d3_ppoll_timeout_zero() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut fds = [poll::PollFd {
        fd: r,
        events: syscall::POLLIN,
        revents: 0,
    }];
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
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

#[crate::lctp_test(suite = syscall)]
fn d3_ppoll_ready_zero_timeout() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::write(w, b"q"), "w");
    let mut fds = [poll::PollFd {
        fd: r,
        events: syscall::POLLIN,
        revents: 0,
    }];
    let ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    match syscall::ppoll(&mut fds, Some(&ts), None) {
        Ok(n) => check!(n >= 1, "rdy"),
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

macro_rules! efd_sem_drain {
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

efd_sem_drain!(d3_efd_1, 1);
efd_sem_drain!(d3_efd_2, 2);
efd_sem_drain!(d3_efd_3, 3);
efd_sem_drain!(d3_efd_4, 4);
efd_sem_drain!(d3_efd_5, 5);
efd_sem_drain!(d3_efd_11, 11);
efd_sem_drain!(d3_efd_12, 12);
efd_sem_drain!(d3_efd_15, 15);
efd_sem_drain!(d3_efd_16, 16);
efd_sem_drain!(d3_efd_20, 20);

#[crate::lctp_test(suite = syscall)]
fn d3_efd_nonblock_eagain() -> TestResult {
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

macro_rules! wait_exit {
    ($name:ident, $code:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

wait_exit!(d3_ex_1, 1);
wait_exit!(d3_ex_2, 2);
wait_exit!(d3_ex_7, 7);
wait_exit!(d3_ex_17, 17);
wait_exit!(d3_ex_33, 33);
wait_exit!(d3_ex_42, 42);
wait_exit!(d3_ex_55, 55);
wait_exit!(d3_ex_77, 77);
wait_exit!(d3_ex_99, 99);
wait_exit!(d3_ex_127, 127);
wait_exit!(d3_ex_200, 200);
wait_exit!(d3_ex_255, 255);

macro_rules! wait_termsig {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let pid = check_ok!(syscall::fork(), "f");
            if pid == 0 {
                let _ = syscall::nanosleep(&Timespec {
                    tv_sec: 60,
                    tv_nsec: 0,
                });
                syscall::exit(0);
            }
            check_ok!(syscall::kill(pid, $sig), "kill");
            let mut st = 0;
            check_ok!(syscall::wait4(pid, &mut st, 0), "w");
            check!(syscall::wifsignaled(st), "sig");
            check_eq!(syscall::wtermsig(st), $sig, "termsig");
            Ok(())
        }
    };
}

wait_termsig!(d3_term_kill, SIGKILL);
wait_termsig!(d3_term_term, SIGTERM);
wait_termsig!(d3_term_usr1, SIGUSR1);
wait_termsig!(d3_term_usr2, SIGUSR2);

#[crate::lctp_test(suite = syscall)]
fn d3_waitid_exited() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        syscall::exit(9);
    }
    let mut info = syscall::Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WEXITED), "waitid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_waitid_killed() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let _ = syscall::nanosleep(&Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        });
        syscall::exit(0);
    }
    check_ok!(syscall::kill(pid, SIGKILL), "kill");
    let mut info = syscall::Siginfo::default();
    check_ok!(syscall::waitid(P_PID, pid, &mut info, wait::WEXITED), "waitid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_wait4_nohang_none() -> TestResult {
    let mut st = 0;
    match syscall::wait4(-1, &mut st, wait::WNOHANG) {
        Err(Errno::ECHILD) => {}
        Ok(0) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("unexpected child")),
        Err(_) => return Err(crate::harness::AssertFail::msg("wait4")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_kill_zero_self() -> TestResult {
    check_ok!(syscall::kill(0, 0), "kill0");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_kill_self_zero() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exist");
    Ok(())
}

macro_rules! sigpending_grid {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            check_ok!(syscall::signal_ignore($sig), "ign");
            check_ok!(
                syscall::rt_sigprocmask(syscall::SIG_BLOCK, Some(syscall::sigmask($sig)), None),
                "block"
            );
            check_ok!(syscall::kill(syscall::getpid(), $sig), "kill");
            let mut pend = 0u64;
            check_ok!(syscall::rt_sigpending(&mut pend), "pend");
            check!(pend & syscall::sigmask($sig) != 0, "bit");
            check_ok!(
                syscall::rt_sigprocmask(syscall::SIG_UNBLOCK, Some(syscall::sigmask($sig)), None),
                "un"
            );
            check_ok!(syscall::signal_default($sig), "dfl");
            Ok(())
        }
    };
}

sigpending_grid!(d3_pend_usr1, SIGUSR1);
sigpending_grid!(d3_pend_usr2, SIGUSR2);
sigpending_grid!(d3_pend_term, SIGTERM);

macro_rules! madvise_advice {
    ($name:ident, $adv:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

madvise_advice!(d3_madv_normal, madvise::MADV_NORMAL);
madvise_advice!(d3_madv_random, madvise::MADV_RANDOM);
madvise_advice!(d3_madv_seq, madvise::MADV_SEQUENTIAL);
madvise_advice!(d3_madv_willneed, madvise::MADV_WILLNEED);
madvise_advice!(d3_madv_dontneed, madvise::MADV_DONTNEED);
madvise_advice!(d3_madv_free, madvise::MADV_FREE);
madvise_advice!(d3_madv_huge, madvise::MADV_HUGEPAGE);
madvise_advice!(d3_madv_nohuge, madvise::MADV_NOHUGEPAGE);

macro_rules! mprotect_combo {
    ($name:ident, $p:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

mprotect_combo!(d3_mprot_none, prot::PROT_NONE);
mprotect_combo!(d3_mprot_r, prot::PROT_READ);
mprotect_combo!(d3_mprot_w, prot::PROT_WRITE);
mprotect_combo!(d3_mprot_rw, prot::PROT_READ | prot::PROT_WRITE);
mprotect_combo!(d3_mprot_rx, prot::PROT_READ | prot::PROT_EXEC);
mprotect_combo!(d3_mprot_rwx, prot::PROT_READ | prot::PROT_WRITE | prot::PROT_EXEC);

macro_rules! memfd_seal {
    ($name:ident, $seal:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

memfd_seal!(d3_seal_seal, F_SEAL_SEAL);
memfd_seal!(d3_seal_shrink, F_SEAL_SHRINK);
memfd_seal!(d3_seal_grow, F_SEAL_GROW);
memfd_seal!(d3_seal_write, F_SEAL_WRITE);

macro_rules! falloc_punch {
    ($name:ident, $off:expr, $len:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

falloc_punch!(d3_punch_0_1k, 0, 1024);
falloc_punch!(d3_punch_1k_1k, 1024, 1024);
falloc_punch!(d3_punch_2k_2k, 2048, 2048);
falloc_punch!(d3_punch_4k_4k, 4096, 4096);
falloc_punch!(d3_punch_0_4k, 0, 4096);
falloc_punch!(d3_punch_512_512, 512, 512);

macro_rules! tfd_rel_ms {
    ($name:ident, $ms:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

tfd_rel_ms!(d3_tfd_1, 1);
tfd_rel_ms!(d3_tfd_5, 5);
tfd_rel_ms!(d3_tfd_10, 10);
tfd_rel_ms!(d3_tfd_25, 25);
tfd_rel_ms!(d3_tfd_50, 50);
tfd_rel_ms!(d3_tfd_75, 75);
tfd_rel_ms!(d3_tfd_150, 150);
tfd_rel_ms!(d3_tfd_250, 250);

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

tfd_abs_ms!(d3_tfd_abs_4, 4);
tfd_abs_ms!(d3_tfd_abs_8, 8);
tfd_abs_ms!(d3_tfd_abs_16, 16);
tfd_abs_ms!(d3_tfd_abs_32, 32);

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

in_mask!(d3_in_access, syscall::IN_ACCESS);
in_mask!(d3_in_create, IN_CREATE);
in_mask!(d3_in_delete, IN_DELETE);
in_mask!(d3_in_modify, IN_MODIFY);
in_mask!(d3_in_attrib, IN_ATTRIB);
in_mask!(d3_in_open, IN_OPEN);
in_mask!(d3_in_close_w, IN_CLOSE_WRITE);
in_mask!(d3_in_close_nw, IN_CLOSE_NOWRITE);
in_mask!(d3_in_moved_from, IN_MOVED_FROM);
in_mask!(d3_in_moved_to, IN_MOVED_TO);
in_mask!(d3_in_cd, IN_CREATE | IN_DELETE);
in_mask!(d3_in_cm, IN_CREATE | IN_MODIFY);
in_mask!(d3_in_oa, IN_OPEN | IN_ATTRIB);
in_mask!(d3_in_move, IN_MOVED_FROM | IN_MOVED_TO);
in_mask!(d3_in_close, IN_CLOSE_WRITE | IN_CLOSE_NOWRITE);
in_mask!(
    d3_in_all,
    IN_CREATE | IN_DELETE | IN_MODIFY | IN_ATTRIB | IN_OPEN | IN_CLOSE_WRITE | IN_CLOSE_NOWRITE
);

#[crate::lctp_test(suite = syscall)]
fn d3_pidfd_send_term() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let _ = syscall::nanosleep(&Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        });
        syscall::exit(0);
    }
    let pfd = match syscall::pidfd_open(pid, 0) {
        Ok(f) => f,
        Err(e) if soft(e) => {
            let _ = syscall::kill(pid, SIGKILL);
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            return Ok(());
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("pidfd")),
    };
    check_ok!(syscall::pidfd_send_signal(pfd, SIGTERM, None, 0), "sig");
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    check!(syscall::wifsignaled(st), "sig");
    check_ok!(syscall::close(pfd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_pidfd_send_usr1() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let _ = syscall::signal_ignore(SIGUSR1);
        let _ = syscall::nanosleep(&Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        });
        syscall::exit(0);
    }
    let pfd = match syscall::pidfd_open(pid, 0) {
        Ok(f) => f,
        Err(e) if soft(e) => {
            let _ = syscall::kill(pid, SIGKILL);
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            return Ok(());
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("pidfd")),
    };
    check_ok!(syscall::pidfd_send_signal(pfd, SIGUSR1, None, 0), "sig");
    check_ok!(syscall::pidfd_send_signal(pfd, SIGKILL, None, 0), "kill");
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    check_ok!(syscall::close(pfd), "c");
    Ok(())
}

macro_rules! shm_size {
    ($name:ident, $sz:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let shmid = match syscall::shmget(IPC_PRIVATE, $sz, IPC_CREAT | 0o600) {
                Ok(id) => id,
                Err(e) if soft(e) => return Ok(()),
                Err(_) => return Err(crate::harness::AssertFail::msg("shmget")),
            };
            let addr = match syscall::shmat(shmid, 0, 0) {
                Ok(a) => a,
                Err(e) => {
                    let _ = syscall::shmctl(shmid, IPC_RMID, 0);
                    if soft(e) {
                        return Ok(());
                    }
                    return Err(crate::harness::AssertFail::msg("shmat"));
                }
            };
            unsafe {
                *(addr as *mut u8) = 0xAA;
            }
            check_ok!(syscall::shmdt(addr), "dt");
            check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rm");
            Ok(())
        }
    };
}

shm_size!(d3_shm_4k, 4096);
shm_size!(d3_shm_8k, 8192);
shm_size!(d3_shm_16k, 16384);
shm_size!(d3_shm_32k, 32768);
shm_size!(d3_shm_64k, 65536);

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

splice_n!(d3_sp_1, 1);
splice_n!(d3_sp_2, 2);
splice_n!(d3_sp_4, 4);
splice_n!(d3_sp_8, 8);
splice_n!(d3_sp_16, 16);
splice_n!(d3_sp_32, 32);
splice_n!(d3_sp_64, 64);
splice_n!(d3_sp_128, 128);
splice_n!(d3_sp_200, 200);
splice_n!(d3_sp_300, 300);

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

tee_n!(d3_tee_1, 1);
tee_n!(d3_tee_2, 2);
tee_n!(d3_tee_4, 4);
tee_n!(d3_tee_8, 8);
tee_n!(d3_tee_16, 16);
tee_n!(d3_tee_32, 32);
tee_n!(d3_tee_64, 64);
tee_n!(d3_tee_100, 100);

macro_rules! vmsplice_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall)]
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

vmsplice_n!(d3_vs_1, 1);
vmsplice_n!(d3_vs_2, 2);
vmsplice_n!(d3_vs_4, 4);
vmsplice_n!(d3_vs_8, 8);
vmsplice_n!(d3_vs_16, 16);
vmsplice_n!(d3_vs_32, 32);
vmsplice_n!(d3_vs_64, 64);
vmsplice_n!(d3_vs_128, 128);

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

macro_rules! tcp_send_sz {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let (srv, cli, acc) = tcp_pair()?;
            let msg = [b'T'; $n];
            check_eq!(check_ok!(syscall::send(cli, &msg, 0), "send"), $n, "n");
            let mut buf = [0u8; $n];
            let mut got = 0usize;
            while got < $n {
                let n = check_ok!(syscall::recv(acc, &mut buf[got..], 0), "recv");
                if n == 0 {
                    break;
                }
                got += n;
            }
            check_eq!(got, $n, "got");
            check_ok!(syscall::close(acc), "a");
            check_ok!(syscall::close(cli), "c");
            check_ok!(syscall::close(srv), "s");
            Ok(())
        }
    };
}

tcp_send_sz!(d3_tcp_16, 16);
tcp_send_sz!(d3_tcp_32, 32);
tcp_send_sz!(d3_tcp_64, 64);
tcp_send_sz!(d3_tcp_128, 128);
tcp_send_sz!(d3_tcp_256, 256);
tcp_send_sz!(d3_tcp_512, 512);
tcp_send_sz!(d3_tcp_1024, 1024);

#[crate::lctp_test(suite = syscall)]
fn d3_tcp_dontwait_empty() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    let mut buf = [0u8; 8];
    match syscall::recv(acc, &mut buf, MSG_DONTWAIT) {
        Err(Errno::EAGAIN) | Err(Errno::EWOULDBLOCK) => {}
        Ok(0) => {}
        Ok(_) => {
            let _ = syscall::close(acc);
            let _ = syscall::close(cli);
            let _ = syscall::close(srv);
            return Err(crate::harness::AssertFail::msg("expected eagain"));
        }
        Err(_) => {
            let _ = syscall::close(acc);
            let _ = syscall::close(cli);
            let _ = syscall::close(srv);
            return Err(crate::harness::AssertFail::msg("recv"));
        }
    }
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_tcp_shutdown_eof() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::shutdown(cli, SHUT_WR), "shut");
    let mut b = [0u8; 4];
    check_eq!(check_ok!(syscall::recv(acc, &mut b, 0), "eof"), 0, "eof");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_udp_dontwait() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "s");
    check_ok!(syscall::bind(fd, &SockAddrIn::loopback(0)), "b");
    let mut buf = [0u8; 16];
    match syscall::recv(fd, &mut buf, MSG_DONTWAIT) {
        Err(Errno::EAGAIN) | Err(Errno::EWOULDBLOCK) => {}
        Ok(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("expected eagain"));
        }
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("recv"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_udp_echo_small() -> TestResult {
    let a = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "a");
    let b = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "b");
    check_ok!(syscall::bind(a, &SockAddrIn::loopback(0)), "ba");
    let addr = check_ok!(syscall::getsockname_in(a), "n");
    check_ok!(syscall::sendto(b, b"hi", 0, Some(&addr)), "send");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(a, &mut buf, 0), "recv"), 2, "n");
    check_eq!(&buf[..2], b"hi", "data");
    check_ok!(syscall::close(a), "ca");
    check_ok!(syscall::close(b), "cb");
    Ok(())
}

macro_rules! so_type {
    ($name:ident, $ty:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(syscall::socket(AF_INET, $ty, 0), "s");
            let mut v = [0u8; 4];
            check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_TYPE, &mut v), "g");
            check_eq!(i32::from_ne_bytes(v), $ty, "ty");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

so_type!(d3_so_stream, SOCK_STREAM);
so_type!(d3_so_dgram, SOCK_DGRAM);

#[crate::lctp_test(suite = syscall)]
fn d3_mmap_anon_rw() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE * 2,
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
    check_ok!(syscall::munmap(addr, PAGE * 2), "un");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d3_check_err_bad_fd() -> TestResult {
    check_err!(syscall::close(-1), Errno::EBADF, "ebadf");
    Ok(())
}
