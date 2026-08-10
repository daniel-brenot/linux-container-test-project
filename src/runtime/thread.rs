//! Minimal freestanding thread helper (`clone` + futex join, no libpthread).
//!
//! Threads share the address space (`CLONE_VM`) and exit via `SYS_exit` only.
//! Join waits on `CLONE_CHILD_CLEARTID` (shared futex). Stacks are munmapped
//! after join so the process returns to a single-threaded state.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::syscall::{self, map, prot, CLONE_THREAD_FLAGS, Errno, Timespec};

type Result<T> = core::result::Result<T, Errno>;

/// Default mapped stack size (includes room for the clear-tid word at the base).
pub const THREAD_STACK_SIZE: usize = 128 * 1024;

/// Entry point for a freestanding thread. Return value becomes `SYS_exit` status.
pub type ThreadFn = unsafe extern "C" fn(*mut u8) -> i32;

/// Handle for a joinable freestanding thread.
pub struct Thread {
    tid: i32,
    stack_addr: usize,
    stack_len: usize,
    /// Kernel CLEARTID word (low address of the mapping). Non-zero while alive.
    clear_tid: *const AtomicU32,
}

impl Thread {
    pub fn tid(&self) -> i32 {
        self.tid
    }
}

/// True when `clone` thread flags are rejected in this environment.
pub fn thread_unavailable(err: Errno) -> bool {
    matches!(err, Errno::ENOSYS | Errno::EINVAL | Errno::EPERM)
}

/// Spawn `entry(arg)` on a fresh anonymous stack. Soft-errors from `clone` are
/// returned as `Err` so callers can soft-skip.
pub fn spawn_thread(entry: ThreadFn, arg: *mut u8) -> Result<Thread> {
    let stack_len = THREAD_STACK_SIZE;
    let stack_addr = syscall::mmap(
        0,
        stack_len,
        prot::PROT_READ | prot::PROT_WRITE,
        map::MAP_PRIVATE | map::MAP_ANONYMOUS,
        -1,
        0,
    )?;

    // Clear-tid word lives at the low end; stack grows down from the high end.
    let clear_tid = stack_addr as *mut AtomicU32;
    unsafe {
        clear_tid.write(AtomicU32::new(0));
    }

    let stack_top = (stack_addr + stack_len) as *mut u8;
    // Leave a guard gap above the clear-tid word (one page when possible).
    let _ = stack_addr;

    let flags = CLONE_THREAD_FLAGS;
    let ctid = clear_tid as *mut i32;

    let tid = unsafe {
        match syscall::clone_thread(flags, stack_top, ctid, ctid, entry, arg) {
            Ok(t) => t,
            Err(e) => {
                let _ = syscall::munmap(stack_addr, stack_len);
                return Err(e);
            }
        }
    };

    if tid <= 0 {
        let _ = syscall::munmap(stack_addr, stack_len);
        return Err(Errno::EINVAL);
    }

    Ok(Thread {
        tid,
        stack_addr,
        stack_len,
        clear_tid: clear_tid as *const AtomicU32,
    })
}

/// Wait for `thread` to exit (CLEARTID futex), then unmap its stack.
pub fn join_thread(thread: Thread) -> Result<()> {
    let Thread {
        tid,
        stack_addr,
        stack_len,
        clear_tid,
    } = thread;

    let ctid = unsafe { &*clear_tid };
    // CLEARTID wake is a shared futex; poll with short timeouts + yield so we
    // still make progress if the wake is missed.
    let mut spins = 0u32;
    loop {
        let v = ctid.load(Ordering::Acquire);
        if v == 0 {
            break;
        }
        let timeout = Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000, // 50ms
        };
        match syscall::futex_wait_addr(
            clear_tid as *const u32,
            v,
            Some(&timeout),
            false,
        ) {
            Ok(()) | Err(Errno::EAGAIN) | Err(Errno::EINTR) | Err(Errno::ETIMEDOUT) => {}
            Err(e) => {
                let _ = syscall::munmap(stack_addr, stack_len);
                return Err(e);
            }
        }
        spins = spins.wrapping_add(1);
        if spins & 0xf == 0 {
            let _ = syscall::sched_yield();
        }
        if spins > 2000 {
            let _ = syscall::munmap(stack_addr, stack_len);
            return Err(Errno::ETIMEDOUT);
        }
    }

    let _ = tid;
    let _ = syscall::munmap(stack_addr, stack_len);
    Ok(())
}
