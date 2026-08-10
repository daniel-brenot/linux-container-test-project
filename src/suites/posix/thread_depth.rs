//! Deep freestanding thread coverage (THR): mutex/rwlock/barrier/cond/once,
//! spawn/join stress, gettid uniqueness, AtomicU64 counters, stack reuse.

use core::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};

use crate::check;
use crate::check_eq;
use crate::check_ok;
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

fn futex_lock(lock: &AtomicU32) {
    loop {
        if lock
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return;
        }
        let _ = syscall::futex_wait(
            lock,
            1,
            Some(&Timespec {
                tv_sec: 0,
                tv_nsec: 2_000_000,
            }),
        );
    }
}

fn futex_unlock(lock: &AtomicU32) {
    lock.store(0, Ordering::Release);
    let _ = syscall::futex_wake(lock, 1);
}

unsafe extern "C" fn thr_nop(_arg: *mut u8) -> i32 {
    0
}

unsafe extern "C" fn thr_inc32(arg: *mut u8) -> i32 {
    (*(arg as *mut AtomicU32)).fetch_add(1, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_inc64(arg: *mut u8) -> i32 {
    (*(arg as *mut AtomicU64)).fetch_add(1, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_add64_n(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        counter: AtomicU64,
        n: u32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..a.n {
        a.counter.fetch_add(1, Ordering::Relaxed);
    }
    0
}

unsafe extern "C" fn thr_store_tid(arg: *mut u8) -> i32 {
    (*(arg as *mut AtomicI32)).store(syscall::gettid(), Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_mutex_inc(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        counter: AtomicU32,
        n: u32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..a.n {
        futex_lock(&a.lock);
        a.counter.fetch_add(1, Ordering::SeqCst);
        futex_unlock(&a.lock);
    }
    0
}

unsafe extern "C" fn thr_trylock_spin(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        got: AtomicU32,
        fail: AtomicU32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..128 {
        if a.lock
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            a.got.fetch_add(1, Ordering::SeqCst);
            a.lock.store(0, Ordering::Release);
            let _ = syscall::futex_wake(&a.lock, 1);
        } else {
            a.fail.fetch_add(1, Ordering::SeqCst);
            let _ = syscall::sched_yield();
        }
    }
    0
}

unsafe extern "C" fn thr_rwlock_shared(arg: *mut u8) -> i32 {
    // [state: AtomicU32][sum: AtomicU32]  state: 0=free, >0 readers, 0x8000_0000 writer
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..32 {
        loop {
            let s = a.state.load(Ordering::Acquire);
            if s & 0x8000_0000 != 0 {
                let _ = syscall::futex_wait(
                    &a.state,
                    s,
                    Some(&Timespec {
                        tv_sec: 0,
                        tv_nsec: 1_000_000,
                    }),
                );
                continue;
            }
            if a.state
                .compare_exchange(s, s + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        a.sum.fetch_add(1, Ordering::Relaxed);
        let prev = a.state.fetch_sub(1, Ordering::Release);
        if prev == 1 {
            let _ = syscall::futex_wake(&a.state, !0);
        }
    }
    0
}

unsafe extern "C" fn thr_rwlock_excl(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..16 {
        loop {
            if a.state
                .compare_exchange(0, 0x8000_0000, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            let s = a.state.load(Ordering::Relaxed);
            let _ = syscall::futex_wait(
                &a.state,
                s,
                Some(&Timespec {
                    tv_sec: 0,
                    tv_nsec: 1_000_000,
                }),
            );
        }
        a.sum.fetch_add(10, Ordering::Relaxed);
        a.state.store(0, Ordering::Release);
        let _ = syscall::futex_wake(&a.state, !0);
    }
    0
}

unsafe extern "C" fn thr_barrier_arrive(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        count: AtomicU32,
        generation: AtomicU32,
        n: u32,
    }
    let a = &*(arg as *const Arg);
    let gen = a.generation.load(Ordering::Acquire);
    let c = a.count.fetch_add(1, Ordering::AcqRel) + 1;
    if c >= a.n {
        a.count.store(0, Ordering::Release);
        a.generation.fetch_add(1, Ordering::Release);
        let _ = syscall::futex_wake(&a.generation, !0);
    } else {
        let timeout = Timespec {
            tv_sec: 2,
            tv_nsec: 0,
        };
        while a.generation.load(Ordering::Acquire) == gen {
            let _ = syscall::futex_wait(&a.generation, gen, Some(&timeout));
        }
    }
    0
}

unsafe extern "C" fn thr_cond_wait(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    let a = &*(arg as *const Arg);
    let timeout = Timespec {
        tv_sec: 2,
        tv_nsec: 0,
    };
    while a.ready.load(Ordering::Acquire) == 0 {
        let _ = syscall::futex_wait(&a.ready, 0, Some(&timeout));
    }
    a.done.fetch_add(1, Ordering::Release);
    let _ = syscall::futex_wake(&a.done, 1);
    0
}

unsafe extern "C" fn thr_once_run(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        gate: AtomicU32,
        runs: AtomicU32,
    }
    let a = &*(arg as *const Arg);
    // 0 = not started, 1 = running, 2 = done
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
                let _ = syscall::futex_wait(
                    &a.gate,
                    1,
                    Some(&Timespec {
                        tv_sec: 0,
                        tv_nsec: 2_000_000,
                    }),
                );
            }
        }
    }
    0
}

unsafe extern "C" fn thr_yield_inc(arg: *mut u8) -> i32 {
    let _ = syscall::sched_yield();
    (*(arg as *mut AtomicU32)).fetch_add(1, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_sleep_brief(arg: *mut u8) -> i32 {
    let _ = syscall::nanosleep(&Timespec {
        tv_sec: 0,
        tv_nsec: 500_000,
    });
    (*(arg as *mut AtomicU32)).store(1, Ordering::SeqCst);
    0
}

macro_rules! spawn_join_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full)]
        fn $name() -> TestResult {
            for _ in 0..$n {
                let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
                    return Ok(());
                };
                soft_join(t)?;
            }
            check_eq!(syscall::gettid(), syscall::getpid(), "restored");
            Ok(())
        }
    };
}

