//! POSIX non-pthread depth2: signal mask/pending, mmap MAP_FIXED soft,
//! clock_nanosleep relative, access/faccessat.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, write_file};
use crate::syscall::{
    self, clock, map, oflag, prot, sigmask, Errno, AT_FDCWD, F_OK, R_OK, SIGINT, SIGTERM, SIGUSR1,
    SIGUSR2, SIG_BLOCK, SIG_DFL, SIG_SETMASK, SIG_UNBLOCK, W_OK, X_OK,
};

const PAGE: usize = 4096;

fn restore_sig(sig: i32) -> TestResult {
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(sig)), None),
        "unblock"
    );
    check_ok!(syscall::signal_default(sig), "dfl");
    Ok(())
}

macro_rules! mask_block_unblock {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix, expect = success, case = concat!("rt_sigprocmask blocks ", stringify!($sig), " and then unblocks it"))]
        fn $name() -> TestResult {
            let mut old = 0u64;
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask($sig)), Some(&mut old)),
                "block"
            );
            let mut cur = 0u64;
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut cur)),
                "query"
            );
            check!(cur & sigmask($sig) != 0, "blocked");
            check_ok!(
                syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask($sig)), None),
                "unblock"
            );
            let mut after = 0u64;
            check_ok!(
                syscall::rt_sigprocmask(SIG_SETMASK, Some(old), Some(&mut after)),
                "restore"
            );
            Ok(())
        }
    };
}

mask_block_unblock!(d2_mask_usr1, SIGUSR1);
mask_block_unblock!(d2_mask_usr2, SIGUSR2);
mask_block_unblock!(d2_mask_term, SIGTERM);
mask_block_unblock!(d2_mask_int, SIGINT);

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask blocks SIGUSR1 and SIGUSR2 together and SIG_SETMASK restores the old mask")]
fn d2_mask_block_two_then_setmask() -> TestResult {
    let mut old = 0u64;
    let set = sigmask(SIGUSR1) | sigmask(SIGUSR2);
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(set), Some(&mut old)),
        "block"
    );
    let mut cur = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut cur)),
        "q"
    );
    check!(cur & sigmask(SIGUSR1) != 0, "u1");
    check!(cur & sigmask(SIGUSR2) != 0, "u2");
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None),
        "restore"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigprocmask SIG_SETMASK of an empty mask clears the process signal mask")]
fn d2_mask_setmask_empty() -> TestResult {
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, Some(0), Some(&mut old)),
        "clear"
    );
    let mut cur = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut cur)),
        "q"
    );
    check_eq!(cur, 0, "empty");
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None),
        "restore"
    );
    Ok(())
}

macro_rules! pending_after_kill_self {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix, expect = success, case = concat!("a blocked ", stringify!($sig), " sent to the process appears in rt_sigpending"))]
        fn $name() -> TestResult {
            check_ok!(syscall::signal_ignore($sig), "ign");
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask($sig)), None),
                "block"
            );
            check_ok!(syscall::kill(syscall::getpid(), $sig), "kill");
            let mut pend = 0u64;
            check_ok!(syscall::rt_sigpending(&mut pend), "pending");
            check!(pend & sigmask($sig) != 0, "pending bit");
            // Drain by unblocking while ignored.
            check_ok!(
                syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask($sig)), None),
                "un"
            );
            restore_sig($sig)?;
            Ok(())
        }
    };
}

