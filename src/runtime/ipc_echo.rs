//! Early-process helper modes invoked before the test harness.
//!
//! - `--ipc-channel-echo` — read fd 3, write back, exit
//! - `--ipc-channel-hold` — keep fd 3 open and block until killed (reload/teardown)
//! - `--ipc-channel-hello` — write `HELLO` on fd 3 immediately, then hold
//!   (Theia/Node plugin-host shape: child speaks first on the IPC channel)
//! - node-pty `spawn-helper` argv (`cwd, uid, gid, closeFDs, file`) — CLOEXEC
//!   fd 3 and exec the shell (Theia terminal handshake)
//! - `--theia-ipc-epoll-idle` — plugin-host analogue: inherit fd 3, watch it with
//!   EPOLLIN|EPOLLOUT|EPOLLET, prove epoll_wait sleeps, then write `IDLEOK`

use crate::syscall;
use crate::syscall::clock;
use crate::syscall::epoll;
use crate::syscall::fcntl_cmd;
use crate::syscall::poll;
use crate::syscall::{EPOLLET, EPOLLIN, EPOLLOUT, EPOLL_CTL_ADD, FD_CLOEXEC, TIOCSCTTY};

const CHANNEL_FD: i32 = 3;

unsafe fn argv_cstr<'a>(argv: *const usize, i: usize) -> Option<&'a [u8]> {
    let ptr = *argv.add(i) as *const u8;
    if ptr.is_null() {
        return None;
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
        if len > 64 {
            return None;
        }
    }
    Some(core::slice::from_raw_parts(ptr, len))
}

/// Under xvisor the guest argv is `[ld.so, binary, flag, …]`; Docker is
/// `[binary, flag]`. Scan every argument for helper flags.
unsafe fn argv_flag<'a>(argc: usize, argv: *const usize) -> Option<&'a [u8]> {
    let mut i = 1usize;
    while i < argc {
        let Some(arg) = argv_cstr(argv, i) else {
            break;
        };
        if arg.starts_with(b"--ipc-channel-") || arg.starts_with(b"--plugin-host-") {
            return Some(arg);
        }
        i += 1;
    }
    None
}

fn argv_is_numeric(s: &[u8]) -> bool {
    !s.is_empty() && s.iter().all(|c| c.is_ascii_digit())
}

/// node-pty `posix_spawn(spawn-helper, [cwd, uid, gid, closeFDs, file, …])`.
/// `argv[0]` is the cwd, not the helper path, so detect the argv shape.
unsafe fn maybe_run_pty_spawn_helper(argc: usize, argv: *const usize) -> bool {
    if argc < 5 {
        return false;
    }
    let Some(uid) = argv_cstr(argv, 1) else {
        return false;
    };
    let Some(close_fds) = argv_cstr(argv, 3) else {
        return false;
    };
    let Some(file) = argv_cstr(argv, 4) else {
        return false;
    };
    let uid_ok = uid == b"-1" || argv_is_numeric(uid);
    let close_ok = close_fds == b"0" || close_fds == b"1";
    if !uid_ok || !close_ok || !file.starts_with(b"/") {
        return false;
    }
    run_pty_spawn_helper(argc, argv);
}

fn run_pty_spawn_helper(argc: usize, argv: *const usize) -> ! {
    let _ = syscall::setsid();
    let _ = syscall::ioctl(0, TIOCSCTTY, 0);
    let _ = syscall::fcntl(3, fcntl_cmd::F_SETFD, FD_CLOEXEC as usize);
    let file_ptr = unsafe { *argv.add(4) as *const u8 };
    if file_ptr.is_null() {
        syscall::exit(127);
    }
    let mut path = [0u8; 256];
    let mut plen = 0usize;
    unsafe {
        while plen + 1 < path.len() && *file_ptr.add(plen) != 0 {
            path[plen] = *file_ptr.add(plen);
            plen += 1;
        }
    }
    path[plen] = 0;
    let mut child_argv = [core::ptr::null::<u8>(); 16];
    let mut k = 0usize;
    let mut j = 4usize;
    while j < argc && k < 15 {
        child_argv[k] = unsafe { *argv.add(j) as *const u8 };
        k += 1;
        j += 1;
    }
    let envp: [*const u8; 1] = [core::ptr::null()];
    let _ = syscall::execve(&path[..plen + 1], &child_argv, &envp);
    syscall::exit(127);
}

/// # Safety
/// `argv` must be a valid argc-length argv vector.
pub unsafe fn dispatch_helper(argc: usize, argv: *const usize) -> bool {
    if maybe_run_pty_spawn_helper(argc, argv) {
        // never returns
    }
    match argv_flag(argc, argv) {
        Some(b"--ipc-channel-echo") => {
            run_ipc_channel_echo();
        }
        Some(b"--ipc-channel-hold") => {
            run_ipc_channel_hold();
        }
        Some(b"--ipc-channel-hello") => {
            run_ipc_channel_hello();
        }
        Some(b"--plugin-host-epoll-idle") => {
            run_theia_ipc_epoll_idle();
        }
        Some(b"--plugin-host-fd4") => {
            run_theia_fd4_hello();
        }
        Some(b"--plugin-host-hold") => {
            run_ipc_channel_hold();
        }
        _ => return false,
    }
}

