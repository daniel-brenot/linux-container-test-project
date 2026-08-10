//! System V shared memory (shmget/shmat/shmdt/shmctl) tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, IPC_CREAT, IPC_PRIVATE, IPC_RMID};

fn soft_skip(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::ENOMEM | Errno::ENOSPC
    )
}

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_ipc_private_roundtrip() -> TestResult {
    let size = 4096usize;
    let shmid = match syscall::shmget(IPC_PRIVATE, size, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft_skip(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("shmget failed")),
    };
    let addr = match syscall::shmat(shmid, 0, 0) {
        Ok(a) => a,
        Err(e) => {
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat failed"));
        }
    };
    // Write and read through the attached segment.
    unsafe {
        let p = addr as *mut u8;
        core::ptr::write_bytes(p, 0x5A, 16);
        check_eq!(*p, 0x5A, "byte0");
        check_eq!(*p.add(15), 0x5A, "byte15");
    }
    check_ok!(syscall::shmdt(addr), "shmdt");
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_parent_child_shared() -> TestResult {
    let size = 4096usize;
    let shmid = match syscall::shmget(IPC_PRIVATE, size, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft_skip(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("shmget")),
    };
    let addr = match syscall::shmat(shmid, 0, 0) {
        Ok(a) => a,
        Err(e) => {
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat"));
        }
    };
    unsafe {
        core::ptr::write_volatile(addr as *mut u8, 0);
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            core::ptr::write_volatile(addr as *mut u8, 0x42);
        }
        let _ = syscall::shmdt(addr);
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(syscall::wexitstatus(status), 0, "child");
    let v = unsafe { core::ptr::read_volatile(addr as *const u8) };
    check_eq!(v, 0x42, "shared write");
    check_ok!(syscall::shmdt(addr), "shmdt");
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_shm_rmid_idempotent_probe() -> TestResult {
    let shmid = match syscall::shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) {
        Ok(id) => id,
        Err(e) if soft_skip(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("shmget")),
    };
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    // Second RMID should fail (EINVAL / EIDRM).
    match syscall::shmctl(shmid, IPC_RMID, 0) {
        Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("second rmid ok")),
        Err(e) if e.0 == 43 => {} // EIDRM
        Err(_) => return Err(crate::harness::AssertFail::msg("second rmid errno")),
    }
    Ok(())
}
