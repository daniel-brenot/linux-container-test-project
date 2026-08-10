//! Thin, portable wrappers around Linux syscalls.
//!
//! Prefer `*at` variants so the same code works on x86_64 and aarch64.

use super::arch::{nr, syscall};
use super::errno::{from_ret, Errno, Result};
use super::{
    Stat, Timespec, UtsName, AT_FDCWD, AT_REMOVEDIR, STDERR_FILENO, STDOUT_FILENO,
};

#[inline]
unsafe fn sys0(nr: usize) -> Result<usize> {
    from_ret(syscall(nr, 0, 0, 0, 0, 0, 0))
}

#[inline]
unsafe fn sys1(nr: usize, a0: usize) -> Result<usize> {
    from_ret(syscall(nr, a0, 0, 0, 0, 0, 0))
}

#[inline]
unsafe fn sys2(nr: usize, a0: usize, a1: usize) -> Result<usize> {
    from_ret(syscall(nr, a0, a1, 0, 0, 0, 0))
}

#[inline]
unsafe fn sys3(nr: usize, a0: usize, a1: usize, a2: usize) -> Result<usize> {
    from_ret(syscall(nr, a0, a1, a2, 0, 0, 0))
}

#[inline]
unsafe fn sys4(nr: usize, a0: usize, a1: usize, a2: usize, a3: usize) -> Result<usize> {
    from_ret(syscall(nr, a0, a1, a2, a3, 0, 0))
}

#[inline]
unsafe fn sys5(
    nr: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
) -> Result<usize> {
    from_ret(syscall(nr, a0, a1, a2, a3, a4, 0))
}

#[inline]
unsafe fn sys6(
    nr: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> Result<usize> {
    from_ret(syscall(nr, a0, a1, a2, a3, a4, a5))
}

fn c_str_ptr(path: &[u8]) -> Result<*const u8> {
    if path.is_empty() || path.last() != Some(&0) {
        // Callers must pass NUL-terminated paths.
        return Err(Errno::EINVAL);
    }
    Ok(path.as_ptr())
}

pub fn exit(code: i32) -> ! {
    unsafe {
        let _ = syscall(nr::EXIT_GROUP, code as usize, 0, 0, 0, 0, 0);
        let _ = syscall(nr::EXIT, code as usize, 0, 0, 0, 0, 0);
        core::hint::unreachable_unchecked()
    }
}

pub fn write(fd: i32, buf: &[u8]) -> Result<usize> {
    unsafe { sys3(nr::WRITE, fd as usize, buf.as_ptr() as usize, buf.len()) }
}

pub fn write_all(fd: i32, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        let n = write(fd, buf)?;
        if n == 0 {
            return Err(Errno::EIO);
        }
        buf = &buf[n..];
    }
    Ok(())
}

pub fn read(fd: i32, buf: &mut [u8]) -> Result<usize> {
    unsafe { sys3(nr::READ, fd as usize, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn close(fd: i32) -> Result<()> {
    unsafe { sys1(nr::CLOSE, fd as usize).map(|_| ()) }
}

pub fn openat(dirfd: i32, path: &[u8], flags: i32, mode: u32) -> Result<i32> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys4(
            nr::OPENAT,
            dirfd as usize,
            p as usize,
            flags as usize,
            mode as usize,
        )
        .map(|fd| fd as i32)
    }
}

pub fn open(path: &[u8], flags: i32, mode: u32) -> Result<i32> {
    openat(AT_FDCWD, path, flags, mode)
}

pub fn lseek(fd: i32, offset: i64, whence: i32) -> Result<i64> {
    unsafe {
        sys3(nr::LSEEK, fd as usize, offset as usize, whence as usize).map(|v| v as i64)
    }
}

pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

pub fn fstatat(dirfd: i32, path: &[u8], flags: i32) -> Result<Stat> {
    let p = c_str_ptr(path)?;
    let mut st = Stat::default();
    unsafe {
        sys4(
            nr::NEWFSTATAT,
            dirfd as usize,
            p as usize,
            &mut st as *mut Stat as usize,
            flags as usize,
        )?;
    }
    Ok(st)
}

pub fn stat(path: &[u8]) -> Result<Stat> {
    fstatat(AT_FDCWD, path, 0)
}

pub fn lstat(path: &[u8]) -> Result<Stat> {
    fstatat(AT_FDCWD, path, super::AT_SYMLINK_NOFOLLOW)
}

pub fn ftruncate(fd: i32, length: i64) -> Result<()> {
    unsafe { sys2(nr::FTRUNCATE, fd as usize, length as usize).map(|_| ()) }
}

pub fn fsync(fd: i32) -> Result<()> {
    unsafe { sys1(nr::FSYNC, fd as usize).map(|_| ()) }
}

pub fn fcntl(fd: i32, cmd: i32, arg: usize) -> Result<usize> {
    unsafe { sys3(nr::FCNTL, fd as usize, cmd as usize, arg) }
}

pub fn dup(fd: i32) -> Result<i32> {
    unsafe { sys1(nr::DUP, fd as usize).map(|v| v as i32) }
}

pub fn dup2(oldfd: i32, newfd: i32) -> Result<i32> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        sys2(nr::DUP2, oldfd as usize, newfd as usize).map(|v| v as i32)
    }
    #[cfg(target_arch = "aarch64")]
    {
        // aarch64 has no dup2; dup3 with flags=0 is equivalent.
        dup3(oldfd, newfd, 0)
    }
}

pub fn dup3(oldfd: i32, newfd: i32, flags: i32) -> Result<i32> {
    unsafe {
        sys3(
            nr::DUP3,
            oldfd as usize,
            newfd as usize,
            flags as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn pipe2(flags: i32) -> Result<(i32, i32)> {
    let mut fds = [0i32; 2];
    unsafe {
        sys2(nr::PIPE2, fds.as_mut_ptr() as usize, flags as usize)?;
    }
    Ok((fds[0], fds[1]))
}

pub fn mkdirat(dirfd: i32, path: &[u8], mode: u32) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys3(nr::MKDIRAT, dirfd as usize, p as usize, mode as usize).map(|_| ())
    }
}

pub fn mkdir(path: &[u8], mode: u32) -> Result<()> {
    mkdirat(AT_FDCWD, path, mode)
}

pub fn unlinkat(dirfd: i32, path: &[u8], flags: i32) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys3(nr::UNLINKAT, dirfd as usize, p as usize, flags as usize).map(|_| ())
    }
}

pub fn unlink(path: &[u8]) -> Result<()> {
    unlinkat(AT_FDCWD, path, 0)
}

pub fn rmdir(path: &[u8]) -> Result<()> {
    unlinkat(AT_FDCWD, path, AT_REMOVEDIR)
}

pub fn renameat(olddirfd: i32, oldpath: &[u8], newdirfd: i32, newpath: &[u8]) -> Result<()> {
    let old = c_str_ptr(oldpath)?;
    let new = c_str_ptr(newpath)?;
    unsafe {
        sys4(
            nr::RENAMEAT,
            olddirfd as usize,
            old as usize,
            newdirfd as usize,
            new as usize,
        )
        .map(|_| ())
    }
}

pub fn rename(old: &[u8], new: &[u8]) -> Result<()> {
    renameat(AT_FDCWD, old, AT_FDCWD, new)
}

#[cfg(target_arch = "x86_64")]
pub fn link(old: &[u8], new: &[u8]) -> Result<()> {
    let o = c_str_ptr(old)?;
    let n = c_str_ptr(new)?;
    unsafe { sys2(nr::LINK, o as usize, n as usize).map(|_| ()) }
}

#[cfg(target_arch = "aarch64")]
pub fn link(old: &[u8], new: &[u8]) -> Result<()> {
    linkat(AT_FDCWD, old, AT_FDCWD, new, 0)
}

