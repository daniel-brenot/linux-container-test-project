//! sysinfo, getrusage, and times tests.

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, RUSAGE_CHILDREN, RUSAGE_SELF};

#[crate::lctp_test(suite = syscall)]
fn sysinfo_uptime_positive() -> TestResult {
    let info = check_ok!(syscall::sysinfo(), "sysinfo");
    check!(info.uptime > 0, "uptime");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysinfo_totalram_positive() -> TestResult {
    let info = check_ok!(syscall::sysinfo(), "sysinfo");
    check!(info.totalram > 0, "totalram");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysinfo_freeram_le_total() -> TestResult {
    let info = check_ok!(syscall::sysinfo(), "sysinfo");
    check!(info.freeram <= info.totalram, "free <= total");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysinfo_procs_positive() -> TestResult {
    let info = check_ok!(syscall::sysinfo(), "sysinfo");
    check!(info.procs > 0, "procs");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getrusage_self() -> TestResult {
    let ru = check_ok!(syscall::getrusage(RUSAGE_SELF), "getrusage");
    check!(ru.ru_utime.tv_sec >= 0, "utime sec");
    check!(ru.ru_stime.tv_sec >= 0, "stime sec");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn getrusage_self_non_negative_faults() -> TestResult {
    let ru = check_ok!(syscall::getrusage(RUSAGE_SELF), "getrusage");
    check!(ru.ru_minflt >= 0, "minflt");
    check!(ru.ru_majflt >= 0, "majflt");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn getrusage_children_after_wait() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait4");
    let ru = check_ok!(syscall::getrusage(RUSAGE_CHILDREN), "getrusage children");
    check!(ru.ru_utime.tv_sec >= 0 || ru.ru_stime.tv_sec >= 0, "child time");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn times_self() -> TestResult {
    let t = check_ok!(syscall::times(), "times");
    check!(t.tms_utime >= 0, "utime");
    check!(t.tms_stime >= 0, "stime");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn times_cutime_cstime() -> TestResult {
    let t = check_ok!(syscall::times(), "times");
    check!(t.tms_cutime >= 0, "cutime");
    check!(t.tms_cstime >= 0, "cstime");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn times_after_work() -> TestResult {
    let t1 = check_ok!(syscall::times(), "times1");
    for _ in 0..1000 {
        let _ = syscall::getpid();
    }
    let t2 = check_ok!(syscall::times(), "times2");
    check!(t2.tms_utime >= t1.tms_utime, "utime grew");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysinfo_mem_unit() -> TestResult {
    let info = check_ok!(syscall::sysinfo(), "sysinfo");
    check!(info.mem_unit >= 1, "mem_unit");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn sysinfo_loads_non_negative() -> TestResult {
    let info = check_ok!(syscall::sysinfo(), "sysinfo");
    for l in info.loads {
        // Kernel stores fixed-point; just ensure readable values.
        check!(l < u64::MAX / 2, "load");
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn getrusage_inblock_oublock() -> TestResult {
    let ru = check_ok!(syscall::getrusage(RUSAGE_SELF), "getrusage");
    check!(ru.ru_inblock >= 0, "inblock");
    check!(ru.ru_oublock >= 0, "oublock");
    Ok(())
}