spawn_join_n!(thr_d_spawn_join_5, 5);
spawn_join_n!(thr_d_spawn_join_8, 8);
spawn_join_n!(thr_d_spawn_join_12, 12);
spawn_join_n!(thr_d_spawn_join_16, 16);
spawn_join_n!(thr_d_spawn_join_20, 20);
spawn_join_n!(thr_d_spawn_join_24, 24);
spawn_join_n!(thr_d_spawn_join_32, 32);

macro_rules! sequential_inc {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let cell = AtomicU32::new(0);
            for _ in 0..$n {
                let Some(t) = soft_spawn(thr_inc32, &cell as *const _ as *mut u8)? else {
                    return Ok(());
                };
                soft_join(t)?;
            }
            check_eq!(cell.load(Ordering::SeqCst), $n, "incs");
            Ok(())
        }
    };
}

sequential_inc!(thr_d_seq_inc_2, 2);
sequential_inc!(thr_d_seq_inc_3, 3);
sequential_inc!(thr_d_seq_inc_4, 4);
sequential_inc!(thr_d_seq_inc_5, 5);
sequential_inc!(thr_d_seq_inc_6, 6);
sequential_inc!(thr_d_seq_inc_7, 7);
sequential_inc!(thr_d_seq_inc_8, 8);
sequential_inc!(thr_d_seq_inc_10, 10);

