//! `fork` + `execve` of a real dynamic `/bin/sh` (and soft `/bin/bash` /
//! `node`) so guests catch loader and CSPRNG failures that bare syscalls miss.
//!
//! Primary failure modes under incomplete guests:
//! - glibc `/bin/sh` exits 127 ("cannot open shared object file")
//! - Node ≥22 aborts (SIGABRT / exit 134) when early CSPRNG self-check fails

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, CloneArgs, Errno, F_OK};

const SIGCHLD: u64 = 17;

fn exec_and_wait_status(path: &[u8], argv: &[*const u8]) -> Result<i32, crate::harness::AssertFail> {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(path, argv, &envp);
        // exec failed — distinguish from shell's conventional 127 where possible.
        syscall::exit(126);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifexited(status), "child not exited");
    Ok(syscall::wexitstatus(status))
}

#[crate::lctp_test(suite = syscall)]
fn sh_exec_exit_zero() -> TestResult {
    // `/bin/sh -c 'exit 0'` must load the dynamic linker and run builtins.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "/bin/sh missing");
    let arg0 = b"sh\0";
    let arg1 = b"-c\0";
    let arg2 = b"exit 0\0";
    let argv = [
        arg0.as_ptr(),
        arg1.as_ptr(),
        arg2.as_ptr(),
        core::ptr::null(),
    ];
    let code = exec_and_wait_status(b"/bin/sh\0", &argv)?;
    check_eq!(code, 0, "sh exit (127 = dynamic loader failure)");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sh_exec_echo_ok() -> TestResult {
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "/bin/sh missing");
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(r);
        if syscall::dup2(w, 1).is_err() {
            syscall::exit(125);
        }
        let _ = syscall::close(w);
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"echo ok\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    check_ok!(syscall::close(w), "close write");
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::read(r, &mut buf), "read");
    check_ok!(syscall::close(r), "close read");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "status");
    // Accept "ok\n" or "ok\r\n".
    check!(n >= 2, "short stdout");
    check_eq!(buf[0], b'o', "o");
    check_eq!(buf[1], b'k', "k");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn sh_test_urandom_chr_soft() -> TestResult {
    // Shell-level probe that `/dev/urandom` is a character device (mirrors `test -c`).
    if syscall::access(b"/bin/sh\0", F_OK).is_err() {
        return Ok(());
    }
    let arg0 = b"sh\0";
    let arg1 = b"-c\0";
    let arg2 = b"test -c /dev/urandom\0";
    let argv = [
        arg0.as_ptr(),
        arg1.as_ptr(),
        arg2.as_ptr(),
        core::ptr::null(),
    ];
    let code = exec_and_wait_status(b"/bin/sh\0", &argv)?;
    check_eq!(code, 0, "test -c /dev/urandom");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn bash_exec_exit_zero_soft() -> TestResult {
    // Debian/Ubuntu images often set SHELL=/bin/bash; catch glibc loader gaps.
    if syscall::access(b"/bin/bash\0", F_OK).is_err() {
        return Ok(());
    }
    let arg0 = b"bash\0";
    let arg1 = b"-c\0";
    let arg2 = b"exit 0\0";
    let argv = [
        arg0.as_ptr(),
        arg1.as_ptr(),
        arg2.as_ptr(),
        core::ptr::null(),
    ];
    let code = exec_and_wait_status(b"/bin/bash\0", &argv)?;
    check_eq!(code, 0, "bash exit (127 = dynamic loader failure)");
    Ok(())
}

fn first_node() -> Option<&'static [u8]> {
    const CANDIDATES: &[&[u8]] = &[
        b"/usr/bin/node\0",
        b"/usr/local/bin/node\0",
        b"/bin/node\0",
    ];
    for &p in CANDIDATES {
        if syscall::access(p, F_OK).is_ok() {
            return Some(p);
        }
    }
    None
}

#[crate::lctp_test(suite = syscall, full)]
fn node_eval_print_soft() -> TestResult {
    // Node ≥22 aborts during `InitializeOncePerProcess` if CSPRNG self-check
    // fails (`ncrypto::CSPRNG` → SIGABRT / exit 134). Soft when node is absent.
    let Some(node) = first_node() else {
        return Ok(());
    };
    let arg0 = b"node\0";
    let arg1 = b"-e\0";
    let arg2 = b"console.log(1)\0";
    let argv = [
        arg0.as_ptr(),
        arg1.as_ptr(),
        arg2.as_ptr(),
        core::ptr::null(),
    ];
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(node, &argv, &envp);
        syscall::exit(127);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    if syscall::wifsignaled(status) {
        let sig = syscall::wtermsig(status);
        return Err(crate::harness::AssertFail::msg(
            if sig == 6 {
                "node aborted (SIGABRT / CSPRNG)"
            } else {
                "node signaled"
            },
        ));
    }
    check!(syscall::wifexited(status), "node not exited");
    let code = syscall::wexitstatus(status);
    // 134 is the shell-visible encoding of SIGABRT on some paths; treat as fail.
    check!(code != 134, "node exit 134 (SIGABRT)");
    check_eq!(code, 0, "node -e");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn clone3_fork_like_soft() -> TestResult {
    // Soft: ENOSYS is acceptable; if implemented, child must exit cleanly.
    let mut args = CloneArgs {
        exit_signal: SIGCHLD,
        ..CloneArgs::default()
    };
    match syscall::clone3(&mut args) {
        Ok(0) => syscall::exit(0),
        Ok(pid) => {
            check!(pid > 0, "pid");
            let mut status = 0;
            check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
            check!(syscall::wifexited(status), "exited");
            check_eq!(syscall::wexitstatus(status), 0, "status");
        }
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("clone3")),
    }
    Ok(())
}
