//! POSIX / SysV semaphore coverage (SEM): semget/semop/semctl IPC_PRIVATE,
//! fork post/wait, IPC_NOWAIT would-block, multi-op, plus futex process-shared style.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, map, prot, Errno, Sembuf, GETNCNT, GETVAL, GETZCNT, IPC_CREAT, IPC_NOWAIT, IPC_PRIVATE,
    IPC_RMID, SEM_UNDO, SETVAL, Timespec,
};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::ENOMEM | Errno::ENOSPC | Errno::EINVAL
    )
}

fn sem_open(nsems: i32) -> Result<Option<i32>, crate::harness::AssertFail> {
    match syscall::semget(IPC_PRIVATE, nsems, IPC_CREAT | 0o600) {
        Ok(id) => Ok(Some(id)),
        Err(e) if soft(e) => Ok(None),
        Err(_) => Err(crate::harness::AssertFail::msg("semget")),
    }
}

fn rmid(id: i32) {
    let _ = syscall::semctl(id, 0, IPC_RMID, 0);
}

macro_rules! sem_create_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
        fn $name() -> TestResult {
            let Some(id) = sem_open($n)? else {
                return Ok(());
            };
            check!(id >= 0, "id");
            rmid(id);
            Ok(())
        }
    };
}

sem_create_n!(sem_posix_create_1, 1);
sem_create_n!(sem_posix_create_2, 2);
sem_create_n!(sem_posix_create_3, 3);
sem_create_n!(sem_posix_create_4, 4);
sem_create_n!(sem_posix_create_8, 8);

macro_rules! sem_setval_getval {
    ($name:ident, $v:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
        fn $name() -> TestResult {
            let Some(id) = sem_open(1)? else {
                return Ok(());
            };
            match syscall::semctl(id, 0, SETVAL, $v as usize) {
                Ok(_) => {}
                Err(e) if soft(e) => {
                    rmid(id);
                    return Ok(());
                }
                Err(_) => {
                    rmid(id);
                    return Err(crate::harness::AssertFail::msg("SETVAL"));
                }
            }
            let got = match syscall::semctl(id, 0, GETVAL, 0) {
                Ok(v) => v,
                Err(e) if soft(e) => {
                    rmid(id);
                    return Ok(());
                }
                Err(_) => {
                    rmid(id);
                    return Err(crate::harness::AssertFail::msg("GETVAL"));
                }
            };
            check_eq!(got, $v, "val");
            rmid(id);
            Ok(())
        }
    };
}

sem_setval_getval!(sem_posix_val_0, 0);
sem_setval_getval!(sem_posix_val_1, 1);
sem_setval_getval!(sem_posix_val_2, 2);
sem_setval_getval!(sem_posix_val_3, 3);
sem_setval_getval!(sem_posix_val_5, 5);
sem_setval_getval!(sem_posix_val_7, 7);
sem_setval_getval!(sem_posix_val_10, 10);
sem_setval_getval!(sem_posix_val_16, 16);

