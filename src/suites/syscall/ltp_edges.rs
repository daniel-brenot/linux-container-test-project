//! Extra LTP-ish edge cases across file/process/mem/net/signal/misc.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::create_empty;
use crate::syscall::{
    self, clock, fcntl_cmd, map, oflag, poll, prot, wait, Errno, Flock, IoVec, Rlimit, Timespec,
    AF_INET, FD_CLOEXEC, F_RDLCK, F_UNLCK, LOCK_EX, LOCK_UN, MSG_DONTWAIT, POLLIN,
    RLIMIT_NOFILE, RUSAGE_SELF, SEEK_CUR, SEEK_DATA, SEEK_END, SEEK_HOLE, SEEK_SET, SOCK_CLOEXEC,
    SOCK_DGRAM, SOCK_NONBLOCK, SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR, SO_TYPE, SYNC_FILE_RANGE_WRITE,
    SIGUSR1, SIG_BLOCK, SIG_DFL, SIG_IGN, SIG_SETMASK, Sigaction, SockAddrIn,
};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::EINVAL | Errno::ENOSYS | Errno::EPERM | Errno::EOPNOTSUPP | Errno::ENOTSUP | Errno::ENOMEM
    )
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_dupfd_cloexec_min_0() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD_CLOEXEC, 0), "d") as i32;
    check_ok!(syscall::close(fd), "cf");
    check_ok!(syscall::close(d), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_getfl_rdwr() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "g");
    check!(fl as i32 & 3 == oflag::O_RDWR, "rdwr");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_setlk_read_whole() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"abc"), "w");
    let mut lk = Flock {
        l_type: F_RDLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "lk");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_setlkw_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let mut lk = Flock {
        l_type: F_RDLCK,
        l_whence: SEEK_SET as i16,
        l_start: 0,
        l_len: 1,
        l_pid: 0,
    };
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLKW, &mut lk), "lk");
    lk.l_type = F_UNLCK;
    check_ok!(syscall::fcntl_flock(fd, fcntl_cmd::F_SETLK, &mut lk), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_lseek_data_empty_file_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
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

#[crate::lctp_test(suite = syscall)]
fn edge_lseek_hole_empty_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    // Empty files: SEEK_HOLE may return 0, ENXIO, EINVAL, or ENOSYS depending on FS/kernel.
    match syscall::lseek(fd, 0, SEEK_HOLE) {
        Ok(_) | Err(Errno::EINVAL) | Err(Errno::ENOSYS) | Err(Errno::ENXIO) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("seek hole"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_preadv_two_then_stat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"WXYZ"), "w");
    let mut a = [0u8; 2];
    let mut b = [0u8; 2];
    let mut iov = [
        IoVec {
            iov_base: a.as_mut_ptr(),
            iov_len: 2,
        },
        IoVec {
            iov_base: b.as_mut_ptr(),
            iov_len: 2,
        },
    ];
    check_eq!(check_ok!(syscall::preadv(fd, &mut iov, 0), "pv"), 4, "n");
    check_eq!(&a, b"WX", "a");
    check_eq!(&b, b"YZ", "b");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_pwritev_scatter_4() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let p = [b"a", b"b", b"c", b"d"];
    let mut iov = [
        IoVec {
            iov_base: p[0].as_ptr() as *mut u8,
            iov_len: 1,
        },
        IoVec {
            iov_base: p[1].as_ptr() as *mut u8,
            iov_len: 1,
        },
        IoVec {
            iov_base: p[2].as_ptr() as *mut u8,
            iov_len: 1,
        },
        IoVec {
            iov_base: p[3].as_ptr() as *mut u8,
            iov_len: 1,
        },
    ];
    check_eq!(check_ok!(syscall::pwritev(fd, &mut iov, 0), "pw"), 4, "n");
    let mut out = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut out, 0), "r");
    check_eq!(&out, b"abcd", "d");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_sync_file_range_mid() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"0123456789"), "w");
    check_ok!(
        syscall::sync_file_range(fd, 2, 4, SYNC_FILE_RANGE_WRITE),
        "sfr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fallocate_1k_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    match syscall::fallocate(fd, 0, 0, 1024) {
        Ok(()) => {
            let st = check_ok!(syscall::fstat(fd), "st");
            check_eq!(st.st_size, 1024, "sz");
        }
        Err(e) if soft(e) || e == Errno::ENOSPC => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("falloc"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_copy_file_range_one_byte() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"in")?;
    let wfd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "o");
    check_ok!(syscall::write(wfd, b"Z"), "w");
    check_ok!(syscall::close(wfd), "cw");
    let in_fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "in");
    let out_path = check_ok!(tmp.child(b"out"), "outp");
    let out_fd = check_ok!(
        syscall::open(
            out_path,
            oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC,
            0o644
        ),
        "out"
    );
    let mut oi = 0i64;
    let mut oo = 0i64;
    let n = check_ok!(
        syscall::copy_file_range(in_fd, Some(&mut oi), out_fd, Some(&mut oo), 1, 0),
        "cfr"
    );
    check_eq!(n, 1, "n");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(out_fd), "co");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_sendfile_count_2() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"sf")?;
    let wfd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "w");
    check_ok!(syscall::write(wfd, b"ABCD"), "wr");
    check_ok!(syscall::close(wfd), "cw");
    let in_fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "in");
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    let mut off = 1i64;
    check_eq!(
        check_ok!(syscall::sendfile(w, in_fd, &mut off, 2), "sf"),
        2,
        "n"
    );
    check_ok!(syscall::close(w), "cw2");
    check_ok!(syscall::close(in_fd), "ci");
    let mut b = [0u8; 2];
    check_ok!(syscall::read(r, &mut b), "r");
    check_eq!(&b, b"BC", "d");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_tee_size_8() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    check_ok!(syscall::write(w1, b"12345678"), "w");
    check_eq!(check_ok!(syscall::tee(r1, w2, 8, 0), "tee"), 8, "n");
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_splice_size_4() -> TestResult {
    let (r1, w1) = check_ok!(syscall::pipe2(0), "p1");
    let (r2, w2) = check_ok!(syscall::pipe2(0), "p2");
    check_ok!(syscall::write(w1, b"abcd"), "w");
    check_eq!(
        check_ok!(syscall::splice(r1, None, w2, None, 4, 0), "sp"),
        4,
        "n"
    );
    check_ok!(syscall::close(w1), "cw1");
    check_ok!(syscall::close(w2), "cw2");
    check_ok!(syscall::close(r1), "cr1");
    check_ok!(syscall::close(r2), "cr2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_waitpid_exit_status_matrix_small() -> TestResult {
    for code in [0i32, 3, 100] {
        let pid = check_ok!(syscall::fork(), "f");
        if pid == 0 {
            syscall::exit(code);
        }
        let mut st = 0;
        check_ok!(syscall::waitpid(pid, &mut st, 0), "w");
        check_eq!(syscall::wexitstatus(st), code, "c");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_wait4_wnohang_running() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let req = Timespec {
            tv_sec: 0,
            tv_nsec: 20_000_000,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let mut st = 0;
    match syscall::wait4(pid, &mut st, wait::WNOHANG) {
        Ok(0) => {}
        Ok(p) if p == pid => {}
        Err(Errno::ECHILD) => {}
        Ok(_) | Err(_) => {
            let _ = syscall::wait4(pid, &mut st, 0);
            return Err(crate::harness::AssertFail::msg("nohang"));
        }
    }
    let _ = syscall::wait4(pid, &mut st, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_getpgid_after_setpgid_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        let me = syscall::getpid();
        if syscall::setpgid(0, me).is_ok() {
            if syscall::getpgid(0).ok() == Some(me) {
                syscall::exit(0);
            }
        }
        syscall::exit(1);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    check_eq!(syscall::wexitstatus(st), 0, "ok");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_prctl_dumpable_toggle() -> TestResult {
    let old = check_ok!(
        syscall::prctl(syscall::PR_GET_DUMPABLE, 0, 0, 0, 0),
        "g"
    );
    check_ok!(syscall::prctl(syscall::PR_SET_DUMPABLE, 1, 0, 0, 0), "s1");
    check_ok!(syscall::prctl(syscall::PR_SET_DUMPABLE, 0, 0, 0, 0), "s0");
    let _ = syscall::prctl(syscall::PR_SET_DUMPABLE, old as usize, 0, 0, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_rlimit_nofile_get_twice() -> TestResult {
    let mut a = Rlimit::default();
    let mut b = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, None, Some(&mut a)),
        "a"
    );
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, None, Some(&mut b)),
        "b"
    );
    check_eq!(a.rlim_cur, b.rlim_cur, "cur");
    check_eq!(a.rlim_max, b.rlim_max, "max");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_getrusage_self_nvcsw() -> TestResult {
    let ru = check_ok!(syscall::getrusage(RUSAGE_SELF), "ru");
    check!(ru.ru_nvcsw >= 0, "nvcsw");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_mmap_prot_write_only_soft() -> TestResult {
    match syscall::mmap(
        0,
        4096,
        prot::PROT_WRITE,
        map::MAP_PRIVATE | map::MAP_ANONYMOUS,
        -1,
        0,
    ) {
        Ok(a) => check_ok!(syscall::munmap(a, 4096), "u"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("mmap w")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_madvise_all_common() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            4096,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    for adv in [
        syscall::madvise::MADV_NORMAL,
        syscall::madvise::MADV_RANDOM,
        syscall::madvise::MADV_SEQUENTIAL,
        syscall::madvise::MADV_WILLNEED,
        syscall::madvise::MADV_DONTNEED,
    ] {
        check_ok!(syscall::madvise(addr, 4096, adv), "adv");
    }
    check_ok!(syscall::munmap(addr, 4096), "u");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_mremap_grow_4_to_8() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            4096,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    let n = check_ok!(
        syscall::mremap(addr, 4096, 8192, syscall::MREMAP_MAYMOVE, 0),
        "r"
    );
    check_ok!(syscall::munmap(n, 8192), "u");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_mincore_after_touch() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            4096,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    unsafe {
        core::ptr::write_bytes(addr as *mut u8, 0x11, 4096);
    }
    let mut v = [0u8; 1];
    check_ok!(syscall::mincore(addr, 4096, &mut v), "mc");
    check!(v[0] & 1 != 0, "res");
    check_ok!(syscall::munmap(addr, 4096), "u");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_brk_grow_one_page_soft() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "q");
    match syscall::brk(cur + 4096) {
        Ok(_) => {
            let _ = syscall::brk(cur);
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("brk")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_clock_gettime_boottime() -> TestResult {
    match syscall::clock_gettime(clock::CLOCK_BOOTTIME) {
        Ok(t) => check!(t.tv_sec >= 0, "sec"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("boot")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_clock_getres_monotonic() -> TestResult {
    let r = check_ok!(syscall::clock_getres(clock::CLOCK_MONOTONIC), "r");
    check!(r.tv_nsec > 0 || r.tv_sec > 0, "nz");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_nanosleep_100us() -> TestResult {
    let req = Timespec {
        tv_sec: 0,
        tv_nsec: 100_000,
    };
    check_ok!(syscall::nanosleep(&req), "ns");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_timerfd_abstime_future() -> TestResult {
    let fd = check_ok!(
        syscall::timerfd_create(clock::CLOCK_MONOTONIC, syscall::TFD_CLOEXEC),
        "t"
    );
    let now = check_ok!(syscall::clock_gettime(clock::CLOCK_MONOTONIC), "n");
    let its = syscall::Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: Timespec {
            tv_sec: now.tv_sec + 100,
            tv_nsec: 0,
        },
    };
    check_ok!(
        syscall::timerfd_settime(fd, syscall::TFD_TIMER_ABSTIME, &its),
        "set"
    );
    check_ok!(
        syscall::timerfd_settime(fd, 0, &syscall::Itimerspec::default()),
        "clr"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_udp_dontwait() -> TestResult {
    let fd = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0),
        "s"
    );
    check_ok!(syscall::bind(fd, &SockAddrIn::loopback(0)), "b");
    let mut buf = [0u8; 4];
    check_err!(
        syscall::recv(fd, &mut buf, MSG_DONTWAIT),
        Errno::EAGAIN,
        "eagain"
    );
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_tcp_shutdown_rdwr_pair() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    let one = 1i32.to_ne_bytes();
    check_ok!(
        syscall::setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one),
        "r"
    );
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "b");
    check_ok!(syscall::listen(srv, 1), "l");
    let bound = check_ok!(syscall::getsockname_in(srv), "n");
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "c");
    check_ok!(syscall::connect(cli, &bound), "conn");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "a");
    check_ok!(syscall::shutdown(cli, syscall::SHUT_RDWR), "sh");
    check_ok!(syscall::close(acc), "ca");
    check_ok!(syscall::close(cli), "cc");
    check_ok!(syscall::close(srv), "cs");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_so_type_stream() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    let mut val = [0u8; 4];
    check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_TYPE, &mut val), "g");
    check_eq!(i32::from_ne_bytes(val), SOCK_STREAM, "t");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_kill_zero_self() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "k");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_sigaction_ign_usr1() -> TestResult {
    let mut old = Sigaction::default();
    let mut neu = Sigaction {
        sa_handler: SIG_IGN,
        ..Sigaction::default()
    };
    check_ok!(
        syscall::rt_sigaction(SIGUSR1, Some(&neu), Some(&mut old)),
        "ign"
    );
    neu.sa_handler = SIG_DFL;
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&neu), None), "dfl");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_sigprocmask_block_usr1() -> TestResult {
    let bit = 1u64 << (SIGUSR1 - 1);
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(bit), Some(&mut old)),
        "b"
    );
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "r");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_poll_pipe_timeout_0() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    let mut fds = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    check_eq!(check_ok!(syscall::poll(&mut fds, 0), "poll"), 0, "n");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_eventfd_nonblock() -> TestResult {
    let efd = check_ok!(
        syscall::eventfd(0, syscall::EFD_NONBLOCK | syscall::EFD_CLOEXEC),
        "e"
    );
    let mut b = [0u8; 8];
    check_err!(syscall::read(efd, &mut b), Errno::EAGAIN, "eagain");
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_uname_machine_nonempty() -> TestResult {
    let u = check_ok!(syscall::uname(), "u");
    let end = u.machine.iter().position(|&b| b == 0).unwrap_or(0);
    check!(end > 0, "mach");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_getrandom_nonblock() -> TestResult {
    let mut b = [0u8; 4];
    match syscall::getrandom(&mut b, syscall::GRND_NONBLOCK) {
        Ok(n) => check!(n > 0, "n"),
        Err(Errno::EAGAIN) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("gr")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_ioctl_enotty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let mut ws = syscall::Winsize::default();
    match syscall::ioctl(fd, syscall::TIOCGWINSZ, &mut ws as *mut _ as usize) {
        Err(Errno::ENOTTY) | Err(Errno::EINVAL) => {}
        Ok(_) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("ioctl"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_dup3_cloexec_flag() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let d = check_ok!(syscall::dup3(fd, 91, oflag::O_CLOEXEC), "d");
    let fl = check_ok!(syscall::fcntl(d, fcntl_cmd::F_GETFD, 0), "g");
    check!(fl & FD_CLOEXEC as usize != 0, "ce");
    check_ok!(syscall::close(fd), "cf");
    check_ok!(syscall::close(d), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_flock_ex_nb_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::flock(fd, LOCK_EX | syscall::LOCK_NB), "lk");
    check_ok!(syscall::flock(fd, LOCK_UN), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_membarrier_query() -> TestResult {
    check_ok!(
        syscall::membarrier(syscall::MEMBARRIER_CMD_QUERY, 0),
        "q"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_personality_query() -> TestResult {
    let p = check_ok!(syscall::personality(0xffff_ffff), "p");
    let _ = p;
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_capget() -> TestResult {
    let mut hdr = syscall::CapUserHeader {
        version: syscall::LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let mut data = [syscall::CapUserData::default(); 2];
    check_ok!(syscall::capget(&mut hdr, &mut data), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_sched_yield() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_getcpu() -> TestResult {
    let mut cpu = 0u32;
    check_ok!(syscall::getcpu(Some(&mut cpu), None), "g");
    check!(cpu < 4096, "cpu");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_kcmp_self_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let pid = syscall::getpid();
    match syscall::kcmp(pid, pid, syscall::KCMP_FILE, fd as u64, fd as u64) {
        Ok(0) => {}
        Err(e) if soft(e) => {}
        Ok(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("kcmp"));
        }
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("kcmp e"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_lseek_end_zero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"hi"), "w");
    check_eq!(check_ok!(syscall::lseek(fd, 0, SEEK_END), "e"), 2, "end");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_lseek_cur_forward() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"0123"), "w");
    check_ok!(syscall::lseek(fd, 0, SEEK_SET), "s");
    check_eq!(check_ok!(syscall::lseek(fd, 2, SEEK_CUR), "c"), 2, "pos");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_setfl_append() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let fl = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "g");
    check_ok!(
        syscall::fcntl(
            fd,
            fcntl_cmd::F_SETFL,
            (fl as i32 | oflag::O_APPEND) as usize
        ),
        "s"
    );
    let fl2 = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFL, 0), "g2");
    check!(fl2 as i32 & oflag::O_APPEND != 0, "ap");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_setfl_nonblock_pipe() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    let fl = check_ok!(syscall::fcntl(r, fcntl_cmd::F_GETFL, 0), "g");
    check_ok!(
        syscall::fcntl(
            r,
            fcntl_cmd::F_SETFL,
            (fl as i32 | oflag::O_NONBLOCK) as usize
        ),
        "s"
    );
    let mut b = [0u8; 1];
    check_err!(syscall::read(r, &mut b), Errno::EAGAIN, "e");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn edge_sendfile_64() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let data = [b'X'; 64];
    let path = create_empty(&mut tmp, b"sf64")?;
    let wfd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "w");
    check_ok!(syscall::write(wfd, &data), "wr");
    check_ok!(syscall::close(wfd), "cw");
    let in_fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "in");
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    let mut off = 0i64;
    check_eq!(
        check_ok!(syscall::sendfile(w, in_fd, &mut off, 64), "sf"),
        64,
        "n"
    );
    check_ok!(syscall::close(w), "cw2");
    check_ok!(syscall::close(in_fd), "ci");
    check_ok!(syscall::close(r), "cr");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn edge_mmap_16k() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            16384,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    unsafe {
        *(addr as *mut u8) = 1;
        *((addr + 16383) as *mut u8) = 2;
    }
    check_ok!(syscall::munmap(addr, 16384), "u");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_waitid_exited_status() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        syscall::exit(15);
    }
    let mut info = syscall::Siginfo::default();
    check_ok!(
        syscall::waitid(syscall::P_PID, pid, &mut info, wait::WEXITED),
        "wi"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_setsid_child_leader() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 {
        match syscall::setsid() {
            Ok(s) if s == syscall::getpid() => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    check_eq!(syscall::wexitstatus(st), 0, "ok");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_prctl_no_new_privs() -> TestResult {
    check_ok!(
        syscall::prctl(syscall::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0),
        "set"
    );
    let v = check_ok!(
        syscall::prctl(syscall::PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0),
        "get"
    );
    check_eq!(v, 1, "nnp");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_mlock_soft() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            4096,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m"
    );
    match syscall::mlock(addr, 4096) {
        Ok(()) => {
            let _ = syscall::munlock(addr, 4096);
        }
        Err(e) if soft(e) || e == Errno::EAGAIN => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("mlock"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "u");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_itimer_disarm() -> TestResult {
    let zero = syscall::Itimerval::default();
    check_ok!(syscall::setitimer(syscall::ITIMER_REAL, &zero, None), "clr");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_close_range_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let d = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_DUPFD, 120), "d") as i32;
    match syscall::close_range(d as u32, d as u32, 0) {
        Ok(()) => check_err!(syscall::close(d), Errno::EBADF, "gone"),
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) => check_ok!(syscall::close(d), "cd"),
        Err(_) => {
            let _ = syscall::close(d);
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("cr"));
        }
    }
    check_ok!(syscall::close(fd), "cf");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_getfd_zero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let flags = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "g");
    check!(flags & FD_CLOEXEC as usize == 0, "no ce");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fcntl_setfd_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::fcntl(fd, fcntl_cmd::F_SETFD, FD_CLOEXEC as usize), "s");
    let flags = check_ok!(syscall::fcntl(fd, fcntl_cmd::F_GETFD, 0), "g");
    check!(flags & FD_CLOEXEC as usize != 0, "ce");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_pread_offset_1() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"ABCD"), "w");
    let mut b = [0u8; 2];
    check_eq!(check_ok!(syscall::pread(fd, &mut b, 1), "r"), 2, "n");
    check_eq!(&b, b"BC", "d");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_pwrite_offset_2() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::pwrite(fd, b"ZZ", 2), "pw");
    let mut b = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut b, 0), "r");
    check_eq!(&b[2..4], b"ZZ", "d");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_ftruncate_zero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"data"), "w");
    check_ok!(syscall::ftruncate(fd, 0), "tr");
    let st = check_ok!(syscall::fstat(fd), "st");
    check_eq!(st.st_size, 0, "sz");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fsync_empty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::fsync(fd), "fs");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fdatasync_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::write(fd, b"x"), "w");
    check_ok!(syscall::fdatasync(fd), "fds");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_pipe2_cloexec() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(oflag::O_CLOEXEC), "p");
    let fl = check_ok!(syscall::fcntl(r, fcntl_cmd::F_GETFD, 0), "g");
    check!(fl & FD_CLOEXEC as usize != 0, "ce");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_socketpair_stream() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(syscall::AF_UNIX, SOCK_STREAM, 0), "sp");
    check_ok!(syscall::send(a, b"z", 0), "s");
    let mut buf = [0u8; 1];
    check_eq!(check_ok!(syscall::recv(b, &mut buf, 0), "r"), 1, "n");
    check_ok!(syscall::close(a), "a");
    check_ok!(syscall::close(b), "b");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_epoll_create_add_del() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "p");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = syscall::epoll::EpollEvent { events: syscall::EPOLLIN, data: 1 };
    check_ok!(syscall::epoll_ctl(ep, syscall::EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::epoll_ctl(ep, syscall::EPOLL_CTL_DEL, r, &mut ev), "del");
    check_ok!(syscall::close(ep), "ep");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_clock_thread_cputime() -> TestResult {
    match syscall::clock_gettime(clock::CLOCK_THREAD_CPUTIME_ID) {
        Ok(t) => check!(t.tv_sec >= 0, "sec"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("tcpu")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_clock_realtime_coarse() -> TestResult {
    match syscall::clock_gettime(clock::CLOCK_REALTIME_COARSE) {
        Ok(t) => check!(t.tv_sec > 0, "sec"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("coarse")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_gettimeofday_positive() -> TestResult {
    let tv = check_ok!(syscall::gettimeofday(), "g");
    check!(tv.tv_sec > 1_600_000_000, "sec");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_sched_getaffinity_mask() -> TestResult {
    let mut mask = [0u8; 64];
    match syscall::sched_getaffinity(0, &mut mask) {
        Ok(()) => check!(mask.iter().any(|&b| b != 0), "bits"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("aff")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_getpriority_self() -> TestResult {
    let p = check_ok!(syscall::getpriority(syscall::PRIO_PROCESS, 0), "p");
    check!(p >= -20 && p <= 19, "rng");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_dup_then_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    let d = check_ok!(syscall::dup(fd), "d");
    check_ok!(syscall::write(d, b"hi"), "w");
    let mut b = [0u8; 2];
    check_ok!(syscall::pread(fd, &mut b, 0), "r");
    check_eq!(&b, b"hi", "d");
    check_ok!(syscall::close(fd), "cf");
    check_ok!(syscall::close(d), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_mprotect_read_only() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(0, 4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "m"
    );
    check_ok!(syscall::mprotect(addr, 4096, prot::PROT_READ), "ro");
    check_ok!(syscall::munmap(addr, 4096), "u");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_msync_async_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"f", 0o644), "c");
    check_ok!(syscall::ftruncate(fd, 4096), "tr");
    let addr = check_ok!(
        syscall::mmap(0, 4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0),
        "m"
    );
    unsafe { *(addr as *mut u8) = 1 };
    check_ok!(syscall::msync(addr, 4096, syscall::MS_ASYNC), "ms");
    check_ok!(syscall::munmap(addr, 4096), "u");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_fork_exit_1() -> TestResult {
    let pid = check_ok!(syscall::fork(), "f");
    if pid == 0 { syscall::exit(1); }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "w");
    check_eq!(syscall::wexitstatus(st), 1, "st");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_gettid_positive() -> TestResult {
    check!(syscall::gettid() > 0, "tid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_getppid_nonneg() -> TestResult {
    check!(syscall::getppid() >= 0, "ppid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_uid_gid_match() -> TestResult {
    check_eq!(syscall::getuid(), syscall::geteuid(), "uid");
    check_eq!(syscall::getgid(), syscall::getegid(), "gid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_openat_creat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let dirfd = check_ok!(syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0), "d");
    let fd = check_ok!(syscall::openat(dirfd, b"x\0", oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644), "o");
    check_ok!(syscall::close(fd), "cf");
    check_ok!(syscall::close(dirfd), "cd");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_access_f_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"a")?;
    check_ok!(syscall::access(&path, syscall::F_OK), "fok");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn edge_stat_size_after_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"s")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "o");
    check_ok!(syscall::write(fd, b"12345"), "w");
    check_ok!(syscall::close(fd), "c");
    let st = check_ok!(syscall::stat(&path), "st");
    check_eq!(st.st_size, 5, "sz");
    Ok(())
}
