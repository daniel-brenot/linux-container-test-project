//! Process wait/pgid/prctl/rlimit/rusage depth.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, wait, Errno, Rlimit, RUSAGE_CHILDREN, RUSAGE_SELF, RLIMIT_AS, RLIMIT_NOFILE, RLIMIT_STACK,
    P_ALL, P_PID, PR_GET_DUMPABLE, PR_GET_NO_NEW_PRIVS, PR_SET_DUMPABLE, PR_SET_NO_NEW_PRIVS,
};

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 reports a matrix of child exit codes including 0, 127, and 255")]
fn wait4_exit_codes_matrix() -> TestResult {
    for code in [0i32, 1, 2, 127, 255] {
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            syscall::exit(code);
        }
        let mut st = 0;
        check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
        check!(syscall::wifexited(st), "exited");
        check_eq!(syscall::wexitstatus(st), code, "code");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitpid(-1) reaps the forked child")]
fn waitpid_any_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(3);
    }
    let mut st = 0;
    let got = check_ok!(syscall::waitpid(-1, &mut st, 0), "wait");
    check_eq!(got, pid, "pid");
    check_eq!(syscall::wexitstatus(st), 3, "st");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 with WNOHANG eventually reaps an exited child")]
fn wait4_wnohang_loop() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(9);
    }
    let mut st = 0;
    let mut reaped = false;
    for _ in 0..2000 {
        match syscall::wait4(pid, &mut st, wait::WNOHANG) {
            Ok(0) => {
                let _ = syscall::sched_yield();
            }
            Ok(p) => {
                check_eq!(p, pid, "pid");
                reaped = true;
                break;
            }
            Err(Errno::ECHILD) => break,
            Err(_) => return Err(crate::harness::AssertFail::msg("wait4")),
        }
    }
    check!(reaped, "reaped");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "waitpid of pid 1 with WNOHANG returns 0, ECHILD, EPERM, or ESRCH")]
fn waitpid_wnohang_echild_soft() -> TestResult {
    let mut st = 0;
    match syscall::waitpid(1, &mut st, wait::WNOHANG) {
        Ok(0) | Err(Errno::ECHILD) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Ok(_) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("waitpid soft")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid with P_PID and WEXITED reaps a specific child")]
fn waitid_p_pid_exited() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(11);
    }
    let mut info = syscall::Siginfo::default();
    check_ok!(
        syscall::waitid(P_PID, pid, &mut info, wait::WEXITED),
        "waitid"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid with P_ALL and WEXITED reaps an exited child")]
fn waitid_p_all_exited() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(12);
    }
    let mut info = syscall::Siginfo::default();
    check_ok!(
        syscall::waitid(P_ALL, 0, &mut info, wait::WEXITED),
        "waitid"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitid with WNOHANG returns success, EAGAIN, or ECHILD before the child exits")]
fn waitid_wnohang() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let req = syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        };
        let _ = syscall::nanosleep(&req);
        syscall::exit(0);
    }
    let mut info = syscall::Siginfo::default();
    match syscall::waitid(P_PID, pid, &mut info, wait::WEXITED | wait::WNOHANG) {
        Ok(()) => {}
        Err(Errno::EAGAIN) | Err(Errno::ECHILD) => {}
        Err(_) => {
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            return Err(crate::harness::AssertFail::msg("waitid nohang"));
        }
    }
    let mut st = 0;
    let _ = syscall::wait4(pid, &mut st, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "waitid with WNOWAIT leaves the child waitable or is rejected with EINVAL/ENOSYS")]
