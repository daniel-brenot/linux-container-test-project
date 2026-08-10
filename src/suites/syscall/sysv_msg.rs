//! System V message queues (msgget/msgsnd/msgrcv/msgctl) with IPC_PRIVATE.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, MsgBuf, IPC_CREAT, IPC_PRIVATE, IPC_RMID};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::ENOMEM | Errno::ENOSPC | Errno::EINVAL
    )
}

#[crate::lctp_test(suite = syscall)]
fn sysv_msg_ipc_private_roundtrip() -> TestResult {
    let msqid = match syscall::msgget(IPC_PRIVATE, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("msgget")),
    };
    let mut msg = MsgBuf::default();
    msg.mtype = 1;
    msg.mtext[..4].copy_from_slice(b"ping");
    match syscall::msgsnd(msqid, &msg, 4, 0) {
        Ok(()) => {}
        Err(e) => {
            let _ = syscall::msgctl(msqid, IPC_RMID, 0);
            if soft(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("msgsnd"));
        }
    }
    let mut out = MsgBuf::default();
    let n = match syscall::msgrcv(msqid, &mut out, 32, 0, 0) {
        Ok(n) => n,
        Err(e) => {
            let _ = syscall::msgctl(msqid, IPC_RMID, 0);
            if soft(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("msgrcv"));
        }
    };
    check_eq!(n, 4, "len");
    check_eq!(out.mtype, 1, "mtype");
    check!(&out.mtext[..4] == b"ping", "payload");
    check_ok!(syscall::msgctl(msqid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_msg_type_filter() -> TestResult {
    let msqid = match syscall::msgget(IPC_PRIVATE, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("msgget")),
    };
    let mut m1 = MsgBuf::default();
    m1.mtype = 2;
    m1.mtext[0] = b'A';
    let mut m2 = MsgBuf::default();
    m2.mtype = 3;
    m2.mtext[0] = b'B';
    if syscall::msgsnd(msqid, &m1, 1, 0).is_err() || syscall::msgsnd(msqid, &m2, 1, 0).is_err() {
        let _ = syscall::msgctl(msqid, IPC_RMID, 0);
        return Ok(());
    }
    let mut out = MsgBuf::default();
    let n = check_ok!(syscall::msgrcv(msqid, &mut out, 32, 3, 0), "rcv type 3");
    check_eq!(n, 1, "len");
    check_eq!(out.mtype, 3, "type");
    check_eq!(out.mtext[0], b'B', "B");
    let n2 = check_ok!(syscall::msgrcv(msqid, &mut out, 32, 0, 0), "rcv remaining");
    check_eq!(n2, 1, "len2");
    check_eq!(out.mtype, 2, "type2");
    check_ok!(syscall::msgctl(msqid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_msg_rmid() -> TestResult {
    let msqid = match syscall::msgget(IPC_PRIVATE, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("msgget")),
    };
    check_ok!(syscall::msgctl(msqid, IPC_RMID, 0), "rmid");
    match syscall::msgctl(msqid, IPC_RMID, 0) {
        Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("second rmid ok")),
        Err(e) if e.0 == 43 => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("second rmid errno")),
    }
    Ok(())
}
