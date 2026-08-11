//! System V shared memory (`shmget` / `shmat` / `shmdt` / `shmctl`).
//!
//! Guests that return `ENOSYS` for these calls break real userspace that
//! expects a modern Linux SysV IPC surface. Soft-skip only permission / resource
//! errors — never `ENOSYS`.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, Errno, IPC_CREAT, IPC_EXCL, IPC_PRIVATE, IPC_RMID, IPC_STAT, SHM_RDONLY,
};

fn soft_skip(e: Errno) -> bool {
    matches!(
        e,
        Errno::EPERM | Errno::EACCES | Errno::ENOMEM | Errno::ENOSPC
    )
}

fn shmget_or_soft(size: usize, flg: i32) -> Result<Option<i32>, crate::harness::AssertFail> {
    match syscall::shmget(IPC_PRIVATE, size, flg) {
        Ok(id) => Ok(Some(id)),
        Err(Errno::ENOSYS) => Err(crate::harness::AssertFail::msg("shmget ENOSYS")),
        Err(e) if soft_skip(e) => Ok(None),
        Err(_) => Err(crate::harness::AssertFail::msg("shmget")),
    }
}

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_get_not_enosys() -> TestResult {
    match syscall::shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) {
        Ok(id) => {
            let _ = syscall::shmctl(id, IPC_RMID, 0);
        }
        Err(Errno::ENOSYS) => {
            return Err(crate::harness::AssertFail::msg("shmget ENOSYS"));
        }
        Err(e) if soft_skip(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("shmget")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_ipc_private_roundtrip() -> TestResult {
    let size = 4096usize;
    let Some(shmid) = shmget_or_soft(size, IPC_CREAT | 0o600)? else {
        return Ok(());
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
    let Some(shmid) = shmget_or_soft(size, IPC_CREAT | 0o600)? else {
        return Ok(());
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

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_dual_attach_same_segment() -> TestResult {
    let Some(shmid) = shmget_or_soft(4096, IPC_CREAT | 0o600)? else {
        return Ok(());
    };
    let a = match syscall::shmat(shmid, 0, 0) {
        Ok(a) => a,
        Err(e) => {
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat a"));
        }
    };
    let b = match syscall::shmat(shmid, 0, 0) {
        Ok(b) => b,
        Err(e) => {
            let _ = syscall::shmdt(a);
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat b"));
        }
    };
    unsafe {
        core::ptr::write_volatile(a as *mut u32, 0x1122_3344);
        check_eq!(
            core::ptr::read_volatile(b as *const u32),
            0x1122_3344,
            "alias"
        );
    }
    check_ok!(syscall::shmdt(a), "dt a");
    check_ok!(syscall::shmdt(b), "dt b");
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_creat_excl() -> TestResult {
    let Some(id) = shmget_or_soft(4096, IPC_CREAT | IPC_EXCL | 0o600)? else {
        return Ok(());
    };
    check_ok!(syscall::shmctl(id, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_zero_size_einval() -> TestResult {
    match syscall::shmget(IPC_PRIVATE, 0, IPC_CREAT | 0o600) {
        Err(Errno::EINVAL) => {}
        Err(Errno::ENOSYS) => {
            return Err(crate::harness::AssertFail::msg("shmget ENOSYS"));
        }
        Err(e) if soft_skip(e) => {}
        Ok(id) => {
            let _ = syscall::shmctl(id, IPC_RMID, 0);
            return Err(crate::harness::AssertFail::msg("size 0 ok"));
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("size 0 errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysv_shm_two_segments_independent() -> TestResult {
    let Some(a) = shmget_or_soft(4096, IPC_CREAT | 0o600)? else {
        return Ok(());
    };
    let Some(b) = shmget_or_soft(4096, IPC_CREAT | 0o600)? else {
        let _ = syscall::shmctl(a, IPC_RMID, 0);
        return Ok(());
    };
    check!(a != b, "distinct ids");
    let pa = match syscall::shmat(a, 0, 0) {
        Ok(p) => p,
        Err(e) => {
            let _ = syscall::shmctl(a, IPC_RMID, 0);
            let _ = syscall::shmctl(b, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat a"));
        }
    };
    let pb = match syscall::shmat(b, 0, 0) {
        Ok(p) => p,
        Err(e) => {
            let _ = syscall::shmdt(pa);
            let _ = syscall::shmctl(a, IPC_RMID, 0);
            let _ = syscall::shmctl(b, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat b"));
        }
    };
    unsafe {
        core::ptr::write_volatile(pa as *mut u32, 1);
        core::ptr::write_volatile(pb as *mut u32, 2);
        check_eq!(core::ptr::read_volatile(pa as *const u32), 1, "a");
        check_eq!(core::ptr::read_volatile(pb as *const u32), 2, "b");
    }
    check_ok!(syscall::shmdt(pa), "dt a");
    check_ok!(syscall::shmdt(pb), "dt b");
    check_ok!(syscall::shmctl(a, IPC_RMID, 0), "rmid a");
    check_ok!(syscall::shmctl(b, IPC_RMID, 0), "rmid b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_shm_attach_write_detach_path() -> TestResult {
    let shmid = check_ok!(
        syscall::shmget(IPC_PRIVATE, 8192, IPC_CREAT | 0o600),
        "shmget"
    );
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
        core::ptr::write_bytes(addr as *mut u8, 0xA5, 32);
        check_eq!(*(addr as *const u8), 0xA5, "byte");
    }
    check_ok!(syscall::shmdt(addr), "shmdt");
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_shm_rdonly_attach() -> TestResult {
    let Some(shmid) = shmget_or_soft(4096, IPC_CREAT | 0o600)? else {
        return Ok(());
    };
    let rw = match syscall::shmat(shmid, 0, 0) {
        Ok(a) => a,
        Err(e) => {
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat rw"));
        }
    };
    unsafe {
        core::ptr::write_volatile(rw as *mut u32, 0x55AA_55AA);
    }
    check_ok!(syscall::shmdt(rw), "dt rw");
    let ro = match syscall::shmat(shmid, 0, SHM_RDONLY) {
        Ok(a) => a,
        Err(e) => {
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            if soft_skip(e) {
                return Ok(());
            }
            return Err(crate::harness::AssertFail::msg("shmat ro"));
        }
    };
    let v = unsafe { core::ptr::read_volatile(ro as *const u32) };
    check_eq!(v, 0x55AA_55AA, "ro read");
    check_ok!(syscall::shmdt(ro), "dt ro");
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_shm_stat_soft() -> TestResult {
    let Some(shmid) = shmget_or_soft(8192, IPC_CREAT | 0o600)? else {
        return Ok(());
    };
    let mut buf = [0u8; 256];
    match syscall::shmctl(shmid, IPC_STAT, buf.as_mut_ptr() as usize) {
        Ok(_) => {}
        Err(e) if soft_skip(e) || e == Errno::EINVAL || e == Errno::EFAULT => {}
        Err(Errno::ENOSYS) => {
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            return Err(crate::harness::AssertFail::msg("shmctl STAT ENOSYS"));
        }
        Err(_) => {
            let _ = syscall::shmctl(shmid, IPC_RMID, 0);
            return Err(crate::harness::AssertFail::msg("shmctl STAT"));
        }
    }
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_shm_attach_after_rmid_fails() -> TestResult {
    let Some(shmid) = shmget_or_soft(4096, IPC_CREAT | 0o600)? else {
        return Ok(());
    };
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    match syscall::shmat(shmid, 0, 0) {
        Err(Errno::EINVAL) => {}
        Err(e) if e.0 == 43 => {} // EIDRM
        Err(e) if soft_skip(e) => {}
        Ok(a) => {
            let _ = syscall::shmdt(a);
            return Err(crate::harness::AssertFail::msg("shmat after rmid"));
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("shmat errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_shm_rmid_idempotent_probe() -> TestResult {
    let Some(shmid) = shmget_or_soft(4096, IPC_CREAT | 0o600)? else {
        return Ok(());
    };
    check_ok!(syscall::shmctl(shmid, IPC_RMID, 0), "rmid");
    match syscall::shmctl(shmid, IPC_RMID, 0) {
        Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("second rmid ok")),
        Err(e) if e.0 == 43 => {} // EIDRM
        Err(_) => return Err(crate::harness::AssertFail::msg("second rmid errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sysv_shm_shmdt_bad_addr() -> TestResult {
    check_err!(syscall::shmdt(0), Errno::EINVAL, "null shmdt");
    Ok(())
}
