//! aarch64 Linux syscall instruction (`svc #0`).

use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall(
    number: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> isize {
    let ret: isize;
    // Linux aarch64: x8=nr, x0..x5 = args.
    asm!(
        "svc #0",
        in("x8") number,
        inout("x0") a0 => ret,
        in("x1") a1,
        in("x2") a2,
        in("x3") a3,
        in("x4") a4,
        in("x5") a5,
        options(nostack, preserves_flags)
    );
    ret
}

/// Raw `clone` + trampoline for a freestanding thread (aarch64 ABI).
///
/// Parent returns the child tid. The child calls `entry(arg)`, then `SYS_exit`.
/// Kernel arg order: flags, stack, parent_tid, tls, child_tid.
///
/// # Safety
/// Same constraints as the x86_64 variant.
#[inline(never)]
pub unsafe fn clone_thread(
    flags: usize,
    stack: *mut u8,
    parent_tid: *mut i32,
    child_tid: *mut i32,
    entry: unsafe extern "C" fn(*mut u8) -> i32,
    arg: *mut u8,
) -> isize {
    let mut ret: isize;
    // x9–x12 hold inputs so they do not collide with svc arg regs x0–x4/x8.
    asm!(
        // Child stack: push (entry, arg), 16-byte aligned.
        "mov x1, x9",
        "and x1, x1, #0xfffffffffffffff0",
        "stp x10, x11, [x1, #-16]!",
        // sys_clone(flags, stack, ptid, tls=0, ctid)
        "mov x8, #220",
        "mov x0, x12",
        // x1 = stack
        "mov x2, x13",
        "mov x3, xzr",
        "mov x4, x14",
        "svc #0",
        "cbnz x0, 2f",
        // child
        "ldp x1, x0, [sp], #16",
        "blr x1",
        "mov x8, #93",
        "svc #0",
        "brk #0",
        "2:",
        lateout("x0") ret,
        out("x1") _,
        out("x2") _,
        out("x3") _,
        out("x4") _,
        out("x8") _,
        in("x9") stack as usize,
        in("x10") entry as usize,
        in("x11") arg as usize,
        in("x12") flags,
        in("x13") parent_tid as usize,
        in("x14") child_tid as usize,
        options(nostack)
    );
    ret
}

/// Syscall numbers for aarch64.
pub mod nr {
    pub const READ: usize = 63;
    pub const WRITE: usize = 64;
    pub const CLOSE: usize = 57;
    pub const FSTAT: usize = 80;
    pub const LSEEK: usize = 62;
    pub const MMAP: usize = 222;
    pub const MPROTECT: usize = 226;
    pub const MUNMAP: usize = 215;
    pub const BRK: usize = 214;
    pub const RT_SIGACTION: usize = 134;
    pub const RT_SIGPROCMASK: usize = 135;
    pub const IOCTL: usize = 29;
    pub const PREAD64: usize = 67;
    pub const PWRITE64: usize = 68;
    pub const READV: usize = 65;
    pub const WRITEV: usize = 66;
    pub const SCHED_YIELD: usize = 124;
    pub const MADVISE: usize = 233;
    pub const DUP: usize = 23;
    pub const DUP3: usize = 24;
    pub const NANOSLEEP: usize = 101;
    pub const GETPID: usize = 172;
    pub const SOCKET: usize = 198;
    pub const SOCKETPAIR: usize = 199;
    pub const BIND: usize = 200;
    pub const LISTEN: usize = 201;
    pub const ACCEPT: usize = 202;
    pub const CONNECT: usize = 203;
    pub const GETSOCKNAME: usize = 204;
    pub const SENDTO: usize = 206;
    pub const RECVFROM: usize = 207;
    pub const SHUTDOWN: usize = 210;
    pub const CLONE: usize = 220;
    pub const EXECVE: usize = 221;
    pub const EXIT: usize = 93;
    pub const EXIT_GROUP: usize = 94;
    pub const WAIT4: usize = 260;
    pub const KILL: usize = 129;
    pub const UNAME: usize = 160;
    pub const FCNTL: usize = 25;
    pub const FSYNC: usize = 82;
    pub const FDATASYNC: usize = 83;
    pub const FTRUNCATE: usize = 46;
    pub const GETCWD: usize = 17;
    pub const CHDIR: usize = 49;
    pub const FCHDIR: usize = 50;
    pub const RENAMEAT: usize = 38;
    pub const LINKAT: usize = 37;
    pub const UNLINKAT: usize = 35;
    pub const SYMLINKAT: usize = 36;
    pub const READLINKAT: usize = 78;
    pub const FCHMOD: usize = 52;
    pub const FCHMODAT: usize = 53;
    pub const GETTIMEOFDAY: usize = 169;
    pub const GETUID: usize = 174;
    pub const GETGID: usize = 176;
    pub const GETEUID: usize = 175;
    pub const GETEGID: usize = 177;
    pub const GETPPID: usize = 173;
    pub const GETTID: usize = 178;
    pub const TKILL: usize = 130;
    pub const SET_TID_ADDRESS: usize = 96;
    pub const GETDENTS64: usize = 61;
    pub const CLOCK_GETTIME: usize = 113;
    pub const CLOCK_NANOSLEEP: usize = 115;
    pub const OPENAT: usize = 56;
    pub const MKDIRAT: usize = 34;
    pub const MKNODAT: usize = 33;
    pub const NEWFSTATAT: usize = 79;
    pub const FACCESSAT: usize = 48;
    pub const UTIMENSAT: usize = 88;
    pub const PIPE2: usize = 59;
    pub const EPOLL_CREATE1: usize = 20;
    pub const EPOLL_CTL: usize = 21;
    pub const EPOLL_PWAIT: usize = 22;
    pub const EVENTFD2: usize = 19;
    pub const PRLIMIT64: usize = 261;
    pub const GETRANDOM: usize = 278;
    pub const FALLOCATE: usize = 47;
    pub const PPOLL: usize = 73;
    pub const ACCEPT4: usize = 242;
    pub const CLOCK_GETRES: usize = 114;
    pub const MREMAP: usize = 216;
    pub const MSYNC: usize = 227;
    pub const MINCORE: usize = 232;
    pub const SENDFILE: usize = 71;
    pub const SPLICE: usize = 76;
    pub const TEE: usize = 77;
    pub const COPY_FILE_RANGE: usize = 285;
    pub const MEMFD_CREATE: usize = 279;
    pub const TIMERFD_CREATE: usize = 85;
    pub const TIMERFD_SETTIME: usize = 86;
    pub const TIMERFD_GETTIME: usize = 87;
    pub const SIGNALFD4: usize = 74;
    pub const FUTEX: usize = 98;
    pub const WAITID: usize = 95;
    pub const GETPGID: usize = 155;
    pub const SETPGID: usize = 154;
    pub const GETSID: usize = 156;
    pub const SETSID: usize = 157;
    pub const GETPRIORITY: usize = 141;
    pub const SETPRIORITY: usize = 140;
    pub const SCHED_SETAFFINITY: usize = 122;
    pub const SCHED_GETAFFINITY: usize = 123;
    pub const SCHED_GETSCHEDULER: usize = 120;
    pub const SCHED_GETPARAM: usize = 121;
    pub const PRCTL: usize = 167;
    pub const SYSINFO: usize = 179;
    pub const GETRUSAGE: usize = 165;
    pub const TIMES: usize = 153;
    pub const SYNC: usize = 81;
    pub const SYNCFS: usize = 267;
    pub const FLOCK: usize = 32;
    pub const STATFS: usize = 43;
    pub const FSTATFS: usize = 44;
    pub const GETPEERNAME: usize = 205;
    pub const SETSOCKOPT: usize = 208;
    pub const GETSOCKOPT: usize = 209;
    pub const GETRESUID: usize = 148;
    pub const GETRESGID: usize = 150;
    pub const GETCPU: usize = 168;
    pub const SIGALTSTACK: usize = 132;
    pub const RT_SIGPENDING: usize = 136;
    pub const PREADV: usize = 69;
    pub const PWRITEV: usize = 70;
    pub const PSELECT6: usize = 72;
    pub const RECVMMSG: usize = 243;
    pub const SENDMMSG: usize = 269;
    pub const SENDMSG: usize = 211;
    pub const RECVMSG: usize = 212;
    pub const INOTIFY_INIT1: usize = 26;
    pub const INOTIFY_ADD_WATCH: usize = 27;
    pub const INOTIFY_RM_WATCH: usize = 28;
    pub const EXECVEAT: usize = 281;
    pub const CLONE3: usize = 435;
    pub const CLOSE_RANGE: usize = 436;
    pub const RENAMEAT2: usize = 276;
    pub const PIDFD_OPEN: usize = 434;
    pub const PIDFD_SEND_SIGNAL: usize = 424;
    pub const GETITIMER: usize = 102;
    pub const SETITIMER: usize = 103;
    pub const VMSPLICE: usize = 75;
    pub const STATX: usize = 291;
    pub const OPENAT2: usize = 437;
    pub const SYNC_FILE_RANGE: usize = 84;
    pub const FADVISE64: usize = 223;
    pub const MEMBARRIER: usize = 283;
    pub const PERSONALITY: usize = 92;
    pub const CAPGET: usize = 90;
    pub const UNSHARE: usize = 97;
    pub const READAHEAD: usize = 213;
    pub const PROCESS_VM_READV: usize = 270;
    pub const PROCESS_VM_WRITEV: usize = 271;
    pub const KCMP: usize = 272;
    pub const SHMGET: usize = 194;
    pub const SHMCTL: usize = 195;
    pub const SHMAT: usize = 196;
    pub const SHMDT: usize = 197;
    pub const MQ_OPEN: usize = 180;
    pub const MQ_UNLINK: usize = 181;
    pub const MQ_TIMEDSEND: usize = 182;
    pub const MQ_TIMEDRECEIVE: usize = 183;
    pub const MQ_NOTIFY: usize = 184;
    /// Combined getattr/setattr on aarch64.
    pub const MQ_GETSETATTR: usize = 185;
    pub const MLOCKALL: usize = 230;
    pub const MUNLOCKALL: usize = 231;
    pub const IO_SETUP: usize = 0;
    pub const IO_DESTROY: usize = 1;
    pub const LANDLOCK_CREATE_RULESET: usize = 444;
    pub const LANDLOCK_ADD_RULE: usize = 445;
    pub const LANDLOCK_RESTRICT_SELF: usize = 446;
    pub const USERFAULTFD: usize = 282;
    pub const PIDFD_GETFD: usize = 438;
    pub const CLOCK_SETTIME: usize = 112;
    pub const IO_URING_SETUP: usize = 425;
    pub const IO_URING_ENTER: usize = 426;
    pub const IO_URING_REGISTER: usize = 427;
    pub const TIMER_CREATE: usize = 107;
    pub const TIMER_GETTIME: usize = 108;
    pub const TIMER_SETTIME: usize = 110;
    pub const TIMER_DELETE: usize = 111;
    pub const MSGGET: usize = 186;
    pub const MSGCTL: usize = 187;
    pub const MSGRCV: usize = 188;
    pub const MSGSND: usize = 189;
    pub const SEMGET: usize = 190;
    pub const SEMCTL: usize = 191;
    pub const SEMTIMEDOP: usize = 192;
    pub const FSOPEN: usize = 430;
    pub const FSCONFIG: usize = 431;
    pub const MLOCK: usize = 228;
    pub const MUNLOCK: usize = 229;
    pub const MOUNT: usize = 40;
    pub const PTRACE: usize = 117;
}
