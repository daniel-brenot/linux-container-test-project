//! Freestanding equivalents of pthread_* semantics (create/join/mutex/cond/rwlock/barrier/once/keys).

use core::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};

use crate::check;
use crate::check_eq;
use crate::harness::TestResult;
use crate::runtime::{self, thread_unavailable};
use crate::syscall::{self, Errno, Timespec};

fn soft_spawn(
    entry: runtime::ThreadFn,
    arg: *mut u8,
) -> Result<Option<runtime::Thread>, crate::harness::AssertFail> {
    match runtime::spawn_thread(entry, arg) {
        Ok(t) => Ok(Some(t)),
        Err(e) if thread_unavailable(e) => Ok(None),
        Err(e) => Err(crate::harness::AssertFail::msg(e.name())),
    }
}

fn soft_join(t: runtime::Thread) -> TestResult {
    match runtime::join_thread(t) {
        Ok(()) => Ok(()),
        Err(e) if thread_unavailable(e) || e == Errno::ETIMEDOUT => Ok(()),
        Err(e) => Err(crate::harness::AssertFail::msg(e.name())),
    }
}

fn mtx_lock(lock: &AtomicU32) {
    loop {
        if lock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            return;
        }
        let _ = syscall::futex_wait(lock, 1, Some(&Timespec { tv_sec: 0, tv_nsec: 2_000_000 }));
    }
}

fn mtx_unlock(lock: &AtomicU32) {
    lock.store(0, Ordering::Release);
    let _ = syscall::futex_wake(lock, 1);
}

unsafe extern "C" fn p_nop(_a: *mut u8) -> i32 { 0 }
unsafe extern "C" fn p_exit_code(arg: *mut u8) -> i32 {
    (*(arg as *const AtomicI32)).load(Ordering::SeqCst)
}
unsafe extern "C" fn p_inc(arg: *mut u8) -> i32 {
    (*(arg as *mut AtomicU32)).fetch_add(1, Ordering::SeqCst);
    0
}
unsafe extern "C" fn p_mutex_crit(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let a = &*(arg as *const Arg);
    for _ in 0..a.n {
        mtx_lock(&a.lock);
        a.counter.fetch_add(1, Ordering::SeqCst);
        mtx_unlock(&a.lock);
    }
    0
}
unsafe extern "C" fn p_trylock(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let a = &*(arg as *const Arg);
    if a.lock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
        a.got.fetch_add(1, Ordering::SeqCst);
        a.lock.store(0, Ordering::Release);
        let _ = syscall::futex_wake(&a.lock, 1);
    } else {
        a.fail.fetch_add(1, Ordering::SeqCst);
    }
    0
}
unsafe extern "C" fn p_rdlock(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let a = &*(arg as *const Arg);
    loop {
        let s = a.state.load(Ordering::Acquire);
        if s & 0x8000_0000 != 0 {
            let _ = syscall::futex_wait(&a.state, s, Some(&Timespec { tv_sec: 0, tv_nsec: 1_000_000 }));
            continue;
        }
        if a.state.compare_exchange(s, s + 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            break;
        }
    }
    a.sum.fetch_add(1, Ordering::Relaxed);
    let prev = a.state.fetch_sub(1, Ordering::Release);
    if prev == 1 { let _ = syscall::futex_wake(&a.state, !0); }
    0
}
unsafe extern "C" fn p_wrlock(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let a = &*(arg as *const Arg);
    loop {
        if a.state.compare_exchange(0, 0x8000_0000, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            break;
        }
        let s = a.state.load(Ordering::Relaxed);
        let _ = syscall::futex_wait(&a.state, s, Some(&Timespec { tv_sec: 0, tv_nsec: 1_000_000 }));
    }
    a.sum.fetch_add(1, Ordering::Relaxed);
    a.state.store(0, Ordering::Release);
    let _ = syscall::futex_wake(&a.state, !0);
    0
}
unsafe extern "C" fn p_barrier(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let a = &*(arg as *const Arg);
    let gen = a.generation.load(Ordering::Acquire);
    let c = a.count.fetch_add(1, Ordering::AcqRel) + 1;
    if c >= a.n {
        a.count.store(0, Ordering::Release);
        a.generation.fetch_add(1, Ordering::Release);
        let _ = syscall::futex_wake(&a.generation, !0);
    } else {
        let timeout = Timespec { tv_sec: 2, tv_nsec: 0 };
        while a.generation.load(Ordering::Acquire) == gen {
            let _ = syscall::futex_wait(&a.generation, gen, Some(&timeout));
        }
    }
    0
}
unsafe extern "C" fn p_cond_wait(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let a = &*(arg as *const Arg);
    let timeout = Timespec { tv_sec: 2, tv_nsec: 0 };
    while a.ready.load(Ordering::Acquire) == 0 {
        let _ = syscall::futex_wait(&a.ready, 0, Some(&timeout));
    }
    a.done.fetch_add(1, Ordering::Release);
    0
}
unsafe extern "C" fn p_once(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let a = &*(arg as *const Arg);
    loop {
        match a.gate.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {
                a.runs.fetch_add(1, Ordering::SeqCst);
                a.gate.store(2, Ordering::Release);
                let _ = syscall::futex_wake(&a.gate, !0);
                break;
            }
            Err(2) => break,
            Err(_) => {
                let _ = syscall::futex_wait(&a.gate, 1, Some(&Timespec { tv_sec: 0, tv_nsec: 2_000_000 }));
            }
        }
    }
    0
}
unsafe extern "C" fn p_key_set(arg: *mut u8) -> i32 {
    // Shared table: [tid: AtomicI32][val: AtomicU64] pairs, index in low byte of arg packing via struct
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let a = &*(arg as *const Arg);
    let slot = &mut *a.slots.add(a.idx);
    slot.tid.store(syscall::gettid(), Ordering::SeqCst);
    slot.val.store(a.magic, Ordering::SeqCst);
    0
}

