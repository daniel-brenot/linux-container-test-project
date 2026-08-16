//! `fork` + `execve` of real ELF interpreters from the guest rootfs.
//!
//! Bare syscall probes cannot catch dynamic-linker failures (exit 127:
//! "cannot open shared object file") or early runtime aborts during init.
//! Soft cases only run when the binary is present on the image.

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
        syscall::exit(126);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifexited(status), "child not exited");
    Ok(syscall::wexitstatus(status))
}

fn exec_wait_full(path: &[u8], argv: &[*const u8]) -> Result<i32, crate::harness::AssertFail> {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(path, argv, &envp);
        syscall::exit(127);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    if syscall::wifsignaled(status) {
        return Err(crate::harness::AssertFail::msg("runtime signaled"));
    }
    check!(syscall::wifexited(status), "not exited");
    Ok(syscall::wexitstatus(status))
}

#[crate::lctp_test(suite = syscall, expect = success, case = "execve of /bin/sh -c 'exit 0' waits with exit status 0")]
fn sh_exec_exit_zero() -> TestResult {
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

#[crate::lctp_test(suite = syscall, expect = success, case = "execve of /bin/sh -c 'echo ok' writes ok to a redirected stdout pipe")]
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
    check!(n >= 2, "short stdout");
    check_eq!(buf[0], b'o', "o");
    check_eq!(buf[1], b'k', "k");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "/bin/sh -c 'test -c /dev/urandom' exits 0, or /bin/sh is absent")]
fn sh_test_urandom_chr_soft() -> TestResult {
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

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "execve of /bin/bash -c 'exit 0' exits 0, or bash is absent")]
fn bash_exec_exit_zero_soft() -> TestResult {
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

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "execve of true exits 0, or the true binary is absent")]
fn true_exec_exit_zero_soft() -> TestResult {
    // Busybox/coreutils `true` is a tiny dynamically linked (or applet) binary.
    for path in [b"/usr/bin/true\0" as &[u8], b"/bin/true\0"] {
        if syscall::access(path, F_OK).is_err() {
            continue;
        }
        let arg0 = b"true\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let code = exec_and_wait_status(path, &argv)?;
        check_eq!(code, 0, "true");
        return Ok(());
    }
    Ok(())
}

struct SoftInterp {
    path: &'static [u8],
    argv: [&'static [u8]; 3],
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "an installed python, perl, or node one-liner exits 0, or none of those interpreters are present")]
fn soft_interpreter_eval_exit_zero() -> TestResult {
    // Soft: if a common interpreter is installed, a one-liner must exit 0
    // (covers early runtime init / crypto / loader failures as SIGABRT or 134).
    const CASES: &[SoftInterp] = &[
        SoftInterp {
            path: b"/usr/bin/python3\0",
            argv: [b"python3\0", b"-c\0", b"print(1)\0"],
        },
        SoftInterp {
            path: b"/usr/bin/python\0",
            argv: [b"python\0", b"-c\0", b"print(1)\0"],
        },
        SoftInterp {
            path: b"/usr/bin/perl\0",
            argv: [b"perl\0", b"-e\0", b"print 1\0"],
        },
        SoftInterp {
            path: b"/usr/bin/node\0",
            argv: [b"node\0", b"-e\0", b"console.log(1)\0"],
        },
        SoftInterp {
            path: b"/usr/local/bin/node\0",
            argv: [b"node\0", b"-e\0", b"console.log(1)\0"],
        },
    ];
    let mut ran = false;
    for case in CASES {
        if syscall::access(case.path, F_OK).is_err() {
            continue;
        }
        ran = true;
        let argv = [
            case.argv[0].as_ptr(),
            case.argv[1].as_ptr(),
            case.argv[2].as_ptr(),
            core::ptr::null(),
        ];
        let code = exec_wait_full(case.path, &argv)?;
        check!(code != 134, "exit 134 (SIGABRT)");
        check_eq!(code, 0, "interp eval");
    }
    let _ = ran;
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "clone3 with SIGCHLD creates a child that exits 0, or is rejected as unsupported")]
fn clone3_fork_like_soft() -> TestResult {
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