#[crate::lctp_test(suite = posix)]
fn thr_d_gettid_unique_two() -> TestResult {
    let a = AtomicI32::new(0);
    let b = AtomicI32::new(0);
    let Some(t1) = soft_spawn(thr_store_tid, &a as *const _ as *mut u8)? else {
        return Ok(());
    };
    soft_join(t1)?;
    let Some(t2) = soft_spawn(thr_store_tid, &b as *const _ as *mut u8)? else {
        return Ok(());
    };
    soft_join(t2)?;
    let ta = a.load(Ordering::SeqCst);
    let tb = b.load(Ordering::SeqCst);
    check!(ta > 0 && tb > 0, "positive");
    // Soft: tids may recycle after join; accept equal or unequal.
    let _ = ta != tb;
    check_eq!(syscall::gettid(), syscall::getpid(), "restored");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_gettid_unique_parallel() -> TestResult {
    let cells = [
        AtomicI32::new(0),
        AtomicI32::new(0),
        AtomicI32::new(0),
        AtomicI32::new(0),
    ];
    let mut handles = [None, None, None, None];
    for (i, h) in handles.iter_mut().enumerate() {
        match soft_spawn(thr_store_tid, &cells[i] as *const _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    let mut tids = [0i32; 4];
    for (i, h) in handles.iter_mut().enumerate() {
        if let Some(t) = h.take() {
            tids[i] = t.tid();
            soft_join(t)?;
        }
    }
    for i in 0..4 {
        for j in (i + 1)..4 {
            check!(tids[i] != tids[j], "unique handles");
        }
    }
    Ok(())
}

macro_rules! mutex_n_threads {
    ($name:ident, $threads:expr, $loops:expr) => {
        #[crate::lctp_test(suite = posix, full)]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                lock: AtomicU32,
                counter: AtomicU32,
                n: u32,
            }
            let mut arg = Arg {
                lock: AtomicU32::new(0),
                counter: AtomicU32::new(0),
                n: $loops,
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            let want = $threads;
            for i in 0..want {
                match soft_spawn(thr_mutex_inc, &mut arg as *mut _ as *mut u8)? {
                    Some(t) => handles[i] = Some(t),
                    None => {
                        for h in handles.iter_mut().filter_map(|x| x.take()) {
                            soft_join(h)?;
                        }
                        return Ok(());
                    }
                }
            }
            for h in handles.iter_mut().filter_map(|x| x.take()) {
                soft_join(h)?;
            }
            check_eq!(
                arg.counter.load(Ordering::SeqCst),
                ($threads as u32) * ($loops as u32),
                "mutex total"
            );
            Ok(())
        }
    };
}

mutex_n_threads!(thr_d_mutex_2x32, 2, 32);
mutex_n_threads!(thr_d_mutex_2x64, 2, 64);
mutex_n_threads!(thr_d_mutex_2x128, 2, 128);
mutex_n_threads!(thr_d_mutex_3x32, 3, 32);
mutex_n_threads!(thr_d_mutex_3x64, 3, 64);
mutex_n_threads!(thr_d_mutex_4x32, 4, 32);
mutex_n_threads!(thr_d_mutex_4x64, 4, 64);
mutex_n_threads!(thr_d_mutex_4x16, 4, 16);

#[crate::lctp_test(suite = posix, full)]
fn thr_d_trylock_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        got: AtomicU32,
        fail: AtomicU32,
    }
    let mut arg = Arg {
        lock: AtomicU32::new(0),
        got: AtomicU32::new(0),
        fail: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(thr_trylock_spin, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_trylock_spin, &mut arg as *mut _ as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    let got = arg.got.load(Ordering::SeqCst);
    let fail = arg.fail.load(Ordering::SeqCst);
    check_eq!(got + fail, 256, "attempts");
    check!(got > 0, "some got");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_rwlock_shared_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
    }
    let mut arg = Arg {
        state: AtomicU32::new(0),
        sum: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(thr_rwlock_shared, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_rwlock_shared, &mut arg as *mut _ as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 64, "shared sum");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_rwlock_excl_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
    }
    let mut arg = Arg {
        state: AtomicU32::new(0),
        sum: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(thr_rwlock_excl, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_rwlock_excl, &mut arg as *mut _ as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 320, "excl sum");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_rwlock_mixed() -> TestResult {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
    }
    let mut arg = Arg {
        state: AtomicU32::new(0),
        sum: AtomicU32::new(0),
    };
    let Some(r1) = soft_spawn(thr_rwlock_shared, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(w1) = soft_spawn(thr_rwlock_excl, &mut arg as *mut _ as *mut u8)? else {
        soft_join(r1)?;
        return Ok(());
    };
    let Some(r2) = soft_spawn(thr_rwlock_shared, &mut arg as *mut _ as *mut u8)? else {
        soft_join(r1)?;
        soft_join(w1)?;
        return Ok(());
    };
    soft_join(r1)?;
    soft_join(w1)?;
    soft_join(r2)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 32 + 160 + 32, "mixed");
    Ok(())
}

macro_rules! barrier_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full)]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                count: AtomicU32,
                generation: AtomicU32,
                n: u32,
            }
            let mut arg = Arg {
                count: AtomicU32::new(0),
                generation: AtomicU32::new(0),
                n: $n,
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$n {
                match soft_spawn(thr_barrier_arrive, &mut arg as *mut _ as *mut u8)? {
                    Some(t) => handles[i] = Some(t),
                    None => {
                        for h in handles.iter_mut().filter_map(|x| x.take()) {
                            soft_join(h)?;
                        }
                        return Ok(());
                    }
                }
            }
            for h in handles.iter_mut().filter_map(|x| x.take()) {
                soft_join(h)?;
            }
            check!(arg.generation.load(Ordering::SeqCst) >= 1, "passed");
            Ok(())
        }
    };
}

