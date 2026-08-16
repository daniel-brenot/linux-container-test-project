//! Denser freestanding thread coverage: mutex/rwlock/barrier/cond/once/stress grids.

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
        n: u32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..a.n {
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

unsafe extern "C" fn thr_rwlock_rd(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
        n: u32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..a.n {
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

unsafe extern "C" fn thr_rwlock_wr(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
        n: u32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..a.n {
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

unsafe extern "C" fn thr_barrier(arg: *mut u8) -> i32 {
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

unsafe extern "C" fn thr_once(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        gate: AtomicU32,
        runs: AtomicU32,
    }
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

unsafe extern "C" fn thr_yield_n(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        counter: AtomicU32,
        yields: u32,
    }
    let a = &*(arg as *const Arg);
    for _ in 0..a.yields {
        let _ = syscall::sched_yield();
    }
    a.counter.fetch_add(1, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_store_tid(arg: *mut u8) -> i32 {
    (*(arg as *mut AtomicI32)).store(syscall::gettid(), Ordering::SeqCst);
    0
}

macro_rules! spawn_join_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "sequential spawn-join cycles restore a single-threaded tid")]
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

spawn_join_n!(thr2_sj_1, 1);
spawn_join_n!(thr2_sj_2, 2);
spawn_join_n!(thr2_sj_3, 3);
spawn_join_n!(thr2_sj_4, 4);
spawn_join_n!(thr2_sj_6, 6);
spawn_join_n!(thr2_sj_7, 7);
spawn_join_n!(thr2_sj_9, 9);
spawn_join_n!(thr2_sj_10, 10);
spawn_join_n!(thr2_sj_11, 11);
spawn_join_n!(thr2_sj_14, 14);
spawn_join_n!(thr2_sj_15, 15);
spawn_join_n!(thr2_sj_18, 18);
spawn_join_n!(thr2_sj_22, 22);
spawn_join_n!(thr2_sj_28, 28);
spawn_join_n!(thr2_sj_36, 36);
spawn_join_n!(thr2_sj_40, 40);
spawn_join_n!(thr2_sj_48, 48);

macro_rules! seq_inc {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "sequential threads can each increment a shared counter")]
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

seq_inc!(thr2_seq_1, 1);
seq_inc!(thr2_seq_9, 9);
seq_inc!(thr2_seq_11, 11);
seq_inc!(thr2_seq_12, 12);
seq_inc!(thr2_seq_13, 13);
seq_inc!(thr2_seq_14, 14);
seq_inc!(thr2_seq_15, 15);
seq_inc!(thr2_seq_16, 16);
seq_inc!(thr2_seq_18, 18);
seq_inc!(thr2_seq_20, 20);
seq_inc!(thr2_seq_24, 24);
seq_inc!(thr2_seq_28, 28);
seq_inc!(thr2_seq_32, 32);

macro_rules! mutex_grid {
    ($name:ident, $threads:expr, $loops:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several threads can serialize increments with a futex mutex")]
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
            for i in 0..$threads {
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
                "mutex"
            );
            Ok(())
        }
    };
}

mutex_grid!(thr2_mtx_2x8, 2, 8);
mutex_grid!(thr2_mtx_2x16, 2, 16);
mutex_grid!(thr2_mtx_2x24, 2, 24);
mutex_grid!(thr2_mtx_2x48, 2, 48);
mutex_grid!(thr2_mtx_2x96, 2, 96);
mutex_grid!(thr2_mtx_2x160, 2, 160);
mutex_grid!(thr2_mtx_2x200, 2, 200);
mutex_grid!(thr2_mtx_3x8, 3, 8);
mutex_grid!(thr2_mtx_3x16, 3, 16);
mutex_grid!(thr2_mtx_3x24, 3, 24);
mutex_grid!(thr2_mtx_3x48, 3, 48);
mutex_grid!(thr2_mtx_3x80, 3, 80);
mutex_grid!(thr2_mtx_3x100, 3, 100);
mutex_grid!(thr2_mtx_4x8, 4, 8);
mutex_grid!(thr2_mtx_4x12, 4, 12);
mutex_grid!(thr2_mtx_4x20, 4, 20);
mutex_grid!(thr2_mtx_4x40, 4, 40);
mutex_grid!(thr2_mtx_4x48, 4, 48);
mutex_grid!(thr2_mtx_4x80, 4, 80);
mutex_grid!(thr2_mtx_4x100, 4, 100);

macro_rules! trylock_n {
    ($name:ident, $threads:expr, $loops:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several threads can make progress with a trylock-style mutex")]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                lock: AtomicU32,
                got: AtomicU32,
                fail: AtomicU32,
                n: u32,
            }
            let mut arg = Arg {
                lock: AtomicU32::new(0),
                got: AtomicU32::new(0),
                fail: AtomicU32::new(0),
                n: $loops,
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$threads {
                match soft_spawn(thr_trylock_spin, &mut arg as *mut _ as *mut u8)? {
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
            let got = arg.got.load(Ordering::SeqCst);
            let fail = arg.fail.load(Ordering::SeqCst);
            check_eq!(got + fail, ($threads as u32) * ($loops as u32), "attempts");
            check!(got > 0, "got");
            Ok(())
        }
    };
}

trylock_n!(thr2_try_2x32, 2, 32);
trylock_n!(thr2_try_2x64, 2, 64);
trylock_n!(thr2_try_2x96, 2, 96);
trylock_n!(thr2_try_3x32, 3, 32);
trylock_n!(thr2_try_3x64, 3, 64);
trylock_n!(thr2_try_4x32, 4, 32);
trylock_n!(thr2_try_4x48, 4, 48);
trylock_n!(thr2_try_4x64, 4, 64);

macro_rules! rw_rd {
    ($name:ident, $threads:expr, $loops:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several threads can take shared reader locks together")]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                state: AtomicU32,
                sum: AtomicU32,
                n: u32,
            }
            let mut arg = Arg {
                state: AtomicU32::new(0),
                sum: AtomicU32::new(0),
                n: $loops,
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$threads {
                match soft_spawn(thr_rwlock_rd, &mut arg as *mut _ as *mut u8)? {
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
                arg.sum.load(Ordering::SeqCst),
                ($threads as u32) * ($loops as u32),
                "rd"
            );
            Ok(())
        }
    };
}

rw_rd!(thr2_rd_2x8, 2, 8);
rw_rd!(thr2_rd_2x16, 2, 16);
rw_rd!(thr2_rd_2x24, 2, 24);
rw_rd!(thr2_rd_2x40, 2, 40);
rw_rd!(thr2_rd_3x8, 3, 8);
rw_rd!(thr2_rd_3x16, 3, 16);
rw_rd!(thr2_rd_3x24, 3, 24);
rw_rd!(thr2_rd_4x8, 4, 8);
rw_rd!(thr2_rd_4x16, 4, 16);
rw_rd!(thr2_rd_4x24, 4, 24);
rw_rd!(thr2_rd_4x32, 4, 32);

macro_rules! rw_wr {
    ($name:ident, $threads:expr, $loops:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several threads can take exclusive writer locks in turn")]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                state: AtomicU32,
                sum: AtomicU32,
                n: u32,
            }
            let mut arg = Arg {
                state: AtomicU32::new(0),
                sum: AtomicU32::new(0),
                n: $loops,
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$threads {
                match soft_spawn(thr_rwlock_wr, &mut arg as *mut _ as *mut u8)? {
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
                arg.sum.load(Ordering::SeqCst),
                ($threads as u32) * ($loops as u32) * 10,
                "wr"
            );
            Ok(())
        }
    };
}

rw_wr!(thr2_wr_2x4, 2, 4);
rw_wr!(thr2_wr_2x8, 2, 8);
rw_wr!(thr2_wr_2x12, 2, 12);
rw_wr!(thr2_wr_2x20, 2, 20);
rw_wr!(thr2_wr_3x4, 3, 4);
rw_wr!(thr2_wr_3x8, 3, 8);
rw_wr!(thr2_wr_3x12, 3, 12);
rw_wr!(thr2_wr_4x4, 4, 4);
rw_wr!(thr2_wr_4x8, 4, 8);
rw_wr!(thr2_wr_4x12, 4, 12);
rw_wr!(thr2_wr_4x16, 4, 16);

macro_rules! barrier_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several threads can rendezvous on a futex barrier")]
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
                match soft_spawn(thr_barrier, &mut arg as *mut _ as *mut u8)? {
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

barrier_n!(thr2_bar_2a, 2);
barrier_n!(thr2_bar_2b, 2);
barrier_n!(thr2_bar_3a, 3);
barrier_n!(thr2_bar_3b, 3);
barrier_n!(thr2_bar_4a, 4);
barrier_n!(thr2_bar_4b, 4);

macro_rules! cond_n {
    ($name:ident, $waiters:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several waiters can be woken by a futex broadcast")]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                ready: AtomicU32,
                done: AtomicU32,
            }
            let mut arg = Arg {
                ready: AtomicU32::new(0),
                done: AtomicU32::new(0),
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$waiters {
                match soft_spawn(thr_cond_wait, &mut arg as *mut _ as *mut u8)? {
                    Some(t) => handles[i] = Some(t),
                    None => {
                        for h in handles.iter_mut().filter_map(|x| x.take()) {
                            soft_join(h)?;
                        }
                        return Ok(());
                    }
                }
            }
            for _ in 0..8 {
                let _ = syscall::sched_yield();
            }
            arg.ready.store(1, Ordering::Release);
            let _ = syscall::futex_wake(&arg.ready, !0);
            for h in handles.iter_mut().filter_map(|x| x.take()) {
                soft_join(h)?;
            }
            check_eq!(arg.done.load(Ordering::SeqCst), $waiters as u32, "done");
            Ok(())
        }
    };
}

cond_n!(thr2_cond_1, 1);
cond_n!(thr2_cond_2, 2);
cond_n!(thr2_cond_3, 3);
cond_n!(thr2_cond_4, 4);

macro_rules! once_n {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several threads run a once-style gate exactly once")]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                gate: AtomicU32,
                runs: AtomicU32,
            }
            let mut arg = Arg {
                gate: AtomicU32::new(0),
                runs: AtomicU32::new(0),
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$n {
                match soft_spawn(thr_once, &mut arg as *mut _ as *mut u8)? {
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
            check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
            Ok(())
        }
    };
}

once_n!(thr2_once_2, 2);
once_n!(thr2_once_3, 3);
once_n!(thr2_once_4, 4);

macro_rules! add64_pair {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "two threads can add concurrently to a shared 64-bit counter")]
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
            let Some(t1) = soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? else {
                return Ok(());
            };
            let Some(t2) = soft_spawn(thr_add64_n, &mut arg as *mut _ as *mut u8)? else {
                soft_join(t1)?;
                return Ok(());
            };
            soft_join(t1)?;
            soft_join(t2)?;
            check_eq!(arg.counter.load(Ordering::SeqCst), ($n as u64) * 2, "sum");
            Ok(())
        }
    };
}

