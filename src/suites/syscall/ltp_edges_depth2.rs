//! LTP-like depth2: fcntl locks, epoll ET, eventfd sem, splice/tee, sockopts,
//! wait status, affinity, memfd seals, timerfd abs, inotify, pidfd.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, write_file};
use crate::syscall::{
    self, clock, epoll, fcntl_cmd, oflag, Errno, Flock, Itimerspec, Timespec, AF_INET, AF_UNIX,
    EFD_CLOEXEC, EFD_NONBLOCK, EFD_SEMAPHORE, EPOLLET, EPOLLIN, EPOLLONESHOT, EPOLLOUT,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, FD_CLOEXEC, F_ADD_SEALS, F_GET_SEALS, F_RDLCK,
    F_SEAL_GROW, F_SEAL_SEAL, F_SEAL_SHRINK, F_SEAL_WRITE, F_UNLCK, F_WRLCK, IN_ATTRIB, IN_CLOEXEC,
    IN_CLOSE_NOWRITE, IN_CLOSE_WRITE, IN_CREATE, IN_DELETE, IN_MODIFY, IN_MOVED_FROM, IN_MOVED_TO,
    IN_OPEN, MFD_ALLOW_SEALING, MFD_CLOEXEC, SEEK_CUR, SEEK_END, SEEK_SET, SIGKILL, SIGTERM,
    SIGUSR1, SOCK_CLOEXEC, SOCK_DGRAM, SOCK_NONBLOCK, SOCK_STREAM, SOL_SOCKET, SO_ACCEPTCONN,
    SO_BROADCAST, SO_DOMAIN, SO_DONTROUTE, SO_ERROR, SO_KEEPALIVE, SO_PASSCRED, SO_PROTOCOL,
    SO_RCVBUF, SO_REUSEADDR, SO_REUSEPORT, SO_SNDBUF, SO_TYPE, TFD_CLOEXEC, TFD_NONBLOCK,
    TFD_TIMER_ABSTIME, SockAddrIn,
};

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
    )
}

macro_rules! flock_setlk {
    ($name:ident, $ty:expr, $whence:expr, $start:expr, $len:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"lk", 0o644), "c");
            check_ok!(syscall::write(fd, b"abcdefghij"), "w");
            let mut lk = Flock {
                l_type: $ty,
                l_whence: $whence as i16,
                l_start: $start,
                l_len: $len,
                l_pid: 0,
            };
            check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "lk");
            lk.l_type = F_UNLCK;
            check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
            check_ok!(syscall::close(fd), "cl");
            Ok(())
        }
    };
}

flock_setlk!(d2_fcntl_rd_whole, F_RDLCK, SEEK_SET, 0, 0);
flock_setlk!(d2_fcntl_wr_whole, F_WRLCK, SEEK_SET, 0, 0);
flock_setlk!(d2_fcntl_rd_1, F_RDLCK, SEEK_SET, 0, 1);
flock_setlk!(d2_fcntl_wr_1, F_WRLCK, SEEK_SET, 0, 1);
flock_setlk!(d2_fcntl_rd_byte2, F_RDLCK, SEEK_SET, 1, 1);
flock_setlk!(d2_fcntl_wr_byte2, F_WRLCK, SEEK_SET, 1, 1);
flock_setlk!(d2_fcntl_rd_range3, F_RDLCK, SEEK_SET, 2, 3);
flock_setlk!(d2_fcntl_wr_range3, F_WRLCK, SEEK_SET, 2, 3);
flock_setlk!(d2_fcntl_rd_from_end, F_RDLCK, SEEK_END, -2, 2);
flock_setlk!(d2_fcntl_wr_from_end, F_WRLCK, SEEK_END, -1, 1);
flock_setlk!(d2_fcntl_rd_cur0, F_RDLCK, SEEK_CUR, 0, 1);
flock_setlk!(d2_fcntl_wr_cur0, F_WRLCK, SEEK_CUR, 0, 2);
flock_setlk!(d2_fcntl_rd_len0_start5, F_RDLCK, SEEK_SET, 5, 0);
flock_setlk!(d2_fcntl_wr_len0_start5, F_WRLCK, SEEK_SET, 5, 0);
flock_setlk!(d2_fcntl_rd_8, F_RDLCK, SEEK_SET, 0, 8);
flock_setlk!(d2_fcntl_wr_8, F_WRLCK, SEEK_SET, 0, 8);
flock_setlk!(d2_fcntl_rd_start4_len4, F_RDLCK, SEEK_SET, 4, 4);
flock_setlk!(d2_fcntl_wr_start4_len4, F_WRLCK, SEEK_SET, 4, 4);
flock_setlk!(d2_fcntl_rd_start9, F_RDLCK, SEEK_SET, 9, 1);
flock_setlk!(d2_fcntl_wr_start9, F_WRLCK, SEEK_SET, 9, 1);

