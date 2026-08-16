//! Dense SIG conformance: sigaction/procmask/pending/kill/signalfd grids.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, sigmask, Errno, SIGALRM, SIGCHLD, SIGHUP, SIGINT, SIGPIPE, SIGTERM, SIGUSR1, SIGUSR2,
    SIG_BLOCK, SIG_DFL, SIG_IGN, SIG_SETMASK, SIG_UNBLOCK,
};

fn discard_pending(sig: i32) -> TestResult {
    check_ok!(syscall::signal_ignore(sig), "ignore");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(sig)), None), "unblock");
    check_ok!(syscall::signal_default(sig), "default");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr1_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr1_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr1_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr1_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr1_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR1, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr1_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR1 and read SIG_IGN back")]
fn sigc_dfl_ign_usr1_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR1 and read SIG_IGN back")]
fn sigc_dfl_ign_usr1_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR1 and read SIG_IGN back")]
fn sigc_dfl_ign_usr1_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR1 and read SIG_IGN back")]
fn sigc_dfl_ign_usr1_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR1, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR1, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR1")]
fn sigc_bu_usr1_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR1")]
fn sigc_bu_usr1_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR1")]
fn sigc_bu_usr1_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR1")]
fn sigc_bu_usr1_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR1")]
fn sigc_bu_usr1_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGUSR1 and restore the previous mask")]
fn sigc_setmask_usr1_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGUSR1)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGUSR1 and restore the previous mask")]
fn sigc_setmask_usr1_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGUSR1)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGUSR1 and restore the previous mask")]
fn sigc_setmask_usr1_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGUSR1)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR2, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr2_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR2, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr2_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR2, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr2_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR2, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr2_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR2, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr2_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGUSR2, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_usr2_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR2 and read SIG_IGN back")]
fn sigc_dfl_ign_usr2_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR2 and read SIG_IGN back")]
fn sigc_dfl_ign_usr2_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR2 and read SIG_IGN back")]
fn sigc_dfl_ign_usr2_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGUSR2 and read SIG_IGN back")]
fn sigc_dfl_ign_usr2_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGUSR2, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGUSR2, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR2")]
fn sigc_bu_usr2_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR2")]
fn sigc_bu_usr2_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR2")]
fn sigc_bu_usr2_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR2")]
fn sigc_bu_usr2_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGUSR2")]
fn sigc_bu_usr2_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR2)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGUSR2 and restore the previous mask")]
fn sigc_setmask_usr2_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGUSR2)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGUSR2 and restore the previous mask")]
fn sigc_setmask_usr2_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGUSR2)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGUSR2 and restore the previous mask")]
fn sigc_setmask_usr2_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGUSR2)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGINT, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_int_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGINT, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_int_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGINT, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_int_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGINT, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_int_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGINT, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_int_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGINT, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_int_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGINT and read SIG_IGN back")]
fn sigc_dfl_ign_int_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGINT and read SIG_IGN back")]
fn sigc_dfl_ign_int_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGINT and read SIG_IGN back")]
fn sigc_dfl_ign_int_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGINT and read SIG_IGN back")]
fn sigc_dfl_ign_int_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGINT, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGINT, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGINT")]
fn sigc_bu_int_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGINT)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGINT)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGINT")]
fn sigc_bu_int_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGINT)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGINT)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGINT")]
fn sigc_bu_int_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGINT)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGINT)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGINT")]
fn sigc_bu_int_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGINT)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGINT)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGINT")]
fn sigc_bu_int_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGINT)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGINT)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGINT and restore the previous mask")]
fn sigc_setmask_int_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGINT)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGINT and restore the previous mask")]
fn sigc_setmask_int_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGINT)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGINT and restore the previous mask")]
fn sigc_setmask_int_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGINT)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGTERM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_term_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGTERM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_term_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGTERM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_term_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGTERM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_term_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGTERM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_term_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGTERM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_term_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGTERM and read SIG_IGN back")]
fn sigc_dfl_ign_term_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGTERM and read SIG_IGN back")]
fn sigc_dfl_ign_term_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGTERM and read SIG_IGN back")]
fn sigc_dfl_ign_term_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGTERM and read SIG_IGN back")]
fn sigc_dfl_ign_term_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGTERM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGTERM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGTERM")]
fn sigc_bu_term_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGTERM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGTERM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGTERM")]
fn sigc_bu_term_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGTERM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGTERM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGTERM")]
fn sigc_bu_term_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGTERM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGTERM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGTERM")]
fn sigc_bu_term_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGTERM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGTERM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGTERM")]
fn sigc_bu_term_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGTERM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGTERM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGTERM and restore the previous mask")]
fn sigc_setmask_term_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGTERM)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGTERM and restore the previous mask")]
fn sigc_setmask_term_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGTERM)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGTERM and restore the previous mask")]
fn sigc_setmask_term_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGTERM)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGCHLD, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_chld_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGCHLD, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_chld_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGCHLD, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_chld_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGCHLD, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_chld_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGCHLD, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_chld_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGCHLD, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_chld_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGCHLD and read SIG_IGN back")]
fn sigc_dfl_ign_chld_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGCHLD and read SIG_IGN back")]
fn sigc_dfl_ign_chld_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGCHLD and read SIG_IGN back")]
fn sigc_dfl_ign_chld_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGCHLD and read SIG_IGN back")]
fn sigc_dfl_ign_chld_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGCHLD, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGCHLD, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGCHLD")]
fn sigc_bu_chld_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGCHLD)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGCHLD)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGCHLD")]
fn sigc_bu_chld_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGCHLD)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGCHLD)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGCHLD")]
fn sigc_bu_chld_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGCHLD)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGCHLD)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGCHLD")]
fn sigc_bu_chld_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGCHLD)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGCHLD)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGCHLD")]
fn sigc_bu_chld_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGCHLD)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGCHLD)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGCHLD and restore the previous mask")]
fn sigc_setmask_chld_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGCHLD)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGCHLD and restore the previous mask")]
fn sigc_setmask_chld_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGCHLD)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGCHLD and restore the previous mask")]
fn sigc_setmask_chld_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGCHLD)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGHUP, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_hup_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGHUP, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_hup_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGHUP, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_hup_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGHUP, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_hup_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGHUP, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_hup_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGHUP, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_hup_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGHUP and read SIG_IGN back")]
fn sigc_dfl_ign_hup_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGHUP and read SIG_IGN back")]
fn sigc_dfl_ign_hup_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGHUP and read SIG_IGN back")]
fn sigc_dfl_ign_hup_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGHUP and read SIG_IGN back")]
fn sigc_dfl_ign_hup_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGHUP, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGHUP, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGHUP")]
fn sigc_bu_hup_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGHUP)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGHUP)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGHUP")]
fn sigc_bu_hup_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGHUP)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGHUP)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGHUP")]
fn sigc_bu_hup_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGHUP)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGHUP)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGHUP")]
fn sigc_bu_hup_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGHUP)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGHUP)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGHUP")]
fn sigc_bu_hup_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGHUP)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGHUP)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGHUP and restore the previous mask")]
fn sigc_setmask_hup_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGHUP)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGHUP and restore the previous mask")]
fn sigc_setmask_hup_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGHUP)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGHUP and restore the previous mask")]
fn sigc_setmask_hup_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGHUP)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGPIPE, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_pipe_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGPIPE, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_pipe_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGPIPE, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_pipe_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGPIPE, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_pipe_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGPIPE, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_pipe_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGPIPE, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_pipe_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGPIPE and read SIG_IGN back")]
fn sigc_dfl_ign_pipe_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGPIPE and read SIG_IGN back")]
fn sigc_dfl_ign_pipe_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGPIPE and read SIG_IGN back")]
fn sigc_dfl_ign_pipe_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGPIPE and read SIG_IGN back")]
fn sigc_dfl_ign_pipe_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGPIPE, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGPIPE, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGPIPE")]
fn sigc_bu_pipe_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGPIPE)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGPIPE)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGPIPE")]
fn sigc_bu_pipe_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGPIPE)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGPIPE)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGPIPE")]
fn sigc_bu_pipe_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGPIPE)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGPIPE)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGPIPE")]
fn sigc_bu_pipe_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGPIPE)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGPIPE)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGPIPE")]
fn sigc_bu_pipe_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGPIPE)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGPIPE)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGPIPE and restore the previous mask")]
fn sigc_setmask_pipe_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGPIPE)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGPIPE and restore the previous mask")]
fn sigc_setmask_pipe_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGPIPE)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGPIPE and restore the previous mask")]
fn sigc_setmask_pipe_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGPIPE)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGALRM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_alrm_1() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGALRM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_alrm_2() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGALRM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_alrm_3() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGALRM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_alrm_4() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGALRM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_alrm_5() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_IGN on SIGALRM, read it back, then restore SIG_DFL")]
fn sigc_ign_dfl_alrm_6() -> TestResult {
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGALRM and read SIG_IGN back")]
fn sigc_dfl_ign_alrm_1() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGALRM and read SIG_IGN back")]
fn sigc_dfl_ign_alrm_2() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGALRM and read SIG_IGN back")]
fn sigc_dfl_ign_alrm_3() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigaction can set SIG_DFL then SIG_IGN on SIGALRM and read SIG_IGN back")]
fn sigc_dfl_ign_alrm_4() -> TestResult {
    let dfl = syscall::Sigaction { sa_handler: SIG_DFL, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "DFL");
    let ign = syscall::Sigaction { sa_handler: SIG_IGN, ..syscall::Sigaction::default() };
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&ign), None), "IGN");
    let mut cur = syscall::Sigaction::default();
    check_ok!(syscall::rt_sigaction(SIGALRM, None, Some(&mut cur)), "query");
    check_eq!(cur.sa_handler, SIG_IGN, "handler");
    check_ok!(syscall::rt_sigaction(SIGALRM, Some(&dfl), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGALRM")]
fn sigc_bu_alrm_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGALRM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGALRM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGALRM")]
fn sigc_bu_alrm_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGALRM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGALRM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGALRM")]
fn sigc_bu_alrm_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGALRM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGALRM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGALRM")]
fn sigc_bu_alrm_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGALRM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGALRM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and then unblock SIGALRM")]
fn sigc_bu_alrm_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGALRM)), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGALRM)), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGALRM and restore the previous mask")]
fn sigc_setmask_alrm_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGALRM)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGALRM and restore the previous mask")]
fn sigc_setmask_alrm_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGALRM)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK can install a mask containing SIGALRM and restore the previous mask")]
fn sigc_setmask_alrm_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(sigmask(SIGALRM)), Some(&mut old)), "set");
    check_ok!(syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None), "restore");
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_6() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_7() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_8() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_9() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_10() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_11() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR1 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr1_12() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR1) != 0, "bit");
    discard_pending(SIGUSR1)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_6() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_7() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_8() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_9() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_10() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_11() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, full, expect = success, case = "a blocked SIGUSR2 sent to self is reported by rt_sigpending")]