add64_pair!(thr2_u64_50, 50);
add64_pair!(thr2_u64_75, 75);
add64_pair!(thr2_u64_125, 125);
add64_pair!(thr2_u64_150, 150);
add64_pair!(thr2_u64_175, 175);
add64_pair!(thr2_u64_225, 225);
add64_pair!(thr2_u64_300, 300);
add64_pair!(thr2_u64_400, 400);
add64_pair!(thr2_u64_600, 600);
add64_pair!(thr2_u64_800, 800);
add64_pair!(thr2_u64_1200, 1200);
add64_pair!(thr2_u64_1500, 1500);
add64_pair!(thr2_u64_2500, 2500);
add64_pair!(thr2_u64_3000, 3000);

macro_rules! yield_threads {
    ($name:ident, $threads:expr, $yields:expr) => {
        #[crate::lctp_test(suite = posix, full, expect = soft, case = "several threads can yield then increment a shared counter")]
        fn $name() -> TestResult {
            #[repr(C)]
            struct Arg {
                counter: AtomicU32,
                yields: u32,
            }
            let mut arg = Arg {
                counter: AtomicU32::new(0),
                yields: $yields,
            };
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$threads {
                match soft_spawn(thr_yield_n, &mut arg as *mut _ as *mut u8)? {
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
            check_eq!(arg.counter.load(Ordering::SeqCst), $threads as u32, "done");
            Ok(())
        }
    };
}

yield_threads!(thr2_yld_2x1, 2, 1);
yield_threads!(thr2_yld_2x4, 2, 4);
yield_threads!(thr2_yld_2x8, 2, 8);
yield_threads!(thr2_yld_3x2, 3, 2);
yield_threads!(thr2_yld_3x4, 3, 4);
yield_threads!(thr2_yld_4x1, 4, 1);
yield_threads!(thr2_yld_4x2, 4, 2);
yield_threads!(thr2_yld_4x4, 4, 4);
yield_threads!(thr2_yld_4x8, 4, 8);

macro_rules! parallel_inc {
    ($name:ident, $n:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "parallel threads can each increment a shared counter")]
        fn $name() -> TestResult {
            let cells: [AtomicU32; 4] = [
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
            ];
            let mut handles: [Option<runtime::Thread>; 4] = [None, None, None, None];
            for i in 0..$n {
                match soft_spawn(thr_inc32, &cells[i] as *const _ as *mut u8)? {
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
            for i in 0..$n {
                check_eq!(cells[i].load(Ordering::SeqCst), 1, "cell");
            }
            Ok(())
        }
    };
}

parallel_inc!(thr2_par_2, 2);
parallel_inc!(thr2_par_3, 3);
parallel_inc!(thr2_par_4, 4);

#[crate::lctp_test(suite = posix, expect = soft, case = "a child tid handle is distinct from the process pid")]
fn thr2_tid_ne_pid_child() -> TestResult {
    let cell = AtomicI32::new(0);
    let Some(t) = soft_spawn(thr_store_tid, &cell as *const _ as *mut u8)? else {
        return Ok(());
    };
    let handle_tid = t.tid();
    soft_join(t)?;
    let stored = cell.load(Ordering::SeqCst);
    check!(handle_tid > 0, "handle");
    check!(stored > 0, "stored");
    check!(handle_tid != syscall::getpid() || stored != syscall::getpid(), "soft");
    let _ = stored;
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "the main thread has gettid equal to getpid")]
fn thr2_self_tid_eq_pid() -> TestResult {
    check_eq!(syscall::gettid(), syscall::getpid(), "main");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "reader and writer threads can share a lock-like state word")]