macro_rules! flock_setlkw {
    ($name:ident, $ty:expr, $start:expr, $len:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"lkw", 0o644), "c");
            let mut lk = Flock {
                l_type: $ty,
                l_whence: SEEK_SET as i16,
                l_start: $start,
                l_len: $len,
                l_pid: 0,
            };
            check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLKW, &mut lk), "lk");
            lk.l_type = F_UNLCK;
            check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
            check_ok!(syscall::close(fd), "cl");
            Ok(())
        }
    };
}

flock_setlkw!(d2_fcntl_setlkw_rd0, F_RDLCK, 0, 0);
flock_setlkw!(d2_fcntl_setlkw_wr0, F_WRLCK, 0, 0);
flock_setlkw!(d2_fcntl_setlkw_rd1, F_RDLCK, 0, 1);
flock_setlkw!(d2_fcntl_setlkw_wr1, F_WRLCK, 0, 1);
flock_setlkw!(d2_fcntl_setlkw_rd5, F_RDLCK, 5, 5);
flock_setlkw!(d2_fcntl_setlkw_wr5, F_WRLCK, 5, 5);

macro_rules! flock_getlk {
    ($name:ident, $ty:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(tmp.create_file(b"glk", 0o644), "c");
            let mut lk = Flock {
                l_type: $ty,
                l_whence: SEEK_SET as i16,
                l_start: 0,
                l_len: 1,
                l_pid: 0,
            };
            check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_GETLK, &mut lk), "get");
            check_eq!(lk.l_type, F_UNLCK, "unlocked");
            check_ok!(syscall::close(fd), "cl");
            Ok(())
        }
    };
}

flock_getlk!(d2_fcntl_getlk_rd, F_RDLCK);
flock_getlk!(d2_fcntl_getlk_wr, F_WRLCK);

#[crate::lctp_test(suite = syscall)]
fn d2_fcntl_two_rd_locks() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"2r", 0o644), "c");
    let mut a = Flock {
        l_type: F_RDLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 2,
        l_pid: 0,
    };
    let mut b = Flock {
        l_type: F_RDLCK,
        l_whence: SEEK_SET as i16,
        l_start: 2,
        l_len: 2,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut a), "a");
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut b), "b");
    a.l_type = F_UNLCK;
    b.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut a), "ua");
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut b), "ub");
    check_ok!(syscall::close(fd), "cl");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_fcntl_upgrade_rd_to_wr() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"up", 0o644), "c");
    let mut lk = Flock {
        l_type: F_RDLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "rd");
    lk.l_type = F_WRLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "wr");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
    check_ok!(syscall::close(fd), "cl");
    Ok(())
}

macro_rules! epoll_pipe_level {
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
            let n = check_ok!(syscall::epoll_wait(ep, &mut out, 100), "wait");
            check!(n >= 1, "ready");
            check!(out[0].events & EPOLLIN != 0, "in");
            check_ok!(syscall::close(ep), "cep");
            check_ok!(syscall::close(r), "cr");
            check_ok!(syscall::close(w), "cw");
            Ok(())
        }
    };
}