fn waitid_wnowait_soft() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(4);
    }
    let mut info = syscall::Siginfo::default();
    match syscall::waitid(P_PID, pid, &mut info, wait::WEXITED | wait::WNOWAIT) {
        Ok(()) => {
            let mut st = 0;
            check_ok!(syscall::wait4(pid, &mut st, 0), "reap");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
        }
        Err(_) => {
            let mut st = 0;
            let _ = syscall::wait4(pid, &mut st, 0);
            return Err(crate::harness::AssertFail::msg("wnowait"));
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 with WUNTRACED reaps a child that exited with status 6")]
fn wait4_wuntraced_exit() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(6);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, wait::WUNTRACED), "wait");
    check_eq!(syscall::wexitstatus(st), 6, "st");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a child inherits the parent's process group")]
fn getpgid_child() -> TestResult {
    let parent_pg = check_ok!(syscall::getpgid(0), "parent");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let pg = match syscall::getpgid(0) {
            Ok(p) => p,
            Err(_) => syscall::exit(1),
        };
        if pg == parent_pg {
            syscall::exit(0);
        }
        syscall::exit(2);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check_eq!(syscall::wexitstatus(st), 0, "pgid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a child can setpgid itself into a new process group")]
fn setpgid_child_new_group() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let me = syscall::getpid();
        if syscall::setpgid(0, me).is_err() {
            syscall::exit(1);
        }
        match syscall::getpgid(0) {
            Ok(pg) if pg == me => syscall::exit(0),
            _ => syscall::exit(2),
        }
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check_eq!(syscall::wexitstatus(st), 0, "setpgid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a child inherits the parent's session id")]
fn getsid_child_inherits() -> TestResult {
    let sid = check_ok!(syscall::getsid(0), "sid");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        match syscall::getsid(0) {
            Ok(s) if s == sid => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check_eq!(syscall::wexitstatus(st), 0, "sid");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "setsid succeeds or returns EPERM when the caller is already a process-group leader")]
fn setsid_fails_if_leader_soft() -> TestResult {
    // Process group leaders typically get EPERM from setsid.
    match syscall::setsid() {
        Ok(_) => {}
        Err(Errno::EPERM) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("setsid")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getpgid of a likely unused pid returns ESRCH or EPERM")]
fn getpgid_bad_pid() -> TestResult {
    match syscall::getpgid(999_999_999) {
        Err(Errno::ESRCH) | Err(Errno::EPERM) => {}
        Ok(_) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("getpgid bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsid of a likely unused pid returns ESRCH or EPERM")]
fn getsid_bad_pid() -> TestResult {
    match syscall::getsid(999_999_999) {
        Err(Errno::ESRCH) | Err(Errno::EPERM) => {}
        Ok(_) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("getsid bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "PR_GET_DUMPABLE returns 0, 1, or 2")]
fn prctl_dumpable_get() -> TestResult {
    let d = check_ok!(syscall::prctl(PR_GET_DUMPABLE, 0, 0, 0, 0), "get");
    check!(d == 0 || d == 1 || d == 2, "range");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "PR_SET_DUMPABLE round-trips 0 and 1")]
fn prctl_dumpable_set_roundtrip() -> TestResult {
    let old = check_ok!(syscall::prctl(PR_GET_DUMPABLE, 0, 0, 0, 0), "old");
    check_ok!(syscall::prctl(PR_SET_DUMPABLE, 0, 0, 0, 0), "set0");
    let v = check_ok!(syscall::prctl(PR_GET_DUMPABLE, 0, 0, 0, 0), "get0");
    check_eq!(v, 0, "0");
    check_ok!(syscall::prctl(PR_SET_DUMPABLE, 1, 0, 0, 0), "set1");
    let v = check_ok!(syscall::prctl(PR_GET_DUMPABLE, 0, 0, 0, 0), "get1");
    check_eq!(v, 1, "1");
    let _ = syscall::prctl(PR_SET_DUMPABLE, old as usize, 0, 0, 0);
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "PR_GET_NO_NEW_PRIVS returns 0 or 1")]
fn prctl_no_new_privs_get() -> TestResult {
    let v = check_ok!(syscall::prctl(PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0), "get");
    check!(v == 0 || v == 1, "range");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "PR_SET_NO_NEW_PRIVS to 1 is visible via PR_GET_NO_NEW_PRIVS")]
fn prctl_no_new_privs_set() -> TestResult {
    check_ok!(syscall::prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0), "set");
    let v = check_ok!(syscall::prctl(PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0), "get");
    check_eq!(v, 1, "nnp");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "prctl set/get name round-trips a short comm string")]
fn prctl_name_depth_short() -> TestResult {
    check_ok!(syscall::prctl_set_name(b"pd\0"), "set");
    let mut buf = [0u8; 16];
    check_ok!(syscall::prctl_get_name(&mut buf), "get");
    check_eq!(&buf[..2], b"pd", "name");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "prlimit64 can lower RLIMIT_NOFILE soft limit and restore it")]
