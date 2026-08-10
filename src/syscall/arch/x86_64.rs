//! x86_64 Linux syscall instruction (`syscall`).

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
    // Linux x86_64: rax=nr, rdi/rsi/rdx/r10/r8/r9 = args; rcx/r11 clobbered.
    asm!(
        "syscall",
        inlateout("rax") number as isize => ret,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        in("r10") a3,
        in("r8") a4,
        in("r9") a5,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

/// Raw `clone` + trampoline for a freestanding thread.
///
/// Parent returns the child tid. The child calls `entry(arg)`, then `SYS_exit`
/// (never returns here). `stack` is the high address of the mapped stack; 16
/// bytes below it are used for the trampoline frame.
///
/// # Safety
/// `stack` must be writable and 16-byte aligned; `entry`/`arg`/`tid` pointers
/// must remain valid for the life of the child.
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
    // Fixed regs avoid clashes with syscall arg/return registers.
    asm!(
        // Build child stack frame: [entry][arg], SP 16-byte aligned.
        "mov rsi, r12",
        "and rsi, -16",
        "sub rsi, 16",
        "mov [rsi], r13",
        "mov [rsi + 8], r14",
        // sys_clone(flags, stack, parent_tid, child_tid, tls=0)
        "mov eax, 56",
        "mov rdi, r15",
        // rsi = stack
        "mov rdx, r8",
        "mov r10, r9",
        "xor r8, r8",
        "syscall",
        "test rax, rax",
        "jnz 2f",
        // child
        "xor ebp, ebp",
        "pop rax",
        "pop rdi",
        "call rax",
        "mov edi, eax",
        "mov eax, 60",
        "syscall",
        "ud2",
        "2:",
        lateout("rax") ret,
        lateout("rcx") _,
        lateout("r11") _,
        out("rdi") _,
        out("rsi") _,
        out("rdx") _,
        lateout("r8") _,
        out("r10") _,
        in("r12") stack as usize,
        in("r13") entry as usize,
        in("r14") arg as usize,
        in("r15") flags,
        in("r8") parent_tid as usize,
        in("r9") child_tid as usize,
        options(nostack)
    );
    ret
}

