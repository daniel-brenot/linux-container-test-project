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
}