pending_after_kill_self!(d2_pend_usr1, SIGUSR1);
pending_after_kill_self!(d2_pend_usr2, SIGUSR2);
pending_after_kill_self!(d2_pend_term, SIGTERM);
pending_after_kill_self!(d2_pend_int, SIGINT);

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigpending no longer reports SIGUSR1 after the ignored signal is unblocked")]
fn d2_pending_clears_after_unblock_ignored() -> TestResult {
    check_ok!(syscall::signal_ignore(SIGUSR1), "ign");
    check_ok!(
        syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask(SIGUSR1)), None),
        "block"
    );
    check_ok!(syscall::kill(syscall::getpid(), SIGUSR1), "kill");
    check_ok!(
        syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask(SIGUSR1)), None),
        "un"
    );
    let mut pend = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pend), "pend");
    check!(pend & sigmask(SIGUSR1) == 0, "cleared");
    restore_sig(SIGUSR1)?;
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "rt_sigpending reports no SIGUSR1, SIGUSR2, SIGTERM, or SIGINT when those are at default")]
fn d2_pending_empty_default() -> TestResult {
    let mut old = 0u64;
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, Some(0), Some(&mut old)),
        "clear mask"
    );
    // Ensure common signals ignored then default so nothing pending from us.
    for s in [SIGUSR1, SIGUSR2, SIGTERM, SIGINT] {
        let _ = syscall::signal_ignore(s);
        let _ = syscall::signal_default(s);
    }
    let mut pend = 0u64;
    check_ok!(syscall::rt_sigpending(&mut pend), "pend");
    check!(
        pend & (sigmask(SIGUSR1) | sigmask(SIGUSR2) | sigmask(SIGTERM) | sigmask(SIGINT)) == 0,
        "no pend"
    );
    check_ok!(
        syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None),
        "restore"
    );
    Ok(())
}

macro_rules! mmap_fixed_soft {
    ($name:ident, $prot:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = concat!("mmap() with MAP_FIXED and ", stringify!($prot), " succeeds or is rejected as unsupported"))]
        fn $name() -> TestResult {
            let base = check_ok!(
                syscall::mmap(
                    0,
                    PAGE * 2,
                    prot::PROT_READ | prot::PROT_WRITE,
                    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
                    -1,
                    0
                ),
                "base"
            );
            let target = base + PAGE;
            match syscall::mmap(
                target,
                PAGE,
                $prot,
                map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_FIXED,
                -1,
                0,
            ) {
                Ok(addr) => {
                    check_eq!(addr, target, "fixed");
                    check_ok!(syscall::munmap(addr, PAGE), "un");
                }
                Err(e)
                    if matches!(
                        e,
                        Errno::EINVAL | Errno::ENOMEM | Errno::EPERM | Errno::EACCES
                    ) => {}
                Err(_) => {
                    let _ = syscall::munmap(base, PAGE * 2);
                    return Err(crate::harness::AssertFail::msg("MAP_FIXED"));
                }
            }
            let _ = syscall::munmap(base, PAGE * 2);
            Ok(())
        }
    };
}

mmap_fixed_soft!(d2_mmap_fixed_none, prot::PROT_NONE);
mmap_fixed_soft!(d2_mmap_fixed_r, prot::PROT_READ);
mmap_fixed_soft!(d2_mmap_fixed_rw, prot::PROT_READ | prot::PROT_WRITE);
mmap_fixed_soft!(d2_mmap_fixed_rx, prot::PROT_READ | prot::PROT_EXEC);

#[crate::lctp_test(suite = posix, expect = soft, case = "mmap(MAP_FIXED) over an existing anonymous mapping succeeds or is rejected as unsupported")]
fn d2_mmap_fixed_overwrite_anon_soft() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "m1"
    );
    unsafe {
        *(addr as *mut u8) = 0xAB;
    }
    match syscall::mmap(
        addr,
        PAGE,
        prot::PROT_READ | prot::PROT_WRITE,
        map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_FIXED,
        -1,
        0,
    ) {
        Ok(a) => {
            check_eq!(a, addr, "same");
            let _ = unsafe { *(a as *const u8) };
            check_ok!(syscall::munmap(a, PAGE), "un");
        }
        Err(e) if matches!(e, Errno::EINVAL | Errno::ENOMEM | Errno::EPERM) => {
            check_ok!(syscall::munmap(addr, PAGE), "un");
        }
        Err(_) => {
            let _ = syscall::munmap(addr, PAGE);
            return Err(crate::harness::AssertFail::msg("fixed"));
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mmap(MAP_FIXED) of a file over an anonymous mapping succeeds or is rejected as unsupported")]
fn d2_mmap_fixed_file_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"m", 0o644), "c");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "t");
    let base = check_ok!(
        syscall::mmap(
            0,
            PAGE * 2,
            prot::PROT_READ,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "base"
    );
    let target = base + PAGE;
    match syscall::mmap(
        target,
        PAGE,
        prot::PROT_READ,
        map::MAP_PRIVATE | map::MAP_FIXED,
        fd,
        0,
    ) {
        Ok(a) => {
            check_eq!(a, target, "fixed");
            check_ok!(syscall::munmap(a, PAGE), "un");
        }
        Err(e) if matches!(e, Errno::EINVAL | Errno::ENOMEM | Errno::EPERM | Errno::ENODEV) => {}
        Err(_) => {
            let _ = syscall::munmap(base, PAGE * 2);
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("file fixed"));
        }
    }
    let _ = syscall::munmap(base, PAGE * 2);
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