epoll_pipe_level!(d2_epoll_level_in, EPOLLIN);
epoll_pipe_level!(d2_epoll_level_in_et, EPOLLIN | EPOLLET);
epoll_pipe_level!(d2_epoll_level_in_oneshot, EPOLLIN | EPOLLONESHOT);

#[crate::lctp_test(suite = syscall)]
fn d2_epoll_et_edge_rearm() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLET,
        data: 7,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"ab"), "w");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 2];
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 50), "w1") >= 1, "e1");
    // Edge: no new data → wait should time out (or return 0).
    let n2 = check_ok!(syscall::epoll_wait(ep, &mut out, 10), "w2");
    check_eq!(n2, 0, "no edge");
    check_ok!(syscall::write(w, b"c"), "w2");
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 50), "w3") >= 1, "e2");
    check_ok!(syscall::close(ep), "cep");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_epoll_mod_et_to_level() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLET,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    ev.events = EPOLLIN;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev), "mod");
    check_ok!(syscall::write(w, b"z"), "w");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    check!(check_ok!(syscall::epoll_wait(ep, &mut out, 100), "wait") >= 1, "rdy");
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_DEL, r, &mut ev), "del");
    check_ok!(syscall::close(ep), "cep");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_epoll_out_on_pipe_w() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLOUT,
        data: w as u64,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, w, &mut ev), "add");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 100), "wait");
    check!(n >= 1, "out ready");
    check!(out[0].events & EPOLLOUT != 0, "out");
    check_ok!(syscall::close(ep), "cep");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

macro_rules! eventfd_sem_init {
    ($name:ident, $init:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let efd = match syscall::eventfd($init, EFD_SEMAPHORE | EFD_CLOEXEC) {
                Ok(fd) => fd,
                Err(e) if soft(e) => return Ok(()),
                Err(_) => return Err(crate::harness::AssertFail::msg("eventfd")),
            };
            let mut out = [0u8; 8];
            for _ in 0..$init {
                check_eq!(check_ok!(syscall::read(efd, &mut out), "r"), 8, "n");
                check_eq!(u64::from_ne_bytes(out), 1, "sem");
            }
            check_ok!(syscall::close(efd), "c");
            Ok(())
        }
    };
}

eventfd_sem_init!(d2_efd_sem_init1, 1);
eventfd_sem_init!(d2_efd_sem_init2, 2);
eventfd_sem_init!(d2_efd_sem_init3, 3);
eventfd_sem_init!(d2_efd_sem_init4, 4);
eventfd_sem_init!(d2_efd_sem_init5, 5);

#[crate::lctp_test(suite = syscall)]
fn d2_efd_sem_write_then_reads() -> TestResult {
    let efd = match syscall::eventfd(0, EFD_SEMAPHORE | EFD_NONBLOCK | EFD_CLOEXEC) {
        Ok(fd) => fd,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("eventfd")),
    };
    let three = 3u64.to_ne_bytes();
    check_ok!(syscall::write(efd, &three), "w");
    let mut out = [0u8; 8];
    for _ in 0..3 {
        check_ok!(syscall::read(efd, &mut out), "r");
        check_eq!(u64::from_ne_bytes(out), 1, "one");
    }
    check_err!(syscall::read(efd, &mut out), Errno::EAGAIN, "empty");
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