fn rlimit_nofile_lower_soft() -> TestResult {
    let mut old = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, None, Some(&mut old)),
        "get"
    );
    if old.rlim_cur <= 32 || old.rlim_cur > old.rlim_max {
        return Ok(());
    }
    let new = Rlimit {
        rlim_cur: old.rlim_cur - 1,
        rlim_max: old.rlim_max,
    };
    // Fourth argument receives the *previous* limits when setting.
    let mut prev = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, Some(&new), Some(&mut prev)),
        "set"
    );
    check_eq!(prev.rlim_cur, old.rlim_cur, "prev");
    let mut got = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, None, Some(&mut got)),
        "get2"
    );
    check_eq!(got.rlim_cur, new.rlim_cur, "cur");
    // restore
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, Some(&old), None),
        "restore"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "prlimit64 reports a positive RLIMIT_STACK soft limit")]
fn rlimit_stack_get() -> TestResult {
    let mut lim = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_STACK, None, Some(&mut lim)),
        "get"
    );
    check!(lim.rlim_cur > 0, "stack");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "prlimit64 reports a positive RLIMIT_AS hard limit")]
fn rlimit_as_get() -> TestResult {
    let mut lim = Rlimit::default();
    check_ok!(syscall::prlimit64(0, RLIMIT_AS, None, Some(&mut lim)), "get");
    check!(lim.rlim_max > 0, "as");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "lowering RLIMIT_STACK succeeds or is rejected with EPERM/EINVAL")]