macro_rules! cnsleep_rel {
    ($name:ident, $clk:expr, $nsec:expr) => {
        #[crate::lctp_test(suite = posix, expect = success, case = concat!("clock_nanosleep() on ", stringify!($clk), " for ", stringify!($nsec), " ns succeeds"))]
        fn $name() -> TestResult {
            let req = syscall::Timespec {
                tv_sec: 0,
                tv_nsec: $nsec,
            };
            check_ok!(syscall::clock_nanosleep($clk, 0, &req), "sleep");
            Ok(())
        }
    };
}

cnsleep_rel!(d2_cns_mono_1ms, clock::CLOCK_MONOTONIC, 1_000_000);
cnsleep_rel!(d2_cns_mono_2ms, clock::CLOCK_MONOTONIC, 2_000_000);
cnsleep_rel!(d2_cns_mono_5ms, clock::CLOCK_MONOTONIC, 5_000_000);
cnsleep_rel!(d2_cns_mono_10ms, clock::CLOCK_MONOTONIC, 10_000_000);
cnsleep_rel!(d2_cns_rt_1ms, clock::CLOCK_REALTIME, 1_000_000);
cnsleep_rel!(d2_cns_rt_5ms, clock::CLOCK_REALTIME, 5_000_000);

#[crate::lctp_test(suite = posix, expect = soft, case = "clock_nanosleep(CLOCK_MONOTONIC_COARSE) for 1 ms succeeds or is rejected as unsupported")]
fn d2_cns_mono_coarse_1ms_soft() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    match syscall::clock_nanosleep(clock::CLOCK_MONOTONIC_COARSE, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) | Err(Errno::EOPNOTSUPP) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("cns coarse")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "clock_nanosleep(CLOCK_MONOTONIC) with a zero relative timeout succeeds")]
fn d2_cns_zero_relative() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    check_ok!(
        syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req),
        "zero"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_nanosleep() with tv_nsec equal to 1e9 returns EINVAL")]
fn d2_cns_bad_nsec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000_000,
    };
    check_err!(
        syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req),
        Errno::EINVAL,
        "einval"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "clock_nanosleep() with a negative tv_nsec returns EINVAL")]
fn d2_cns_neg_nsec_einval() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: -1,
    };
    check_err!(
        syscall::clock_nanosleep(clock::CLOCK_MONOTONIC, 0, &req),
        Errno::EINVAL,
        "einval"
    );
    Ok(())
}