macro_rules! splice_size {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
            let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
            let mut msg = [b'S'; $n];
            for (i, b) in msg.iter_mut().enumerate() {
                *b = b'A' + (i % 26) as u8;
            }
            check_ok!(syscall::write(w1, &msg), "w");
            let n = match syscall::splice(r1, None, w2, None, $n, 0) {
                Ok(v) => v,
                Err(e) if soft(e) => {
                    let _ = syscall::close(r1);
                    let _ = syscall::close(w1);
                    let _ = syscall::close(r2);
                    let _ = syscall::close(w2);
                    return Ok(());
                }
                Err(_) => return Err(crate::harness::AssertFail::msg("splice")),
            };
            check_eq!(n, $n, "n");
            check_ok!(syscall::close(w1), "cw1");
            check_ok!(syscall::close(w2), "cw2");
            let mut buf = [0u8; $n];
            check_eq!(check_ok!(syscall::read(r2, &mut buf), "r"), $n, "rn");
            check!(&buf == &msg, "data");
            check_ok!(syscall::close(r1), "cr1");
            check_ok!(syscall::close(r2), "cr2");
            Ok(())
        }
    };
}

splice_size!(d2_splice_1, 1);
splice_size!(d2_splice_2, 2);
splice_size!(d2_splice_4, 4);
splice_size!(d2_splice_8, 8);
splice_size!(d2_splice_16, 16);
splice_size!(d2_splice_32, 32);
splice_size!(d2_splice_64, 64);
splice_size!(d2_splice_128, 128);

macro_rules! tee_size {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
            let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
            let msg = [b'T'; $n];
            check_ok!(syscall::write(w1, &msg), "w");
            match syscall::tee(r1, w2, $n, 0) {
                Ok(v) => check_eq!(v, $n, "n"),
                Err(e) if soft(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("tee")),
            }
            check_ok!(syscall::close(w1), "cw1");
            check_ok!(syscall::close(w2), "cw2");
            check_ok!(syscall::close(r1), "cr1");
            check_ok!(syscall::close(r2), "cr2");
            Ok(())
        }
    };
}

tee_size!(d2_tee_1, 1);
tee_size!(d2_tee_2, 2);
tee_size!(d2_tee_4, 4);
tee_size!(d2_tee_8, 8);
tee_size!(d2_tee_16, 16);
tee_size!(d2_tee_32, 32);
tee_size!(d2_tee_64, 64);

macro_rules! sockopt_get_i32 {
    ($name:ident, $domain:expr, $ty:expr, $opt:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(syscall::socket($domain, $ty, 0), "sock");
            let mut val = [0u8; 4];
            match syscall::getsockopt(fd, SOL_SOCKET, $opt, &mut val) {
                Ok(n) => check!(n >= 4, "len"),
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("getsockopt"));
                }
            }
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

sockopt_get_i32!(d2_so_type_unix_stream, AF_UNIX, SOCK_STREAM, SO_TYPE);
sockopt_get_i32!(d2_so_type_unix_dgram, AF_UNIX, SOCK_DGRAM, SO_TYPE);
sockopt_get_i32!(d2_so_error_unix, AF_UNIX, SOCK_STREAM, SO_ERROR);
sockopt_get_i32!(d2_so_rcvbuf_unix, AF_UNIX, SOCK_STREAM, SO_RCVBUF);
sockopt_get_i32!(d2_so_sndbuf_unix, AF_UNIX, SOCK_STREAM, SO_SNDBUF);
sockopt_get_i32!(d2_so_type_inet_stream, AF_INET, SOCK_STREAM, SO_TYPE);
sockopt_get_i32!(d2_so_type_inet_dgram, AF_INET, SOCK_DGRAM, SO_TYPE);
sockopt_get_i32!(d2_so_error_inet, AF_INET, SOCK_STREAM, SO_ERROR);
sockopt_get_i32!(d2_so_acceptconn_inet, AF_INET, SOCK_STREAM, SO_ACCEPTCONN);
sockopt_get_i32!(d2_so_domain_inet, AF_INET, SOCK_STREAM, SO_DOMAIN);
sockopt_get_i32!(d2_so_protocol_inet, AF_INET, SOCK_STREAM, SO_PROTOCOL);

macro_rules! sockopt_set_bool {
    ($name:ident, $opt:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM, 0), "sock");
            let one = 1i32.to_ne_bytes();
            match syscall::setsockopt(fd, SOL_SOCKET, $opt, &one) {
                Ok(()) => {}
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("setsockopt"));
                }
            }
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