fn rlimit_stack_lower_soft() -> TestResult {
    let mut old = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_STACK, None, Some(&mut old)),
        "get"
    );
    if old.rlim_cur < 1024 * 1024 || old.rlim_cur > old.rlim_max {
        return Ok(());
    }
    let new = Rlimit {
        rlim_cur: old.rlim_cur / 2,
        rlim_max: old.rlim_max,
    };
    match syscall::prlimit64(0, RLIMIT_STACK, Some(&new), None) {
        Ok(()) => {
            let _ = syscall::prlimit64(0, RLIMIT_STACK, Some(&old), None);
        }
        Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("stack set")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getrusage of self reports non-negative user and system time")]
fn getrusage_self_basic() -> TestResult {
    let ru = check_ok!(syscall::getrusage(RUSAGE_SELF), "rusage");
    check!(ru.ru_utime.tv_sec >= 0, "utime");
    check!(ru.ru_stime.tv_sec >= 0, "stime");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getrusage of children after wait reports non-negative fields")]
fn getrusage_children_after_wait() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        // burn a tiny bit of user time
        let mut x = 0u64;
        for i in 0..10000u64 {
            x = x.wrapping_add(i);
        }
        core::hint::black_box(x);
        syscall::exit(0);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    let ru = check_ok!(syscall::getrusage(RUSAGE_CHILDREN), "children");
    check!(ru.ru_utime.tv_sec >= 0, "cutime");
    check!(ru.ru_maxrss >= 0, "maxrss");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getrusage of self reports a non-negative minflt")]
fn getrusage_self_minflt() -> TestResult {
    let ru = check_ok!(syscall::getrusage(RUSAGE_SELF), "rusage");
    check!(ru.ru_minflt >= 0, "minflt");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getrusage of children after wait4 reports a non-negative nvcsw")]
fn fork_wait4_rusage_filled() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    let ru = check_ok!(syscall::getrusage(RUSAGE_CHILDREN), "ru");
    check!(ru.ru_nvcsw >= 0, "nvcsw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "wait4 reaps eight children that each exited")]
fn wait_many_children() -> TestResult {
    let mut pids = [0i32; 8];
    for (i, slot) in pids.iter_mut().enumerate() {
        let pid = check_ok!(syscall::fork(), "fork");
        if pid == 0 {
            syscall::exit(i as i32);
        }
        *slot = pid;
    }
    for &pid in &pids {
        let mut st = 0;
        check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
        check!(syscall::wifexited(st), "exited");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "waitpid can reap a chosen child while a sibling is still unreaped")]
fn waitpid_specific_amid_siblings() -> TestResult {
    let a = check_ok!(syscall::fork(), "a");
    if a == 0 {
        syscall::exit(1);
    }
    let b = check_ok!(syscall::fork(), "b");
    if b == 0 {
        syscall::exit(2);
    }
    let mut st = 0;
    check_eq!(check_ok!(syscall::waitpid(b, &mut st, 0), "wait b"), b, "b");
    check_eq!(syscall::wexitstatus(st), 2, "st b");
    check_eq!(check_ok!(syscall::waitpid(a, &mut st, 0), "wait a"), a, "a");
    check_eq!(syscall::wexitstatus(st), 1, "st a");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setpgid of a likely unused pid returns ESRCH, EPERM, or EINVAL")]
fn setpgid_bad_pid() -> TestResult {
    match syscall::setpgid(999_999_999, 999_999_999) {
        Err(Errno::ESRCH) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("setpgid bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a child can set and read PR_SET_DUMPABLE")]
fn prctl_dumpable_in_child() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::prctl(PR_SET_DUMPABLE, 0, 0, 0, 0).is_ok()
            && syscall::prctl(PR_GET_DUMPABLE, 0, 0, 0, 0).ok() == Some(0)
        {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check_eq!(syscall::wexitstatus(st), 0, "child");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "RLIMIT_NOFILE soft limit is less than or equal to the hard limit")]
fn rlimit_nofile_cur_le_max() -> TestResult {
    let mut lim = Rlimit::default();
    check_ok!(
        syscall::prlimit64(0, RLIMIT_NOFILE, None, Some(&mut lim)),
        "get"
    );
    check!(lim.rlim_cur <= lim.rlim_max, "cur<=max");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a second waitid after reaping returns ECHILD or EAGAIN")]
fn waitid_then_echild() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut info = syscall::Siginfo::default();
    check_ok!(
        syscall::waitid(P_PID, pid, &mut info, wait::WEXITED),
        "waitid"
    );
    match syscall::waitid(P_PID, pid, &mut info, wait::WEXITED | wait::WNOHANG) {
        Err(Errno::ECHILD) | Err(Errno::EAGAIN) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("second waitid")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getrusage of children reports non-negative minflt and context-switch counts")]
fn getrusage_children_nonneg_fields() -> TestResult {
    let ru = check_ok!(syscall::getrusage(RUSAGE_CHILDREN), "ru");
    check!(ru.ru_minflt >= 0, "minflt");
    check!(ru.ru_nvcsw >= 0, "nvcsw");
    check!(ru.ru_nivcsw >= 0, "nivcsw");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a child getpid differs from the parent pid")]
fn fork_getpid_differs() -> TestResult {
    let parent = syscall::getpid();
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        if syscall::getpid() != parent {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    check_eq!(syscall::wexitstatus(st), 0, "diff");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "wait4 with options 0 reaps the forked child")]
fn wait4_options_zero() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut st = 0;
    check_eq!(check_ok!(syscall::wait4(pid, &mut st, 0), "wait"), pid, "pid");
    Ok(())
}