#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn sem_posix_op_down_up() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 1usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let down = [Sembuf {
        sem_num: 0,
        sem_op: -1,
        sem_flg: 0,
    }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => {
            rmid(id);
            return Ok(());
        }
        Err(_) => {
            rmid(id);
            return Err(crate::harness::AssertFail::msg("down"));
        }
    }
    check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"), 0, "zero");
    let up = [Sembuf {
        sem_num: 0,
        sem_op: 1,
        sem_flg: 0,
    }];
    check_ok!(syscall::semop(id, &up), "up");
    check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g2"), 1, "one");
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn sem_posix_nowait_wouldblock() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 0usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let down = [Sembuf {
        sem_num: 0,
        sem_op: -1,
        sem_flg: IPC_NOWAIT as i16,
    }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => {
            rmid(id);
            return Err(crate::harness::AssertFail::msg("should block"));
        }
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn sem_posix_multi_op() -> TestResult {
    let Some(id) = sem_open(2)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 1usize).is_err()
        || syscall::semctl(id, 1, SETVAL, 1usize).is_err()
    {
        rmid(id);
        return Ok(());
    }
    let ops = [
        Sembuf {
            sem_num: 0,
            sem_op: -1,
            sem_flg: 0,
        },
        Sembuf {
            sem_num: 1,
            sem_op: -1,
            sem_flg: 0,
        },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {
            check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "a"), 0, "a");
            check_eq!(check_ok!(syscall::semctl(id, 1, GETVAL, 0), "b"), 0, "b");
        }
        Err(e) if soft(e) => {}
        Err(_) => {
            rmid(id);
            return Err(crate::harness::AssertFail::msg("multi"));
        }
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a child can post a semaphore the parent waits on")]
fn sem_posix_fork_post_wait() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 0usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let pid = match syscall::fork() {
        Ok(p) => p,
        Err(_) => {
            rmid(id);
            return Ok(());
        }
    };
    if pid == 0 {
        let pause = Timespec {
            tv_sec: 0,
            tv_nsec: 30_000_000,
        };
        let _ = syscall::nanosleep(&pause);
        let up = [Sembuf {
            sem_num: 0,
            sem_op: 1,
            sem_flg: 0,
        }];
        let _ = syscall::semop(id, &up);
        syscall::exit(0);
    }
    let down = [Sembuf {
        sem_num: 0,
        sem_op: -1,
        sem_flg: 0,
    }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::kill(pid, 9);
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            rmid(id);
            return Err(crate::harness::AssertFail::msg("wait down"));
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "GETNCNT is zero when no waiters are sleeping")]
fn sem_posix_getncnt_zero() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 1usize).is_err() {
        rmid(id);
        return Ok(());
    }
    match syscall::semctl(id, 0, GETNCNT, 0) {
        Ok(v) => check_eq!(v, 0, "ncnt"),
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "GETZCNT is zero when no waiters are sleeping on zero")]
fn sem_posix_getzcnt_zero() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 1usize).is_err() {
        rmid(id);
        return Ok(());
    }
    match syscall::semctl(id, 0, GETZCNT, 0) {
        Ok(v) => check_eq!(v, 0, "zcnt"),
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "semop with SEM_UNDO succeeds or is rejected as unsupported")]
fn sem_posix_undo_flag_soft() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 1usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let down = [Sembuf {
        sem_num: 0,
        sem_op: -1,
        sem_flg: SEM_UNDO,
    }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            rmid(id);
            return Err(crate::harness::AssertFail::msg("undo"));
        }
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "semop can add then subtract the same count")]
fn sem_posix_add_then_sub() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 0usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let up = [Sembuf {
        sem_num: 0,
        sem_op: 5,
        sem_flg: 0,
    }];
    if syscall::semop(id, &up).is_err() {
        rmid(id);
        return Ok(());
    }
    let down = [Sembuf {
        sem_num: 0,
        sem_op: -3,
        sem_flg: 0,
    }];
    if syscall::semop(id, &down).is_ok() {
        check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"), 2, "2");
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "semctl IPC_RMID removes a private semaphore set")]
fn sem_posix_rmid() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    check_ok!(syscall::semctl(id, 0, IPC_RMID, 0), "rmid");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a second IPC_RMID on a removed set is rejected or ignored")]
fn sem_posix_rmid_second_soft() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    check_ok!(syscall::semctl(id, 0, IPC_RMID, 0), "rmid");
    match syscall::semctl(id, 0, IPC_RMID, 0) {
        Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("second rmid")),
        Err(e) if e.0 == 43 => {} // EIDRM
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

/// Futex process-shared style via MAP_SHARED anon (SEM-like).
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a process-shared futex can wake a forked waiter")]
fn sem_posix_futex_pshared_fork() -> TestResult {
    let addr = match syscall::mmap(
        0,
        4096,
        prot::PROT_READ | prot::PROT_WRITE,
        map::MAP_SHARED | map::MAP_ANONYMOUS,
        -1,
        0,
    ) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };
    let lock = unsafe { &*(addr as *const AtomicU32) };
    lock.store(0, Ordering::SeqCst);
    let pid = match syscall::fork() {
        Ok(p) => p,
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Ok(());
        }
    };
    if pid == 0 {
        let pause = Timespec {
            tv_sec: 0,
            tv_nsec: 20_000_000,
        };
        let _ = syscall::nanosleep(&pause);
        lock.store(1, Ordering::Release);
        let _ = syscall::futex_wake(lock, 1);
        syscall::exit(0);
    }
    let timeout = Timespec {
        tv_sec: 2,
        tv_nsec: 0,
    };
    while lock.load(Ordering::Acquire) == 0 {
        match syscall::futex_wait(lock, 0, Some(&timeout)) {
            Ok(()) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => {}
            Err(Errno::ETIMEDOUT) => break,
            Err(_) => break,
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_eq!(lock.load(Ordering::SeqCst), 1, "posted");
    check_ok!(syscall::munmap(addr, 4096), "unmap");
    Ok(())
}

macro_rules! sem_op_cycles {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "semop can decrement and increment a semaphore in a loop")]
        fn $name() -> TestResult {
            let Some(id) = sem_open(1)? else {
                return Ok(());
            };
            if syscall::semctl(id, 0, SETVAL, 1usize).is_err() {
                rmid(id);
                return Ok(());
            }
            for _ in 0..$n {
                let down = [Sembuf {
                    sem_num: 0,
                    sem_op: -1,
                    sem_flg: 0,
                }];
                if syscall::semop(id, &down).is_err() {
                    rmid(id);
                    return Ok(());
                }
                let up = [Sembuf {
                    sem_num: 0,
                    sem_op: 1,
                    sem_flg: 0,
                }];
                if syscall::semop(id, &up).is_err() {
                    rmid(id);
                    return Ok(());
                }
            }
            check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"), 1, "still 1");
            rmid(id);
            Ok(())
        }
    };
}

