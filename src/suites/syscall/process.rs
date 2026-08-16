//! Process-related syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, wait, Errno};

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 reports a child that exited with status 42")]
fn fork_exit_status() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(42);
    }
    let mut status = 0;
    check_eq!(check_ok!(syscall::wait4(pid, &mut status, 0), "wait4"), pid, "pid");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 42, "status");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 reports a child that exited with status 0")]
fn fork_exit_zero() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "status zero");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a child sees getppid equal to the parent's pid")]
fn fork_getppid() -> TestResult {
    let parent = syscall::getpid();
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::getppid() == parent {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    check_eq!(syscall::wexitstatus(status), 0, "ppid mismatch");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getpid returns a positive pid that is stable across two calls")]
fn getpid_stable() -> TestResult {
    let a = syscall::getpid();
    let b = syscall::getpid();
    check!(a > 0, "getpid <= 0");
    check_eq!(a, b, "getpid unstable");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getppid returns a non-negative value")]
fn getppid_non_negative() -> TestResult {
    // PID 1 (typical container entrypoint) has ppid 0.
    check!(syscall::getppid() >= 0, "getppid negative");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getuid equals geteuid in this process")]
fn getuid_euid_match() -> TestResult {
    check_eq!(syscall::getuid(), syscall::geteuid(), "uid != euid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getgid equals getegid in this process")]
fn getgid_egid_match() -> TestResult {
    check_eq!(syscall::getgid(), syscall::getegid(), "gid != egid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 with WNOHANG and no children returns 0 or ECHILD")]
fn wait_nohang_nochild() -> TestResult {
    let mut status = 0;
    match syscall::wait4(-1, &mut status, wait::WNOHANG) {
        Ok(0) => {}
        Err(Errno::ECHILD) => {}
        Ok(pid) => {
            // Reap unexpected child if any.
            let _ = syscall::wait4(pid, &mut status, 0);
            return Err(crate::harness::AssertFail::msg("unexpected child"));
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("wait WNOHANG errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 reaps a child that exited with status 7")]
fn wait4_reap_zombie() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(7);
    }
    let mut status = 0;
    check_eq!(check_ok!(syscall::wait4(pid, &mut status, 0), "wait4"), pid, "pid");
    check_eq!(syscall::wexitstatus(status), 7, "status");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "two successive fork/wait4 pairs report the requested exit codes")]
fn fork_double_child() -> TestResult {
    for code in [11i32, 22] {
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            syscall::exit(code);
        }
        let mut status = 0;
        check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
        check_eq!(syscall::wexitstatus(status), code, "status");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "gettid equals getpid in a single-threaded process")]
fn gettid_equals_pid() -> TestResult {
    check_eq!(syscall::gettid(), syscall::getpid(), "tid != pid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitpid reaps a child that exited with status 19")]
fn waitpid_alias_reap() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(19);
    }
    let mut status = 0;
    check_eq!(
        check_ok!(syscall::waitpid(pid, &mut status, 0), "waitpid"),
        pid,
        "pid"
    );
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 19, "status");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitpid with WUNTRACED still reaps a child that exited normally")]
fn waitpid_wuntraced_soft() -> TestResult {
    // Without stopping the child, WUNTRACED still reaps a normal exit.
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(5);
    }
    let mut status = 0;
    check_ok!(
        syscall::waitpid(pid, &mut status, wait::WUNTRACED),
        "wait WUNTRACED"
    );
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 5, "status");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "waitpid with WNOHANG eventually reaps an exited child")]
fn waitpid_wnohang_after_exit() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(8);
    }
    // Give the child a moment to exit; loop with WNOHANG.
    let mut status = 0;
    let mut got = 0i32;
    for _ in 0..1000 {
        match syscall::waitpid(pid, &mut status, wait::WNOHANG) {
            Ok(0) => {
                let _ = syscall::sched_yield();
            }
            Ok(p) => {
                got = p;
                break;
            }
            Err(Errno::ECHILD) => break,
            Err(_) => return Err(crate::harness::AssertFail::msg("waitpid nohang")),
        }
    }
    check_eq!(got, pid, "reaped");
    check_eq!(syscall::wexitstatus(status), 8, "status");
    Ok(())
}
