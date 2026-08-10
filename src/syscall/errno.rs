//! Linux errno values returned as negated syscall results.

use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Errno(pub i32);

pub type Result<T> = core::result::Result<T, Errno>;

impl Errno {
    pub const EPERM: Self = Self(1);
    pub const ENOENT: Self = Self(2);
    pub const ESRCH: Self = Self(3);
    pub const EINTR: Self = Self(4);
    pub const EIO: Self = Self(5);
    pub const ENXIO: Self = Self(6);
    pub const E2BIG: Self = Self(7);
    pub const ENOEXEC: Self = Self(8);
    pub const EBADF: Self = Self(9);
    pub const ECHILD: Self = Self(10);
    pub const EAGAIN: Self = Self(11);
    pub const ENOMEM: Self = Self(12);
    pub const EACCES: Self = Self(13);
    pub const EFAULT: Self = Self(14);
    pub const EEXIST: Self = Self(17);
    pub const EXDEV: Self = Self(18);
    pub const ENODEV: Self = Self(19);
    pub const ENOTDIR: Self = Self(20);
    pub const EISDIR: Self = Self(21);
    pub const EINVAL: Self = Self(22);
    pub const ENFILE: Self = Self(23);
    pub const EMFILE: Self = Self(24);
    pub const ENOTTY: Self = Self(25);
    pub const ETXTBSY: Self = Self(26);
    pub const EFBIG: Self = Self(27);
    pub const ENOSPC: Self = Self(28);
    pub const ESPIPE: Self = Self(29);
    pub const EROFS: Self = Self(30);
    pub const EMLINK: Self = Self(31);
    pub const EPIPE: Self = Self(32);
    pub const ERANGE: Self = Self(34);
    pub const ENAMETOOLONG: Self = Self(36);
    pub const ENOSYS: Self = Self(38);
    pub const ENOTEMPTY: Self = Self(39);
    pub const ELOOP: Self = Self(40);
    pub const ENOMSG: Self = Self(42);
    pub const ENOTSOCK: Self = Self(88);
    pub const EOPNOTSUPP: Self = Self(95);
    pub const EAFNOSUPPORT: Self = Self(97);
    pub const EADDRINUSE: Self = Self(98);
    pub const ECONNREFUSED: Self = Self(111);
    pub const ETIMEDOUT: Self = Self(110);
    pub const EWOULDBLOCK: Self = Self(11);
    pub const ENOTSUP: Self = Self(95);
    pub const EBUSY: Self = Self(16);

    pub fn as_isize(self) -> isize {
        -(self.0 as isize)
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            1 => "EPERM",
            2 => "ENOENT",
            3 => "ESRCH",
            4 => "EINTR",
            5 => "EIO",
            9 => "EBADF",
            10 => "ECHILD",
            11 => "EAGAIN",
            12 => "ENOMEM",
            13 => "EACCES",
            14 => "EFAULT",
            17 => "EEXIST",
            20 => "ENOTDIR",
            21 => "EISDIR",
            22 => "EINVAL",
            28 => "ENOSPC",
            30 => "EROFS",
            38 => "ENOSYS",
            39 => "ENOTEMPTY",
            40 => "ELOOP",
            _ => "EUNKNOWN",
        }
    }
}

impl fmt::Debug for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.0)
    }
}

/// Convert a raw syscall return into `Result`.
#[inline(always)]
pub fn from_ret(ret: isize) -> Result<usize> {
    if ret < 0 {
        // Syscall errors are in -4095..-1.
        Err(Errno((-ret) as i32))
    } else {
        Ok(ret as usize)
    }
}
