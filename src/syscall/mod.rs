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
    /// Linux `O_TMPFILE` (`__O_TMPFILE | O_DIRECTORY`).
    pub const O_TMPFILE: i32 = 0o20000000 | O_DIRECTORY;
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
    pub const MAP_POPULATE: i32 = 0x8000;
}

/// `waitid` / `wait4` options.
pub mod wait {
    pub const WNOHANG: i32 = 1;
    pub const WUNTRACED: i32 = 2;
    pub const WEXITED: i32 = 4;
    pub const WCONTINUED: i32 = 8;
    pub const WNOWAIT: i32 = 0x0100_0000;
}

/// `clock_gettime` clock ids.
pub mod clock {
    pub const CLOCK_REALTIME: i32 = 0;
    pub const CLOCK_MONOTONIC: i32 = 1;
    pub const CLOCK_PROCESS_CPUTIME_ID: i32 = 2;
    pub const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
    pub const CLOCK_MONOTONIC_RAW: i32 = 4;
    pub const CLOCK_REALTIME_COARSE: i32 = 5;
    pub const CLOCK_MONOTONIC_COARSE: i32 = 6;
    pub const CLOCK_BOOTTIME: i32 = 7;
}

/// `fcntl` commands.
pub mod fcntl_cmd {
    pub const F_DUPFD: i32 = 0;
    pub const F_GETFD: i32 = 1;
    pub const F_SETFD: i32 = 2;
    pub const F_GETFL: i32 = 3;
    pub const F_SETFL: i32 = 4;
    pub const F_GETLK: i32 = 5;
    pub const F_SETLK: i32 = 6;
    pub const F_SETLKW: i32 = 7;
    pub const F_DUPFD_CLOEXEC: i32 = 1030;
}

/// `fcntl` advisory lock types (`struct flock.l_type`).
pub const F_RDLCK: i16 = 0;
pub const F_WRLCK: i16 = 1;
pub const F_UNLCK: i16 = 2;

/// Kernel `struct flock` (64-bit).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Flock {
    pub l_type: i16,
    pub l_whence: i16,
    pub l_start: i64,
    pub l_len: i64,
    pub l_pid: i32,
}

pub const FD_CLOEXEC: i32 = 1;

/// `unlinkat` flag: remove a directory.
pub const AT_REMOVEDIR: i32 = 0x200;