fn sigc_pend_usr2_12() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR2)), None), "block");
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR2), "kill");
    let mut pending = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pending), "pending");
    check!(pending & sigmask(SIGUSR2) != 0, "bit");
    discard_pending(SIGUSR2)?;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_1() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_2() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_3() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_4() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_5() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_6() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_7() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_8() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_9() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_10() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_11() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_12() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_13() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_14() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "kill with signal 0 on the calling process succeeds")]
fn sigc_kill_zero_15() -> TestResult {
    check_ok!(syscall::kill(syscall::getpid(), 0), "exists");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_1() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_2() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_3() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_4() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_5() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_6() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_7() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "kill with an invalid signal number returns EINVAL or is otherwise rejected")]
fn sigc_kill_bad_sig_8() -> TestResult {
    match syscall::kill(syscall::getpid(), 999) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected")),
        Err(_) => Ok(()),
    }
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_1() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_2() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_3() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_4() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_5() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_6() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_7() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR1 or rejected as unsupported")]
fn sigc_signalfd_usr1_8() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR1), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_1() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_2() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_3() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_4() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_5() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_6() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_7() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGUSR2 or rejected as unsupported")]
fn sigc_signalfd_usr2_8() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGUSR2), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_1() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_2() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_3() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_4() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_5() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_6() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_7() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "signalfd can be created for SIGTERM or rejected as unsupported")]
fn sigc_signalfd_term_8() -> TestResult {
    match syscall::signalfd(-1, sigmask(SIGTERM), 0) {
        Ok(fd) => { check!(fd >= 0, "fd"); check_ok!(syscall::close(fd), "close"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("signalfd")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_1() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_2() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_3() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_4() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_5() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_6() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_7() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_8() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_9() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask with a null set can query the current mask")]
fn sigc_procmask_query_10() -> TestResult {
    let mut old = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut old)), "query");
    let _ = old;
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_1() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_2() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_3() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_4() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_5() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_6() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_7() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block SIGUSR1 twice and then unblock it")]
fn sigc_double_block_usr1_8() -> TestResult {
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b1");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None), "b2");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None), "u");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_1() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_2() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_3() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_4() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_5() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_6() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_7() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask can block and unblock SIGUSR1 and SIGUSR2 together")]
fn sigc_block_pair_8() -> TestResult {
    let m = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, Some(m), None), "block");
    check_ok!(syscall::rt_sigprocmask(SIG_UNBLOCK, Some(m), None), "unblock");
    Ok(())
}