pub fn linkat(
    olddirfd: i32,
    oldpath: &[u8],
    newdirfd: i32,
    newpath: &[u8],
    flags: i32,
) -> Result<()> {
    let o = c_str_ptr(oldpath)?;
    let n = c_str_ptr(newpath)?;
    unsafe {
        sys5(
            nr::LINKAT,
            olddirfd as usize,
            o as usize,
            newdirfd as usize,
            n as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn symlinkat(target: &[u8], newdirfd: i32, linkpath: &[u8]) -> Result<()> {
    let t = c_str_ptr(target)?;
    let l = c_str_ptr(linkpath)?;
    unsafe {
        sys3(nr::SYMLINKAT, t as usize, newdirfd as usize, l as usize).map(|_| ())
    }
}

pub fn symlink(target: &[u8], linkpath: &[u8]) -> Result<()> {
    symlinkat(target, AT_FDCWD, linkpath)
}

pub fn readlinkat(dirfd: i32, path: &[u8], buf: &mut [u8]) -> Result<usize> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys4(
            nr::READLINKAT,
            dirfd as usize,
            p as usize,
            buf.as_mut_ptr() as usize,
            buf.len(),
        )
    }
}

pub fn readlink(path: &[u8], buf: &mut [u8]) -> Result<usize> {
    readlinkat(AT_FDCWD, path, buf)
}

pub fn fchmod(fd: i32, mode: u32) -> Result<()> {
    unsafe { sys2(nr::FCHMOD, fd as usize, mode as usize).map(|_| ()) }
}

pub fn fchmodat(dirfd: i32, path: &[u8], mode: u32, flags: i32) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys4(
            nr::FCHMODAT,
            dirfd as usize,
            p as usize,
            mode as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn chmod(path: &[u8], mode: u32) -> Result<()> {
    fchmodat(AT_FDCWD, path, mode, 0)
}

pub fn faccessat(dirfd: i32, path: &[u8], mode: i32, flags: i32) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys4(
            nr::FACCESSAT,
            dirfd as usize,
            p as usize,
            mode as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn access(path: &[u8], mode: i32) -> Result<()> {
    faccessat(AT_FDCWD, path, mode, 0)
}

pub const F_OK: i32 = 0;
pub const X_OK: i32 = 1;
pub const W_OK: i32 = 2;
pub const R_OK: i32 = 4;

pub fn chdir(path: &[u8]) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe { sys1(nr::CHDIR, p as usize).map(|_| ()) }
}

pub fn fchdir(fd: i32) -> Result<()> {
    unsafe { sys1(nr::FCHDIR, fd as usize).map(|_| ()) }
}

pub fn getcwd(buf: &mut [u8]) -> Result<usize> {
    unsafe { sys2(nr::GETCWD, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn getpid() -> i32 {
    unsafe { sys0(nr::GETPID).unwrap_or(0) as i32 }
}

pub fn getppid() -> i32 {
    unsafe { sys0(nr::GETPPID).unwrap_or(0) as i32 }
}

pub fn gettid() -> i32 {
    unsafe { sys0(nr::GETTID).unwrap_or(0) as i32 }
}

pub fn getuid() -> u32 {
    unsafe { sys0(nr::GETUID).unwrap_or(0) as u32 }
}

pub fn geteuid() -> u32 {
    unsafe { sys0(nr::GETEUID).unwrap_or(0) as u32 }
}

pub fn getgid() -> u32 {
    unsafe { sys0(nr::GETGID).unwrap_or(0) as u32 }
}

pub fn getegid() -> u32 {
    unsafe { sys0(nr::GETEGID).unwrap_or(0) as u32 }
}

pub fn uname() -> Result<UtsName> {
    let mut u = UtsName::default();
    unsafe {
        sys1(nr::UNAME, &mut u as *mut UtsName as usize)?;
    }
    Ok(u)
}

pub fn clock_gettime(clock_id: i32) -> Result<Timespec> {
    let mut ts = Timespec::default();
    unsafe {
        sys2(
            nr::CLOCK_GETTIME,
            clock_id as usize,
            &mut ts as *mut Timespec as usize,
        )?;
    }
    Ok(ts)
}

pub fn nanosleep(req: &Timespec) -> Result<()> {
    unsafe {
        sys2(
            nr::NANOSLEEP,
            req as *const Timespec as usize,
            0,
        )
        .map(|_| ())
    }
}

pub fn mmap(
    addr: usize,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> Result<usize> {
    unsafe {
        sys6(
            nr::MMAP,
            addr,
            len,
            prot as usize,
            flags as usize,
            fd as usize,
            offset as usize,
        )
    }
}

pub fn munmap(addr: usize, len: usize) -> Result<()> {
    unsafe { sys2(nr::MUNMAP, addr, len).map(|_| ()) }
}

pub fn mprotect(addr: usize, len: usize, prot: i32) -> Result<()> {
    unsafe { sys3(nr::MPROTECT, addr, len, prot as usize).map(|_| ()) }
}

pub fn getrandom(buf: &mut [u8], flags: u32) -> Result<usize> {
    unsafe {
        sys3(
            nr::GETRANDOM,
            buf.as_mut_ptr() as usize,
            buf.len(),
            flags as usize,
        )
    }
}

pub fn fork() -> Result<i32> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        sys0(nr::FORK).map(|v| v as i32)
    }
    #[cfg(target_arch = "aarch64")]
    {
        // aarch64 has no fork(2); emulate with clone(SIGCHLD).
        const SIGCHLD: usize = 17;
        unsafe { sys5(nr::CLONE, SIGCHLD, 0, 0, 0, 0).map(|v| v as i32) }
    }
}

pub fn wait4(pid: i32, status: &mut i32, options: i32) -> Result<i32> {
    unsafe {
        sys4(
            nr::WAIT4,
            pid as usize,
            status as *mut i32 as usize,
            options as usize,
            0,
        )
        .map(|v| v as i32)
    }
}

pub fn kill(pid: i32, sig: i32) -> Result<()> {
    unsafe { sys2(nr::KILL, pid as usize, sig as usize).map(|_| ()) }
}

pub fn pread(fd: i32, buf: &mut [u8], offset: i64) -> Result<usize> {
    unsafe {
        sys4(
            nr::PREAD64,
            fd as usize,
            buf.as_mut_ptr() as usize,
            buf.len(),
            offset as usize,
        )
    }
}

pub fn pwrite(fd: i32, buf: &[u8], offset: i64) -> Result<usize> {
    unsafe {
        sys4(
            nr::PWRITE64,
            fd as usize,
            buf.as_ptr() as usize,
            buf.len(),
            offset as usize,
        )
    }
}

pub fn getdents64(fd: i32, buf: &mut [u8]) -> Result<usize> {
    unsafe {
        sys3(
            nr::GETDENTS64,
            fd as usize,
            buf.as_mut_ptr() as usize,
            buf.len(),
        )
    }
}

/// Decode a wait status produced by `wait4`.
pub fn wifexited(status: i32) -> bool {
    (status & 0x7f) == 0
}

pub fn wexitstatus(status: i32) -> i32 {
    (status >> 8) & 0xff
}

pub fn wifsignaled(status: i32) -> bool {
    (((status & 0x7f) + 1) as i8) >= 2
}

pub fn wtermsig(status: i32) -> i32 {
    status & 0x7f
}

pub fn print(s: &str) {
    let _ = write_all(STDOUT_FILENO, s.as_bytes());
}

pub fn eprint(s: &str) {
    let _ = write_all(STDERR_FILENO, s.as_bytes());
}

pub fn fstat(fd: i32) -> Result<Stat> {
    let mut st = Stat::default();
    unsafe {
        sys2(nr::FSTAT, fd as usize, &mut st as *mut Stat as usize)?;
    }
    Ok(st)
}

pub fn fdatasync(fd: i32) -> Result<()> {
    unsafe { sys1(nr::FDATASYNC, fd as usize).map(|_| ()) }
}

pub fn truncate(path: &[u8], length: i64) -> Result<()> {
    let fd = open(path, super::oflag::O_WRONLY, 0)?;
    let res = ftruncate(fd, length);
    let _ = close(fd);
    res
}

pub fn readv(fd: i32, iov: &mut [super::IoVec]) -> Result<usize> {
    unsafe {
        sys3(
            nr::READV,
            fd as usize,
            iov.as_mut_ptr() as usize,
            iov.len(),
        )
    }
}

pub fn writev(fd: i32, iov: &mut [super::IoVec]) -> Result<usize> {
    unsafe {
        sys3(
            nr::WRITEV,
            fd as usize,
            iov.as_mut_ptr() as usize,
            iov.len(),
        )
    }
}

pub fn sched_yield() -> Result<()> {
    unsafe { sys0(nr::SCHED_YIELD).map(|_| ()) }
}

pub fn madvise(addr: usize, len: usize, advice: i32) -> Result<()> {
    unsafe { sys3(nr::MADVISE, addr, len, advice as usize).map(|_| ()) }
}

pub fn brk(addr: usize) -> Result<usize> {
    // brk(2) returns the new program break (not an error-in-negative style
    // consistently across arches via libc; raw syscall returns the break).
    let ret = unsafe { syscall(nr::BRK, addr, 0, 0, 0, 0, 0) };
    if ret < 0 {
        from_ret(ret)
    } else {
        Ok(ret as usize)
    }
}

pub fn fallocate(fd: i32, mode: i32, offset: i64, len: i64) -> Result<()> {
    unsafe {
        sys4(
            nr::FALLOCATE,
            fd as usize,
            mode as usize,
            offset as usize,
            len as usize,
        )
        .map(|_| ())
    }
}

pub fn gettimeofday() -> Result<super::Timeval> {
    let mut tv = super::Timeval::default();
    unsafe {
        sys2(nr::GETTIMEOFDAY, &mut tv as *mut super::Timeval as usize, 0)?;
    }
    Ok(tv)
}

pub fn clock_nanosleep(clock_id: i32, flags: i32, req: &Timespec) -> Result<()> {
    unsafe {
        sys4(
            nr::CLOCK_NANOSLEEP,
            clock_id as usize,
            flags as usize,
            req as *const Timespec as usize,
            0,
        )
        .map(|_| ())
    }
}

pub fn prlimit64(
    pid: i32,
    resource: i32,
    new_limit: Option<&super::Rlimit>,
    old_limit: Option<&mut super::Rlimit>,
) -> Result<()> {
    let new_ptr = new_limit
        .map(|r| r as *const super::Rlimit as usize)
        .unwrap_or(0);
    let old_ptr = old_limit
        .map(|r| r as *mut super::Rlimit as usize)
        .unwrap_or(0);
    unsafe {
        sys4(
            nr::PRLIMIT64,
            pid as usize,
            resource as usize,
            new_ptr,
            old_ptr,
        )
        .map(|_| ())
    }
}

pub fn socket(domain: i32, ty: i32, protocol: i32) -> Result<i32> {
    unsafe {
        sys3(
            nr::SOCKET,
            domain as usize,
            ty as usize,
            protocol as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn socketpair(domain: i32, ty: i32, protocol: i32) -> Result<(i32, i32)> {
    let mut sv = [0i32; 2];
    unsafe {
        sys4(
            nr::SOCKETPAIR,
            domain as usize,
            ty as usize,
            protocol as usize,
            sv.as_mut_ptr() as usize,
        )?;
    }
    Ok((sv[0], sv[1]))
}

pub fn send(fd: i32, buf: &[u8], flags: i32) -> Result<usize> {
    unsafe {
        sys6(
            nr::SENDTO,
            fd as usize,
            buf.as_ptr() as usize,
            buf.len(),
            flags as usize,
            0,
            0,
        )
    }
}

pub fn recv(fd: i32, buf: &mut [u8], flags: i32) -> Result<usize> {
    unsafe {
        sys6(
            nr::RECVFROM,
            fd as usize,
            buf.as_mut_ptr() as usize,
            buf.len(),
            flags as usize,
            0,
            0,
        )
    }
}

pub fn sendto(
    fd: i32,
    buf: &[u8],
    flags: i32,
    addr: Option<&super::SockAddrIn>,
) -> Result<usize> {
    let (addr_ptr, addr_len) = match addr {
        Some(a) => (
            a as *const super::SockAddrIn as usize,
            core::mem::size_of::<super::SockAddrIn>(),
        ),
        None => (0, 0),
    };
    unsafe {
        sys6(
            nr::SENDTO,
            fd as usize,
            buf.as_ptr() as usize,
            buf.len(),
            flags as usize,
            addr_ptr,
            addr_len,
        )
    }
}

pub fn recvfrom(
    fd: i32,
    buf: &mut [u8],
    flags: i32,
    addr: Option<&mut super::SockAddrIn>,
    addrlen: Option<&mut u32>,
) -> Result<usize> {
    let addr_ptr = addr
        .map(|a| a as *mut super::SockAddrIn as usize)
        .unwrap_or(0);
    let len_ptr = addrlen.map(|l| l as *mut u32 as usize).unwrap_or(0);
    unsafe {
        sys6(
            nr::RECVFROM,
            fd as usize,
            buf.as_mut_ptr() as usize,
            buf.len(),
            flags as usize,
            addr_ptr,
            len_ptr,
        )
    }
}

pub fn sendmsg(fd: i32, msg: &super::MsgHdr, flags: i32) -> Result<usize> {
    unsafe {
        sys3(
            nr::SENDMSG,
            fd as usize,
            msg as *const super::MsgHdr as usize,
            flags as usize,
        )
    }
}

pub fn recvmsg(fd: i32, msg: &mut super::MsgHdr, flags: i32) -> Result<usize> {
    unsafe {
        sys3(
            nr::RECVMSG,
            fd as usize,
            msg as *mut super::MsgHdr as usize,
            flags as usize,
        )
    }
}

pub fn shutdown(fd: i32, how: i32) -> Result<()> {
    unsafe { sys2(nr::SHUTDOWN, fd as usize, how as usize).map(|_| ()) }
}

pub fn eventfd(initval: u32, flags: i32) -> Result<i32> {
    unsafe {
        sys2(nr::EVENTFD2, initval as usize, flags as usize).map(|v| v as i32)
    }
}

pub fn poll(fds: &mut [super::poll::PollFd], timeout_ms: i32) -> Result<usize> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        sys3(
            nr::POLL,
            fds.as_mut_ptr() as usize,
            fds.len(),
            timeout_ms as usize,
        )
    }
    #[cfg(target_arch = "aarch64")]
    {
        let ts = if timeout_ms < 0 {
            None
        } else {
            Some(Timespec {
                tv_sec: (timeout_ms as i64) / 1000,
                tv_nsec: ((timeout_ms as i64) % 1000) * 1_000_000,
            })
        };
        unsafe {
            sys5(
                nr::PPOLL,
                fds.as_mut_ptr() as usize,
                fds.len(),
                ts.as_ref()
                    .map(|t| t as *const Timespec as usize)
                    .unwrap_or(0),
                0,
                0,
            )
        }
    }
}

pub fn epoll_create1(flags: i32) -> Result<i32> {
    unsafe { sys1(nr::EPOLL_CREATE1, flags as usize).map(|v| v as i32) }
}

pub fn epoll_ctl(
    epfd: i32,
    op: i32,
    fd: i32,
    event: &mut super::epoll::EpollEvent,
) -> Result<()> {
    unsafe {
        sys4(
            nr::EPOLL_CTL,
            epfd as usize,
            op as usize,
            fd as usize,
            event as *mut super::epoll::EpollEvent as usize,
        )
        .map(|_| ())
    }
}

pub fn epoll_wait(
    epfd: i32,
    events: &mut [super::epoll::EpollEvent],
    timeout_ms: i32,
) -> Result<usize> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        sys4(
            nr::EPOLL_WAIT,
            epfd as usize,
            events.as_mut_ptr() as usize,
            events.len(),
            timeout_ms as usize,
        )
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        sys5(
            nr::EPOLL_PWAIT,
            epfd as usize,
            events.as_mut_ptr() as usize,
            events.len(),
            timeout_ms as usize,
            0,
        )
    }
}

pub fn mknodat(dirfd: i32, path: &[u8], mode: u32, dev: u64) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys4(
            nr::MKNODAT,
            dirfd as usize,
            p as usize,
            mode as usize,
            dev as usize,
        )
        .map(|_| ())
    }
}

pub fn utimensat(
    dirfd: i32,
    path: &[u8],
    times: &[Timespec; 2],
    flags: i32,
) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys4(
            nr::UTIMENSAT,
            dirfd as usize,
            p as usize,
            times.as_ptr() as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn chown(path: &[u8], uid: u32, gid: u32) -> Result<()> {
    // Prefer fchownat for portability.
    #[cfg(target_arch = "x86_64")]
    {
        const FCHOWNAT: usize = 260;
        let p = c_str_ptr(path)?;
        unsafe {
            sys5(
                FCHOWNAT,
                AT_FDCWD as usize,
                p as usize,
                uid as usize,
                gid as usize,
                0,
            )
            .map(|_| ())
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        const FCHOWNAT: usize = 54;
        let p = c_str_ptr(path)?;
        unsafe {
            sys5(
                FCHOWNAT,
                AT_FDCWD as usize,
                p as usize,
                uid as usize,
                gid as usize,
                0,
            )
            .map(|_| ())
        }
    }
}

// --- Extended syscall wrappers (LTP-style tests) ---

pub fn clock_getres(clock_id: i32) -> Result<super::Timespec> {
    let mut ts = super::Timespec::default();
    unsafe {
        sys2(
            nr::CLOCK_GETRES,
            clock_id as usize,
            &mut ts as *mut super::Timespec as usize,
        )?;
    }
    Ok(ts)
}

pub fn mremap(
    old_addr: usize,
    old_len: usize,
    new_len: usize,
    flags: i32,
    new_addr: usize,
) -> Result<usize> {
    unsafe {
        sys5(
            nr::MREMAP,
            old_addr,
            old_len,
            new_len,
            flags as usize,
            new_addr,
        )
    }
}

pub fn msync(addr: usize, len: usize, flags: i32) -> Result<()> {
    unsafe { sys3(nr::MSYNC, addr, len, flags as usize).map(|_| ()) }
}

pub fn mincore(addr: usize, len: usize, vec: &mut [u8]) -> Result<()> {
    unsafe {
        sys3(
            nr::MINCORE,
            addr,
            len,
            vec.as_mut_ptr() as usize,
        )
        .map(|_| ())
    }
}

pub fn sendfile(out_fd: i32, in_fd: i32, offset: &mut i64, count: usize) -> Result<usize> {
    unsafe {
        sys4(
            nr::SENDFILE,
            out_fd as usize,
            in_fd as usize,
            offset as *mut i64 as usize,
            count,
        )
    }
}

pub fn splice(
    fd_in: i32,
    off_in: Option<&mut i64>,
    fd_out: i32,
    off_out: Option<&mut i64>,
    len: usize,
    flags: u32,
) -> Result<usize> {
    let oi = off_in.map(|p| p as *mut i64 as usize).unwrap_or(0);
    let oo = off_out.map(|p| p as *mut i64 as usize).unwrap_or(0);
    unsafe {
        sys6(
            nr::SPLICE,
            fd_in as usize,
            oi,
            fd_out as usize,
            oo,
            len,
            flags as usize,
        )
    }
}

pub fn copy_file_range(
    fd_in: i32,
    off_in: Option<&mut i64>,
    fd_out: i32,
    off_out: Option<&mut i64>,
    len: usize,
    flags: u32,
) -> Result<usize> {
    let oi = off_in.map(|p| p as *mut i64 as usize).unwrap_or(0);
    let oo = off_out.map(|p| p as *mut i64 as usize).unwrap_or(0);
    unsafe {
        sys6(
            nr::COPY_FILE_RANGE,
            fd_in as usize,
            oi,
            fd_out as usize,
            oo,
            len,
            flags as usize,
        )
    }
}

pub fn memfd_create(name: &[u8], flags: u32) -> Result<i32> {
    unsafe {
        sys2(nr::MEMFD_CREATE, name.as_ptr() as usize, flags as usize)
            .map(|v| v as i32)
    }
}

pub fn timerfd_create(clockid: i32, flags: i32) -> Result<i32> {
    unsafe {
        sys2(nr::TIMERFD_CREATE, clockid as usize, flags as usize).map(|v| v as i32)
    }
}

pub fn timerfd_settime(fd: i32, flags: i32, new_value: &super::Itimerspec) -> Result<()> {
    unsafe {
        sys4(
            nr::TIMERFD_SETTIME,
            fd as usize,
            flags as usize,
            new_value as *const super::Itimerspec as usize,
            0,
        )
        .map(|_| ())
    }
}

pub fn timerfd_gettime(fd: i32) -> Result<super::Itimerspec> {
    let mut its = super::Itimerspec::default();
    unsafe {
        sys2(
            nr::TIMERFD_GETTIME,
            fd as usize,
            &mut its as *mut super::Itimerspec as usize,
        )?;
    }
    Ok(its)
}

pub fn futex_wait(
    uaddr: &core::sync::atomic::AtomicU32,
    val: u32,
    timeout: Option<&super::Timespec>,
) -> Result<()> {
    let op = super::FUTEX_WAIT | super::FUTEX_PRIVATE_FLAG;
    let to = timeout
        .map(|t| t as *const super::Timespec as usize)
        .unwrap_or(0);
    unsafe {
        sys6(
            nr::FUTEX,
            uaddr as *const _ as usize,
            op as usize,
            val as usize,
            to,
            0,
            0,
        )
        .map(|_| ())
    }
}

pub fn futex_wake(uaddr: &core::sync::atomic::AtomicU32, count: u32) -> Result<usize> {
    let op = super::FUTEX_WAKE | super::FUTEX_PRIVATE_FLAG;
    unsafe {
        sys6(
            nr::FUTEX,
            uaddr as *const _ as usize,
            op as usize,
            count as usize,
            0,
            0,
            0,
        )
    }
}

pub fn waitid(idtype: i32, id: i32, infop: &mut super::Siginfo, options: i32) -> Result<()> {
    unsafe {
        sys5(
            nr::WAITID,
            idtype as usize,
            id as usize,
            infop as *mut super::Siginfo as usize,
            options as usize,
            0,
        )
        .map(|_| ())
    }
}

pub fn getpgid(pid: i32) -> Result<i32> {
    unsafe { sys1(nr::GETPGID, pid as usize).map(|v| v as i32) }
}

pub fn setpgid(pid: i32, pgid: i32) -> Result<()> {
    unsafe { sys2(nr::SETPGID, pid as usize, pgid as usize).map(|_| ()) }
}

pub fn getsid(pid: i32) -> Result<i32> {
    unsafe { sys1(nr::GETSID, pid as usize).map(|v| v as i32) }
}

pub fn setsid() -> Result<i32> {
    unsafe { sys0(nr::SETSID).map(|v| v as i32) }
}

pub fn getpriority(which: i32, who: i32) -> Result<i32> {
    let ret = unsafe { syscall(nr::GETPRIORITY, which as usize, who as usize, 0, 0, 0, 0) };
    if ret < 0 {
        from_ret(ret).map(|_| 0)
    } else {
        // getpriority returns nice in 1..40 range; userspace expects -20..19.
        Ok(20 - ret as i32)
    }
}

pub fn sched_getaffinity(pid: i32, mask: &mut [u8]) -> Result<()> {
    unsafe {
        sys3(
            nr::SCHED_GETAFFINITY,
            pid as usize,
            mask.len(),
            mask.as_mut_ptr() as usize,
        )
        .map(|_| ())
    }
}

pub fn sched_setaffinity(pid: i32, mask: &[u8]) -> Result<()> {
    unsafe {
        sys3(
            nr::SCHED_SETAFFINITY,
            pid as usize,
            mask.len(),
            mask.as_ptr() as usize,
        )
        .map(|_| ())
    }
}

pub fn sched_getscheduler(pid: i32) -> Result<i32> {
    unsafe { sys1(nr::SCHED_GETSCHEDULER, pid as usize).map(|v| v as i32) }
}

pub fn prctl_get_name(buf: &mut [u8; 16]) -> Result<()> {
    unsafe {
        sys2(
            nr::PRCTL,
            super::PR_GET_NAME as usize,
            buf.as_mut_ptr() as usize,
        )
        .map(|_| ())
    }
}

pub fn prctl_set_name(name: &[u8]) -> Result<()> {
    unsafe {
        sys2(
            nr::PRCTL,
            super::PR_SET_NAME as usize,
            name.as_ptr() as usize,
        )
        .map(|_| ())
    }
}

pub fn sysinfo() -> Result<super::Sysinfo> {
    let mut info = super::Sysinfo::default();
    unsafe {
        sys1(nr::SYSINFO, &mut info as *mut super::Sysinfo as usize)?;
    }
    Ok(info)
}

pub fn getrusage(who: i32) -> Result<super::Rusage> {
    let mut ru = super::Rusage::default();
    unsafe {
        sys2(
            nr::GETRUSAGE,
            who as usize,
            &mut ru as *mut super::Rusage as usize,
        )?;
    }
    Ok(ru)
}

pub fn times() -> Result<super::Tms> {
    let mut t = super::Tms::default();
    unsafe {
        sys1(nr::TIMES, &mut t as *mut super::Tms as usize)?;
    }
    Ok(t)
}

pub fn sync() -> Result<()> {
    unsafe { sys0(nr::SYNC).map(|_| ()) }
}

pub fn syncfs(fd: i32) -> Result<()> {
    unsafe { sys1(nr::SYNCFS, fd as usize).map(|_| ()) }
}

pub fn flock(fd: i32, op: i32) -> Result<()> {
    unsafe { sys2(nr::FLOCK, fd as usize, op as usize).map(|_| ()) }
}

pub fn statfs(path: &[u8]) -> Result<super::Statfs> {
    let p = c_str_ptr(path)?;
    let mut st = super::Statfs::default();
    unsafe {
        sys2(nr::STATFS, p as usize, &mut st as *mut super::Statfs as usize)?;
    }
    Ok(st)
}

pub fn fstatfs(fd: i32) -> Result<super::Statfs> {
    let mut st = super::Statfs::default();
    unsafe {
        sys2(nr::FSTATFS, fd as usize, &mut st as *mut super::Statfs as usize)?;
    }
    Ok(st)
}

pub fn getsockname(fd: i32, addr: &mut [u8], len: &mut u32) -> Result<()> {
    unsafe {
        sys3(
            nr::GETSOCKNAME,
            fd as usize,
            addr.as_mut_ptr() as usize,
            len as *mut u32 as usize,
        )
        .map(|_| ())
    }
}

pub fn getpeername(fd: i32, addr: &mut [u8], len: &mut u32) -> Result<()> {
    unsafe {
        sys3(
            nr::GETPEERNAME,
            fd as usize,
            addr.as_mut_ptr() as usize,
            len as *mut u32 as usize,
        )
        .map(|_| ())
    }
}

pub fn setsockopt(fd: i32, level: i32, optname: i32, optval: &[u8]) -> Result<()> {
    unsafe {
        sys5(
            nr::SETSOCKOPT,
            fd as usize,
            level as usize,
            optname as usize,
            optval.as_ptr() as usize,
            optval.len(),
        )
        .map(|_| ())
    }
}

pub fn getsockopt(fd: i32, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize> {
    let mut len = optval.len() as u32;
    unsafe {
        sys5(
            nr::GETSOCKOPT,
            fd as usize,
            level as usize,
            optname as usize,
            optval.as_mut_ptr() as usize,
            &mut len as *mut u32 as usize,
        )
        .map(|_| len as usize)
    }
}

pub fn getresuid() -> Result<(u32, u32, u32)> {
    let mut ruid = 0u32;
    let mut euid = 0u32;
    let mut suid = 0u32;
    unsafe {
        sys3(
            nr::GETRESUID,
            &mut ruid as *mut u32 as usize,
            &mut euid as *mut u32 as usize,
            &mut suid as *mut u32 as usize,
        )?;
    }
    Ok((ruid, euid, suid))
}

pub fn getresgid() -> Result<(u32, u32, u32)> {
    let mut rgid = 0u32;
    let mut egid = 0u32;
    let mut sgid = 0u32;
    unsafe {
        sys3(
            nr::GETRESGID,
            &mut rgid as *mut u32 as usize,
            &mut egid as *mut u32 as usize,
            &mut sgid as *mut u32 as usize,
        )?;
    }
    Ok((rgid, egid, sgid))
}

pub fn rt_sigprocmask(
    how: i32,
    set: Option<super::Sigset>,
    oldset: Option<&mut super::Sigset>,
) -> Result<()> {
    // Keep `set` alive for the syscall; do not take a pointer to a temporary.
    let set_storage = set;
    let set_ptr = match &set_storage {
        Some(s) => s as *const super::Sigset as usize,
        None => 0,
    };
    let old_ptr = oldset
        .map(|s| s as *mut super::Sigset as usize)
        .unwrap_or(0);
    unsafe {
        sys4(
            nr::RT_SIGPROCMASK,
            how as usize,
            set_ptr,
            old_ptr,
            core::mem::size_of::<super::Sigset>(),
        )
        .map(|_| ())
    }
}

pub fn rt_sigpending(set: &mut super::Sigset) -> Result<()> {
    unsafe {
        sys2(
            nr::RT_SIGPENDING,
            set as *mut super::Sigset as usize,
            core::mem::size_of::<super::Sigset>(),
        )
        .map(|_| ())
    }
}

pub fn rt_sigaction(
    sig: i32,
    act: Option<&super::Sigaction>,
    oldact: Option<&mut super::Sigaction>,
) -> Result<()> {
    let act_ptr = act.map(|a| a as *const super::Sigaction as usize).unwrap_or(0);
    let old_ptr = oldact
        .map(|a| a as *mut super::Sigaction as usize)
        .unwrap_or(0);
    unsafe {
        sys4(
            nr::RT_SIGACTION,
            sig as usize,
            act_ptr,
            old_ptr,
            core::mem::size_of::<super::Sigset>(),
        )
        .map(|_| ())
    }
}

/// Install `SIG_IGN` so a later unblock of a pending signal does not terminate the process.
pub fn signal_ignore(sig: i32) -> Result<()> {
    let act = super::Sigaction {
        sa_handler: super::SIG_IGN,
        ..super::Sigaction::default()
    };
    rt_sigaction(sig, Some(&act), None)
}

pub fn signal_default(sig: i32) -> Result<()> {
    let act = super::Sigaction {
        sa_handler: super::SIG_DFL,
        ..super::Sigaction::default()
    };
    rt_sigaction(sig, Some(&act), None)
}

pub fn sigmask(sig: i32) -> super::Sigset {
    1u64 << (sig - 1)
}

pub fn bind(fd: i32, addr: &super::SockAddrIn) -> Result<()> {
    unsafe {
        sys3(
            nr::BIND,
            fd as usize,
            addr as *const super::SockAddrIn as usize,
            core::mem::size_of::<super::SockAddrIn>(),
        )
        .map(|_| ())
    }
}

pub fn listen(fd: i32, backlog: i32) -> Result<()> {
    unsafe { sys2(nr::LISTEN, fd as usize, backlog as usize).map(|_| ()) }
}

pub fn connect(fd: i32, addr: &super::SockAddrIn) -> Result<()> {
    unsafe {
        sys3(
            nr::CONNECT,
            fd as usize,
            addr as *const super::SockAddrIn as usize,
            core::mem::size_of::<super::SockAddrIn>(),
        )
        .map(|_| ())
    }
}

pub fn accept4(
    fd: i32,
    addr: Option<&mut super::SockAddrIn>,
    addrlen: Option<&mut u32>,
    flags: i32,
) -> Result<i32> {
    let addr_ptr = addr
        .map(|a| a as *mut super::SockAddrIn as usize)
        .unwrap_or(0);
    let len_ptr = addrlen.map(|l| l as *mut u32 as usize).unwrap_or(0);
    unsafe {
        sys4(
            nr::ACCEPT4,
            fd as usize,
            addr_ptr,
            len_ptr,
            flags as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn getsockname_in(fd: i32) -> Result<super::SockAddrIn> {
    let mut addr = super::SockAddrIn::default();
    let mut len = core::mem::size_of::<super::SockAddrIn>() as u32;
    unsafe {
        sys3(
            nr::GETSOCKNAME,
            fd as usize,
            &mut addr as *mut super::SockAddrIn as usize,
            &mut len as *mut u32 as usize,
        )?;
    }
    Ok(addr)
}

pub fn getpeername_in(fd: i32) -> Result<super::SockAddrIn> {
    let mut addr = super::SockAddrIn::default();
    let mut len = core::mem::size_of::<super::SockAddrIn>() as u32;
    unsafe {
        sys3(
            nr::GETPEERNAME,
            fd as usize,
            &mut addr as *mut super::SockAddrIn as usize,
            &mut len as *mut u32 as usize,
        )?;
    }
    Ok(addr)
}

pub fn signalfd(fd: i32, mask: super::Sigset, flags: i32) -> Result<i32> {
    // Keep mask alive for the syscall duration.
    let mask_storage = mask;
    unsafe {
        sys4(
            nr::SIGNALFD4,
            fd as usize,
            &mask_storage as *const super::Sigset as usize,
            core::mem::size_of::<super::Sigset>(),
            flags as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn ppoll(
    fds: &mut [super::poll::PollFd],
    timeout: Option<&Timespec>,
    sigmask: Option<&super::Sigset>,
) -> Result<usize> {
    let ts_ptr = timeout
        .map(|t| t as *const Timespec as usize)
        .unwrap_or(0);
    let mask_ptr = sigmask
        .map(|s| s as *const super::Sigset as usize)
        .unwrap_or(0);
    let mask_size = if mask_ptr != 0 {
        core::mem::size_of::<super::Sigset>()
    } else {
        0
    };
    unsafe {
        sys5(
            nr::PPOLL,
            fds.as_mut_ptr() as usize,
            fds.len(),
            ts_ptr,
            mask_ptr,
            mask_size,
        )
    }
}

pub fn tee(fd_in: i32, fd_out: i32, len: usize, flags: u32) -> Result<usize> {
    unsafe {
        sys4(
            nr::TEE,
            fd_in as usize,
            fd_out as usize,
            len,
            flags as usize,
        )
    }
}

pub fn close_range(first: u32, last: u32, flags: u32) -> Result<()> {
    unsafe {
        sys3(
            nr::CLOSE_RANGE,
            first as usize,
            last as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn renameat2(
    olddirfd: i32,
    oldpath: &[u8],
    newdirfd: i32,
    newpath: &[u8],
    flags: u32,
) -> Result<()> {
    let old = c_str_ptr(oldpath)?;
    let new = c_str_ptr(newpath)?;
    unsafe {
        sys5(
            nr::RENAMEAT2,
            olddirfd as usize,
            old as usize,
            newdirfd as usize,
            new as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn inotify_init1(flags: i32) -> Result<i32> {
    unsafe { sys1(nr::INOTIFY_INIT1, flags as usize).map(|v| v as i32) }
}

pub fn inotify_add_watch(fd: i32, path: &[u8], mask: u32) -> Result<i32> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys3(
            nr::INOTIFY_ADD_WATCH,
            fd as usize,
            p as usize,
            mask as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn inotify_rm_watch(fd: i32, wd: i32) -> Result<()> {
    unsafe {
        sys2(nr::INOTIFY_RM_WATCH, fd as usize, wd as usize).map(|_| ())
    }
}

pub fn ioctl(fd: i32, request: usize, arg: usize) -> Result<usize> {
    unsafe { sys3(nr::IOCTL, fd as usize, request, arg) }
}

pub fn preadv(fd: i32, iov: &mut [super::IoVec], offset: i64) -> Result<usize> {
    unsafe {
        sys5(
            nr::PREADV,
            fd as usize,
            iov.as_mut_ptr() as usize,
            iov.len(),
            offset as usize,
            0,
        )
    }
}

pub fn pwritev(fd: i32, iov: &mut [super::IoVec], offset: i64) -> Result<usize> {
    unsafe {
        sys5(
            nr::PWRITEV,
            fd as usize,
            iov.as_mut_ptr() as usize,
            iov.len(),
            offset as usize,
            0,
        )
    }
}

pub fn pidfd_open(pid: i32, flags: u32) -> Result<i32> {
    unsafe {
        sys2(nr::PIDFD_OPEN, pid as usize, flags as usize).map(|v| v as i32)
    }
}

pub fn pidfd_send_signal(
    pidfd: i32,
    sig: i32,
    info: Option<&super::Siginfo>,
    flags: u32,
) -> Result<()> {
    let info_ptr = info
        .map(|i| i as *const super::Siginfo as usize)
        .unwrap_or(0);
    unsafe {
        sys4(
            nr::PIDFD_SEND_SIGNAL,
            pidfd as usize,
            sig as usize,
            info_ptr,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn setitimer(
    which: i32,
    new_value: &super::Itimerval,
    old_value: Option<&mut super::Itimerval>,
) -> Result<()> {
    let old_ptr = old_value
        .map(|o| o as *mut super::Itimerval as usize)
        .unwrap_or(0);
    unsafe {
        sys3(
            nr::SETITIMER,
            which as usize,
            new_value as *const super::Itimerval as usize,
            old_ptr,
        )
        .map(|_| ())
    }
}

pub fn getitimer(which: i32, curr: &mut super::Itimerval) -> Result<()> {
    unsafe {
        sys2(
            nr::GETITIMER,
            which as usize,
            curr as *mut super::Itimerval as usize,
        )
        .map(|_| ())
    }
}

pub fn vmsplice(fd: i32, iov: &[super::IoVec], flags: u32) -> Result<usize> {
    unsafe {
        sys4(
            nr::VMSPLICE,
            fd as usize,
            iov.as_ptr() as usize,
            iov.len(),
            flags as usize,
        )
    }
}

/// Sixth argument to `pselect6`: pointer to `{ sigset, sigsetsize }`.
#[repr(C)]
struct Pselect6SigsetArg {
    ss: *const super::Sigset,
    ss_len: usize,
}

pub fn pselect6(
    nfds: i32,
    readfds: Option<&mut super::FdSet>,
    writefds: Option<&mut super::FdSet>,
    exceptfds: Option<&mut super::FdSet>,
    timeout: Option<&Timespec>,
    sigmask: Option<&super::Sigset>,
) -> Result<usize> {
    let r = readfds
        .map(|f| f as *mut super::FdSet as usize)
        .unwrap_or(0);
    let w = writefds
        .map(|f| f as *mut super::FdSet as usize)
        .unwrap_or(0);
    let e = exceptfds
        .map(|f| f as *mut super::FdSet as usize)
        .unwrap_or(0);
    let ts = timeout
        .map(|t| t as *const Timespec as usize)
        .unwrap_or(0);
    // Keep the optional sigset and arg struct alive across the syscall.
    let mask_storage = sigmask.copied();
    let arg = mask_storage.as_ref().map(|m| Pselect6SigsetArg {
        ss: m as *const super::Sigset,
        ss_len: core::mem::size_of::<super::Sigset>(),
    });
    let arg_ptr = arg
        .as_ref()
        .map(|a| a as *const Pselect6SigsetArg as usize)
        .unwrap_or(0);
    unsafe { sys6(nr::PSELECT6, nfds as usize, r, w, e, ts, arg_ptr) }
}

pub fn getcpu(cpu: Option<&mut u32>, node: Option<&mut u32>) -> Result<()> {
    let c = cpu.map(|p| p as *mut u32 as usize).unwrap_or(0);
    let n = node.map(|p| p as *mut u32 as usize).unwrap_or(0);
    unsafe { sys3(nr::GETCPU, c, n, 0).map(|_| ()) }
}

pub fn sigaltstack(
    new: Option<&super::Stack>,
    old: Option<&mut super::Stack>,
) -> Result<()> {
    let n = new
        .map(|s| s as *const super::Stack as usize)
        .unwrap_or(0);
    let o = old
        .map(|s| s as *mut super::Stack as usize)
        .unwrap_or(0);
    unsafe { sys2(nr::SIGALTSTACK, n, o).map(|_| ()) }
}

pub fn statx(
    dirfd: i32,
    path: &[u8],
    flags: i32,
    mask: u32,
    buf: &mut super::Statx,
) -> Result<()> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys5(
            nr::STATX,
            dirfd as usize,
            p as usize,
            flags as usize,
            mask as usize,
            buf as *mut super::Statx as usize,
        )
        .map(|_| ())
    }
}

pub fn openat2(dirfd: i32, path: &[u8], how: &super::OpenHow) -> Result<i32> {
    let p = c_str_ptr(path)?;
    unsafe {
        sys4(
            nr::OPENAT2,
            dirfd as usize,
            p as usize,
            how as *const super::OpenHow as usize,
            core::mem::size_of::<super::OpenHow>(),
        )
        .map(|v| v as i32)
    }
}

pub fn sync_file_range(fd: i32, offset: i64, nbytes: i64, flags: u32) -> Result<()> {
    unsafe {
        sys4(
            nr::SYNC_FILE_RANGE,
            fd as usize,
            offset as usize,
            nbytes as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn fadvise64(fd: i32, offset: i64, len: i64, advice: i32) -> Result<()> {
    unsafe {
        sys4(
            nr::FADVISE64,
            fd as usize,
            offset as usize,
            len as usize,
            advice as usize,
        )
        .map(|_| ())
    }
}

/// Alias matching the POSIX name; same kernel entry as `fadvise64`.
pub fn posix_fadvise(fd: i32, offset: i64, len: i64, advice: i32) -> Result<()> {
    fadvise64(fd, offset, len, advice)
}

pub fn membarrier(cmd: i32, flags: u32) -> Result<i32> {
    unsafe {
        sys2(nr::MEMBARRIER, cmd as usize, flags as usize).map(|v| v as i32)
    }
}

pub fn personality(persona: u32) -> Result<u32> {
    unsafe { sys1(nr::PERSONALITY, persona as usize).map(|v| v as u32) }
}

pub fn capget(
    header: &mut super::CapUserHeader,
    data: &mut [super::CapUserData],
) -> Result<()> {
    unsafe {
        sys2(
            nr::CAPGET,
            header as *mut super::CapUserHeader as usize,
            data.as_mut_ptr() as usize,
        )
        .map(|_| ())
    }
}

pub fn unshare(flags: u64) -> Result<()> {
    unsafe { sys1(nr::UNSHARE, flags as usize).map(|_| ()) }
}

pub fn readahead(fd: i32, offset: i64, count: usize) -> Result<()> {
    unsafe {
        sys3(
            nr::READAHEAD,
            fd as usize,
            offset as usize,
            count,
        )
        .map(|_| ())
    }
}

pub fn process_vm_readv(
    pid: i32,
    local_iov: &mut [super::IoVec],
    remote_iov: &[super::IoVec],
    flags: u64,
) -> Result<usize> {
    unsafe {
        sys6(
            nr::PROCESS_VM_READV,
            pid as usize,
            local_iov.as_mut_ptr() as usize,
            local_iov.len(),
            remote_iov.as_ptr() as usize,
            remote_iov.len(),
            flags as usize,
        )
    }
}

pub fn process_vm_writev(
    pid: i32,
    local_iov: &[super::IoVec],
    remote_iov: &[super::IoVec],
    flags: u64,
) -> Result<usize> {
    unsafe {
        sys6(
            nr::PROCESS_VM_WRITEV,
            pid as usize,
            local_iov.as_ptr() as usize,
            local_iov.len(),
            remote_iov.as_ptr() as usize,
            remote_iov.len(),
            flags as usize,
        )
    }
}

pub fn kcmp(pid1: i32, pid2: i32, typ: i32, idx1: u64, idx2: u64) -> Result<i32> {
    unsafe {
        sys5(
            nr::KCMP,
            pid1 as usize,
            pid2 as usize,
            typ as usize,
            idx1 as usize,
            idx2 as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn shmget(key: i32, size: usize, shmflg: i32) -> Result<i32> {
    unsafe {
        sys3(
            nr::SHMGET,
            key as usize,
            size,
            shmflg as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn shmat(shmid: i32, shmaddr: usize, shmflg: i32) -> Result<usize> {
    unsafe {
        sys3(
            nr::SHMAT,
            shmid as usize,
            shmaddr,
            shmflg as usize,
        )
    }
}

pub fn shmdt(shmaddr: usize) -> Result<()> {
    unsafe { sys1(nr::SHMDT, shmaddr).map(|_| ()) }
}

pub fn shmctl(shmid: i32, cmd: i32, buf: usize) -> Result<i32> {
    unsafe {
        sys3(nr::SHMCTL, shmid as usize, cmd as usize, buf).map(|v| v as i32)
    }
}

pub fn mq_open(
    name: &[u8],
    oflag: i32,
    mode: u32,
    attr: Option<&super::MqAttr>,
) -> Result<i32> {
    let p = c_str_ptr(name)?;
    let a = attr
        .map(|x| x as *const super::MqAttr as usize)
        .unwrap_or(0);
    unsafe {
        sys4(
            nr::MQ_OPEN,
            p as usize,
            oflag as usize,
            mode as usize,
            a,
        )
        .map(|v| v as i32)
    }
}

pub fn mq_unlink(name: &[u8]) -> Result<()> {
    let p = c_str_ptr(name)?;
    unsafe { sys1(nr::MQ_UNLINK, p as usize).map(|_| ()) }
}

pub fn landlock_create_ruleset(
    attr: Option<&super::LandlockRulesetAttr>,
    size: usize,
    flags: u32,
) -> Result<i32> {
    let a = attr
        .map(|x| x as *const super::LandlockRulesetAttr as usize)
        .unwrap_or(0);
    unsafe {
        sys3(
            nr::LANDLOCK_CREATE_RULESET,
            a,
            size,
            flags as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn landlock_restrict_self(ruleset_fd: i32, flags: u32) -> Result<()> {
    unsafe {
        sys2(
            nr::LANDLOCK_RESTRICT_SELF,
            ruleset_fd as usize,
            flags as usize,
        )
        .map(|_| ())
    }
}

pub fn userfaultfd(flags: i32) -> Result<i32> {
    unsafe { sys1(nr::USERFAULTFD, flags as usize).map(|v| v as i32) }
}

pub fn pidfd_getfd(pidfd: i32, targetfd: i32, flags: u32) -> Result<i32> {
    unsafe {
        sys3(
            nr::PIDFD_GETFD,
            pidfd as usize,
            targetfd as usize,
            flags as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn clock_settime(clock_id: i32, tp: &Timespec) -> Result<()> {
    unsafe {
        sys2(
            nr::CLOCK_SETTIME,
            clock_id as usize,
            tp as *const Timespec as usize,
        )
        .map(|_| ())
    }
}

pub fn setpriority(which: i32, who: i32, prio: i32) -> Result<()> {
    unsafe {
        sys3(
            nr::SETPRIORITY,
            which as usize,
            who as usize,
            prio as usize,
        )
        .map(|_| ())
    }
}

/// `futimens(fd, times)` — `utimensat` with null path.
pub fn futimens(fd: i32, times: &[Timespec; 2]) -> Result<()> {
    unsafe {
        sys4(
            nr::UTIMENSAT,
            fd as usize,
            0,
            times.as_ptr() as usize,
            0,
        )
        .map(|_| ())
    }
}

pub fn fcntl_flock(fd: i32, cmd: i32, lock: &mut super::Flock) -> Result<()> {
    unsafe {
        sys3(
            nr::FCNTL,
            fd as usize,
            cmd as usize,
            lock as *mut super::Flock as usize,
        )
        .map(|_| ())
    }
}

/// `waitpid` — alias of `wait4` with null rusage.
pub fn waitpid(pid: i32, status: &mut i32, options: i32) -> Result<i32> {
    wait4(pid, status, options)
}

pub fn wifstopped(status: i32) -> bool {
    (status & 0xff) == 0x7f
}

pub fn wstopsig(status: i32) -> i32 {
    (status >> 8) & 0xff
}

pub fn io_uring_setup(entries: u32, params: &mut super::IoUringParams) -> Result<i32> {
    unsafe {
        sys2(
            nr::IO_URING_SETUP,
            entries as usize,
            params as *mut super::IoUringParams as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn io_uring_enter(
    fd: i32,
    to_submit: u32,
    min_complete: u32,
    flags: u32,
    sig: usize,
) -> Result<usize> {
    unsafe {
        sys6(
            nr::IO_URING_ENTER,
            fd as usize,
            to_submit as usize,
            min_complete as usize,
            flags as usize,
            sig,
            0,
        )
    }
}

pub fn io_uring_register(fd: i32, opcode: u32, arg: usize, nr_args: u32) -> Result<usize> {
    unsafe {
        sys4(
            nr::IO_URING_REGISTER,
            fd as usize,
            opcode as usize,
            arg,
            nr_args as usize,
        )
    }
}

pub fn timer_create(clock_id: i32, sevp: Option<&super::Sigevent>, timerid: &mut usize) -> Result<()> {
    let sev = sevp
        .map(|s| s as *const super::Sigevent as usize)
        .unwrap_or(0);
    unsafe {
        sys3(
            nr::TIMER_CREATE,
            clock_id as usize,
            sev,
            timerid as *mut usize as usize,
        )
        .map(|_| ())
    }
}

pub fn timer_settime(
    timerid: usize,
    flags: i32,
    new_value: &super::Itimerspec,
    old_value: Option<&mut super::Itimerspec>,
) -> Result<()> {
    let old = old_value
        .map(|o| o as *mut super::Itimerspec as usize)
        .unwrap_or(0);
    unsafe {
        sys4(
            nr::TIMER_SETTIME,
            timerid,
            flags as usize,
            new_value as *const super::Itimerspec as usize,
            old,
        )
        .map(|_| ())
    }
}

pub fn timer_gettime(timerid: usize) -> Result<super::Itimerspec> {
    let mut cur = super::Itimerspec::default();
    unsafe {
        sys2(
            nr::TIMER_GETTIME,
            timerid,
            &mut cur as *mut super::Itimerspec as usize,
        )?;
    }
    Ok(cur)
}

pub fn timer_delete(timerid: usize) -> Result<()> {
    unsafe { sys1(nr::TIMER_DELETE, timerid).map(|_| ()) }
}

pub fn semget(key: i32, nsems: i32, semflg: i32) -> Result<i32> {
    unsafe {
        sys3(
            nr::SEMGET,
            key as usize,
            nsems as usize,
            semflg as usize,
        )
        .map(|v| v as i32)
    }
}

pub fn semop(semid: i32, sops: &[super::Sembuf]) -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        sys3(
            nr::SEMOP,
            semid as usize,
            sops.as_ptr() as usize,
            sops.len(),
        )
        .map(|_| ())
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // aarch64 exposes semtimedop; NULL timeout == semop.
        sys4(
            nr::SEMTIMEDOP,
            semid as usize,
            sops.as_ptr() as usize,
            sops.len(),
            0,
        )
        .map(|_| ())
    }
}

pub fn semctl(semid: i32, semnum: i32, cmd: i32, arg: usize) -> Result<i32> {
    unsafe {
        sys4(
            nr::SEMCTL,
            semid as usize,
            semnum as usize,
            cmd as usize,
            arg,
        )
        .map(|v| v as i32)
    }
}

pub fn msgget(key: i32, msgflg: i32) -> Result<i32> {
    unsafe {
        sys2(nr::MSGGET, key as usize, msgflg as usize).map(|v| v as i32)
    }
}

pub fn msgsnd(msqid: i32, msgp: &super::MsgBuf, msgsz: usize, msgflg: i32) -> Result<()> {
    unsafe {
        sys4(
            nr::MSGSND,
            msqid as usize,
            msgp as *const super::MsgBuf as usize,
            msgsz,
            msgflg as usize,
        )
        .map(|_| ())
    }
}

pub fn msgrcv(
    msqid: i32,
    msgp: &mut super::MsgBuf,
    msgsz: usize,
    msgtyp: i64,
    msgflg: i32,
) -> Result<usize> {
    unsafe {
        sys5(
            nr::MSGRCV,
            msqid as usize,
            msgp as *mut super::MsgBuf as usize,
            msgsz,
            msgtyp as usize,
            msgflg as usize,
        )
    }
}

pub fn msgctl(msqid: i32, cmd: i32, buf: usize) -> Result<i32> {
    unsafe {
        sys3(nr::MSGCTL, msqid as usize, cmd as usize, buf).map(|v| v as i32)
    }
}

pub fn fsopen(fsname: &[u8], flags: u32) -> Result<i32> {
    let p = c_str_ptr(fsname)?;
    unsafe {
        sys2(nr::FSOPEN, p as usize, flags as usize).map(|v| v as i32)
    }
}

pub fn fsconfig(fd: i32, cmd: u32, key: usize, value: usize, aux: i32) -> Result<()> {
    unsafe {
        sys5(
            nr::FSCONFIG,
            fd as usize,
            cmd as usize,
            key,
            value,
            aux as usize,
        )
        .map(|_| ())
    }
}

pub fn mlock(addr: usize, len: usize) -> Result<()> {
    unsafe { sys2(nr::MLOCK, addr, len).map(|_| ()) }
}

pub fn munlock(addr: usize, len: usize) -> Result<()> {
    unsafe { sys2(nr::MUNLOCK, addr, len).map(|_| ()) }
}

pub fn prctl(option: i32, arg2: usize, arg3: usize, arg4: usize, arg5: usize) -> Result<i32> {
    unsafe {
        sys5(
            nr::PRCTL,
            option as usize,
            arg2,
            arg3,
            arg4,
            arg5,
        )
        .map(|v| v as i32)
    }
}