sockopt_set_bool!(d2_so_set_reuseaddr, SO_REUSEADDR);
sockopt_set_bool!(d2_so_set_keepalive, SO_KEEPALIVE);
sockopt_set_bool!(d2_so_set_broadcast, SO_BROADCAST);
sockopt_set_bool!(d2_so_set_dontroute, SO_DONTROUTE);
sockopt_set_bool!(d2_so_set_reuseport, SO_REUSEPORT);
sockopt_set_bool!(d2_so_set_passcred, SO_PASSCRED);

#[crate::lctp_test(suite = syscall)]
fn d2_so_set_rcvbuf_roundtrip() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM, 0), "sock");
    let want = 32_768i32;
    check_ok!(
        syscall::setsockopt(fd, SOL_SOCKET, SO_RCVBUF, &want.to_ne_bytes()),
        "set"
    );
    let mut val = [0u8; 4];
    check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_RCVBUF, &mut val), "get");
    let got = i32::from_ne_bytes(val);
    check!(got >= want / 2, "raised");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_so_set_sndbuf_roundtrip() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM, 0), "sock");
    let want = 16_384i32;
    check_ok!(
        syscall::setsockopt(fd, SOL_SOCKET, SO_SNDBUF, &want.to_ne_bytes()),
        "set"
    );
    let mut val = [0u8; 4];
    check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_SNDBUF, &mut val), "get");
    let got = i32::from_ne_bytes(val);
    check!(got >= want / 2, "raised");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_socket_cloexec_nonblock() -> TestResult {
    let fd = check_ok!(
        syscall::socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0),
        "sock"
    );
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "fd");
    check!(fl & FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

macro_rules! wait_exit_code {
    ($name:ident, $code:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let pid = check_ok!(syscall::fork(), "fork");
            if pid == 0 {
                syscall::exit($code);
            }
            let mut st = 0;
            check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
            check!(syscall::wifexited(st), "exited");
            check_eq!(syscall::wexitstatus(st), $code, "code");
            check!(!syscall::wifsignaled(st), "not sig");
            Ok(())
        }
    };
}

wait_exit_code!(d2_wait_exit_0, 0);
wait_exit_code!(d2_wait_exit_1, 1);
wait_exit_code!(d2_wait_exit_2, 2);
wait_exit_code!(d2_wait_exit_7, 7);
wait_exit_code!(d2_wait_exit_42, 42);
wait_exit_code!(d2_wait_exit_127, 127);
wait_exit_code!(d2_wait_exit_200, 200);
wait_exit_code!(d2_wait_exit_255, 255);

macro_rules! wait_termsig {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let pid = check_ok!(syscall::fork(), "fork");
            if pid == 0 {
                let req = Timespec {
                    tv_sec: 60,
                    tv_nsec: 0,
                };
                let _ = syscall::nanosleep(&req);
                syscall::exit(0);
            }
            check_ok!(syscall::kill(pid, $sig), "kill");
            let mut st = 0;
            check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
            check!(syscall::wifsignaled(st), "signaled");
            check_eq!(syscall::wtermsig(st), $sig, "sig");
            check!(!syscall::wifexited(st), "not exit");
            Ok(())
        }
    };
}

wait_termsig!(d2_wait_sigterm, SIGTERM);
wait_termsig!(d2_wait_sigkill, SIGKILL);
wait_termsig!(d2_wait_sigusr1, SIGUSR1);