/// Read from fd 3, echo bytes back, exit.
pub fn run_ipc_channel_echo() -> ! {
    let mut buf = [0u8; 64];
    let n = match syscall::read(CHANNEL_FD, &mut buf) {
        Ok(n) => n,
        Err(_) => syscall::exit(2),
    };
    if n == 0 {
        syscall::exit(3);
    }
    let mut off = 0usize;
    while off < n {
        match syscall::write(CHANNEL_FD, &buf[off..n]) {
            Ok(0) => syscall::exit(4),
            Ok(w) => off += w,
            Err(_) => syscall::exit(5),
        }
    }
    syscall::exit(0);
}

/// Write a handshake, then hold the channel until hangup or kill.
///
/// Node `child_process.fork` / Theia plugin-host send on the IPC socket
/// before the parent writes. A runtime that turns `fork` into an instant
/// zombie (exit 0, EOF on the channel) fails this path.
pub fn run_ipc_channel_hello() -> ! {
    let msg = b"HELLO";
    let mut off = 0usize;
    while off < msg.len() {
        match syscall::write(CHANNEL_FD, &msg[off..]) {
            Ok(0) => syscall::exit(4),
            Ok(w) => off += w,
            Err(_) => syscall::exit(5),
        }
    }
    run_ipc_channel_hold();
}

/// Hold the IPC channel open until the process is killed or the peer hangs up.
pub fn run_ipc_channel_hold() -> ! {
    let mut buf = [0u8; 64];
    loop {
        let mut fds = [poll::PollFd {
            fd: CHANNEL_FD,
            events: syscall::POLLIN | syscall::POLLHUP,
            revents: 0,
        }];
        match syscall::poll(&mut fds, 1000) {
            Ok(0) => continue,
            Ok(_) => {
                if fds[0].revents & syscall::POLLHUP != 0 {
                    syscall::exit(0);
                }
                match syscall::read(CHANNEL_FD, &mut buf) {
                    Ok(0) => syscall::exit(0), // peer closed
                    Ok(_) => continue,         // discard; stay alive
                    Err(_) => syscall::exit(2),
                }
            }
            Err(_) => syscall::exit(3),
        }
    }
}

/// Nested plugin-host analogue: fd 3 is an already-connected socketpair.
///
/// libuv watches that channel with EPOLLIN|EPOLLOUT|EPOLLET. After the first
/// writable edge, `epoll_wait` must sleep — a runtime that keeps returning
/// POLLOUT pegs the nested process (Theia plugin-host at ~300% CPU) and the
/// workbench never finishes restoring the layout.
fn run_theia_ipc_epoll_idle() -> ! {
    let ep = match syscall::epoll_create1(0) {
        Ok(fd) => fd,
        Err(_) => syscall::exit(10),
    };
    let mut ev = epoll::EpollEvent {
        events: EPOLLIN | EPOLLOUT | EPOLLET,
        data: CHANNEL_FD as u64,
    };
    if syscall::epoll_ctl(ep, EPOLL_CTL_ADD, CHANNEL_FD, &mut ev).is_err() {
        syscall::exit(11);
    }
    let mut out = [epoll::EpollEvent { events: 0, data: 0 }; 8];
    for _ in 0..64 {
        match syscall::epoll_wait(ep, &mut out, 0) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => syscall::exit(12),
        }
    }
    let t0 = match syscall::clock_gettime(clock::CLOCK_MONOTONIC) {
        Ok(t) => t,
        Err(_) => syscall::exit(13),
    };
    let n = match syscall::epoll_wait(ep, &mut out, 50) {
        Ok(n) => n,
        Err(_) => syscall::exit(14),
    };
    let t1 = match syscall::clock_gettime(clock::CLOCK_MONOTONIC) {
        Ok(t) => t,
        Err(_) => syscall::exit(15),
    };
    let live = out.iter().take(n).any(|e| {
        e.events & (EPOLLIN | EPOLLOUT) != 0 && e.events & syscall::EPOLLHUP == 0
    });
    let ms = (t1.tv_sec - t0.tv_sec).saturating_mul(1000)
        + (t1.tv_nsec - t0.tv_nsec) / 1_000_000;
    if live || ms < 20 {
        syscall::exit(16);
    }
    let msg = b"IDLEOK";
    let mut off = 0usize;
    while off < msg.len() {
        match syscall::write(CHANNEL_FD, &msg[off..]) {
            Ok(0) => syscall::exit(4),
            Ok(w) => off += w,
            Err(_) => syscall::exit(5),
        }
    }
    run_ipc_channel_hold();
}

/// Theia plugin-host extra stdio slot (`stdio[4]`, BinaryMessagePipe) is a pipe.
fn run_theia_fd4_hello() -> ! {
    let msg = b"FD4OK";
    let mut off = 0usize;
    while off < msg.len() {
        match syscall::write(4, &msg[off..]) {
            Ok(0) => syscall::exit(20),
            Ok(w) => off += w,
            Err(_) => syscall::exit(20),
        }
    }
    loop {
        let req = syscall::Timespec {
            tv_sec: 1,
            tv_nsec: 0,
        };
        let _ = syscall::nanosleep(&req);
    }
}
