//! Raw Linux syscall ABI and thin wrappers.
//!
//! All kernel interaction goes through this module so tests exercise the
//! syscall interface directly rather than a C library.

#![allow(dead_code)]

mod arch;
mod errno;
mod wrappers;

pub use errno::Errno;
pub use wrappers::*;

/// Linux `AT_FDCWD` — interpret relative paths against the current directory.
pub const AT_FDCWD: i32 = -100;

/// `open` / `openat` flags (subset).
pub mod oflag {
    pub const O_RDONLY: i32 = 0;
    pub const O_WRONLY: i32 = 1;
    pub const O_RDWR: i32 = 2;
    pub const O_CREAT: i32 = 0o100;
    pub const O_EXCL: i32 = 0o200;
    pub const O_NOCTTY: i32 = 0o400;
    pub const O_TRUNC: i32 = 0o1000;
    pub const O_APPEND: i32 = 0o2000;
    pub const O_NONBLOCK: i32 = 0o4000;
    pub const O_DIRECTORY: i32 = 0o200000;
    pub const O_NOFOLLOW: i32 = 0o400000;
    pub const O_CLOEXEC: i32 = 0o2000000;
    pub const O_PATH: i32 = 0o10000000;
}

/// `mmap` / `mprotect` protection bits.
pub mod prot {
    pub const PROT_NONE: i32 = 0;
    pub const PROT_READ: i32 = 1;
    pub const PROT_WRITE: i32 = 2;
    pub const PROT_EXEC: i32 = 4;
}

/// `mmap` flags.
pub mod map {
    pub const MAP_SHARED: i32 = 0x01;
    pub const MAP_PRIVATE: i32 = 0x02;
    pub const MAP_FIXED: i32 = 0x10;
    pub const MAP_ANONYMOUS: i32 = 0x20;
}

/// `waitid` / `wait4` options.
pub mod wait {
    pub const WNOHANG: i32 = 1;
    pub const WUNTRACED: i32 = 2;
    pub const WEXITED: i32 = 4;
}

/// `clock_gettime` clock ids.
pub mod clock {
    pub const CLOCK_REALTIME: i32 = 0;
    pub const CLOCK_MONOTONIC: i32 = 1;
    pub const CLOCK_PROCESS_CPUTIME_ID: i32 = 2;
    pub const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
    pub const CLOCK_MONOTONIC_RAW: i32 = 4;
}

/// `fcntl` commands.
pub mod fcntl_cmd {
    pub const F_DUPFD: i32 = 0;
    pub const F_GETFD: i32 = 1;
    pub const F_SETFD: i32 = 2;
    pub const F_GETFL: i32 = 3;
    pub const F_SETFL: i32 = 4;
    pub const F_DUPFD_CLOEXEC: i32 = 1030;
}

pub const FD_CLOEXEC: i32 = 1;

/// `unlinkat` flag: remove a directory.
pub const AT_REMOVEDIR: i32 = 0x200;

/// `fchmodat` / `faccessat` flag: do not follow symlinks.
pub const AT_SYMLINK_NOFOLLOW: i32 = 0x100;

/// Standard fds.
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Stat {
    // Layout matches the kernel `struct stat` used by `newfstatat` on
    // x86_64 and aarch64 (64-bit). Fields after the ones we care about are
    // padding so size stays correct for the syscall.
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub __pad0: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    pub st_ctime_nsec: i64,
    pub __unused: [i64; 3],
}

impl Default for Stat {
    fn default() -> Self {
        // Safety: all-zero is a valid bit pattern for this POD struct.
        unsafe { core::mem::zeroed() }
    }
}

impl Stat {
    pub fn is_reg(&self) -> bool {
        (self.st_mode & 0o170000) == 0o100000
    }

    pub fn is_dir(&self) -> bool {
        (self.st_mode & 0o170000) == 0o040000
    }