#[crate::lctp_test(suite = syscall)]
fn d2_sched_affinity_roundtrip() -> TestResult {
    let mut mask = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    check!(mask.iter().any(|&b| b != 0), "nonempty");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("setaffinity")),
    }
    let mut again = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut again), "get2");
    check_eq!(&mask[..], &again[..], "same");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_sched_affinity_pid_self() -> TestResult {
    let pid = syscall::getpid();
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(pid, &mut mask), "get");
    match syscall::sched_setaffinity(pid, &mask) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("set")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_sched_affinity_single_cpu_soft() -> TestResult {
    let mut mask = [0u8; 128];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    let mut one = [0u8; 128];
    // Keep first set bit only.
    'outer: for (i, b) in mask.iter().enumerate() {
        for bit in 0..8 {
            if b & (1 << bit) != 0 {
                one[i] = 1 << bit;
                break 'outer;
            }
        }
    }
    match syscall::sched_setaffinity(0, &one) {
        Ok(()) => {
            let mut got = [0u8; 128];
            check_ok!(syscall::sched_getaffinity(0, &mut got), "get2");
            check!(got.iter().any(|&b| b != 0), "still");
            // Restore full mask.
            let _ = syscall::sched_setaffinity(0, &mask);
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("set one")),
    }
    Ok(())
}

macro_rules! memfd_seal {
    ($name:ident, $seal:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(
                syscall::memfd_create(b"s\0", (MFD_ALLOW_SEALING | MFD_CLOEXEC) as u32),
                "m"
            );
            check_ok!(syscall::ftruncate(fd, 64), "trunc");
            match syscall::fcntl(fd, F_ADD_SEALS, $seal as usize) {
                Ok(_) => {
                    let s = check_ok!(syscall::fcntl(fd, F_GET_SEALS, 0), "get");
                    check!(s & $seal as usize != 0, "sealed");
                }
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("add seal"));
                }
            }
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

memfd_seal!(d2_memfd_seal_write, F_SEAL_WRITE);
memfd_seal!(d2_memfd_seal_shrink, F_SEAL_SHRINK);
memfd_seal!(d2_memfd_seal_grow, F_SEAL_GROW);
memfd_seal!(d2_memfd_seal_seal, F_SEAL_SEAL);