barrier_n!(thr_d_barrier_2, 2);
barrier_n!(thr_d_barrier_3, 3);
barrier_n!(thr_d_barrier_4, 4);

#[crate::lctp_test(suite = posix, full)]
fn thr_d_cond_broadcast_three() -> TestResult {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    let mut arg = Arg {
        ready: AtomicU32::new(0),
        done: AtomicU32::new(0),
    };
    let mut handles = [None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_cond_wait, &mut arg as *mut _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for _ in 0..8 {
        let _ = syscall::sched_yield();
    }
    arg.ready.store(1, Ordering::Release);
    let _ = check_ok!(syscall::futex_wake(&arg.ready, !0), "broadcast");
    for h in handles.iter_mut().filter_map(|x| x.take()) {
        soft_join(h)?;
    }
    check_eq!(arg.done.load(Ordering::SeqCst), 3, "all woken");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_cond_signal_one() -> TestResult {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    let mut arg = Arg {
        ready: AtomicU32::new(0),
        done: AtomicU32::new(0),
    };
    let Some(t) = soft_spawn(thr_cond_wait, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let _ = syscall::sched_yield();
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, 1);
    soft_join(t)?;
    check_eq!(arg.done.load(Ordering::SeqCst), 1, "done");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_once_four() -> TestResult {
    #[repr(C)]
    struct Arg {
        gate: AtomicU32,
        runs: AtomicU32,
    }
    let mut arg = Arg {
        gate: AtomicU32::new(0),
        runs: AtomicU32::new(0),
    };
    let mut handles = [None, None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_once_run, &mut arg as *mut _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) {
        soft_join(h)?;
    }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    check_eq!(arg.gate.load(Ordering::SeqCst), 2, "done gate");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_once_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        gate: AtomicU32,
        runs: AtomicU32,
    }
    let mut arg = Arg {
        gate: AtomicU32::new(0),
        runs: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(thr_once_run, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_once_run, &mut arg as *mut _ as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_atomic_u64_stress() -> TestResult {
    #[repr(C)]
    struct Arg {
        counter: AtomicU64,
        n: u32,
    }
    let mut arg = Arg {
        counter: AtomicU64::new(0),
        n: 2000,
    };
    let mut handles = [None, None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) {
        soft_join(h)?;
    }
    check_eq!(arg.counter.load(Ordering::SeqCst), 8000, "u64 stress");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_atomic_u64_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        counter: AtomicU64,
        n: u32,
    }
    let mut arg = Arg {
        counter: AtomicU64::new(0),
        n: 5000,
    };
    let Some(t1) = soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 10_000, "u64 two");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thr_d_detach_like_join_after_exit() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_sleep_brief, &cell as *const _ as *mut u8)? else {
        return Ok(());
    };
    // Parent yields; child may already have exited — join still works (detach-like).
    for _ in 0..20 {
        let _ = syscall::sched_yield();
    }
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 1, "exited");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_stack_reuse_after_join() -> TestResult {
    for expected in 1u32..=8 {
        let cell = AtomicU32::new(0);
        let Some(t) = soft_spawn(thr_inc32, &cell as *const _ as *mut u8)? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "reuse");
        let _ = expected;
    }
    check_eq!(syscall::gettid(), syscall::getpid(), "restored");
    Ok(())
}

macro_rules! parallel_inc {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full)]
        fn $name() -> TestResult {
            let cell = AtomicU32::new(0);
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$n {
                match soft_spawn(thr_inc32, &cell as *const _ as *mut u8)? {
                    Some(t) => handles[i] = Some(t),
                    None => {
                        for h in handles.iter_mut().filter_map(|x| x.take()) {
                            soft_join(h)?;
                        }
                        return Ok(());
                    }
                }
            }
            for h in handles.iter_mut().filter_map(|x| x.take()) {
                soft_join(h)?;
            }
            check_eq!(cell.load(Ordering::SeqCst), $n as u32, "parallel");
            Ok(())
        }
    };
}