/// Syscall numbers for x86_64.
pub mod nr {
    pub const READ: usize = 0;
    pub const WRITE: usize = 1;
    pub const CLOSE: usize = 3;
    pub const FSTAT: usize = 5;
    pub const LSEEK: usize = 8;
    pub const MMAP: usize = 9;
    pub const MPROTECT: usize = 10;
    pub const MUNMAP: usize = 11;
    pub const BRK: usize = 12;
    pub const RT_SIGACTION: usize = 13;
    pub const RT_SIGPROCMASK: usize = 14;
    pub const IOCTL: usize = 16;
    pub const PREAD64: usize = 17;
    pub const PWRITE64: usize = 18;
    pub const READV: usize = 19;
    pub const WRITEV: usize = 20;
    pub const ACCESS: usize = 21;
    pub const SCHED_YIELD: usize = 24;
    pub const MADVISE: usize = 28;
    pub const DUP: usize = 32;
    pub const DUP2: usize = 33;
    pub const NANOSLEEP: usize = 35;
    pub const GETPID: usize = 39;
    pub const SOCKET: usize = 41;
    pub const CONNECT: usize = 42;
    pub const ACCEPT: usize = 43;
    pub const SENDTO: usize = 44;
    pub const RECVFROM: usize = 45;
    pub const BIND: usize = 49;
    pub const LISTEN: usize = 50;
    pub const GETSOCKNAME: usize = 51;
    pub const SOCKETPAIR: usize = 53;
    pub const CLONE: usize = 56;
    pub const FORK: usize = 57;
    pub const EXIT: usize = 60;
    pub const WAIT4: usize = 61;
    pub const KILL: usize = 62;
    pub const UNAME: usize = 63;
    pub const FCNTL: usize = 72;
    pub const FSYNC: usize = 74;
    pub const FDATASYNC: usize = 75;
    pub const FTRUNCATE: usize = 77;
    pub const GETCWD: usize = 79;
    pub const CHDIR: usize = 80;
    pub const FCHDIR: usize = 81;
    pub const RENAME: usize = 82;
    pub const MKDIR: usize = 83;
    pub const RMDIR: usize = 84;
    pub const LINK: usize = 86;
    pub const UNLINK: usize = 87;
    pub const SYMLINK: usize = 88;
    pub const READLINK: usize = 89;
    pub const CHMOD: usize = 90;
    pub const FCHMOD: usize = 91;
    pub const GETTIMEOFDAY: usize = 96;
    pub const GETUID: usize = 102;
    pub const GETGID: usize = 104;
    pub const GETEUID: usize = 107;
    pub const GETEGID: usize = 108;
    pub const GETPPID: usize = 110;
    pub const GETTID: usize = 186;
    pub const TKILL: usize = 200;
    pub const FUTEX: usize = 202;
    pub const EPOLL_CREATE: usize = 213;
    pub const EPOLL_CTL: usize = 233;
    pub const EPOLL_WAIT: usize = 232;
    pub const GETDENTS64: usize = 217;
    pub const SET_TID_ADDRESS: usize = 218;
    pub const CLOCK_GETTIME: usize = 228;
    pub const CLOCK_NANOSLEEP: usize = 230;
    pub const EXIT_GROUP: usize = 231;
    pub const OPENAT: usize = 257;
    pub const MKDIRAT: usize = 258;
    pub const MKNODAT: usize = 259;
    pub const NEWFSTATAT: usize = 262;
    pub const UNLINKAT: usize = 263;
    pub const RENAMEAT: usize = 264;
    pub const LINKAT: usize = 265;
    pub const SYMLINKAT: usize = 266;
    pub const READLINKAT: usize = 267;
    pub const FCHMODAT: usize = 268;
    pub const FACCESSAT: usize = 269;
    pub const UTIMENSAT: usize = 280;
    pub const EPOLL_CREATE1: usize = 291;
    pub const DUP3: usize = 292;
    pub const PIPE2: usize = 293;
    pub const EVENTFD2: usize = 290;
    pub const PRLIMIT64: usize = 302;
    pub const GETRANDOM: usize = 318;
    pub const EXECVEAT: usize = 322;
    pub const FALLOCATE: usize = 285;
    pub const POLL: usize = 7;
    pub const PPOLL: usize = 271;
    pub const ACCEPT4: usize = 288;
    pub const SHUTDOWN: usize = 48;
    pub const CLOCK_GETRES: usize = 229;
    pub const MREMAP: usize = 25;
    pub const MSYNC: usize = 26;
    pub const MINCORE: usize = 27;
    pub const SENDFILE: usize = 40;
    pub const SPLICE: usize = 275;
    pub const TEE: usize = 276;
    pub const COPY_FILE_RANGE: usize = 326;
    pub const MEMFD_CREATE: usize = 319;
    pub const TIMERFD_CREATE: usize = 283;
    pub const TIMERFD_SETTIME: usize = 286;
    pub const TIMERFD_GETTIME: usize = 287;
    pub const SIGNALFD4: usize = 289;
    pub const WAITID: usize = 247;
    pub const GETPGID: usize = 121;
    pub const SETPGID: usize = 109;
    pub const GETSID: usize = 124;
    pub const SETSID: usize = 112;
    pub const GETPRIORITY: usize = 140;
    pub const SETPRIORITY: usize = 141;
    pub const SCHED_SETAFFINITY: usize = 203;
    pub const SCHED_GETAFFINITY: usize = 204;
    pub const SCHED_GETSCHEDULER: usize = 145;
    pub const SCHED_GETPARAM: usize = 143;
    pub const PRCTL: usize = 157;
    pub const SYSINFO: usize = 99;
    pub const GETRUSAGE: usize = 98;
    pub const TIMES: usize = 100;
    pub const SYNC: usize = 162;
    pub const SYNCFS: usize = 306;
    pub const FLOCK: usize = 73;
    pub const STATFS: usize = 137;
    pub const FSTATFS: usize = 138;
    pub const GETPEERNAME: usize = 52;
    pub const SETSOCKOPT: usize = 54;
    pub const GETSOCKOPT: usize = 55;
    pub const GETRESUID: usize = 118;
    pub const GETRESGID: usize = 120;
    pub const GETCPU: usize = 309;
    pub const SIGALTSTACK: usize = 131;
    pub const RT_SIGPENDING: usize = 127;
    pub const PREADV: usize = 295;
    pub const PWRITEV: usize = 296;
    pub const PSELECT6: usize = 270;
    pub const RECVMMSG: usize = 299;
    pub const SENDMMSG: usize = 307;
    pub const SENDMSG: usize = 46;
    pub const RECVMSG: usize = 47;
    pub const INOTIFY_INIT1: usize = 294;
    pub const INOTIFY_ADD_WATCH: usize = 254;
    pub const INOTIFY_RM_WATCH: usize = 255;
    pub const CLOSE_RANGE: usize = 436;
    pub const RENAMEAT2: usize = 316;
    pub const PIDFD_OPEN: usize = 434;
    pub const PIDFD_SEND_SIGNAL: usize = 424;
    pub const GETITIMER: usize = 36;
    pub const SETITIMER: usize = 38;
    pub const VMSPLICE: usize = 278;
    pub const STATX: usize = 332;
    pub const OPENAT2: usize = 437;
    pub const SYNC_FILE_RANGE: usize = 277;
    pub const FADVISE64: usize = 221;
    pub const MEMBARRIER: usize = 324;
    pub const PERSONALITY: usize = 135;
    pub const CAPGET: usize = 125;
    pub const UNSHARE: usize = 272;
    pub const READAHEAD: usize = 187;
    pub const PROCESS_VM_READV: usize = 310;
    pub const PROCESS_VM_WRITEV: usize = 311;
    pub const KCMP: usize = 312;
    pub const SHMGET: usize = 29;
    pub const SHMAT: usize = 30;
    pub const SHMCTL: usize = 31;
    pub const SHMDT: usize = 67;
    pub const MQ_OPEN: usize = 240;
    pub const MQ_UNLINK: usize = 241;
    pub const LANDLOCK_CREATE_RULESET: usize = 444;
    pub const LANDLOCK_ADD_RULE: usize = 445;
    pub const LANDLOCK_RESTRICT_SELF: usize = 446;
    pub const USERFAULTFD: usize = 323;
    pub const PIDFD_GETFD: usize = 438;
    pub const CLOCK_SETTIME: usize = 227;
    pub const IO_URING_SETUP: usize = 425;
    pub const IO_URING_ENTER: usize = 426;
    pub const IO_URING_REGISTER: usize = 427;
    pub const TIMER_CREATE: usize = 222;
    pub const TIMER_SETTIME: usize = 223;
    pub const TIMER_GETTIME: usize = 224;
    pub const TIMER_DELETE: usize = 226;
    pub const SEMGET: usize = 64;
    pub const SEMOP: usize = 65;
    pub const SEMCTL: usize = 66;
    pub const MSGGET: usize = 68;
    pub const MSGSND: usize = 69;
    pub const MSGRCV: usize = 70;
    pub const MSGCTL: usize = 71;
    pub const FSOPEN: usize = 430;
    pub const FSCONFIG: usize = 431;
    pub const MLOCK: usize = 149;
    pub const MUNLOCK: usize = 150;
}
