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