    pub fn is_lnk(&self) -> bool {
        (self.st_mode & 0o170000) == 0o120000
    }

    pub fn is_fifo(&self) -> bool {
        (self.st_mode & 0o170000) == 0o010000
    }

    pub fn mode_bits(&self) -> u32 {
        self.st_mode & 0o7777
    }
}

/// File type bits for `mknodat`.
pub const S_IFIFO: u32 = 0o010000;
pub const S_IFCHR: u32 = 0o020000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFBLK: u32 = 0o060000;
pub const S_IFREG: u32 = 0o100000;
pub const S_IFLNK: u32 = 0o120000;
pub const S_IFSOCK: u32 = 0o140000;

/// Socket domain / type.
pub const AF_UNIX: i32 = 1;
pub const AF_INET: i32 = 2;
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32 = 2;
pub const SOCK_CLOEXEC: i32 = 0o2000000;
pub const SOCK_NONBLOCK: i32 = 0o4000;
pub const SHUT_RD: i32 = 0;
pub const SHUT_WR: i32 = 1;
pub const SHUT_RDWR: i32 = 2;

/// Signals.
pub const SIGHUP: i32 = 1;
pub const SIGINT: i32 = 2;
pub const SIGQUIT: i32 = 3;
pub const SIGKILL: i32 = 9;
pub const SIGUSR1: i32 = 10;
pub const SIGUSR2: i32 = 12;
pub const SIGTERM: i32 = 15;
pub const SIGCHLD: i32 = 17;

/// `sigaction` special handlers / flags (kernel ABI).
pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;

/// poll(2) events.
pub const POLLIN: i16 = 0x0001;
pub const POLLOUT: i16 = 0x0004;
pub const POLLERR: i16 = 0x0008;
pub const POLLHUP: i16 = 0x0010;

/// epoll.
pub const EPOLLIN: u32 = 0x001;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008;
pub const EPOLLHUP: u32 = 0x010;
pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;

/// utimensat special nsec values.
pub const UTIME_NOW: i64 = (1 << 30) - 1;
pub const UTIME_OMIT: i64 = (1 << 30) - 2;

/// prlimit resource ids.
pub const RLIMIT_NOFILE: i32 = 7;

pub mod madvise {
    pub const MADV_NORMAL: i32 = 0;
    pub const MADV_RANDOM: i32 = 1;
    pub const MADV_SEQUENTIAL: i32 = 2;
    pub const MADV_WILLNEED: i32 = 3;
    pub const MADV_DONTNEED: i32 = 4;
}

pub mod poll {
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct PollFd {
        pub fd: i32,
        pub events: i16,
        pub revents: i16,
    }
}