/// `fchmodat` / `faccessat` flag: do not follow symlinks.
pub const AT_SYMLINK_NOFOLLOW: i32 = 0x100;
/// `linkat` flag: follow symlinks on `oldpath`.
pub const AT_SYMLINK_FOLLOW: i32 = 0x400;
/// `*at` flag: empty path refers to `dirfd` itself.
pub const AT_EMPTY_PATH: i32 = 0x1000;

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

    pub fn is_chr(&self) -> bool {
        (self.st_mode & 0o170000) == S_IFCHR
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
pub const SIGPIPE: i32 = 13;
pub const SIGALRM: i32 = 14;
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
pub const EPOLLET: u32 = 0x8000_0000;
pub const EPOLLONESHOT: u32 = 1 << 30;
pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;
/// `epoll_create1` flag (same bit as `O_CLOEXEC`).
pub const EPOLL_CLOEXEC: i32 = 0o2000000;

/// utimensat special nsec values.
pub const UTIME_NOW: i64 = (1 << 30) - 1;
pub const UTIME_OMIT: i64 = (1 << 30) - 2;

/// prlimit resource ids.
pub const RLIMIT_CPU: i32 = 0;
pub const RLIMIT_STACK: i32 = 3;
pub const RLIMIT_NOFILE: i32 = 7;
pub const RLIMIT_AS: i32 = 9;

pub mod madvise {
    pub const MADV_NORMAL: i32 = 0;
    pub const MADV_RANDOM: i32 = 1;
    pub const MADV_SEQUENTIAL: i32 = 2;
    pub const MADV_WILLNEED: i32 = 3;
    pub const MADV_DONTNEED: i32 = 4;
    pub const MADV_FREE: i32 = 8;
    pub const MADV_HUGEPAGE: i32 = 14;
    pub const MADV_NOHUGEPAGE: i32 = 15;
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
    /// Linux `struct epoll_event`.
    ///
    /// On x86_64 the UAPI type is packed (12 bytes). On aarch64 it is naturally
    /// aligned (16 bytes with padding before `data`).
    #[cfg_attr(target_arch = "x86_64", repr(C, packed))]
    #[cfg_attr(target_arch = "aarch64", repr(C))]
    #[derive(Clone, Copy)]
    pub struct EpollEvent {
        pub events: u32,
        pub data: u64,
    }

    impl EpollEvent {
        pub const fn new(events: u32, data: u64) -> Self {
            Self { events, data }
        }

        pub fn events(self) -> u32 {
            self.events
        }

        pub fn data(self) -> u64 {
            self.data
        }
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
pub const P_PIDFD: i32 = 3;

/// rt_sigprocmask `how`.
pub const SIG_BLOCK: i32 = 0;
pub const SIG_UNBLOCK: i32 = 1;
pub const SIG_SETMASK: i32 = 2;

/// Socket option levels and names.
pub const SOL_SOCKET: i32 = 1;
pub const SO_DEBUG: i32 = 1;
pub const SO_REUSEADDR: i32 = 2;
pub const SO_TYPE: i32 = 3;
pub const SO_ERROR: i32 = 4;
pub const SO_DONTROUTE: i32 = 5;
pub const SO_BROADCAST: i32 = 6;
pub const SO_SNDBUF: i32 = 7;
pub const SO_RCVBUF: i32 = 8;
pub const SO_KEEPALIVE: i32 = 9;
pub const SO_OOBINLINE: i32 = 10;
pub const SO_LINGER: i32 = 13;
pub const SO_REUSEPORT: i32 = 15;
pub const SO_PASSCRED: i32 = 16;
pub const SO_ACCEPTCONN: i32 = 30;
pub const SO_PROTOCOL: i32 = 38;
pub const SO_DOMAIN: i32 = 39;
/// `send` / `recv` flags.
pub const MSG_DONTWAIT: i32 = 0x40;
pub const MSG_PEEK: i32 = 0x02;
/// `getrandom` flags.
pub const GRND_NONBLOCK: u32 = 0x0001;
pub const GRND_RANDOM: u32 = 0x0002;
/// `fallocate` modes.
pub const FALLOC_FL_KEEP_SIZE: i32 = 0x01;
pub const FALLOC_FL_PUNCH_HOLE: i32 = 0x02;
pub const FALLOC_FL_ZERO_RANGE: i32 = 0x10;

/// prctl options.
pub const PR_GET_DUMPABLE: i32 = 3;
pub const PR_SET_DUMPABLE: i32 = 4;
pub const PR_SET_NAME: i32 = 15;
pub const PR_GET_NAME: i32 = 16;
pub const PR_SET_NO_NEW_PRIVS: i32 = 38;
pub const PR_GET_NO_NEW_PRIVS: i32 = 39;

/// fcntl sealing (memfd).
pub const F_ADD_SEALS: i32 = 1033;
pub const F_GET_SEALS: i32 = 1034;
pub const F_SEAL_SEAL: i32 = 0x0001;
pub const F_SEAL_SHRINK: i32 = 0x0002;
pub const F_SEAL_GROW: i32 = 0x0004;
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
/// ioctl: get pty number for `/dev/ptmx` (`_IOR('T', 0x30, unsigned int)`).
pub const TIOCGPTN: usize = 0x8004_5430;
/// ioctl: lock/unlock pty (`_IOW('T', 0x31, int)`); unlock with `0`.
pub const TIOCSPTLCK: usize = 0x4004_5431;
/// ioctl: make fd the controlling terminal (`TIOCSCTTY`).
pub const TIOCSCTTY: usize = 0x540E;

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

/// Ancillary data: pass open file descriptors.
pub const SCM_RIGHTS: i32 = 1;

/// `setitimer` / `getitimer` which.
pub const ITIMER_REAL: i32 = 0;

/// `sigaltstack` flags.
pub const SS_ONSTACK: i32 = 1;
pub const SS_DISABLE: i32 = 2;

/// `fd_set` capacity used with `pselect6`.
pub const FD_SETSIZE: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Itimerval {
    pub it_interval: Timeval,
    pub it_value: Timeval,
}

/// Kernel `struct msghdr` (64-bit).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MsgHdr {
    pub msg_name: *mut u8,
    pub msg_namelen: i32,
    pub msg_iov: *mut IoVec,
    pub msg_iovlen: usize,
    pub msg_control: *mut u8,
    pub msg_controllen: usize,
    pub msg_flags: u32,
}

impl Default for MsgHdr {
    fn default() -> Self {
        // Safety: all-zero is a valid bit pattern for this POD struct.
        unsafe { core::mem::zeroed() }
    }
}

/// Kernel `struct cmsghdr`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CmsgHdr {
    pub cmsg_len: usize,
    pub cmsg_level: i32,
    pub cmsg_type: i32,
}

/// Kernel `stack_t` for `sigaltstack`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Stack {
    pub ss_sp: *mut u8,
    pub ss_flags: i32,
    pub ss_size: usize,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            ss_sp: core::ptr::null_mut(),
            ss_flags: 0,
            ss_size: 0,
        }
    }
}