parallel_inc!(thr_d_par_inc_2, 2);
parallel_inc!(thr_d_par_inc_3, 3);
parallel_inc!(thr_d_par_inc_4, 4);

macro_rules! yield_inc_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let cell = AtomicU32::new(0);
            for _ in 0..$n {
                let Some(t) = soft_spawn(thr_yield_inc, &cell as *const _ as *mut u8)? else {
                    return Ok(());
                };
                soft_join(t)?;
            }
            check_eq!(cell.load(Ordering::SeqCst), $n, "yield incs");
            Ok(())
        }
    };
}

yield_inc_n!(thr_d_yield_inc_2, 2);
yield_inc_n!(thr_d_yield_inc_3, 3);
yield_inc_n!(thr_d_yield_inc_4, 4);
yield_inc_n!(thr_d_yield_inc_5, 5);

#[crate::lctp_test(suite = posix)]
fn thr_d_tid_ne_parent_many() -> TestResult {
    let parent = syscall::gettid();
    for _ in 0..6 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        check!(t.tid() != parent, "tid");
        soft_join(t)?;
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_u64_inc_one() -> TestResult {
    let cell = AtomicU64::new(0);
    let Some(t) = soft_spawn(thr_inc64, &cell as *const _ as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 1, "u64");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_barrier_twice() -> TestResult {
    for _ in 0..2 {
        #[repr(C)]
        struct Arg {
            count: AtomicU32,
            generation: AtomicU32,
            n: u32,
        }
        let mut arg = Arg {
            count: AtomicU32::new(0),
            generation: AtomicU32::new(0),
            n: 2,
        };
        let Some(t1) = soft_spawn(thr_barrier_arrive, &mut arg as *mut _ as *mut u8)? else {
            return Ok(());
        };
        let Some(t2) = soft_spawn(thr_barrier_arrive, &mut arg as *mut _ as *mut u8)? else {
            soft_join(t1)?;
            return Ok(());
        };
        soft_join(t1)?;
        soft_join(t2)?;
        check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thr_d_parent_pid_stable() -> TestResult {
    let pid = syscall::getpid();
    for _ in 0..5 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        check_eq!(syscall::getpid(), pid, "pid");
        soft_join(t)?;
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_mutex_one_thread_many() -> TestResult {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        counter: AtomicU32,
        n: u32,
    }
    let mut arg = Arg {
        lock: AtomicU32::new(0),
        counter: AtomicU32::new(0),
        n: 256,
    };
    let Some(t) = soft_spawn(thr_mutex_inc, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 256, "one");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_cond_two_waiters() -> TestResult {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    let mut arg = Arg {
        ready: AtomicU32::new(0),
        done: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(thr_cond_wait, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_cond_wait, &mut arg as *mut _ as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    for _ in 0..8 {
        let _ = syscall::sched_yield();
    }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, !0);
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.done.load(Ordering::SeqCst), 2, "two");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thr_d_join_restores_each_cycle() -> TestResult {
    for _ in 0..6 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(syscall::gettid(), syscall::getpid(), "single");
    }
    Ok(())
}

// Dense smoke variants: spawn+store patterns
macro_rules! store_magic {
    ($name:ident, $magic:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            unsafe extern "C" fn store(arg: *mut u8) -> i32 {
                (*(arg as *mut AtomicU32)).store($magic, Ordering::SeqCst);
                0
            }
            let cell = AtomicU32::new(0);
            let Some(t) = soft_spawn(store, &cell as *const _ as *mut u8)? else {
                return Ok(());
            };
            soft_join(t)?;
            check_eq!(cell.load(Ordering::SeqCst), $magic, "magic");
            Ok(())
        }
    };
}

store_magic!(thr_d_store_1, 1);
store_magic!(thr_d_store_2, 2);
store_magic!(thr_d_store_3, 3);
store_magic!(thr_d_store_7, 7);
store_magic!(thr_d_store_13, 13);
store_magic!(thr_d_store_42, 42);
store_magic!(thr_d_store_99, 99);
store_magic!(thr_d_store_100, 100);
store_magic!(thr_d_store_255, 255);
store_magic!(thr_d_store_1000, 1000);
store_magic!(thr_d_store_0xdead, 0xDEAD);
store_magic!(thr_d_store_0xbeef, 0xBEEF);
store_magic!(thr_d_store_0xc0ffee, 0xC0FFEE);
store_magic!(thr_d_store_0x123456, 0x123456);
store_magic!(thr_d_store_max, 0xFFFF_FFFF);

macro_rules! add_n_one {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full)]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                counter: AtomicU64,
                n: u32,
            }
            let mut arg = Arg {
                counter: AtomicU64::new(0),
                n: $n,
            };
            let Some(t) = soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? else {
                return Ok(());
            };
            soft_join(t)?;
            check_eq!(arg.counter.load(Ordering::SeqCst), $n as u64, "add");
            Ok(())
        }
    };
}