pub mod epoll {
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct EpollEvent {
        pub events: u32,
        pub data: u64,
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IoVec {
    pub iov_base: *mut u8,
    pub iov_len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Rlimit {
    pub rlim_cur: u64,
    pub rlim_max: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UtsName {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

impl Default for UtsName {
    fn default() -> Self {
        // Safety: all-zero is a valid bit pattern for this POD struct.
        unsafe { core::mem::zeroed() }
    }
}

/// Memory sync / remap flags.
pub const MS_ASYNC: i32 = 1;
pub const MS_SYNC: i32 = 4;
pub const MREMAP_MAYMOVE: i32 = 1;

/// memfd_create flags.
pub const MFD_CLOEXEC: i32 = 0x0001;
pub const MFD_ALLOW_SEALING: i32 = 0x0002;

/// timerfd_create flags.
pub const TFD_CLOEXEC: i32 = 0o2000000;
pub const TFD_NONBLOCK: i32 = 0o4000;
pub const TFD_TIMER_ABSTIME: i32 = 1;

/// flock(2) operations.
pub const LOCK_SH: i32 = 1;
pub const LOCK_EX: i32 = 2;
pub const LOCK_UN: i32 = 8;
pub const LOCK_NB: i32 = 4;

/// getpriority / setpriority `which`.
pub const PRIO_PROCESS: i32 = 0;

/// getrusage `who`.
pub const RUSAGE_SELF: i32 = 0;
pub const RUSAGE_CHILDREN: i32 = -1;

/// sched_getscheduler policy.
pub const SCHED_OTHER: i32 = 0;

/// waitid id types.
pub const P_PID: i32 = 1;
pub const P_ALL: i32 = 0;

/// rt_sigprocmask `how`.
pub const SIG_BLOCK: i32 = 0;
pub const SIG_UNBLOCK: i32 = 1;
pub const SIG_SETMASK: i32 = 2;

/// Socket option levels and names.
pub const SOL_SOCKET: i32 = 1;
pub const SO_TYPE: i32 = 3;
pub const SO_REUSEADDR: i32 = 2;
pub const SO_RCVBUF: i32 = 8;

/// prctl options for thread name.
pub const PR_SET_NAME: i32 = 15;
pub const PR_GET_NAME: i32 = 16;

/// fcntl sealing (memfd).
pub const F_ADD_SEALS: i32 = 1033;
pub const F_GET_SEALS: i32 = 1034;
pub const F_SEAL_WRITE: i32 = 0x0008;

/// futex op codes (private).
pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;
pub const FUTEX_PRIVATE_FLAG: u32 = 128;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Itimerspec {
    pub it_interval: Timespec,
    pub it_value: Timespec,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Statfs {
    pub f_type: i64,
    pub f_bsize: i64,
    pub f_blocks: u64,
    pub f_bfree: u64,
    pub f_bavail: u64,
    pub f_files: u64,
    pub f_ffree: u64,
    pub f_fsid: [i32; 2],
    pub f_namelen: i64,
    pub f_frsize: i64,
    pub f_flags: i64,
    pub f_spare: [i64; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Sysinfo {
    pub uptime: i64,
    pub loads: [u64; 3],
    pub totalram: u64,
    pub freeram: u64,
    pub sharedram: u64,
    pub bufferram: u64,
    pub totalswap: u64,
    pub freeswap: u64,
    pub procs: u16,
    pub pad: u16,
    pub totalhigh: u64,
    pub freehigh: u64,
    pub mem_unit: u32,
    pub _f: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Rusage {
    pub ru_utime: Timeval,
    pub ru_stime: Timeval,
    pub ru_maxrss: i64,
    pub ru_ixrss: i64,
    pub ru_idrss: i64,
    pub ru_isrss: i64,
    pub ru_minflt: i64,
    pub ru_majflt: i64,
    pub ru_nswap: i64,
    pub ru_inblock: i64,
    pub ru_oublock: i64,
    pub ru_msgsnd: i64,
    pub ru_msgrcv: i64,
    pub ru_nsignals: i64,
    pub ru_nvcsw: i64,
    pub ru_nivcsw: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Tms {
    pub tms_utime: i64,
    pub tms_stime: i64,
    pub tms_cutime: i64,
    pub tms_cstime: i64,
}

/// Minimal `siginfo_t` buffer for waitid(2).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Siginfo {
    pub data: [u8; 128],
}

impl Default for Siginfo {
    fn default() -> Self {
        Self { data: [0u8; 128] }
    }
}

/// Linux `kernel_sigset_t` on 64-bit targets.
pub type Sigset = u64;

/// Kernel `struct sigaction` layout used by `rt_sigaction` on x86_64/aarch64.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sigaction {
    pub sa_handler: usize,
    pub sa_flags: usize,
    pub sa_restorer: usize,
    pub sa_mask: Sigset,
}

impl Default for Sigaction {
    fn default() -> Self {
        Self {
            sa_handler: SIG_DFL,
            sa_flags: 0,
            sa_restorer: 0,
            sa_mask: 0,
        }
    }
}

/// IPv4 socket address (`struct sockaddr_in`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SockAddrIn {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub sin_zero: [u8; 8],
}

impl Default for SockAddrIn {
    fn default() -> Self {
        Self {
            sin_family: 0,
            sin_port: 0,
            sin_addr: 0,
            sin_zero: [0; 8],
        }
    }
}

impl SockAddrIn {
    /// Build a loopback address with `port` in host byte order.
    pub fn loopback(port: u16) -> Self {
        Self {
            sin_family: AF_INET as u16,
            sin_port: port.to_be(),
            sin_addr: INADDR_LOOPBACK.to_be(),
            sin_zero: [0; 8],
        }
    }

    pub fn port_host(&self) -> u16 {
        u16::from_be(self.sin_port)
    }
}

/// IPv4 loopback in host order (`127.0.0.1`).
pub const INADDR_LOOPBACK: u32 = 0x7f00_0001;

/// signalfd4 flags.
pub const SFD_CLOEXEC: i32 = 0o2000000;
pub const SFD_NONBLOCK: i32 = 0o4000;

/// Kernel `struct signalfd_siginfo` (128 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SignalfdSiginfo {
    pub ssi_signo: u32,
    pub ssi_errno: i32,
    pub ssi_code: i32,
    pub ssi_pid: u32,
    pub ssi_uid: u32,
    pub ssi_fd: i32,
    pub ssi_tid: u32,
    pub ssi_band: u32,
    pub ssi_overrun: u32,
    pub ssi_trapno: u32,
    pub ssi_status: i32,
    pub ssi_int: i32,
    pub ssi_ptr: u64,
    pub ssi_utime: u64,
    pub ssi_stime: u64,
    pub ssi_addr: u64,
    pub ssi_addr_lsb: u16,
    pub __pad2: u16,
    pub ssi_syscall: i32,
    pub ssi_call_addr: u64,
    pub ssi_arch: u32,
    pub __pad: [u8; 28],
}

impl Default for SignalfdSiginfo {
    fn default() -> Self {
        // Safety: all-zero is a valid bit pattern for this POD struct.
        unsafe { core::mem::zeroed() }
    }
}

/// renameat2 flags.
pub const RENAME_NOREPLACE: u32 = 1 << 0;
pub const RENAME_EXCHANGE: u32 = 1 << 1;
pub const RENAME_WHITEOUT: u32 = 1 << 2;

/// close_range flags.
pub const CLOSE_RANGE_UNSHARE: u32 = 1 << 1;
pub const CLOSE_RANGE_CLOEXEC: u32 = 1 << 2;

/// ioctl: get terminal window size.
pub const TIOCGWINSZ: usize = 0x5413;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Winsize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

/// inotify_init1 flags.
pub const IN_CLOEXEC: i32 = 0o2000000;
pub const IN_NONBLOCK: i32 = 0o4000;

/// inotify event masks.
pub const IN_ACCESS: u32 = 0x0000_0001;
pub const IN_MODIFY: u32 = 0x0000_0002;
pub const IN_ATTRIB: u32 = 0x0000_0004;
pub const IN_CLOSE_WRITE: u32 = 0x0000_0008;
pub const IN_CLOSE_NOWRITE: u32 = 0x0000_0010;
pub const IN_OPEN: u32 = 0x0000_0020;
pub const IN_MOVED_FROM: u32 = 0x0000_0040;
pub const IN_MOVED_TO: u32 = 0x0000_0080;
pub const IN_CREATE: u32 = 0x0000_0100;
pub const IN_DELETE: u32 = 0x0000_0200;
pub const IN_DELETE_SELF: u32 = 0x0000_0400;
pub const IN_MOVE_SELF: u32 = 0x0000_0800;

/// Fixed header of `struct inotify_event` (name[] follows).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct InotifyEvent {
    pub wd: i32,
    pub mask: u32,
    pub cookie: u32,
    pub len: u32,
}

/// pidfd_open flags.
pub const PIDFD_NONBLOCK: u32 = 0o4000;
