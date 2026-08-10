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
}