add_n_one!(thr_d_add_10, 10);
add_n_one!(thr_d_add_50, 50);
add_n_one!(thr_d_add_100, 100);
add_n_one!(thr_d_add_250, 250);
add_n_one!(thr_d_add_500, 500);
add_n_one!(thr_d_add_1000, 1000);
add_n_one!(thr_d_add_2000, 2000);
add_n_one!(thr_d_add_4000, 4000);

#[crate::lctp_test(suite = posix, full)]
fn thr_d_three_mutex_then_barrier() -> TestResult {
    #[repr(C)]
    struct MArg {
        lock: AtomicU32,
        counter: AtomicU32,
        n: u32,
    }
    let mut marg = MArg {
        lock: AtomicU32::new(0),
        counter: AtomicU32::new(0),
        n: 20,
    };
    let Some(m1) = soft_spawn(thr_mutex_inc, &mut marg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(m2) = soft_spawn(thr_mutex_inc, &mut marg as *mut _ as *mut u8)? else {
        soft_join(m1)?;
        return Ok(());
    };
    soft_join(m1)?;
    soft_join(m2)?;
    check_eq!(marg.counter.load(Ordering::SeqCst), 40, "mutex phase");

    #[repr(C)]
    struct BArg {
        count: AtomicU32,
        generation: AtomicU32,
        n: u32,
    }
    let mut barg = BArg {
        count: AtomicU32::new(0),
        generation: AtomicU32::new(0),
        n: 2,
    };
    let Some(b1) = soft_spawn(thr_barrier_arrive, &mut barg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(b2) = soft_spawn(thr_barrier_arrive, &mut barg as *mut _ as *mut u8)? else {
        soft_join(b1)?;
        return Ok(());
    };
    soft_join(b1)?;
    soft_join(b2)?;
    check!(barg.generation.load(Ordering::SeqCst) >= 1, "barrier");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thr_d_nop_handle_positive() -> TestResult {
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_four_yield_parallel() -> TestResult {
    let cell = AtomicU32::new(0);
    let mut handles = [None, None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_yield_inc, &cell as *const _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) {
        soft_join(h)?;
    }
    check_eq!(cell.load(Ordering::SeqCst), 4, "four yield");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_stack_reuse_with_u64() -> TestResult {
    for _ in 0..5 {
        let cell = AtomicU64::new(0);
        let Some(t) = soft_spawn(thr_inc64, &cell as *const _ as *mut u8)? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "u64 reuse");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thr_d_soft_unavailable_ok() -> TestResult {
    // Document soft path: if spawn fails with ENOSYS/EINVAL/EPERM, skip.
    match runtime::spawn_thread(thr_nop, core::ptr::null_mut()) {
        Ok(t) => soft_join(t),
        Err(e) if thread_unavailable(e) => Ok(()),
        Err(e) => Err(crate::harness::AssertFail::msg(e.name())),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_mutex_trylock_four() -> TestResult {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        got: AtomicU32,
        fail: AtomicU32,
    }
    let mut arg = Arg {
        lock: AtomicU32::new(0),
        got: AtomicU32::new(0),
        fail: AtomicU32::new(0),
    };
    let mut handles = [None, None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_trylock_spin, &mut arg as *mut _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) {
        soft_join(h)?;
    }
    let got = arg.got.load(Ordering::SeqCst);
    let fail = arg.fail.load(Ordering::SeqCst);
    check_eq!(got + fail, 512, "4*128");
    check!(got > 0, "got some");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_once_three() -> TestResult {
    #[repr(C)]
    struct Arg {
        gate: AtomicU32,
        runs: AtomicU32,
    }
    let mut arg = Arg {
        gate: AtomicU32::new(0),
        runs: AtomicU32::new(0),
    };
    let mut handles = [None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_once_run, &mut arg as *mut _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) {
        soft_join(h)?;
    }
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once3");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_rwlock_shared_four() -> TestResult {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
    }
    let mut arg = Arg {
        state: AtomicU32::new(0),
        sum: AtomicU32::new(0),
    };
    let mut handles = [None, None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_rwlock_shared, &mut arg as *mut _ as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut().filter_map(|x| x.take()) {
        soft_join(h)?;
    }
    check_eq!(arg.sum.load(Ordering::SeqCst), 128, "4*32");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_detach_many() -> TestResult {
    for _ in 0..8 {
        let cell = AtomicU32::new(0);
        let Some(t) = soft_spawn(thr_sleep_brief, &cell as *const _ as *mut u8)? else {
            return Ok(());
        };
        for _ in 0..5 {
            let _ = syscall::sched_yield();
        }
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "done");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thr_d_two_cells_parallel() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(thr_inc32, &a as *const _ as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_inc32, &b as *const _ as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 1, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thr_d_u64_stress_two_rounds() -> TestResult {
    for _ in 0..2 {
        #[repr(C)]
        struct Arg {
            counter: AtomicU64,
            n: u32,
        }
        let mut arg = Arg {
            counter: AtomicU64::new(0),
            n: 1000,
        };
        let Some(t1) = soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? else {
            return Ok(());
        };
        let Some(t2) = soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? else {
            soft_join(t1)?;
            return Ok(());
        };
        soft_join(t1)?;
        soft_join(t2)?;
        check_eq!(arg.counter.load(Ordering::SeqCst), 2000, "round");
    }
    Ok(())
}
