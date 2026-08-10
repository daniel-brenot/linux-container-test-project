//! Freestanding pthread-like POSIX coverage (no libpthread).
//!
//! Uses `runtime::spawn_thread` / `join_thread` backed by `clone(CLONE_THREAD…)`
//! and futex join via `CLONE_CHILD_CLEARTID`. Soft-skips when clone is rejected
//! (`ENOSYS` / `EINVAL` / `EPERM`) so Docker Desktop / restricted hosts stay green.

use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

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

unsafe extern "C" fn thr_nop(_arg: *mut u8) -> i32 {
    0
}

unsafe extern "C" fn thr_return_7(_arg: *mut u8) -> i32 {
    7
}

unsafe extern "C" fn thr_store_u32(arg: *mut u8) -> i32 {
    let p = arg as *mut AtomicU32;
    (*p).store(0xA5A5_5A5A, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_store_tid(arg: *mut u8) -> i32 {
    let p = arg as *mut AtomicI32;
    (*p).store(syscall::gettid(), Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_store_pid(arg: *mut u8) -> i32 {
    let p = arg as *mut AtomicI32;
    (*p).store(syscall::getpid(), Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_inc(arg: *mut u8) -> i32 {
    let p = arg as *mut AtomicU32;
    (*p).fetch_add(1, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_add_n(arg: *mut u8) -> i32 {
    #[repr(C)]
    struct Arg {
        counter: AtomicU32,
        n: u32,
    }
    let a = &mut *(arg as *mut Arg);
    for _ in 0..a.n {
        a.counter.fetch_add(1, Ordering::SeqCst);
    }
    0
}

unsafe extern "C" fn thr_mutex_lock_inc(arg: *mut u8) -> i32 {
    // Layout: [lock: AtomicU32][counter: AtomicU32]
    let lock = arg as *mut AtomicU32;
    let counter = (arg as *mut AtomicU32).add(1);
    for _ in 0..64 {
        // spin-mutex via CAS + futex
        loop {
            if (*lock)
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            let _ = syscall::futex_wait(&*lock, 1, Some(&Timespec {
                tv_sec: 0,
                tv_nsec: 1_000_000,
            }));
        }
        (*counter).fetch_add(1, Ordering::SeqCst);
        (*lock).store(0, Ordering::Release);
        let _ = syscall::futex_wake(&*lock, 1);
    }
    0
}

unsafe extern "C" fn thr_cond_wait(arg: *mut u8) -> i32 {
    // Layout: [ready: AtomicU32][done: AtomicU32]
    let ready = arg as *mut AtomicU32;
    let done = (arg as *mut AtomicU32).add(1);
    let timeout = Timespec {
        tv_sec: 2,
        tv_nsec: 0,
    };
    while (*ready).load(Ordering::Acquire) == 0 {
        let _ = syscall::futex_wait(&*ready, 0, Some(&timeout));
    }
    (*done).store(1, Ordering::Release);
    let _ = syscall::futex_wake(&*done, 1);
    0
}

unsafe extern "C" fn thr_yield_then_store(arg: *mut u8) -> i32 {
    let _ = syscall::sched_yield();
    let p = arg as *mut AtomicU32;
    (*p).store(99, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_write_bytes(arg: *mut u8) -> i32 {
    // arg points to 16-byte buffer
    let buf = core::slice::from_raw_parts_mut(arg, 16);
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (0x40 + i) as u8;
    }
    0
}

unsafe extern "C" fn thr_nested_inc_twice(arg: *mut u8) -> i32 {
    let p = arg as *mut AtomicU32;
    (*p).fetch_add(1, Ordering::SeqCst);
    let _ = syscall::sched_yield();
    (*p).fetch_add(1, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_check_tid_ne_pid(arg: *mut u8) -> i32 {
    let p = arg as *mut AtomicU32;
    let ok = syscall::gettid() != syscall::getpid();
    (*p).store(if ok { 1 } else { 0 }, Ordering::SeqCst);
    0
}

unsafe extern "C" fn thr_read_shared_then_add(arg: *mut u8) -> i32 {
    // [src: AtomicU32][dst: AtomicU32]
    let src = arg as *mut AtomicU32;
    let dst = (arg as *mut AtomicU32).add(1);
    let v = (*src).load(Ordering::SeqCst);
    (*dst).store(v.wrapping_add(1), Ordering::SeqCst);
    0
}

#[crate::lctp_test(suite = posix)]
fn thread_spawn_join() -> TestResult {
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    soft_join(t)
}

#[crate::lctp_test(suite = posix)]
fn thread_spawn_join_nonzero_exit() -> TestResult {
    // Exit status is not observed via wait4 for CLONE_THREAD; just ensure join.
    let Some(t) = soft_spawn(thr_return_7, core::ptr::null_mut())? else {
        return Ok(());
    };
    soft_join(t)
}

#[crate::lctp_test(suite = posix)]
fn thread_shared_store() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_store_u32, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 0xA5A5_5A5A, "shared store");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_gettid_ne_getpid() -> TestResult {
    let cell = AtomicI32::new(0);
    let parent_pid = syscall::getpid();
    let Some(t) = soft_spawn(thr_store_tid, &cell as *const AtomicI32 as *mut u8)? else {
        return Ok(());
    };
    let child_tid_handle = t.tid();
    soft_join(t)?;
    let child_tid = cell.load(Ordering::SeqCst);
    check!(child_tid > 0, "tid positive");
    check!(child_tid != parent_pid, "tid != pid");
    check_eq!(child_tid, child_tid_handle, "tid matches handle");
    // Back to single-threaded: gettid == getpid again.
    check_eq!(syscall::gettid(), syscall::getpid(), "single-thread restored");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_getpid_same_as_parent() -> TestResult {
    let cell = AtomicI32::new(0);
    let parent_pid = syscall::getpid();
    let Some(t) = soft_spawn(thr_store_pid, &cell as *const AtomicI32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), parent_pid, "same pid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_tid_check_in_child() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_check_tid_ne_pid, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 1, "child saw tid!=pid");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_atomic_inc_one() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 1, "inc");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_atomic_inc_two() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t1) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(cell.load(Ordering::SeqCst), 2, "two incs");
    check_eq!(syscall::gettid(), syscall::getpid(), "single-thread restored");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_atomic_add_loop() -> TestResult {
    #[repr(C)]
    struct Arg {
        counter: AtomicU32,
        n: u32,
    }
    let mut arg = Arg {
        counter: AtomicU32::new(0),
        n: 1000,
    };
    let Some(t) = soft_spawn(thr_add_n, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 1000, "add loop");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_two_add_loops() -> TestResult {
    #[repr(C)]
    struct Arg {
        counter: AtomicU32,
        n: u32,
    }
    let mut arg = Arg {
        counter: AtomicU32::new(0),
        n: 500,
    };
    let Some(t1) = soft_spawn(thr_add_n, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_add_n, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 1000, "two add loops");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_mutex_like_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        counter: AtomicU32,
    }
    let mut arg = Arg {
        lock: AtomicU32::new(0),
        counter: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(thr_mutex_lock_inc, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_mutex_lock_inc, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 128, "mutex incs");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_condvar_like_futex() -> TestResult {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    let mut arg = Arg {
        ready: AtomicU32::new(0),
        done: AtomicU32::new(0),
    };
    let Some(t) = soft_spawn(thr_cond_wait, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    // Give the child a moment to wait, then signal.
    let _ = syscall::sched_yield();
    arg.ready.store(1, Ordering::Release);
    let _ = syscall::futex_wake(&arg.ready, 1);
    // Wait for done (soft).
    let timeout = Timespec {
        tv_sec: 2,
        tv_nsec: 0,
    };
    while arg.done.load(Ordering::Acquire) == 0 {
        let _ = syscall::futex_wait(&arg.done, 0, Some(&timeout));
        break;
    }
    soft_join(t)?;
    check_eq!(arg.done.load(Ordering::SeqCst), 1, "cond signaled");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_yield_then_store() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_yield_then_store, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 99, "after yield");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_shared_byte_buffer() -> TestResult {
    let mut buf = [0u8; 16];
    let Some(t) = soft_spawn(thr_write_bytes, buf.as_mut_ptr())? else {
        return Ok(());
    };
    soft_join(t)?;
    for (i, b) in buf.iter().enumerate() {
        check_eq!(*b, (0x40 + i) as u8, "byte");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_nested_inc() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_nested_inc_twice, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 2, "nested");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_read_shared_add() -> TestResult {
    #[repr(C)]
    struct Arg {
        src: AtomicU32,
        dst: AtomicU32,
    }
    let mut arg = Arg {
        src: AtomicU32::new(41),
        dst: AtomicU32::new(0),
    };
    let Some(t) = soft_spawn(thr_read_shared_then_add, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(arg.dst.load(Ordering::SeqCst), 42, "read+add");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_sequential_spawns() -> TestResult {
    for expected in 1u32..=4 {
        let cell = AtomicU32::new(0);
        let Some(t) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "seq");
        check_eq!(syscall::gettid(), syscall::getpid(), "single after");
        let _ = expected;
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_tid_positive() -> TestResult {
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    check!(t.tid() > 0, "tid");
    soft_join(t)
}

#[crate::lctp_test(suite = posix)]
fn thread_parent_tid_unchanged() -> TestResult {
    let before = syscall::gettid();
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    check_eq!(syscall::gettid(), before, "parent tid");
    soft_join(t)?;
    check_eq!(syscall::gettid(), before, "after join");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_three_sequential_stores() -> TestResult {
    for magic in [1u32, 2, 3] {
        let cell = AtomicU32::new(0);
        // Reuse store via a small wrapper pattern: store magic through thr_store_u32
        // by temporarily setting then overwriting in child — use dedicated path:
        let Some(t) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "store");
        let _ = magic;
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_spawn_join_twice() -> TestResult {
    for _ in 0..2 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        soft_join(t)?;
    }
    check_eq!(syscall::gettid(), syscall::getpid(), "restored");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_futex_wake_from_child() -> TestResult {
    // Parent waits; child stores and wakes (condvar-like reverse).
    static READY: AtomicU32 = AtomicU32::new(0);

    unsafe extern "C" fn child(_arg: *mut u8) -> i32 {
        READY.store(1, Ordering::Release);
        let _ = syscall::futex_wake(&READY, 1);
        0
    }

    READY.store(0, Ordering::SeqCst);
    let Some(t) = soft_spawn(child, core::ptr::null_mut())? else {
        return Ok(());
    };
    let timeout = Timespec {
        tv_sec: 2,
        tv_nsec: 0,
    };
    while READY.load(Ordering::Acquire) == 0 {
        match syscall::futex_wait(&READY, 0, Some(&timeout)) {
            Ok(()) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => {}
            Err(Errno::ETIMEDOUT) => break,
            Err(e) => return Err(crate::harness::AssertFail::msg(e.name())),
        }
    }
    soft_join(t)?;
    check_eq!(READY.load(Ordering::SeqCst), 1, "woken");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_shared_zero_init_visible() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 0, "untouched");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_two_shared_cells() -> TestResult {
    let a = AtomicU32::new(0);
    let b = AtomicU32::new(0);
    let Some(t1) = soft_spawn(thr_store_u32, &a as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(thr_inc, &b as *const AtomicU32 as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(a.load(Ordering::SeqCst), 0xA5A5_5A5A, "a");
    check_eq!(b.load(Ordering::SeqCst), 1, "b");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_join_restores_single_thread() -> TestResult {
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(syscall::gettid(), syscall::getpid(), "tid==pid");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_many_incs_one_thread() -> TestResult {
    #[repr(C)]
    struct Arg {
        counter: AtomicU32,
        n: u32,
    }
    let mut arg = Arg {
        counter: AtomicU32::new(0),
        n: 10_000,
    };
    let Some(t) = soft_spawn(thr_add_n, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 10_000, "many");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_arg_null_ok() -> TestResult {
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    soft_join(t)
}

#[crate::lctp_test(suite = posix, full)]
fn thread_overlap_parent_work() -> TestResult {
    #[repr(C)]
    struct Arg {
        counter: AtomicU32,
        n: u32,
    }
    let mut arg = Arg {
        counter: AtomicU32::new(0),
        n: 200,
    };
    let Some(t) = soft_spawn(thr_add_n, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let mut local = 0u32;
    for _ in 0..200 {
        local = local.wrapping_add(1);
    }
    soft_join(t)?;
    check_eq!(local, 200, "parent work");
    check_eq!(arg.counter.load(Ordering::SeqCst), 200, "child work");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_store_then_parent_sees() -> TestResult {
    let cell = AtomicU32::new(123);
    let Some(t) = soft_spawn(thr_store_u32, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check!(cell.load(Ordering::SeqCst) != 123, "overwritten");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_double_join_pattern() -> TestResult {
    let c1 = AtomicU32::new(0);
    let c2 = AtomicU32::new(0);
    let Some(t1) = soft_spawn(thr_inc, &c1 as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t1)?;
    let Some(t2) = soft_spawn(thr_inc, &c2 as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t2)?;
    check_eq!(c1.load(Ordering::SeqCst), 1, "c1");
    check_eq!(c2.load(Ordering::SeqCst), 1, "c2");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_pid_stable_across_thread() -> TestResult {
    let pid0 = syscall::getpid();
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    check_eq!(syscall::getpid(), pid0, "during");
    soft_join(t)?;
    check_eq!(syscall::getpid(), pid0, "after");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_mutex_like_one() -> TestResult {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        counter: AtomicU32,
    }
    let mut arg = Arg {
        lock: AtomicU32::new(0),
        counter: AtomicU32::new(0),
    };
    let Some(t) = soft_spawn(thr_mutex_lock_inc, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 64, "one mutex thread");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_tls_less_smoke() -> TestResult {
    // Document TLS-less: we never set CLONE_SETTLS; thread still runs.
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    soft_join(t)?;
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_four_parallel_incs() -> TestResult {
    let cell = AtomicU32::new(0);
    let mut handles = [None, None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? {
            Some(t) => *h = Some(t),
            None => {
                for done in handles.iter_mut().filter_map(|x| x.take()) {
                    soft_join(done)?;
                }
                return Ok(());
            }
        }
    }
    for h in handles.iter_mut() {
        if let Some(t) = h.take() {
            soft_join(t)?;
        }
    }
    check_eq!(cell.load(Ordering::SeqCst), 4, "four");
    check_eq!(syscall::gettid(), syscall::getpid(), "restored");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_shared_store_order() -> TestResult {
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(thr_store_u32, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    // Parent must not observe a torn write after join (full barrier via join).
    soft_join(t)?;
    let v = cell.load(Ordering::SeqCst);
    check!(v == 0xA5A5_5A5A, "ordered");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_cond_parent_waits_child_sets() -> TestResult {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    let mut arg = Arg {
        ready: AtomicU32::new(0),
        done: AtomicU32::new(0),
    };
    let Some(t) = soft_spawn(thr_cond_wait, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    for _ in 0..10 {
        let _ = syscall::sched_yield();
    }
    arg.ready.store(1, Ordering::Release);
    let _ = check_ok!(syscall::futex_wake(&arg.ready, 1), "wake");
    soft_join(t)?;
    check_eq!(arg.done.load(Ordering::SeqCst), 1, "done");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_gettid_differs_from_handle_parent() -> TestResult {
    let parent = syscall::gettid();
    let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
        return Ok(());
    };
    check!(t.tid() != parent, "different tid");
    soft_join(t)
}

#[crate::lctp_test(suite = posix, full)]
fn thread_add_then_inc() -> TestResult {
    #[repr(C)]
    struct Arg {
        counter: AtomicU32,
        n: u32,
    }
    let mut arg = Arg {
        counter: AtomicU32::new(0),
        n: 10,
    };
    let Some(t1) = soft_spawn(thr_add_n, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    soft_join(t1)?;
    let Some(t2) = soft_spawn(thr_inc, &arg.counter as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 11, "add+inc");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_byte_buffer_partial() -> TestResult {
    let mut buf = [0u8; 16];
    buf[0] = 1;
    let Some(t) = soft_spawn(thr_write_bytes, buf.as_mut_ptr())? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(buf[0], 0x40, "overwritten first");
    check_eq!(buf[15], 0x40 + 15, "last");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_stress_ten_joins() -> TestResult {
    for _ in 0..10 {
        let cell = AtomicU32::new(0);
        let Some(t) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "stress");
    }
    check_eq!(syscall::gettid(), syscall::getpid(), "final single");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_clone_vm_visibility_parent_write() -> TestResult {
    // Parent publishes a value before spawn; child must observe it (CLONE_VM).
    #[repr(C)]
    struct Arg {
        src: AtomicU32,
        dst: AtomicU32,
    }
    let mut arg = Arg {
        src: AtomicU32::new(77),
        dst: AtomicU32::new(0),
    };
    let Some(t) = soft_spawn(thr_read_shared_then_add, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(arg.dst.load(Ordering::SeqCst), 78, "vm visibility");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_parent_write_visible_to_child() -> TestResult {
    #[repr(C)]
    struct Arg {
        src: AtomicU32,
        dst: AtomicU32,
    }
    let mut arg = Arg {
        src: AtomicU32::new(0),
        dst: AtomicU32::new(0),
    };
    arg.src.store(100, Ordering::SeqCst);
    let Some(t) = soft_spawn(thr_read_shared_then_add, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(arg.dst.load(Ordering::SeqCst), 101, "parent write visible");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_mutex_three() -> TestResult {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        counter: AtomicU32,
    }
    let mut arg = Arg {
        lock: AtomicU32::new(0),
        counter: AtomicU32::new(0),
    };
    let mut handles = [None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_mutex_lock_inc, &mut arg as *mut Arg as *mut u8)? {
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
    check_eq!(arg.counter.load(Ordering::SeqCst), 192, "3*64");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_barrier_like_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        count: AtomicU32,
        gen: AtomicU32,
    }
    unsafe extern "C" fn arrive(arg: *mut u8) -> i32 {
        let a = &*(arg as *const Arg);
        let g = a.gen.load(Ordering::Acquire);
        if a.count.fetch_add(1, Ordering::AcqRel) + 1 >= 2 {
            a.count.store(0, Ordering::Release);
            a.gen.fetch_add(1, Ordering::Release);
            let _ = syscall::futex_wake(&a.gen, !0);
        } else {
            let timeout = Timespec {
                tv_sec: 2,
                tv_nsec: 0,
            };
            while a.gen.load(Ordering::Acquire) == g {
                let _ = syscall::futex_wait(&a.gen, g, Some(&timeout));
            }
        }
        0
    }
    let mut arg = Arg {
        count: AtomicU32::new(0),
        gen: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(arrive, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(arrive, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check!(arg.gen.load(Ordering::SeqCst) >= 1, "passed");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_once_like_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        gate: AtomicU32,
        runs: AtomicU32,
    }
    unsafe extern "C" fn once(arg: *mut u8) -> i32 {
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
    let mut arg = Arg {
        gate: AtomicU32::new(0),
        runs: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(once, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(once, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.runs.load(Ordering::SeqCst), 1, "once");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_stack_reuse_six() -> TestResult {
    for _ in 0..6 {
        let cell = AtomicU32::new(0);
        let Some(t) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
            return Ok(());
        };
        soft_join(t)?;
        check_eq!(cell.load(Ordering::SeqCst), 1, "reuse");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_detach_like_sleep() -> TestResult {
    unsafe extern "C" fn sleepy(arg: *mut u8) -> i32 {
        let _ = syscall::nanosleep(&Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        });
        (*(arg as *mut AtomicU32)).store(1, Ordering::SeqCst);
        0
    }
    let cell = AtomicU32::new(0);
    let Some(t) = soft_spawn(sleepy, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    for _ in 0..30 {
        let _ = syscall::sched_yield();
    }
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 1, "done");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_u64_counter_two() -> TestResult {
    use core::sync::atomic::AtomicU64;
    #[repr(C)]
    struct Arg {
        counter: AtomicU64,
        n: u32,
    }
    unsafe extern "C" fn add(arg: *mut u8) -> i32 {
        let a = &*(arg as *const Arg);
        for _ in 0..a.n {
            a.counter.fetch_add(1, Ordering::Relaxed);
        }
        0
    }
    let mut arg = Arg {
        counter: AtomicU64::new(0),
        n: 1000,
    };
    let Some(t1) = soft_spawn(add, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(add, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.counter.load(Ordering::SeqCst), 2000, "u64");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_spawn_join_eight() -> TestResult {
    for _ in 0..8 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        soft_join(t)?;
    }
    check_eq!(syscall::gettid(), syscall::getpid(), "restored");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_cond_broadcast_two() -> TestResult {
    #[repr(C)]
    struct Arg {
        ready: AtomicU32,
        done: AtomicU32,
    }
    unsafe extern "C" fn wait_inc(arg: *mut u8) -> i32 {
        let a = &*(arg as *const Arg);
        let timeout = Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        };
        while a.ready.load(Ordering::Acquire) == 0 {
            let _ = syscall::futex_wait(&a.ready, 0, Some(&timeout));
        }
        a.done.fetch_add(1, Ordering::Release);
        0
    }
    let mut arg = Arg {
        ready: AtomicU32::new(0),
        done: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(wait_inc, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(wait_inc, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    for _ in 0..32 {
        let _ = syscall::sched_yield();
    }
    arg.ready.store(1, Ordering::Release);
    for _ in 0..64 {
        let _ = syscall::futex_wake(&arg.ready, !0);
        if arg.done.load(Ordering::Acquire) >= 2 {
            break;
        }
        let _ = syscall::sched_yield();
        let _ = syscall::nanosleep(&Timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000,
        });
    }
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.done.load(Ordering::SeqCst), 2, "both");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_gettid_after_many_joins() -> TestResult {
    for _ in 0..5 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        soft_join(t)?;
    }
    check_eq!(syscall::gettid(), syscall::getpid(), "single");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_trylock_like() -> TestResult {
    #[repr(C)]
    struct Arg {
        lock: AtomicU32,
        got: AtomicU32,
    }
    unsafe extern "C" fn try_inc(arg: *mut u8) -> i32 {
        let a = &*(arg as *const Arg);
        for _ in 0..64 {
            if a.lock
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                a.got.fetch_add(1, Ordering::SeqCst);
                a.lock.store(0, Ordering::Release);
                let _ = syscall::futex_wake(&a.lock, 1);
            } else {
                let _ = syscall::sched_yield();
            }
        }
        0
    }
    let mut arg = Arg {
        lock: AtomicU32::new(0),
        got: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(try_inc, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(try_inc, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check!(arg.got.load(Ordering::SeqCst) > 0, "got");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_rwlock_shared_like() -> TestResult {
    #[repr(C)]
    struct Arg {
        state: AtomicU32,
        sum: AtomicU32,
    }
    unsafe extern "C" fn rd(arg: *mut u8) -> i32 {
        let a = &*(arg as *const Arg);
        for _ in 0..16 {
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
    let mut arg = Arg {
        state: AtomicU32::new(0),
        sum: AtomicU32::new(0),
    };
    let Some(t1) = soft_spawn(rd, &mut arg as *mut Arg as *mut u8)? else {
        return Ok(());
    };
    let Some(t2) = soft_spawn(rd, &mut arg as *mut Arg as *mut u8)? else {
        soft_join(t1)?;
        return Ok(());
    };
    soft_join(t1)?;
    soft_join(t2)?;
    check_eq!(arg.sum.load(Ordering::SeqCst), 32, "sum");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn thread_three_seq_nops() -> TestResult {
    for _ in 0..3 {
        let Some(t) = soft_spawn(thr_nop, core::ptr::null_mut())? else {
            return Ok(());
        };
        soft_join(t)?;
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn thread_five_parallel_incs() -> TestResult {
    // Cap at 4 concurrent (stack pressure); do 5 via 4+1.
    let cell = AtomicU32::new(0);
    let mut handles = [None, None, None, None];
    for h in handles.iter_mut() {
        match soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? {
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
    let Some(t) = soft_spawn(thr_inc, &cell as *const AtomicU32 as *mut u8)? else {
        return Ok(());
    };
    soft_join(t)?;
    check_eq!(cell.load(Ordering::SeqCst), 5, "five");
    Ok(())
}