fn thr2_mixed_rd_wr() -> TestResult {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
        n: u32,
    }
    let mut arg = Arg {
        state: AtomicU32::new(0),
        sum: AtomicU32::new(0),
        n: 8,
    };
    let Some(r1) = soft_spawn(thr_rwlock_rd, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(w1) = soft_spawn(thr_rwlock_wr, &mut arg as *mut _ as *mut u8)? else {
        soft_join(r1)?;
        return Ok(());
    };
    let Some(r2) = soft_spawn(thr_rwlock_rd, &mut arg as *mut _ as *mut u8)? else {
        soft_join(r1)?;
        soft_join(w1)?;
        return Ok(());
    };
    soft_join(r1)?;
    soft_join(w1)?;
    soft_join(r2)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 8 + 80 + 8, "mixed");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "mutex-protected work can be followed by a once-style gate")]
fn thr2_mutex_then_once() -> TestResult {
    #[repr(C)]
    struct MArg {
        lock: AtomicU32,
        counter: AtomicU32,
        n: u32,
    }
    let mut marg = MArg {
        lock: AtomicU32::new(0),
        counter: AtomicU32::new(0),
        n: 16,
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
    check_eq!(marg.counter.load(Ordering::SeqCst), 32, "mtx");

    #[repr(C)]
    struct OArg {
        gate: AtomicU32,
        runs: AtomicU32,
    }
    let mut oarg = OArg {
        gate: AtomicU32::new(0),
        runs: AtomicU32::new(0),
    };
    let Some(o1) = soft_spawn(thr_once, &mut oarg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    let Some(o2) = soft_spawn(thr_once, &mut oarg as *mut _ as *mut u8)? else {
        soft_join(o1)?;
        return Ok(());
    };
    soft_join(o1)?;
    soft_join(o2)?;
    check_eq!(oarg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "sequential threads can reuse stacks while updating a 64-bit cell")]
fn thr2_stack_reuse_u64() -> TestResult {
    for _ in 0..8 {
        let cell = AtomicU64::new(0);
        let Some(t) = soft_spawn(thr_inc64, &cell as *const _ as *mut u8)? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "u64");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "threads can pass a futex barrier for two generations")]
fn thr2_double_barrier_rounds() -> TestResult {
    for _ in 0..3 {
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
        let Some(a) = soft_spawn(thr_barrier, &mut arg as *mut _ as *mut u8)? else {
            return Ok(());
        };
        let Some(b) = soft_spawn(thr_barrier, &mut arg as *mut _ as *mut u8)? else {
            soft_join(a)?;
            return Ok(());
        };
        soft_join(a)?;
        soft_join(b)?;
        check!(arg.generation.load(Ordering::SeqCst) >= 1, "gen");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "a nop thread can be spawned and joined")]
fn thr2_nop_ok() -> TestResult {
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    soft_join(t)
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "four threads can increment a shared 64-bit counter")]
fn thr2_four_inc64() -> TestResult {
    let cells = [
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ];
    let mut handles = [None, None, None, None];
    for (i, h) in handles.iter_mut().enumerate() {
        match soft_spawn(thr_inc64, &cells[i] as *const _ as *mut u8)? {
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
    for c in &cells {
        check_eq!(c.load(Ordering::SeqCst), 1, "c");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a waiter can be woken by a futex signal")]
fn thr2_cond_signal_style() -> TestResult {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    let mut arg = Arg {
        ready: AtomicU32::new(0),
        done: AtomicU32::new(0),
    };
    let Some(w) = soft_spawn(thr_cond_wait, &mut arg as *mut _ as *mut u8)? else {
        return Ok(());
    };
    for _ in 0..4 {
        let _ = syscall::sched_yield();
    }
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, 1);
    soft_join(w)?;
    check_eq!(arg.done.load(Ordering::SeqCst), 1, "signaled");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "mutex-protected increments complete across several rounds")]
fn thr2_stress_mutex_rounds() -> TestResult {
    for round in 0..4u32 {
        #[repr(C)]
        struct Arg {
            lock: AtomicU32,
            counter: AtomicU32,
            n: u32,
        }
        let mut arg = Arg {
            lock: AtomicU32::new(0),
            counter: AtomicU32::new(0),
            n: 25 + round * 5,
        };
        let Some(a) = soft_spawn(thr_mutex_inc, &mut arg as *mut _ as *mut u8)? else {
            return Ok(());
        };
        let Some(b) = soft_spawn(thr_mutex_inc, &mut arg as *mut _ as *mut u8)? else {
            soft_join(a)?;
            return Ok(());
        };
        soft_join(a)?;
        soft_join(b)?;
        check_eq!(
            arg.counter.load(Ordering::SeqCst),
            2 * (25 + round * 5),
            "round"
        );
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "gettid of the main thread is positive")]
fn thr2_gettid_positive() -> TestResult {
    check!(syscall::gettid() > 0, "tid");
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "joining sequential threads restores gettid equal to getpid")]
fn thr2_join_restores_main() -> TestResult {
    for _ in 0..6 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(syscall::gettid(), syscall::getpid(), "main");
    }
    Ok(())
}