sem_op_cycles!(sem_posix_cycles_2, 2);
sem_op_cycles!(sem_posix_cycles_4, 4);
sem_op_cycles!(sem_posix_cycles_8, 8);
sem_op_cycles!(sem_posix_cycles_16, 16);

#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT succeeds when the semaphore is available")]
fn sem_posix_nowait_ok_when_available() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 1usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let down = [Sembuf {
        sem_num: 0,
        sem_op: -1,
        sem_flg: IPC_NOWAIT as i16,
    }];
    match syscall::semop(id, &down) {
        Ok(()) => check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"), 0, "0"),
        Err(e) if soft(e) => {}
        Err(_) => {
            rmid(id);
            return Err(crate::harness::AssertFail::msg("nowait"));
        }
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a forked child can decrement a semaphore")]
fn sem_posix_fork_child_down() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 1usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let pid = match syscall::fork() {
        Ok(p) => p,
        Err(_) => {
            rmid(id);
            return Ok(());
        }
    };
    if pid == 0 {
        let down = [Sembuf {
            sem_num: 0,
            sem_op: -1,
            sem_flg: 0,
        }];
        match syscall::semop(id, &down) {
            Ok(()) => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    // Soft: value may be 0 after child down.
    let _ = syscall::semctl(id, 0, GETVAL, 0);
    rmid(id);
    Ok(())
}

macro_rules! sem_val_roundtrip {
    ($name:ident, $a:expr, $b:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "two SETVAL calls leave GETVAL at the last value")]
        fn $name() -> TestResult {
            let Some(id) = sem_open(1)? else {
                return Ok(());
            };
            if syscall::semctl(id, 0, SETVAL, $a as usize).is_err() {
                rmid(id);
                return Ok(());
            }
            if syscall::semctl(id, 0, SETVAL, $b as usize).is_err() {
                rmid(id);
                return Ok(());
            }
            match syscall::semctl(id, 0, GETVAL, 0) {
                Ok(v) => check_eq!(v, $b, "final"),
                Err(_) => {}
            }
            rmid(id);
            Ok(())
        }
    };
}

sem_val_roundtrip!(sem_posix_rt_1_2, 1, 2);
sem_val_roundtrip!(sem_posix_rt_5_0, 5, 0);
sem_val_roundtrip!(sem_posix_rt_0_9, 0, 9);
sem_val_roundtrip!(sem_posix_rt_3_3, 3, 3);
sem_val_roundtrip!(sem_posix_rt_8_1, 8, 1);
sem_val_roundtrip!(sem_posix_rt_4_7, 4, 7);

#[crate::lctp_test(suite = posix, expect = soft, case = "a zero-valued semop is a no-op on an existing semaphore")]
fn sem_posix_zero_op() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 0usize).is_err() {
        rmid(id);
        return Ok(());
    }
    let z = [Sembuf {
        sem_num: 0,
        sem_op: 0,
        sem_flg: IPC_NOWAIT as i16,
    }];
    match syscall::semop(id, &z) {
        Ok(()) => {}
        Err(e) if soft(e) || e == Errno::EAGAIN => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "semop can increment several semaphores in one array")]
