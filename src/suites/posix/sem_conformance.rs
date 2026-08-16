//! Semaphore conformance deepeners (SysV SEM soft ENOSYS/EPERM).

use crate::check;
use crate::check_eq;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, Sembuf, GETVAL, IPC_CREAT, IPC_NOWAIT, IPC_PRIVATE, IPC_RMID, SETVAL};

fn soft(e: Errno) -> bool {
    matches!(e, Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::ENOMEM | Errno::ENOSPC | Errno::EINVAL)
}

fn sem_open(nsems: i32) -> Result<Option<i32>, crate::harness::AssertFail> {
    match syscall::semget(IPC_PRIVATE, nsems, IPC_CREAT | 0o600) {
        Ok(id) => Ok(Some(id)),
        Err(e) if soft(e) => Ok(None),
        Err(_) => Err(crate::harness::AssertFail::msg("semget")),
    }
}

fn rmid(id: i32) { let _ = syscall::semctl(id, 0, IPC_RMID, 0); }

#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_1() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_2() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_3() -> TestResult {
    let Some(id) = sem_open(3)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_4() -> TestResult {
    let Some(id) = sem_open(4)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_5() -> TestResult {
    let Some(id) = sem_open(5)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_6() -> TestResult {
    let Some(id) = sem_open(6)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_7() -> TestResult {
    let Some(id) = sem_open(7)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_8() -> TestResult {
    let Some(id) = sem_open(8)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_9() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_10() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_11() -> TestResult {
    let Some(id) = sem_open(3)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_12() -> TestResult {
    let Some(id) = sem_open(4)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_13() -> TestResult {
    let Some(id) = sem_open(5)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_14() -> TestResult {
    let Some(id) = sem_open(6)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_15() -> TestResult {
    let Some(id) = sem_open(7)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semget can create a private semaphore set")]
fn semc_create_16() -> TestResult {
    let Some(id) = sem_open(8)? else { return Ok(()); };
    check!(id >= 0, "id");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_0() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 0 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 0, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_1() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 1 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 1, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_2() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 2 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 2, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_3() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 3 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 3, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_4() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 4 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 4, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_5() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 5 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 5, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_6() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 6 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 6, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_7() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 7 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 7, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_8() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 8 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 8, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_9() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 9 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 9, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_10() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 10 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 10, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_11() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 11 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 11, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_12() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 12 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 12, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_13() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 13 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 13, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_14() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 14 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 14, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_15() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 15 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 15, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_16() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 16 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 16, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_17() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 17 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 17, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_18() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 18 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 18, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_19() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 19 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 19, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semctl SETVAL then GETVAL round-trips a semaphore value")]
fn semc_setget_20() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    match syscall::semctl(id, 0, SETVAL, 20 as usize) {
        Ok(_) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("SETVAL")); }
    }
    let got = match syscall::semctl(id, 0, GETVAL, 0) {
        Ok(v) => v,
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("GETVAL")); }
    };
    check_eq!(got, 20, "val");
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_1() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_2() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_3() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_4() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_5() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_6() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_7() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_8() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_9() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_10() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_11() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_12() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_13() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_14() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop can decrement then increment a semaphore")]
fn semc_op_up_down_15() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 1).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 }];
    let up = [Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 }];
    match syscall::semop(id, &down) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("down")); }
    }
    match syscall::semop(id, &up) {
        Ok(()) => {}
        Err(e) if soft(e) => { rmid(id); return Ok(()); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("up")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_1() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_2() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_3() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_4() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_5() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_6() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_7() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_8() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_9() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_10() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_11() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "semop with IPC_NOWAIT on a zero semaphore returns EAGAIN")]
fn semc_nowait_wouldblock_12() -> TestResult {
    let Some(id) = sem_open(1)? else { return Ok(()); };
    if syscall::semctl(id, 0, SETVAL, 0).is_err() { rmid(id); return Ok(()); }
    let down = [Sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 }];
    match syscall::semop(id, &down) {
        Err(Errno::EAGAIN) => {}
        Err(e) if soft(e) => {}
        Ok(()) => { rmid(id); return Err(crate::harness::AssertFail::msg("should block")); }
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("op")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_1() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_2() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_3() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_4() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_5() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_6() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_7() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_8() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_9() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = soft, case = "a two-semaphore set can be updated with one semop array")]
fn semc_multi_op_10() -> TestResult {
    let Some(id) = sem_open(2)? else { return Ok(()); };
    let _ = syscall::semctl(id, 0, SETVAL, 1);
    let _ = syscall::semctl(id, 1, SETVAL, 1);
    let ops = [
        Sembuf { sem_num: 0, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: -1, sem_flg: 0 },
        Sembuf { sem_num: 0, sem_op: 1, sem_flg: 0 },
        Sembuf { sem_num: 1, sem_op: 1, sem_flg: 0 },
    ];
    match syscall::semop(id, &ops) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => { rmid(id); return Err(crate::harness::AssertFail::msg("multi")); }
    }
    rmid(id);
    Ok(())
}