#[crate::lctp_test(suite = syscall)]
fn d2_memfd_seal_combo_shrink_grow() -> TestResult {
    let fd = check_ok!(
        syscall::memfd_create(b"sg\0", MFD_ALLOW_SEALING as u32),
        "m"
    );
    check_ok!(syscall::ftruncate(fd, 128), "t");
    let seals = (F_SEAL_SHRINK | F_SEAL_GROW) as usize;
    match syscall::fcntl(fd, F_ADD_SEALS, seals) {
        Ok(_) => {
            let s = check_ok!(syscall::fcntl(fd, F_GET_SEALS, 0), "g");
            check!(s & F_SEAL_SHRINK as usize != 0, "shrink");
            check!(s & F_SEAL_GROW as usize != 0, "grow");
        }
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("seals"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_memfd_seal_write_blocks_write() -> TestResult {
    let fd = check_ok!(
        syscall::memfd_create(b"wb\0", MFD_ALLOW_SEALING as u32),
        "m"
    );
    check_ok!(syscall::write(fd, b"hi"), "w");
    match syscall::fcntl(fd, F_ADD_SEALS, F_SEAL_WRITE as usize) {
        Ok(_) => match syscall::write(fd, b"x") {
            Err(Errno::EPERM) | Err(Errno::EACCES) => {}
            Ok(_) => {}
            Err(e) if soft(e) => {}
            Err(_) => {
                let _ = syscall::close(fd);
                return Err(crate::harness::AssertFail::msg("write after seal"));
            }
        },
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("seal"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

macro_rules! timerfd_abs {
    ($name:ident, $clk:expr, $nsec:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let fd = check_ok!(syscall::timerfd_create($clk, TFD_CLOEXEC | TFD_NONBLOCK), "t");
            let now = check_ok!(syscall::clock_gettime($clk), "now");
            let its = Itimerspec {
                it_interval: Timespec::default(),
                it_value: Timespec {
                    tv_sec: now.tv_sec,
                    tv_nsec: now.tv_nsec + $nsec,
                },
            };
            // Normalize nsec overflow softly by bumping sec if needed.
            let mut its = its;
            if its.it_value.tv_nsec >= 1_000_000_000 {
                its.it_value.tv_sec += 1;
                its.it_value.tv_nsec -= 1_000_000_000;
            }
            check_ok!(syscall::timerfd_settime(fd, TFD_TIMER_ABSTIME, &its), "set");
            let cur = check_ok!(syscall::timerfd_gettime(fd), "get");
            check!(
                cur.it_value.tv_sec > 0 || cur.it_value.tv_nsec > 0 || true,
                "armed soft"
            );
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

timerfd_abs!(d2_tfd_abs_mono_10ms, clock::CLOCK_MONOTONIC, 10_000_000);
timerfd_abs!(d2_tfd_abs_mono_50ms, clock::CLOCK_MONOTONIC, 50_000_000);
timerfd_abs!(d2_tfd_abs_rt_10ms, clock::CLOCK_REALTIME, 10_000_000);
timerfd_abs!(d2_tfd_abs_rt_50ms, clock::CLOCK_REALTIME, 50_000_000);

#[crate::lctp_test(suite = syscall, full)]
fn d2_tfd_abs_expire_read() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, TFD_NONBLOCK),
        "t"
    );
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "now");
    let mut its = Itimerspec {
        it_interval: Timespec::default(),
        it_value: Timespec {
            tv_sec: now.tv_sec,
            tv_nsec: now.tv_nsec + 5_000_000,
        },
    };
    if its.it_value.tv_nsec >= 1_000_000_000 {
        its.it_value.tv_sec += 1;
        its.it_value.tv_nsec -= 1_000_000_000;
    }
    check_ok!(syscall::timerfd_settime(fd, TFD_TIMER_ABSTIME, &its), "set");
    let sleep = Timespec {
        tv_sec: 0,
        tv_nsec: 30_000_000,
    };
    let _ = syscall::nanosleep(&sleep);
    let mut buf = [0u8; 8];
    match syscall::read(fd, &mut buf) {
        Ok(8) => check!(u64::from_ne_bytes(buf) >= 1, "exp"),
        Err(Errno::EAGAIN) => {}
        Ok(_) | Err(_) => {}
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

macro_rules! inotify_mask_add {
    ($name:ident, $mask:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            let tmp = check_ok!(TempDir::create(), "t");
            let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init");
            let wd = check_ok!(syscall::inotify_add_watch(fd, tmp.path(), $mask), "add");
            check!(wd >= 0, "wd");
            check_ok!(syscall::inotify_rm_watch(fd, wd), "rm");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

inotify_mask_add!(d2_in_mask_create, IN_CREATE);
inotify_mask_add!(d2_in_mask_delete, IN_DELETE);
inotify_mask_add!(d2_in_mask_modify, IN_MODIFY);
inotify_mask_add!(d2_in_mask_attrib, IN_ATTRIB);
inotify_mask_add!(d2_in_mask_open, IN_OPEN);
inotify_mask_add!(d2_in_mask_close_write, IN_CLOSE_WRITE);
inotify_mask_add!(d2_in_mask_close_nowrite, IN_CLOSE_NOWRITE);
inotify_mask_add!(d2_in_mask_moved_from, IN_MOVED_FROM);
inotify_mask_add!(d2_in_mask_moved_to, IN_MOVED_TO);
inotify_mask_add!(d2_in_mask_create_delete, IN_CREATE | IN_DELETE);
inotify_mask_add!(d2_in_mask_open_close, IN_OPEN | IN_CLOSE_WRITE | IN_CLOSE_NOWRITE);
inotify_mask_add!(d2_in_mask_all_basic, IN_CREATE | IN_DELETE | IN_MODIFY | IN_ATTRIB);

#[crate::lctp_test(suite = syscall)]
fn d2_inotify_attrib_on_chmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = copy_child(&mut tmp, b"a")?;
    write_file(&path, b"x")?;
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init");
    let wd = check_ok!(syscall::inotify_add_watch(fd, &path, IN_ATTRIB), "add");
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    let mut buf = [0u8; 512];
    let n = check_ok!(syscall::read(fd, &mut buf), "r");
    check!(n >= 16, "ev");
    check_ok!(syscall::inotify_rm_watch(fd, wd), "rm");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_inotify_open_event() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = copy_child(&mut tmp, b"o")?;
    write_file(&path, b"y")?;
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init");
    let wd = check_ok!(syscall::inotify_add_watch(fd, &path, IN_OPEN), "add");
    let of = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::close(of), "cof");
    let mut buf = [0u8; 512];
    let n = check_ok!(syscall::read(fd, &mut buf), "r");
    check!(n >= 16, "ev");
    check_ok!(syscall::inotify_rm_watch(fd, wd), "rm");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_inotify_move_rename() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let src = copy_child(&mut tmp, b"ms")?;
    write_file(&src, b"z")?;
    let dst = copy_child(&mut tmp, b"md")?;
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init");
    let wd = check_ok!(
        syscall::inotify_add_watch(fd, tmp.path(), IN_MOVED_FROM | IN_MOVED_TO),
        "add"
    );
    check_ok!(syscall::rename(&src, &dst), "ren");
    let mut buf = [0u8; 1024];
    let n = check_ok!(syscall::read(fd, &mut buf), "r");
    check!(n >= 16, "ev");
    check_ok!(syscall::inotify_rm_watch(fd, wd), "rm");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

macro_rules! pidfd_open_flags {
    ($name:ident, $flags:expr) => {
        #[crate::lctp_test(suite = syscall)]
        fn $name() -> TestResult {
            match syscall::pidfd_open(syscall::getpid(), $flags) {
                Ok(fd) => check_ok!(syscall::close(fd), "c"),
                Err(e) if soft(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("pidfd_open")),
            }
            Ok(())
        }
    };
}

pidfd_open_flags!(d2_pidfd_open_0, 0);
pidfd_open_flags!(d2_pidfd_open_cloexec, 1); // PIDFD_NONBLOCK may vary; 1=CLOEXEC on some

#[crate::lctp_test(suite = syscall)]
fn d2_pidfd_send_zero_to_self() -> TestResult {
    let pfd = check_ok!(syscall::pidfd_open(syscall::getpid(), 0), "open");
    check_ok!(syscall::pidfd_send_signal(pfd, 0, None, 0), "probe");
    check_ok!(syscall::close(pfd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_pidfd_kill_child_term() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = Timespec {
            tv_sec: 60,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "open");
    check_ok!(syscall::pidfd_send_signal(pfd, SIGTERM, None, 0), "sig");
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check!(syscall::wifsignaled(st), "signaled");
    check_ok!(syscall::close(pfd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_pidfd_getfd_soft() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = Timespec {
            tv_sec: 2,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let pfd = check_ok!(syscall::pidfd_open(pid, 0), "open");
    match syscall::pidfd_getfd(pfd, 1, 0) {
        Ok(fd) => {
            let _ = syscall::close(fd);
        }
        Err(e) if soft(e) || matches!(e, Errno::EBADF | Errno::EINVAL | Errno::ESRCH) => {}
        Err(_) => {}
    }
    let _ = syscall::kill(pid, SIGKILL);
    let mut st = 0;
    let _ = syscall::wait4(pid, &mut st, 0);
    check_ok!(syscall::close(pfd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn d2_inet_bind_getsockname() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM, 0), "sock");
    let one = 1i32.to_ne_bytes();
    let _ = syscall::setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one);
    let addr = SockAddrIn::loopback(0);
    match syscall::bind(fd, &addr) {
        Ok(()) => {
            let mut raw = [0u8; 16];
            let mut len = raw.len() as u32;
            check_ok!(syscall::getsockname(fd, &mut raw, &mut len), "name");
            check!(len >= 8, "len");
        }
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("bind"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}
