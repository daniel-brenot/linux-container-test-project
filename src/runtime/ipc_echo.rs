//! Early-process helper modes invoked before the test harness.
//!
//! - `--ipc-channel-echo` — read fd 3, write back, exit
//! - `--ipc-channel-hold` — keep fd 3 open and block until killed (reload/teardown)
//! - `--ipc-channel-hello` — write `HELLO` on fd 3 immediately, then hold
//!   (Theia/Node plugin-host shape: child speaks first on the IPC channel)

use crate::syscall;
use crate::syscall::poll;

const CHANNEL_FD: i32 = 3;

unsafe fn argv1<'a>(argc: usize, argv: *const usize) -> Option<&'a [u8]> {
    if argc < 2 {
        return None;
    }
    let ptr = *argv.add(1) as *const u8;
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

/// # Safety
/// `argv` must be a valid argc-length argv vector.
pub unsafe fn dispatch_helper(argc: usize, argv: *const usize) -> bool {
    match argv1(argc, argv) {
        Some(b"--ipc-channel-echo") => {
            run_ipc_channel_echo();
        }
        Some(b"--ipc-channel-hold") => {
            run_ipc_channel_hold();
        }
        Some(b"--ipc-channel-hello") => {
            run_ipc_channel_hello();
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