/// Linux `fd_set` bitmap for `pselect6` (FD_SETSIZE bits).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FdSet {
    pub bits: [u64; FD_SETSIZE / 64],
}

impl Default for FdSet {
    fn default() -> Self {
        Self {
            bits: [0u64; FD_SETSIZE / 64],
        }
    }
}

impl FdSet {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn set(&mut self, fd: i32) {
        if fd < 0 {
            return;
        }
        let fd = fd as usize;
        if fd >= FD_SETSIZE {
            return;
        }
        self.bits[fd / 64] |= 1u64 << (fd % 64);
    }

    pub fn clear(&mut self, fd: i32) {
        if fd < 0 {
            return;
        }
        let fd = fd as usize;
        if fd >= FD_SETSIZE {
            return;
        }
        self.bits[fd / 64] &= !(1u64 << (fd % 64));
    }

    pub fn is_set(&self, fd: i32) -> bool {
        if fd < 0 {
            return false;
        }
        let fd = fd as usize;
        if fd >= FD_SETSIZE {
            return false;
        }
        (self.bits[fd / 64] & (1u64 << (fd % 64))) != 0
    }
}

/// CMSG_ALIGN — align control message lengths to `sizeof(usize)`.
pub fn cmsg_align(len: usize) -> usize {
    let a = core::mem::size_of::<usize>();
    (len + a - 1) & !(a - 1)
}

/// CMSG_LEN — data length plus aligned header.
pub fn cmsg_len(data_len: usize) -> usize {
    cmsg_align(core::mem::size_of::<CmsgHdr>()) + data_len
}

/// CMSG_SPACE — space needed in the control buffer for `data_len` bytes.
pub fn cmsg_space(data_len: usize) -> usize {
    cmsg_align(core::mem::size_of::<CmsgHdr>()) + cmsg_align(data_len)
}

/// `statx` mask bits.
pub const STATX_TYPE: u32 = 0x0000_0001;
pub const STATX_MODE: u32 = 0x0000_0002;
pub const STATX_NLINK: u32 = 0x0000_0004;
pub const STATX_UID: u32 = 0x0000_0008;
pub const STATX_GID: u32 = 0x0000_0010;
pub const STATX_ATIME: u32 = 0x0000_0020;
pub const STATX_MTIME: u32 = 0x0000_0040;
pub const STATX_CTIME: u32 = 0x0000_0080;
pub const STATX_INO: u32 = 0x0000_0100;
pub const STATX_SIZE: u32 = 0x0000_0200;
pub const STATX_BLOCKS: u32 = 0x0000_0400;
pub const STATX_BASIC_STATS: u32 = 0x0000_07ff;

/// Kernel `struct statx_timestamp`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct StatxTimestamp {
    pub tv_sec: i64,
    pub tv_nsec: u32,
    pub __reserved: i32,
}

