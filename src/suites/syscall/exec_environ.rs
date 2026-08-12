//! `execve` environ integrity and inherited IPC "channel fd" spawns.
//!
//! Covers the Linux surface behind nested helper processes that inherit an IPC
//! socket (fd number published via environ) and must not leave the parent's
//! environ unusable after the child image is torn down. Also stresses large
//! environ blocks and failed-then-successful spawn sequences.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, oflag, poll, wait, AF_UNIX, F_OK, POLLIN, SOCK_STREAM};

fn wait_exit_bounded(pid: i32) -> Result<i32, crate::harness::AssertFail> {
    let mut status = 0;
    for _ in 0..200 {
        match syscall::wait4(pid, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid => {
                check!(syscall::wifexited(status), "exited");
                return Ok(syscall::wexitstatus(status));
            }
            Ok(_) | Err(syscall::Errno::ECHILD) => {}
            Err(_) => return Err(crate::harness::AssertFail::msg("wait4")),
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    let _ = syscall::kill(pid, 9);
    let _ = syscall::wait4(pid, &mut status, 0);
    Err(crate::harness::AssertFail::msg("child hung"))
}

fn self_exe(buf: &mut [u8; 256]) -> Result<usize, crate::harness::AssertFail> {
    let n = check_ok!(
        syscall::readlink(b"/proc/self/exe\0", buf),
        "readlink /proc/self/exe"
    );
    check!(n > 0 && n < buf.len(), "exe path len");
    buf[n] = 0;
    Ok(n + 1)
}

fn u32_to_dec(mut n: u32, out: &mut [u8]) -> usize {
    if n == 0 {
        out[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 10];
    let mut i = 0usize;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in 0..i {
        out[j] = tmp[i - 1 - j];
    }
    i
}

fn buf_has(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return false;
    }
    for i in 0..=hay.len() - needle.len() {
        if &hay[i..i + needle.len()] == needle {
            return true;
        }
    }
    false
}

#[crate::lctp_test(suite = syscall)]
fn exec_environ_var_roundtrip() -> TestResult {
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let (out_r, out_w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(out_r);
        if syscall::dup2(out_w, 1).is_err() || syscall::dup2(out_w, 2).is_err() {
            syscall::exit(125);
        }
        let _ = syscall::close(out_w);
        // Keep strings in a local buffer (same pattern as the large-environ case).
        let mut arena = [0u8; 128];
        arena[..21].copy_from_slice(b"LCTP_MARK=environ_ok\0");
        arena[21..40].copy_from_slice(b"PATH=/usr/bin:/bin\0");
        let envp = [arena.as_ptr(), arena[21..].as_ptr(), core::ptr::null()];
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"echo \"$LCTP_MARK\"\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(out_w);
    let mut buf = [0u8; 64];
    let mut n = 0usize;
    let mut status = 0i32;
    let mut reaped = false;
    for _ in 0..80 {
        if !reaped {
            if let Ok(p) = syscall::wait4(pid, &mut status, wait::WNOHANG) {
                if p == pid {
                    reaped = true;
                }
            }
        }
        match syscall::read(out_r, &mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => n += k,
            Err(syscall::Errno::EAGAIN) | Err(syscall::Errno::EINTR) => {}
            Err(_) => break,
        }
        if buf_has(&buf[..n], b"environ_ok") {
            break;
        }
        if reaped {
            while n < buf.len() {
                match syscall::read(out_r, &mut buf[n..]) {
                    Ok(0) | Err(_) => break,
                    Ok(k) => n += k,
                }
            }
            break;
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 25_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    let _ = syscall::close(out_r);
    if !reaped {
        let code = wait_exit_bounded(pid)?;
        check_eq!(code, 0, "status");
    } else {
        check!(syscall::wifexited(status), "exited");
        check_eq!(syscall::wexitstatus(status), 0, "status");
    }
    check!(buf_has(&buf[..n], b"environ_ok"), "value");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn exec_environ_many_entries() -> TestResult {
    // Large environ exercises guest reinstall / allocation of the env block.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let mut arena = [0u8; 12_288];
    let mut ptrs = [core::ptr::null::<u8>(); 96];
    let mut off = 0usize;
    let mut count = 0usize;
    for i in 0u32..80 {
        // LCTP_E<i>=v<i>
        let mut key = [0u8; 32];
        key[0..6].copy_from_slice(b"LCTP_E");
        let mut d = [0u8; 10];
        let nd = u32_to_dec(i, &mut d);
        key[6..6 + nd].copy_from_slice(&d[..nd]);
        key[6 + nd] = b'=';
        key[7 + nd] = b'v';
        let mut d2 = [0u8; 10];
        let nd2 = u32_to_dec(i, &mut d2);
        key[8 + nd..8 + nd + nd2].copy_from_slice(&d2[..nd2]);
        let len = 8 + nd + nd2; // without NUL yet
        if off + len + 1 > arena.len() {
            break;
        }
        arena[off..off + len].copy_from_slice(&key[..len]);
        arena[off + len] = 0;
        ptrs[count] = arena[off..].as_ptr();
        off += len + 1;
        count += 1;
    }
    // Probe var in the middle.
    let path_env = b"PATH=/usr/bin:/bin\0";
    if off + path_env.len() <= arena.len() {
        arena[off..off + path_env.len()].copy_from_slice(path_env);
        ptrs[count] = arena[off..].as_ptr();
        off += path_env.len();
        count += 1;
    }
    let probe = b"echo -n \"$LCTP_E42\"\0";
    let (out_r, out_w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(out_r);
        if syscall::dup2(out_w, 1).is_err() {
            syscall::exit(125);
        }
        let _ = syscall::close(out_w);
        ptrs[count] = core::ptr::null();
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            probe.as_ptr(),
            core::ptr::null(),
        ];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &ptrs[..count + 1]);
        syscall::exit(127);
    }
    let _ = syscall::close(out_w);
    let mut buf = [0u8; 32];
    let mut n = 0usize;
    for _ in 0..40 {
        match syscall::read(out_r, &mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => n += k,
            Err(_) => break,
        }
        if n >= 3 {
            break;
        }
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 25_000_000,
        };
        let _ = syscall::nanosleep(&req);
    }
    let _ = syscall::close(out_r);
    let code = wait_exit_bounded(pid)?;
    check_eq!(code, 0, "status");
    check!(n >= 3 && &buf[..3] == b"v42", "LCTP_E42");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn exec_fail_then_environ_ok() -> TestResult {
    // A failed spawn must not leave subsequent execve environ unusable.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let pid1 = check_ok!(syscall::fork(), "fork1");
    if pid1 == 0 {
        let envp = [core::ptr::null::<u8>()];
        let arg0 = b"missing\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(b"/tmp/lctp-no-such-helper\0", &argv, &envp);
        syscall::exit(127);
    }
    let c1 = wait_exit_bounded(pid1)?;
    check!(c1 != 0, "missing should fail");

    let (out_r, out_w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid2 = check_ok!(syscall::fork(), "fork2");
    if pid2 == 0 {
        let _ = syscall::close(out_r);
        if syscall::dup2(out_w, 1).is_err() {
            syscall::exit(125);
        }
        let _ = syscall::close(out_w);
        let env0 = b"LCTP_AFTER=alive\0";
        let env1 = b"PATH=/usr/bin:/bin\0";
        let envp = [env0.as_ptr(), env1.as_ptr(), core::ptr::null()];
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"echo -n \"$LCTP_AFTER\"\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(out_w);
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::read(out_r, &mut buf), "read");
    let _ = syscall::close(out_r);
    let c2 = wait_exit_bounded(pid2)?;
    check_eq!(c2, 0, "second spawn");
    check!(n >= 5 && &buf[..5] == b"alive", "environ after fail");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn exec_channel_fd_socketpair_shell() -> TestResult {
    // Publish an inherited IPC fd via environ (`CHANNEL_FD=3`) and speak on it.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(a);
        if syscall::dup2(b, 3).is_err() {
            syscall::exit(125);
        }
        if b != 3 {
            let _ = syscall::close(b);
        }
        let env0 = b"CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        // Read a line from the channel fd and write it back on the same fd.
        let arg2 = b"IFS= read -r line <&3; printf %s \"$line\" >&3\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    check_ok!(syscall::write(a, b"ping\n"), "write");
    let mut fds = [poll::PollFd {
        fd: a,
        events: POLLIN,
        revents: 0,
    }];
    let pr = check_ok!(syscall::poll(&mut fds, 3000), "poll");
    check!(pr >= 1, "readable");
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::read(a, &mut buf), "read");
    let _ = syscall::close(a);
    let code = wait_exit_bounded(pid)?;
    check_eq!(code, 0, "shell status");
    check!(n >= 4 && &buf[..4] == b"ping", "channel echo");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn exec_nested_same_image_channel_fd() -> TestResult {
    // Nested exec of *this* ET_EXEC with an IPC socket on fd 3 — same shape as
    // a helper process sharing the parent's load address with a channel fd.
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(a);
        if syscall::dup2(b, 3).is_err() {
            syscall::exit(125);
        }
        if b != 3 {
            let _ = syscall::close(b);
        }
        // Intentionally non-CLOEXEC channel; close other end already done.
        let mut arena = [0u8; 4096];
        let mut ptrs = [core::ptr::null::<u8>(); 40];
        let mut off = 0usize;
        let mut count = 0usize;
        let fixed = b"CHANNEL_FD=3\0";
        arena[off..off + fixed.len()].copy_from_slice(fixed);
        ptrs[count] = arena[off..].as_ptr();
        off += fixed.len();
        count += 1;
        for i in 0u32..24 {
            let mut e = [0u8; 24];
            e[0..7].copy_from_slice(b"LCTP_N=");
            let mut d = [0u8; 10];
            let nd = u32_to_dec(i, &mut d);
            e[7..7 + nd].copy_from_slice(&d[..nd]);
            let elen = 7 + nd;
            e[elen] = 0;
            if off + elen + 1 > arena.len() {
                break;
            }
            arena[off..off + elen + 1].copy_from_slice(&e[..elen + 1]);
            ptrs[count] = arena[off..].as_ptr();
            off += elen + 1;
            count += 1;
        }
        ptrs[count] = core::ptr::null();
        let arg0 = exe[..exe_len].as_ptr(); // path as argv0-ish; real argv0 below
        let mut arg0buf = [0u8; 256];
        arg0buf[..exe_len].copy_from_slice(&exe[..exe_len]);
        let flag = b"--ipc-channel-echo\0";
        let argv = [arg0buf.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = arg0;
        let _ = syscall::execve(&exe[..exe_len], &argv, &ptrs[..count + 1]);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    check_ok!(syscall::write(a, b"nest-ok"), "write");
    let mut fds = [poll::PollFd {
        fd: a,
        events: POLLIN,
        revents: 0,
    }];
    check!(check_ok!(syscall::poll(&mut fds, 5000), "poll") >= 1, "rdy");
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::read(a, &mut buf), "read");
    let _ = syscall::close(a);
    let code = wait_exit_bounded(pid)?;
    check_eq!(code, 0, "nested helper");
    check_eq!(n, 7, "len");
    check_eq!(&buf[..7], b"nest-ok", "payload");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn exec_nested_channel_then_parent_environ_spawn() -> TestResult {
    // After a nested same-image IPC helper returns, parent must still be able
    // to spawn a child with a fresh environ (getenv / env block still valid).
    let mut exe = [0u8; 256];
    let exe_len = self_exe(&mut exe)?;
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "sp");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(a);
        if syscall::dup2(b, 3).is_err() {
            syscall::exit(125);
        }
        if b != 3 {
            let _ = syscall::close(b);
        }
        let env0 = b"CHANNEL_FD=3\0";
        let envp = [env0.as_ptr(), core::ptr::null()];
        let mut arg0buf = [0u8; 256];
        arg0buf[..exe_len].copy_from_slice(&exe[..exe_len]);
        let flag = b"--ipc-channel-echo\0";
        let argv = [arg0buf.as_ptr(), flag.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(&exe[..exe_len], &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(b);
    let _ = syscall::write(a, b"x");
    let mut tmp = [0u8; 8];
    let mut fds = [poll::PollFd {
        fd: a,
        events: POLLIN,
        revents: 0,
    }];
    let _ = syscall::poll(&mut fds, 5000);
    let _ = syscall::read(a, &mut tmp);
    let _ = syscall::close(a);
    let code = wait_exit_bounded(pid)?;
    check_eq!(code, 0, "helper");

    // Parent continues: environ-carrying spawn must work.
    let (out_r, out_w) = check_ok!(syscall::pipe2(oflag::O_CLOEXEC), "pipe");
    let pid2 = check_ok!(syscall::fork(), "fork2");
    if pid2 == 0 {
        let _ = syscall::close(out_r);
        if syscall::dup2(out_w, 1).is_err() {
            syscall::exit(125);
        }
        let _ = syscall::close(out_w);
        let env0 = b"LCTP_PARENT=restored\0";
        let env1 = b"PATH=/usr/bin:/bin\0";
        let envp = [env0.as_ptr(), env1.as_ptr(), core::ptr::null()];
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"echo -n \"$LCTP_PARENT\"\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(out_w);
    let mut buf = [0u8; 32];
    let n = check_ok!(syscall::read(out_r, &mut buf), "read");
    let _ = syscall::close(out_r);
    let c2 = wait_exit_bounded(pid2)?;
    check_eq!(c2, 0, "parent spawn");
    check!(n >= 8 && &buf[..8] == b"restored", "parent environ");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn exec_empty_environ_ok() -> TestResult {
    let true_path: &[u8] = if syscall::access(b"/bin/true\0", F_OK).is_ok() {
        b"/bin/true\0"
    } else if syscall::access(b"/usr/bin/true\0", F_OK).is_ok() {
        b"/usr/bin/true\0"
    } else {
        return Ok(());
    };
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let envp = [core::ptr::null::<u8>()];
        let arg0 = b"true\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let _ = syscall::execve(true_path, &argv, &envp);
        syscall::exit(127);
    }
    let code = wait_exit_bounded(pid)?;
    check_eq!(code, 0, "empty env");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn exec_repeated_spawns_environ_stable() -> TestResult {
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    for i in 0u32..8 {
        let (out_r, out_w) = check_ok!(syscall::pipe2(0), "pipe");
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            let _ = syscall::close(out_r);
            if syscall::dup2(out_w, 1).is_err() {
                syscall::exit(125);
            }
            let _ = syscall::close(out_w);
            let mut env = [0u8; 32];
            env[..9].copy_from_slice(b"LCTP_SEQ=");
            let mut d = [0u8; 10];
            let nd = u32_to_dec(i, &mut d);
            env[9..9 + nd].copy_from_slice(&d[..nd]);
            env[9 + nd] = 0;
            let envp = [env.as_ptr(), core::ptr::null()];
            let arg0 = b"sh\0";
            let arg1 = b"-c\0";
            let arg2 = b"echo -n \"$LCTP_SEQ\"\0";
            let argv = [
                arg0.as_ptr(),
                arg1.as_ptr(),
                arg2.as_ptr(),
                core::ptr::null(),
            ];
            let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
            syscall::exit(127);
        }
        let _ = syscall::close(out_w);
        let mut buf = [0u8; 16];
        let n = check_ok!(syscall::read(out_r, &mut buf), "read");
        let _ = syscall::close(out_r);
        let code = wait_exit_bounded(pid)?;
        check_eq!(code, 0, "seq status");
        let mut expect = [0u8; 10];
        let ne = u32_to_dec(i, &mut expect);
        check_eq!(n, ne, "seq len");
        check_eq!(&buf[..ne], &expect[..ne], "seq val");
    }
    Ok(())
}