macro_rules! access_ok {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix, expect = success, case = concat!("access() succeeds after chmod ", stringify!($chmod), " for mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $chmod), "chmod");
            check_ok!(syscall::access(&path, $mode), "access");
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

access_ok!(d2_acc_f_ok, 0o000, F_OK);
access_ok!(d2_acc_r_400, 0o400, R_OK);
access_ok!(d2_acc_r_440, 0o440, R_OK);
access_ok!(d2_acc_r_444, 0o444, R_OK);
access_ok!(d2_acc_r_600, 0o600, R_OK);
access_ok!(d2_acc_r_644, 0o644, R_OK);
access_ok!(d2_acc_w_200, 0o200, W_OK);
access_ok!(d2_acc_w_600, 0o600, W_OK);
access_ok!(d2_acc_w_620, 0o620, W_OK);
access_ok!(d2_acc_w_666, 0o666, W_OK);
access_ok!(d2_acc_x_100, 0o100, X_OK);
access_ok!(d2_acc_x_500, 0o500, X_OK);
access_ok!(d2_acc_x_700, 0o700, X_OK);
access_ok!(d2_acc_x_755, 0o755, X_OK);
access_ok!(d2_acc_rw_600, 0o600, R_OK | W_OK);
access_ok!(d2_acc_rw_660, 0o660, R_OK | W_OK);
access_ok!(d2_acc_rx_500, 0o500, R_OK | X_OK);
access_ok!(d2_acc_rx_550, 0o550, R_OK | X_OK);
access_ok!(d2_acc_wx_300, 0o300, W_OK | X_OK);
access_ok!(d2_acc_rwx_700, 0o700, R_OK | W_OK | X_OK);
access_ok!(d2_acc_rwx_755, 0o755, R_OK | W_OK | X_OK);
access_ok!(d2_acc_rwx_777, 0o777, R_OK | W_OK | X_OK);

macro_rules! access_eacces {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix, expect = failure, case = concat!("access() after chmod ", stringify!($chmod), " for mode ", stringify!($mode), " returns EACCES"))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $chmod), "chmod");
            check_err!(syscall::access(&path, $mode), Errno::EACCES, "eacces");
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

access_eacces!(d2_acc_e_r_000, 0o000, R_OK);
access_eacces!(d2_acc_e_r_200, 0o200, R_OK);
access_eacces!(d2_acc_e_w_000, 0o000, W_OK);
access_eacces!(d2_acc_e_w_400, 0o400, W_OK);
access_eacces!(d2_acc_e_x_000, 0o000, X_OK);
access_eacces!(d2_acc_e_x_600, 0o600, X_OK);
access_eacces!(d2_acc_e_rw_400, 0o400, R_OK | W_OK);
access_eacces!(d2_acc_e_rx_600, 0o600, R_OK | X_OK);
access_eacces!(d2_acc_e_wx_400, 0o400, W_OK | X_OK);
access_eacces!(d2_acc_e_rwx_000, 0o000, R_OK | W_OK | X_OK);

macro_rules! faccessat_ok {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix, expect = success, case = concat!("faccessat() succeeds after chmod ", stringify!($chmod), " for mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $chmod), "chmod");
            check_ok!(syscall::faccessat(AT_FDCWD, &path, $mode, 0), "fa");
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

faccessat_ok!(d2_fa_f_ok, 0o000, F_OK);
faccessat_ok!(d2_fa_r_400, 0o400, R_OK);
faccessat_ok!(d2_fa_w_200, 0o200, W_OK);
faccessat_ok!(d2_fa_x_100, 0o100, X_OK);
faccessat_ok!(d2_fa_rw_600, 0o600, R_OK | W_OK);
faccessat_ok!(d2_fa_rx_500, 0o500, R_OK | X_OK);
faccessat_ok!(d2_fa_wx_300, 0o300, W_OK | X_OK);
faccessat_ok!(d2_fa_rwx_700, 0o700, R_OK | W_OK | X_OK);
faccessat_ok!(d2_fa_r_644, 0o644, R_OK);
faccessat_ok!(d2_fa_w_666, 0o666, W_OK);
faccessat_ok!(d2_fa_x_755, 0o755, X_OK);
faccessat_ok!(d2_fa_rwx_777, 0o777, R_OK | W_OK | X_OK);

macro_rules! faccessat_eacces {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix, expect = failure, case = concat!("faccessat() after chmod ", stringify!($chmod), " for mode ", stringify!($mode), " returns EACCES"))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $chmod), "chmod");
            check_err!(
                syscall::faccessat(AT_FDCWD, &path, $mode, 0),
                Errno::EACCES,
                "eacces"
            );
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

faccessat_eacces!(d2_fa_e_r_000, 0o000, R_OK);
faccessat_eacces!(d2_fa_e_w_000, 0o000, W_OK);
faccessat_eacces!(d2_fa_e_x_000, 0o000, X_OK);
faccessat_eacces!(d2_fa_e_r_200, 0o200, R_OK);
faccessat_eacces!(d2_fa_e_w_400, 0o400, W_OK);
faccessat_eacces!(d2_fa_e_x_600, 0o600, X_OK);
faccessat_eacces!(d2_fa_e_rw_200, 0o200, R_OK | W_OK);
faccessat_eacces!(d2_fa_e_rx_600, 0o600, R_OK | X_OK);

