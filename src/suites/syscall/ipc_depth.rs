//! IPC depth: SysV, pipe/poll/epoll timeouts, eventfd edges.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, epoll, poll, Errno, EFD_CLOEXEC, EFD_NONBLOCK, EFD_SEMAPHORE, EPOLLIN, EPOLL_CTL_ADD,
    EPOLL_CTL_DEL, EPOLL_CTL_MOD, GETVAL, IPC_CREAT, IPC_PRIVATE, IPC_RMID, POLLIN, POLLOUT, SETVAL,
    Sembuf, MsgBuf,
};

fn soft_ipc(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::ENOMEM | Errno::ENOSPC | Errno::EINVAL
    )
}

#[crate::lctp_test(suite = syscall)]
fn ipc_shm_create_rmid() -> TestResult {
    match syscall::shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) {
        Ok(id) => {
            check_ok!(syscall::shmctl(id, IPC_RMID, 0), "rmid");
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("shmget")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_shm_attach_write() -> TestResult {
    let id = match syscall::shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) {
        Ok(i) => i,
        Err(e) if soft_ipc(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("shmget")),
    };
    let addr = match syscall::shmat(id, 0, 0) {
        Ok(a) => a,
        Err(e) => {
            let _ = syscall::shmctl(id, IPC_RMID, 0);
            if soft_ipc(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat"));
        }
    };
    unsafe {
        *(addr as *mut u32) = 0xDEAD_BEEF;
        check_eq!(*(addr as *const u32), 0xDEAD_BEEF, "val");
    }
    check_ok!(syscall::shmdt(addr), "dt");
    check_ok!(syscall::shmctl(id, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_shm_rdonly_attach_soft() -> TestResult {
    let id = match syscall::shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) {
        Ok(i) => i,
        Err(e) if soft_ipc(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("shmget")),
    };
    match syscall::shmat(id, 0, syscall::SHM_RDONLY) {
        Ok(a) => {
            check_ok!(syscall::shmdt(a), "dt");
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => {
            let _ = syscall::shmctl(id, IPC_RMID, 0);
            return Err(crate::harness::AssertFail::msg("shmat ro"));
        }
    }
    let _ = syscall::shmctl(id, IPC_RMID, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_sem_get_setval() -> TestResult {
    let id = match syscall::semget(IPC_PRIVATE, 1, IPC_CREAT | 0o600) {
        Ok(i) => i,
        Err(e) if soft_ipc(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("semget")),
    };
    match syscall::semctl(id, 0, SETVAL, 3) {
        Ok(_) => {
            let v = check_ok!(syscall::semctl(id, 0, GETVAL, 0), "getval");
            check_eq!(v, 3, "val");
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => {
            let _ = syscall::semctl(id, 0, IPC_RMID, 0);
            return Err(crate::harness::AssertFail::msg("setval"));
        }
    }
    let _ = syscall::semctl(id, 0, IPC_RMID, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_semop_add_sub() -> TestResult {
    let id = match syscall::semget(IPC_PRIVATE, 1, IPC_CREAT | 0o600) {
        Ok(i) => i,
        Err(e) if soft_ipc(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("semget")),
    };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() {
        let _ = syscall::semctl(id, 0, IPC_RMID, 0);
        return Ok(());
    }
    let mut ops = [Sembuf {
        sem_num: 0,
        sem_op: -1,
        sem_flg: 0,
    }];
    match syscall::semop(id, &mut ops) {
        Ok(()) => {
            ops[0].sem_op = 1;
            let _ = syscall::semop(id, &mut ops);
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => {
            let _ = syscall::semctl(id, 0, IPC_RMID, 0);
            return Err(crate::harness::AssertFail::msg("semop"));
        }
    }
    let _ = syscall::semctl(id, 0, IPC_RMID, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_msg_send_recv() -> TestResult {
    let id = match syscall::msgget(IPC_PRIVATE, IPC_CREAT | 0o600) {
        Ok(i) => i,
        Err(e) if soft_ipc(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("msgget")),
    };
    let mut msg = MsgBuf::default();
    msg.mtype = 1;
    msg.mtext[..4].copy_from_slice(b"ping");
    match syscall::msgsnd(id, &msg, 4, 0) {
        Ok(()) => {
            let mut out = MsgBuf::default();
            let n = check_ok!(syscall::msgrcv(id, &mut out, 32, 1, 0), "rcv");
            check_eq!(n, 4, "n");
            check_eq!(&out.mtext[..4], b"ping", "d");
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => {
            let _ = syscall::msgctl(id, IPC_RMID, 0);
            return Err(crate::harness::AssertFail::msg("msgsnd"));
        }
    }
    let _ = syscall::msgctl(id, IPC_RMID, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_msg_two_types() -> TestResult {
    let id = match syscall::msgget(IPC_PRIVATE, IPC_CREAT | 0o600) {
        Ok(i) => i,
        Err(e) if soft_ipc(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("msgget")),
    };
    let mut m1 = MsgBuf {
        mtype: 1,
        mtext: [0; 32],
    };
    m1.mtext[0] = b'A';
    let mut m2 = MsgBuf {
        mtype: 2,
        mtext: [0; 32],
    };
    m2.mtext[0] = b'B';
    if syscall::msgsnd(id, &m1, 1, 0).is_err() || syscall::msgsnd(id, &m2, 1, 0).is_err() {
        let _ = syscall::msgctl(id, IPC_RMID, 0);
        return Ok(());
    }
    let mut out = MsgBuf::default();
    let n = check_ok!(syscall::msgrcv(id, &mut out, 32, 2, 0), "rcv2");
    check_eq!(n, 1, "n");
    check_eq!(out.mtext[0], b'B', "B");
    let n = check_ok!(syscall::msgrcv(id, &mut out, 32, 1, 0), "rcv1");
    check_eq!(out.mtext[0], b'A', "A");
    let _ = n;
    let _ = syscall::msgctl(id, IPC_RMID, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_pipe_poll_timeout_0() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut fds = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut fds, 0), "poll");
    check_eq!(n, 0, "empty");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_pipe_poll_timeout_1ms() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut fds = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut fds, 1), "poll");
    check_eq!(n, 0, "timeout");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_pipe_poll_readable() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::write(w, b"x"), "w");
    let mut fds = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut fds, 0), "poll");
    check_eq!(n, 1, "ready");
    check!(fds[0].revents & POLLIN != 0, "in");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_pipe_poll_pollout() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut fds = [poll::PollFd {
        fd: w,
        events: POLLOUT,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut fds, 0), "poll");
    check!(n >= 1, "out");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_epoll_timeout_0() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: r as u64,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 0), "wait");
    check_eq!(n, 0, "timeout");
    check_ok!(syscall::close(ep), "ep");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_epoll_timeout_1() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 1), "wait");
    check_eq!(n, 0, "to");
    check_ok!(syscall::close(ep), "ep");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_epoll_ready_after_write() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 42,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::write(w, b"z"), "w");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 0), "wait");
    check_eq!(n, 1, "ready");
    check_eq!(out[0].data, 42, "data");
    check_ok!(syscall::close(ep), "ep");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_epoll_mod_del() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 1,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    ev.data = 2;
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_MOD, r, &mut ev), "mod");
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_DEL, r, &mut ev), "del");
    check_ok!(syscall::close(ep), "ep");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_eventfd_nonblock_eagain() -> TestResult {
    let efd = check_ok!(syscall::eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK), "efd");
    let mut buf = [0u8; 8];
    check_err!(syscall::read(efd, &mut buf), Errno::EAGAIN, "eagain");
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_eventfd_semaphore_soft() -> TestResult {
    let efd = match syscall::eventfd(2, EFD_SEMAPHORE | EFD_CLOEXEC) {
        Ok(f) => f,
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("efd sem")),
    };
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::read(efd, &mut buf), "r"), 8, "n");
    check_eq!(u64::from_le_bytes(buf), 1, "sem dec");
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_eventfd_write_read_roundtrip() -> TestResult {
    let efd = check_ok!(syscall::eventfd(0, EFD_CLOEXEC), "efd");
    let v = 7u64.to_le_bytes();
    check_ok!(syscall::write(efd, &v), "w");
    let mut out = [0u8; 8];
    check_ok!(syscall::read(efd, &mut out), "r");
    check_eq!(u64::from_le_bytes(out), 7, "v");
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_pipe_epoll_hup_on_close_write() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let ep = check_ok!(syscall::epoll_create1(0), "ep");
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN,
        data: 0,
    };
    check_ok!(syscall::epoll_ctl(ep, EPOLL_CTL_ADD, r, &mut ev), "add");
    check_ok!(syscall::close(w), "cw");
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 1];
    let n = check_ok!(syscall::epoll_wait(ep, &mut out, 100), "wait");
    check!(n >= 1, "event");
    check_ok!(syscall::close(ep), "ep");
    check_ok!(syscall::close(r), "r");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_poll_two_fds() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::write(w, b"a"), "w");
    let mut fds = [
        poll::PollFd {
            fd: r,
            events: POLLIN,
            revents: 0,
        },
        poll::PollFd {
            fd: w,
            events: POLLOUT,
            revents: 0,
        },
    ];
    let n = check_ok!(syscall::poll(&mut fds, 0), "poll");
    check!(n >= 1, "ready");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn ipc_shm_large_soft() -> TestResult {
    match syscall::shmget(IPC_PRIVATE, 65536, IPC_CREAT | 0o600) {
        Ok(id) => {
            check_ok!(syscall::shmctl(id, IPC_RMID, 0), "rmid");
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("shm large")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_sem_nsems_2() -> TestResult {
    match syscall::semget(IPC_PRIVATE, 2, IPC_CREAT | 0o600) {
        Ok(id) => {
            let _ = syscall::semctl(id, 0, SETVAL, 0);
            let _ = syscall::semctl(id, 1, SETVAL, 1);
            let _ = syscall::semctl(id, 0, IPC_RMID, 0);
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("sem 2")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_eventfd_init_nonzero() -> TestResult {
    let efd = check_ok!(syscall::eventfd(9, EFD_CLOEXEC), "efd");
    let mut out = [0u8; 8];
    check_ok!(syscall::read(efd, &mut out), "r");
    check_eq!(u64::from_le_bytes(out), 9, "init");
    check_ok!(syscall::close(efd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_epoll_create1_cloexec() -> TestResult {
    let ep = check_ok!(syscall::epoll_create1(syscall::oflag::O_CLOEXEC), "ep");
    let flags = check_ok!(syscall::fcntl(ep, syscall::fcntl_cmd::F_GETFD, 0), "fd");
    check!(flags & syscall::FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(ep), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_pipe2_direct_roundtrip() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    check_ok!(syscall::write(w, b"ipc"), "w");
    let mut b = [0u8; 3];
    check_eq!(check_ok!(syscall::read(r, &mut b), "r"), 3, "n");
    check_eq!(&b, b"ipc", "d");
    check_ok!(syscall::close(r), "cr");
    check_ok!(syscall::close(w), "cw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn ipc_poll_timeout_10ms() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let mut fds = [poll::PollFd {
        fd: r,
        events: POLLIN,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut fds, 10), "poll");
    check_eq!(n, 0, "to");
    check_ok!(syscall::close(r), "r");
    check_ok!(syscall::close(w), "w");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn ipc_msg_rmid() -> TestResult {
    match syscall::msgget(IPC_PRIVATE, IPC_CREAT | 0o600) {
        Ok(id) => {
            check_ok!(syscall::msgctl(id, IPC_RMID, 0), "rmid");
        }
        Err(e) if soft_ipc(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("msgget")),
    }
    Ok(())
}
