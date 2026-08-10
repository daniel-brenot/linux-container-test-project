//! System V semaphores (semget/semop/semctl) with IPC_PRIVATE.

use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, Sembuf, GETVAL, IPC_CREAT, IPC_PRIVATE, IPC_RMID, SETVAL};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::ENOMEM | Errno::ENOSPC | Errno::EINVAL
    )
}

#[crate::lctp_test(suite = syscall)]
fn sysv_sem_ipc_private_setval_getval() -> TestResult {
    let semid = match syscall::semget(IPC_PRIVATE, 1, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("semget")),
    };
    match syscall::semctl(semid, 0, SETVAL, 3usize) {
        Ok(_) => {}
        Err(e) => {
            let _ = syscall::semctl(semid, 0, IPC_RMID, 0);
            if soft(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("SETVAL"));
        }
    }
    let v = match syscall::semctl(semid, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) => {
            let _ = syscall::semctl(semid, 0, IPC_RMID, 0);
            if soft(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("GETVAL"));
        }
    };
    check_eq!(v, 3, "val");
    check_ok!(syscall::semctl(semid, 0, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysv_sem_op_down_up() -> TestResult {
    let semid = match syscall::semget(IPC_PRIVATE, 1, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("semget")),
    };
    if syscall::semctl(semid, 0, SETVAL, 1usize).is_err() {
        let _ = syscall::semctl(semid, 0, IPC_RMID, 0);
        return Ok(());
    }
    let down = [Sembuf {
        sem_num: 0,
        sem_op: -1,
        sem_flg: 0,
    }];
    match syscall::semop(semid, &down) {
        Ok(()) => {}
        Err(e) => {
            let _ = syscall::semctl(semid, 0, IPC_RMID, 0);
            if soft(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("semop down"));
        }
    }
    let v = check_ok!(syscall::semctl(semid, 0, GETVAL, 0), "getval");
    check_eq!(v, 0, "after down");
    let up = [Sembuf {
        sem_num: 0,
        sem_op: 1,
        sem_flg: 0,
    }];
    check_ok!(syscall::semop(semid, &up), "semop up");
    let v2 = check_ok!(syscall::semctl(semid, 0, GETVAL, 0), "getval2");
    check_eq!(v2, 1, "after up");
    check_ok!(syscall::semctl(semid, 0, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_sem_rmid_idempotent() -> TestResult {
    let semid = match syscall::semget(IPC_PRIVATE, 1, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("semget")),
    };
    check_ok!(syscall::semctl(semid, 0, IPC_RMID, 0), "rmid");
    match syscall::semctl(semid, 0, IPC_RMID, 0) {
        Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("second rmid ok")),
        Err(e) if e.0 == 43 => {} // EIDRM
        Err(_) => return Err(crate::harness::AssertFail::msg("second rmid errno")),
    }
    Ok(())
}