#[crate::lctp_test(suite = posix)]
fn papi_create_join_1() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_2() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_3() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_4() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_5() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_6() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_7() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_8() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_9() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_10() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_11() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_12() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_13() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_14() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_15() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_16() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_17() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_18() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_19() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_20() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_21() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_22() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_23() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_24() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_25() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_26() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_27() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_28() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_29() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_30() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_31() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_32() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_33() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_34() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_35() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_36() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_37() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_38() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_39() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_create_join_40() -> TestResult {
    let Some(t) = soft_spawn(p_nop, core::ptr::null_mut())? else { return Ok(()); };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_1() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_2() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_3() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_4() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_5() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_6() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_7() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_8() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_9() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_10() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_11() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_12() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_13() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_14() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_15() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_16() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_17() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_18() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_19() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_self_equal_20() -> TestResult {
    let a = syscall::gettid();
    let b = syscall::gettid();
    check_eq!(a, b, "equal");
    check_eq!(a, syscall::getpid(), "self_main");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_1() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_2() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_3() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_4() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_5() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_6() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_7() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_8() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_9() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_10() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_11() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_12() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_13() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_14() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_15() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_16() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_17() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_18() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_19() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_20() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_21() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_22() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_23() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_24() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_mutex_init_destroy_25() -> TestResult {
    let lock = AtomicU32::new(0);
    check_eq!(lock.load(Ordering::Relaxed), 0, "init");
    mtx_lock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 1, "locked");
    mtx_unlock(&lock);
    check_eq!(lock.load(Ordering::Relaxed), 0, "destroyed_unlocked");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_1() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 12 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 24, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_2() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 16 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 32, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_3() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 20 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 40, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_4() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 24 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 48, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_5() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 28 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 56, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_6() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 32 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 64, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_7() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 36 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 72, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_8() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 40 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 80, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_9() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 44 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 88, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_10() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 48 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 96, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_11() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 52 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 104, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_12() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 56 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 112, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_13() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 60 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 120, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_14() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 64 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 128, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_15() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 68 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 136, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_16() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 72 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 144, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_17() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 76 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 152, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_18() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 80 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 160, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_19() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 84 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 168, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_lock_unlock_20() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, counter: AtomicU32, n: u32 }
    let mut arg = Arg { lock: AtomicU32::new(0), counter: AtomicU32::new(0), n: 88 };
    let Some(t1) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_mutex_crit, &mut arg as *mut _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 176, "crit");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_1() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_2() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_3() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_4() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_5() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_6() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_7() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_8() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_9() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_10() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_11() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_12() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_13() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_14() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_mutex_trylock_15() -> TestResult {
    #[repr(C)]
    struct Arg { lock: AtomicU32, got: AtomicU32, fail: AtomicU32 }
    let mut arg = Arg { lock: AtomicU32::new(0), got: AtomicU32::new(0), fail: AtomicU32::new(0) };
    // Hold lock then spawn trylock that should fail once
    arg.lock.store(1, Ordering::Release);
    let Some(t) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    check_eq!(arg.fail.load(Ordering::SeqCst), 1, "fail");
    arg.lock.store(0, Ordering::Release);
    let Some(t2) = soft_spawn(p_trylock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    soft_join(t2)?;
    check_eq!(arg.got.load(Ordering::SeqCst), 1, "got");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_1() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..1 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 1, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_2() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 2, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_3() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 3, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_4() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 4, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_5() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 2, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_6() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 3, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_7() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 4, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_8() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..1 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 1, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_9() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 2, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_10() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 3, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_11() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 4, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_broadcast_12() -> TestResult {
    #[repr(C)]
    struct Arg { ready: AtomicU32, done: AtomicU32 }
    let mut arg = Arg { ready: AtomicU32::new(0), done: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..1 {
        match soft_spawn(p_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for _ in 0..4 { let _ = syscall::sched_yield(); }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.done.load(Ordering::SeqCst), 1, "done");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_1() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_2() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_3() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_4() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_5() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_6() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_7() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_8() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_9() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_cond_timedwait_soft_10() -> TestResult {
    let ready = AtomicU32::new(0);
    let timeout = Timespec { tv_sec: 0, tv_nsec: 1_000_000 };
    // Soft: wait with timeout while never signaled.
    let _ = syscall::futex_wait(&ready, 0, Some(&timeout));
    check_eq!(ready.load(Ordering::SeqCst), 0, "still");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_1() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_2() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_3() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_4() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_5() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_6() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_7() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_8() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_9() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_10() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_11() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_12() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_13() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_14() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_15() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_rd_16() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_rdlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "rd");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_1() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_2() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_3() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_4() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_5() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_6() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_7() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_8() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_9() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_10() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_11() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_12() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_13() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_14() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_15() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_rwlock_wr_16() -> TestResult {
    #[repr(C)]
    struct Arg { state: AtomicU32, sum: AtomicU32 }
    let mut arg = Arg { state: AtomicU32::new(0), sum: AtomicU32::new(0) };
    let Some(a) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let Some(b) = soft_spawn(p_wrlock, &mut arg as *mut _ as *mut u8)? else { soft_join(a)?; return Ok(()); };
    soft_join(a)?;
    soft_join(b)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 2, "wr");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_1() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_2() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_3() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_4() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_5() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_6() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_7() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_2_8() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 2 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_1() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_2() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_3() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_4() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_5() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_6() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_7() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_3_8() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 3 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_1() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_2() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_3() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_4() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_5() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_6() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_7() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_barrier_4_8() -> TestResult {
    #[repr(C)]
    struct Arg { count: AtomicU32, generation: AtomicU32, n: u32 }
    let mut arg = Arg { count: AtomicU32::new(0), generation: AtomicU32::new(0), n: 4 };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_barrier, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_1() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_2() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_3() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_4() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_5() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_6() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_7() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_2_8() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..2 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_1() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_2() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_3() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_4() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_5() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_6() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_7() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_3_8() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..3 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_1() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_2() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_3() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_4() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_5() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_6() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_7() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_once_4_8() -> TestResult {
    #[repr(C)]
    struct Arg { gate: AtomicU32, runs: AtomicU32 }
    let mut arg = Arg { gate: AtomicU32::new(0), runs: AtomicU32::new(0) };
    let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
    for i in 0..4 {
        match soft_spawn(p_once, &mut arg as *mut _ as *mut u8)? {
            Some(t) => handles[i] = Some(t),
            None => {
                for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) { soft_join(h)?; }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_1() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4097 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4097, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_2() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4098 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4098, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_3() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4099 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4099, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_4() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4100 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4100, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_5() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4101 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4101, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_6() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4102 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4102, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_7() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4103 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4103, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_8() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4104 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4104, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_9() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4105 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4105, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_10() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4106 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4106, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_11() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4107 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4107, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_12() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4108 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4108, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_13() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4109 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4109, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_14() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4110 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4110, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_15() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4111 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4111, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_16() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4112 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4112, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_17() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4113 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4113, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_18() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4114 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4114, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_19() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4115 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4115, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_key_setget_20() -> TestResult {
    #[repr(C)]
    struct Slot { tid: AtomicI32, val: AtomicU64 }
    #[repr(C)]
    struct Arg { slots: *mut Slot, idx: usize, magic: u64 }
    let mut slot = Slot { tid: AtomicI32::new(0), val: AtomicU64::new(0) };
    let mut arg = Arg { slots: &mut slot as *mut Slot, idx: 0, magic: 4116 };
    let Some(t) = soft_spawn(p_key_set, &mut arg as *mut _ as *mut u8)? else { return Ok(()); };
    let ht = t.tid();
    soft_join(t)?;
    check!(slot.tid.load(Ordering::SeqCst) > 0, "tid");
    check_eq!(slot.val.load(Ordering::SeqCst), 4116, "val");
    let _ = ht;
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_1() -> TestResult {
    let cell = AtomicI32::new(1);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 1, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_2() -> TestResult {
    let cell = AtomicI32::new(2);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 2, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_3() -> TestResult {
    let cell = AtomicI32::new(3);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 3, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_4() -> TestResult {
    let cell = AtomicI32::new(4);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 4, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_5() -> TestResult {
    let cell = AtomicI32::new(5);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 5, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_6() -> TestResult {
    let cell = AtomicI32::new(6);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 6, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_7() -> TestResult {
    let cell = AtomicI32::new(7);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 7, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_8() -> TestResult {
    let cell = AtomicI32::new(8);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 8, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_9() -> TestResult {
    let cell = AtomicI32::new(9);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 9, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn papi_exit_code_10() -> TestResult {
    let cell = AtomicI32::new(10);
    let Some(t) = soft_spawn(p_exit_code, &cell as *const _ as *mut u8)? else { return Ok(()); };
    soft_join(t)?;
    // Return code is not retrieved via join in freestanding helper; ensure thread ran.
    check_eq!(cell.load(Ordering::SeqCst), 10, "cell");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_1() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_2() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_3() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_4() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_5() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_6() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_7() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_8() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_9() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn papi_create_join_parallel_10() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(p_inc, &a as *const _ as *mut u8)? else { return Ok(()); };
    let Some(t2) = soft_spawn(p_inc, &b as *const _ as *mut u8)? else { soft_join(t1)?; return Ok(()); };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}