#[crate::lctp_test(suite = posix, expect = success, case = "faccessat() with F_OK and R_OK succeeds for a file relative to a directory fd")]
fn d2_faccessat_dirfd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let _ = create_empty(&mut tmp, b"f")?;
    let dirfd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "dir"
    );
    check_ok!(syscall::faccessat(dirfd, b"f\0", F_OK, 0), "fa");
    check_ok!(syscall::faccessat(dirfd, b"f\0", R_OK, 0), "r");
    check_ok!(syscall::close(dirfd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "faccessat() of a path that does not exist returns ENOENT")]
fn d2_faccessat_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"nope")?;
    check_err!(
        syscall::faccessat(AT_FDCWD, &p, F_OK, 0),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(X_OK) and access(R_OK|X_OK) succeed on a searchable directory")]
fn d2_access_dir_search() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::access(&d, X_OK), "x");
    check_ok!(syscall::access(&d, R_OK | X_OK), "rx");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "access(X_OK) of a directory without search permission returns EACCES")]
fn d2_access_dir_no_search_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&d, 0o600), "chmod");
    check_err!(syscall::access(&d, X_OK), Errno::EACCES, "eacces");
    check_ok!(syscall::chmod(&d, 0o755), "restore");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access() on a symlink follows the target so R_OK succeeds and W_OK returns EACCES")]
fn d2_access_symlink_follow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let f = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&f, 0o400), "chmod");
    let l = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"f\0", &l), "sym");
    check_ok!(syscall::access(&l, R_OK), "r");
    check_err!(syscall::access(&l, W_OK), Errno::EACCES, "w");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mmap(MAP_FIXED) onto the middle of an anonymous mapping succeeds or is rejected as unsupported")]
fn d2_mmap_anon_then_fixed_neighbor() -> TestResult {
    let a = check_ok!(
        syscall::mmap(
            0,
            PAGE * 4,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "a"
    );
    let mid = a + PAGE;
    match syscall::mmap(
        mid,
        PAGE,
        prot::PROT_READ,
        map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_FIXED,
        -1,
        0,
    ) {
        Ok(x) => {
            check_eq!(x, mid, "mid");
            check_ok!(syscall::munmap(x, PAGE), "un mid");
        }
        Err(e) if matches!(e, Errno::EINVAL | Errno::ENOMEM | Errno::EPERM) => {}
        Err(_) => {
            let _ = syscall::munmap(a, PAGE * 4);
            return Err(crate::harness::AssertFail::msg("fixed mid"));
        }
    }
    check_ok!(syscall::munmap(a, PAGE * 4), "un a");
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "clock_nanosleep(CLOCK_REALTIME_COARSE) for 1 ms succeeds or is rejected as unsupported")]
fn d2_cns_realtime_coarse_soft() -> TestResult {
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    };
    match syscall::clock_nanosleep(clock::CLOCK_REALTIME_COARSE, 0, &req) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) | Err(Errno::EOPNOTSUPP) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("cns coarse")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "two successive rt_sigprocmask queries return the same signal mask")]
fn d2_signal_mask_query_idempotent() -> TestResult {
    let mut a = 0u64;
    let mut b = 0u64;
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut a)), "a");
    check_ok!(syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut b)), "b");
    check_eq!(a, b, "same");
    let _ = SIG_DFL;
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = failure, case = "access(F_OK) of a path that does not exist returns ENOENT")]
fn d2_access_missing_enoent() -> TestResult {
    check_err!(
        syscall::access(b"/tmp/lctp-posix-d2-missing\0", F_OK),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = success, case = "access(R_OK|W_OK) succeeds on a newly written regular file")]
fn d2_write_file_then_access_rw() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"w")?;
    write_file(&p, b"hi")?;
    check_ok!(syscall::access(&p, R_OK | W_OK), "rw");
    Ok(())
}