/// Kernel `struct statx` (subset used by tests; size matches ABI).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Statx {
    pub stx_mask: u32,
    pub stx_blksize: u32,
    pub stx_attributes: u64,
    pub stx_nlink: u32,
    pub stx_uid: u32,
    pub stx_gid: u32,
    pub stx_mode: u16,
    pub __spare0: u16,
    pub stx_ino: u64,
    pub stx_size: u64,
    pub stx_blocks: u64,
    pub stx_attributes_mask: u64,
    pub stx_atime: StatxTimestamp,
    pub stx_btime: StatxTimestamp,
    pub stx_ctime: StatxTimestamp,
    pub stx_mtime: StatxTimestamp,
    pub stx_rdev_major: u32,
    pub stx_rdev_minor: u32,
    pub stx_dev_major: u32,
    pub stx_dev_minor: u32,
    pub stx_mnt_id: u64,
    pub stx_dio_mem_align: u32,
    pub stx_dio_offset_align: u32,
    pub __spare3: [u64; 12],
}

impl Default for Statx {
    fn default() -> Self {
        // Safety: all-zero is a valid bit pattern for this POD struct.
        unsafe { core::mem::zeroed() }
    }
}

impl Statx {
    pub fn is_reg(&self) -> bool {
        (self.stx_mode as u32 & 0o170000) == 0o100000
    }

    pub fn is_lnk(&self) -> bool {
        (self.stx_mode as u32 & 0o170000) == 0o120000
    }

    pub fn mode_bits(&self) -> u32 {
        self.stx_mode as u32 & 0o7777
    }
}

/// `openat2` resolve flags.
pub const RESOLVE_NO_XDEV: u64 = 0x01;
pub const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
pub const RESOLVE_NO_SYMLINKS: u64 = 0x04;
pub const RESOLVE_BENEATH: u64 = 0x08;
pub const RESOLVE_IN_ROOT: u64 = 0x10;

/// Kernel `struct open_how`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenHow {
    pub flags: u64,
    pub mode: u64,
    pub resolve: u64,
}

/// `sync_file_range` flags.
pub const SYNC_FILE_RANGE_WAIT_BEFORE: u32 = 1;
pub const SYNC_FILE_RANGE_WRITE: u32 = 2;
pub const SYNC_FILE_RANGE_WAIT_AFTER: u32 = 4;

/// `posix_fadvise` / `fadvise64` advice values.
pub const POSIX_FADV_NORMAL: i32 = 0;
pub const POSIX_FADV_RANDOM: i32 = 1;
pub const POSIX_FADV_SEQUENTIAL: i32 = 2;
pub const POSIX_FADV_WILLNEED: i32 = 3;
pub const POSIX_FADV_DONTNEED: i32 = 4;
pub const POSIX_FADV_NOREUSE: i32 = 5;

/// `membarrier` commands.
pub const MEMBARRIER_CMD_QUERY: i32 = 0;