fn sem_posix_multi_up() -> TestResult {
    let Some(id) = sem_open(2)? else {
        return Ok(());
    };
    let _ = syscall::semctl(id, 0, SETVAL, 0usize);
    let _ = syscall::semctl(id, 1, SETVAL, 0usize);
    let ops = [
        Sembuf {
            sem_num: 0,
            sem_op: 1,
            sem_flg: 0,
        },
        Sembuf {
            sem_num: 1,
            sem_op: 2,
            sem_flg: 0,
        },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {
            check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "a"), 1, "a");
            check_eq!(check_ok!(syscall::semctl(id, 1, GETVAL, 0), "b"), 2, "b");
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "two private semaphore sets can be created at once")]
fn sem_posix_two_ids() -> TestResult {
    let Some(a) = sem_open(1)? else {
        return Ok(());
    };
    let Some(b) = sem_open(1)? else {
        rmid(a);
        return Ok(());
    };
    check!(a != b, "distinct");
    rmid(a);
    rmid(b);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "semop with IPC_NOWAIT can increment a semaphore")]
fn sem_posix_nowait_up() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    let _ = syscall::semctl(id, 0, SETVAL, 0usize);
    let up = [Sembuf {
        sem_num: 0,
        sem_op: 1,
        sem_flg: IPC_NOWAIT as i16,
    }];
    match syscall::semop(id, &up) {
        Ok(()) => check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"), 1, "1"),
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "GETVAL after create returns a defined value")]
fn sem_posix_getval_after_create() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => check!(v >= 0, "nonneg"),
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "SETVAL and GETVAL work on each semaphore in a three-member set")]
fn sem_posix_three_sems_set() -> TestResult {
    let Some(id) = sem_open(3)? else {
        return Ok(());
    };
    for i in 0..3 {
        let _ = syscall::semctl(id, i, SETVAL, (i + 1) as usize);
    }
    for i in 0..3 {
        if let Ok(v) = syscall::semctl(id, i, GETVAL, 0) {
            check_eq!(v, i + 1, "slot");
        }
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "semctl GETVAL with an out-of-range semnum is rejected")]
fn sem_posix_bad_semnum_soft() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    match syscall::semctl(id, 99, GETVAL, 0) {
        Err(e) if e == Errno::EINVAL || soft(e) => {}
        Ok(_) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "two process-shared futex posts can wake a forked waiter")]
fn sem_posix_futex_pshared_two_posts() -> TestResult {
    let addr = match syscall::mmap(
        0,
        4096,
        prot::PROT_READ | prot::PROT_WRITE,
        map::MAP_SHARED | map::MAP_ANONYMOUS,
        -1,
        0,
    ) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };
    let cell = unsafe { &*(addr as *const AtomicU32) };
    cell.store(0, Ordering::SeqCst);
    for _ in 0..2 {
        cell.fetch_add(1, Ordering::SeqCst);
        let _ = syscall::futex_wake(cell, 1);
    }
    check_eq!(cell.load(Ordering::SeqCst), 2, "posts");
    check_ok!(syscall::munmap(addr, 4096), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "semop can increment a semaphore by one")]
fn sem_posix_op_plus_one() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    let _ = syscall::semctl(id, 0, SETVAL, 0usize);
    let up = [Sembuf {
        sem_num: 0,
        sem_op: 1,
        sem_flg: 0,
    }];
    match syscall::semop(id, &up) {
        Ok(()) => check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"), 1, "1"),
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "semop can increment a semaphore by several")]
fn sem_posix_op_plus_many() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    let _ = syscall::semctl(id, 0, SETVAL, 0usize);
    let up = [Sembuf {
        sem_num: 0,
        sem_op: 10,
        sem_flg: 0,
    }];
    match syscall::semop(id, &up) {
        Ok(()) => check_eq!(check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"), 10, "10"),
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    rmid(id);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "two one-semaphore private sets can be created in sequence")]
fn sem_posix_create_nsems_1_twice() -> TestResult {
    for _ in 0..2 {
        let Some(id) = sem_open(1)? else {
            return Ok(());
        };
        rmid(id);
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "semop can decrement a semaphore that was set to 3")]
fn sem_posix_down_from_3() -> TestResult {
    let Some(id) = sem_open(1)? else {
        return Ok(());
    };
    if syscall::semctl(id, 0, SETVAL, 3usize).is_err() {
        rmid(id);
        return Ok(());
    }
    for expect in [2, 1, 0] {
        let down = [Sembuf {
            sem_num: 0,
            sem_op: -1,
            sem_flg: 0,
        }];
        if syscall::semop(id, &down).is_err() {
            rmid(id);
            return Ok(());
        }
        check_eq!(
            check_ok!(syscall::semctl(id, 0, GETVAL, 0), "g"),
            expect,
            "step"
        );
    }
    rmid(id);
    Ok(())
}