/// Linux capability ABI version 3.
pub const LINUX_CAPABILITY_VERSION_3: u32 = 0x2008_0522;
pub const LINUX_CAPABILITY_U32S_3: usize = 2;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CapUserHeader {
    pub version: u32,
    pub pid: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CapUserData {
    pub effective: u32,
    pub permitted: u32,
    pub inheritable: u32,
}

/// `clone` / `unshare` flags (subset used by freestanding thread helper).
pub const CLONE_VM: u64 = 0x0000_0100;
pub const CLONE_FS: u64 = 0x0000_0200;
/// `unshare` / `clone` flag: share file descriptor table until unshared.
pub const CLONE_FILES: u64 = 0x0000_0400;
pub const CLONE_SIGHAND: u64 = 0x0000_0800;
pub const CLONE_THREAD: u64 = 0x0001_0000;
pub const CLONE_SYSVSEM: u64 = 0x0004_0000;
pub const CLONE_PARENT_SETTID: u64 = 0x0010_0000;
pub const CLONE_CHILD_CLEARTID: u64 = 0x0020_0000;
pub const CLONE_CHILD_SETTID: u64 = 0x0100_0000;
/// `unshare` flag: new user namespace (often EPERM when unprivileged).
pub const CLONE_NEWUSER: u64 = 0x1000_0000;

/// Flags for a freestanding POSIX-like thread (`CLONE_THREAD` group).
pub const CLONE_THREAD_FLAGS: u64 = CLONE_VM
    | CLONE_FS
    | CLONE_FILES
    | CLONE_SIGHAND
    | CLONE_THREAD
    | CLONE_SYSVSEM
    | CLONE_PARENT_SETTID
    | CLONE_CHILD_CLEARTID;

/// `kcmp` comparison types.
pub const KCMP_FILE: i32 = 0;

/// System V IPC key / shm / sem / msg flags.
pub const IPC_PRIVATE: i32 = 0;
pub const IPC_CREAT: i32 = 0o1000;
pub const IPC_EXCL: i32 = 0o2000;
pub const IPC_NOWAIT: i32 = 0o4000;
pub const IPC_RMID: i32 = 0;
pub const IPC_SET: i32 = 1;
pub const IPC_STAT: i32 = 2;
pub const IPC_INFO: i32 = 3;
pub const SHM_RDONLY: i32 = 0o10000;
pub const SHM_RND: i32 = 0o20000;
/// `semop` flags.
pub const SEM_UNDO: i16 = 0o10000;
/// `semctl` commands.
pub const GETVAL: i32 = 12;
pub const GETALL: i32 = 13;
pub const GETNCNT: i32 = 14;
pub const GETZCNT: i32 = 15;
pub const SETVAL: i32 = 16;
pub const SETALL: i32 = 17;
/// `mlockall` flags.
pub const MCL_CURRENT: i32 = 1;
pub const MCL_FUTURE: i32 = 2;
/// `clock_nanosleep` / POSIX timer absolute flag.
pub const TIMER_ABSTIME: i32 = 1;
/// eventfd2 flags.
pub const EFD_SEMAPHORE: i32 = 1;
pub const EFD_CLOEXEC: i32 = 0o2000000;
pub const EFD_NONBLOCK: i32 = 0o4000;
/// POSIX timer notification.
pub const SIGEV_SIGNAL: i32 = 0;
pub const SIGEV_NONE: i32 = 1;
/// `fsconfig` commands (probe only).
pub const FSCONFIG_SET_FLAG: u32 = 0;
pub const FSCONFIG_CMD_CREATE: u32 = 6;

/// Kernel `struct mq_attr`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MqAttr {
    pub mq_flags: i64,
    pub mq_maxmsg: i64,
    pub mq_msgsize: i64,
    pub mq_curmsgs: i64,
}

/// Landlock: query ABI version via `landlock_create_ruleset`.
pub const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1 << 0;
/// Minimal FS access right for ruleset attr probes.
pub const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct LandlockRulesetAttr {
    pub handled_access_fs: u64,
}

/// userfaultfd flags.
pub const UFFD_CLOEXEC: i32 = 0o2000000;
pub const UFFD_NONBLOCK: i32 = 0o4000;

/// Kernel `struct io_uring_params` (offsets opaque; size must match ABI).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IoUringParams {
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub flags: u32,
    pub sq_thread_cpu: u32,
    pub sq_thread_idle: u32,
    pub features: u32,
    pub wq_fd: u32,
    pub resv: [u32; 3],
    pub sq_off: [u8; 40],
    pub cq_off: [u8; 40],
}

impl Default for IoUringParams {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

/// Kernel `struct sigevent` (subset used with SIGEV_NONE).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sigevent {
    pub sigev_value: usize,
    pub sigev_signo: i32,
    pub sigev_notify: i32,
    pub _pad: [u8; 48],
}

impl Default for Sigevent {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

/// Kernel `struct sembuf`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Sembuf {
    pub sem_num: u16,
    pub sem_op: i16,
    pub sem_flg: i16,
}

/// Message buffer header + small payload for SysV msgsnd/msgrcv.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MsgBuf {
    pub mtype: i64,
    pub mtext: [u8; 32],
}

impl Default for MsgBuf {
    fn default() -> Self {
        Self {
            mtype: 0,
            mtext: [0; 32],
        }
    }
}
