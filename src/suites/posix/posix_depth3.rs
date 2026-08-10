//! POSIX non-pthread depth3: errno grids, path .. /. //, access/faccessat,
//! clock_nanosleep, mmap SHARED/PRIVATE, signal IGN/DFL/block.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, write_file};
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

macro_rules! errno_bad_fd {
    ($name:ident, $call:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            check_err!($call, Errno::EBADF, "ebadf");
            Ok(())
        }
    };
}

errno_bad_fd!(d3_ebadf_close, syscall::close(-1));
errno_bad_fd!(d3_ebadf_fsync, syscall::fsync(-1));
errno_bad_fd!(d3_ebadf_fdatasync, syscall::fdatasync(-1));
errno_bad_fd!(d3_ebadf_fstat, syscall::fstat(-1));

#[crate::lctp_test(suite = posix)]
fn d3_ebadf_read() -> TestResult {
    let mut b = [0u8; 4];
    check_err!(syscall::read(-1, &mut b), Errno::EBADF, "ebadf");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_ebadf_write() -> TestResult {
    check_err!(syscall::write(-1, b"x"), Errno::EBADF, "ebadf");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_ebadf_lseek() -> TestResult {
    check_err!(syscall::lseek(-1, 0, 0), Errno::EBADF, "ebadf");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_enoent_open() -> TestResult {
    check_err!(
        syscall::open(b"/tmp/lctp-posix-d3-missing\0", oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_enoent_stat() -> TestResult {
    check_err!(
        syscall::stat(b"/tmp/lctp-posix-d3-missing\0"),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_enoent_unlink() -> TestResult {
    check_err!(
        syscall::unlink(b"/tmp/lctp-posix-d3-missing\0"),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_enoent_access() -> TestResult {
    check_err!(
        syscall::access(b"/tmp/lctp-posix-d3-missing\0", F_OK),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_enotdir_open_file_as_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::open(&p, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_eisdir_write_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    match syscall::open(&d, oflag::O_WRONLY, 0) {
        Err(Errno::EISDIR) => {}
        Ok(fd) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("expected eisdir"));
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("open dir wronly")),
    }
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

fn chdir_restore(saved: &[u8], n: usize) -> TestResult {
    check_ok!(syscall::chdir(&saved[..n]), "restore cwd");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_path_dot() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let mut cwd = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut cwd), "cwd");
    check_ok!(syscall::chdir(&d), "chdir");
    let st = check_ok!(syscall::stat(b".\0"), "stat");
    check!(st.is_dir(), "dir");
    chdir_restore(&cwd, n)?;
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_path_dotdot() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let mut cwd = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut cwd), "cwd");
    check_ok!(syscall::chdir(&d), "chdir");
    let st = check_ok!(syscall::stat(b"..\0"), "stat");
    check!(st.is_dir(), "dir");
    chdir_restore(&cwd, n)?;
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_path_slash_slash() -> TestResult {
    // //tmp should resolve like /tmp on Linux
    match syscall::stat(b"//tmp\0") {
        Ok(st) => check!(st.is_dir(), "dir"),
        Err(Errno::ENOENT) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("//tmp")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_path_dot_slash() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let _ = create_empty(&mut tmp, b"f")?;
    let mut cwd = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut cwd), "cwd");
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    check_ok!(syscall::stat(b"./f\0"), "stat");
    chdir_restore(&cwd, n)?;
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_path_dotdot_slash() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let sub = create_dir(&mut tmp, b"sub", 0o755)?;
    let _ = create_empty(&mut tmp, b"f")?;
    let mut cwd = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut cwd), "cwd");
    check_ok!(syscall::chdir(&sub), "chdir");
    check_ok!(syscall::stat(b"../f\0"), "stat");
    chdir_restore(&cwd, n)?;
    check_ok!(syscall::rmdir(&sub), "rm");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_path_multi_slash() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let mut child = [0u8; 160];
    // build d///x style via join then rewrite — just open d/x normally
    let p = join_path(&d, b"x", &mut child)?;
    write_file(p, b"z")?;
    let mut slashy = [0u8; 180];
    let blen = d.iter().position(|&c| c == 0).unwrap();
    slashy[..blen].copy_from_slice(&d[..blen]);
    slashy[blen..blen + 4].copy_from_slice(b"///x");
    slashy[blen + 4] = 0;
    check_ok!(
        syscall::stat(crate::suites::common::truncate_cstr(&slashy)),
        "stat"
    );
    check_ok!(syscall::unlink(p), "ul");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

macro_rules! access_ok {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix)]
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

access_ok!(d3_acc_f_000, 0o000, F_OK);
access_ok!(d3_acc_r_400, 0o400, R_OK);
access_ok!(d3_acc_r_440, 0o440, R_OK);
access_ok!(d3_acc_r_444, 0o444, R_OK);
access_ok!(d3_acc_r_500, 0o500, R_OK);
access_ok!(d3_acc_r_540, 0o540, R_OK);
access_ok!(d3_acc_r_600, 0o600, R_OK);
access_ok!(d3_acc_r_640, 0o640, R_OK);
access_ok!(d3_acc_r_644, 0o644, R_OK);
access_ok!(d3_acc_r_700, 0o700, R_OK);
access_ok!(d3_acc_w_200, 0o200, W_OK);
access_ok!(d3_acc_w_220, 0o220, W_OK);
access_ok!(d3_acc_w_600, 0o600, W_OK);
access_ok!(d3_acc_w_620, 0o620, W_OK);
access_ok!(d3_acc_w_660, 0o660, W_OK);
access_ok!(d3_acc_w_700, 0o700, W_OK);
access_ok!(d3_acc_x_100, 0o100, X_OK);
access_ok!(d3_acc_x_500, 0o500, X_OK);
access_ok!(d3_acc_x_700, 0o700, X_OK);
access_ok!(d3_acc_x_711, 0o711, X_OK);
access_ok!(d3_acc_x_755, 0o755, X_OK);
access_ok!(d3_acc_rw_600, 0o600, R_OK | W_OK);
access_ok!(d3_acc_rw_660, 0o660, R_OK | W_OK);
access_ok!(d3_acc_rx_500, 0o500, R_OK | X_OK);
access_ok!(d3_acc_rx_550, 0o550, R_OK | X_OK);
access_ok!(d3_acc_wx_300, 0o300, W_OK | X_OK);
access_ok!(d3_acc_rwx_700, 0o700, R_OK | W_OK | X_OK);
access_ok!(d3_acc_rwx_755, 0o755, R_OK | W_OK | X_OK);
access_ok!(d3_acc_rwx_777, 0o777, R_OK | W_OK | X_OK);

macro_rules! access_eacces {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix)]
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

access_eacces!(d3_e_r_000, 0o000, R_OK);
access_eacces!(d3_e_r_200, 0o200, R_OK);
access_eacces!(d3_e_r_300, 0o300, R_OK);
access_eacces!(d3_e_w_000, 0o000, W_OK);
access_eacces!(d3_e_w_400, 0o400, W_OK);
access_eacces!(d3_e_w_500, 0o500, W_OK);
access_eacces!(d3_e_x_000, 0o000, X_OK);
access_eacces!(d3_e_x_600, 0o600, X_OK);
access_eacces!(d3_e_x_200, 0o200, X_OK);
access_eacces!(d3_e_rw_400, 0o400, R_OK | W_OK);
access_eacces!(d3_e_rw_200, 0o200, R_OK | W_OK);
access_eacces!(d3_e_rx_600, 0o600, R_OK | X_OK);
access_eacces!(d3_e_wx_400, 0o400, W_OK | X_OK);
access_eacces!(d3_e_rwx_000, 0o000, R_OK | W_OK | X_OK);

macro_rules! faccessat_ok {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix)]
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

faccessat_ok!(d3_fa_f_000, 0o000, F_OK);
faccessat_ok!(d3_fa_r_400, 0o400, R_OK);
faccessat_ok!(d3_fa_r_644, 0o644, R_OK);
faccessat_ok!(d3_fa_w_200, 0o200, W_OK);
faccessat_ok!(d3_fa_w_666, 0o666, W_OK);
faccessat_ok!(d3_fa_x_100, 0o100, X_OK);
faccessat_ok!(d3_fa_x_755, 0o755, X_OK);
faccessat_ok!(d3_fa_rw_600, 0o600, R_OK | W_OK);
faccessat_ok!(d3_fa_rx_500, 0o500, R_OK | X_OK);
faccessat_ok!(d3_fa_wx_300, 0o300, W_OK | X_OK);
faccessat_ok!(d3_fa_rwx_700, 0o700, R_OK | W_OK | X_OK);
faccessat_ok!(d3_fa_rwx_777, 0o777, R_OK | W_OK | X_OK);

macro_rules! faccessat_eacces {
    ($name:ident, $chmod:expr, $mode:expr) => {
        #[crate::lctp_test(suite = posix)]
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

faccessat_eacces!(d3_fae_r_000, 0o000, R_OK);
faccessat_eacces!(d3_fae_w_000, 0o000, W_OK);
faccessat_eacces!(d3_fae_x_000, 0o000, X_OK);
faccessat_eacces!(d3_fae_r_200, 0o200, R_OK);
faccessat_eacces!(d3_fae_w_400, 0o400, W_OK);
faccessat_eacces!(d3_fae_x_600, 0o600, X_OK);
faccessat_eacces!(d3_fae_rw_200, 0o200, R_OK | W_OK);
faccessat_eacces!(d3_fae_rx_600, 0o600, R_OK | X_OK);

macro_rules! cnsleep {
    ($name:ident, $clk:expr, $nsec:expr) => {
        #[crate::lctp_test(suite = posix)]
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

cnsleep!(d3_cns_mono_100us, clock::CLOCK_MONOTONIC, 100_000);
cnsleep!(d3_cns_mono_250us, clock::CLOCK_MONOTONIC, 250_000);
cnsleep!(d3_cns_mono_500us, clock::CLOCK_MONOTONIC, 500_000);
cnsleep!(d3_cns_mono_1ms, clock::CLOCK_MONOTONIC, 1_000_000);
cnsleep!(d3_cns_mono_2ms, clock::CLOCK_MONOTONIC, 2_000_000);
cnsleep!(d3_cns_mono_3ms, clock::CLOCK_MONOTONIC, 3_000_000);
cnsleep!(d3_cns_mono_4ms, clock::CLOCK_MONOTONIC, 4_000_000);
cnsleep!(d3_cns_mono_5ms, clock::CLOCK_MONOTONIC, 5_000_000);
cnsleep!(d3_cns_mono_8ms, clock::CLOCK_MONOTONIC, 8_000_000);
cnsleep!(d3_cns_mono_12ms, clock::CLOCK_MONOTONIC, 12_000_000);
cnsleep!(d3_cns_rt_100us, clock::CLOCK_REALTIME, 100_000);
cnsleep!(d3_cns_rt_500us, clock::CLOCK_REALTIME, 500_000);
cnsleep!(d3_cns_rt_1ms, clock::CLOCK_REALTIME, 1_000_000);
cnsleep!(d3_cns_rt_2ms, clock::CLOCK_REALTIME, 2_000_000);
cnsleep!(d3_cns_rt_5ms, clock::CLOCK_REALTIME, 5_000_000);
cnsleep!(d3_cns_rt_10ms, clock::CLOCK_REALTIME, 10_000_000);

#[crate::lctp_test(suite = posix)]
fn d3_cns_zero() -> TestResult {
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

#[crate::lctp_test(suite = posix)]
fn d3_cns_bad_nsec() -> TestResult {
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

#[crate::lctp_test(suite = posix)]
fn d3_cns_neg_nsec() -> TestResult {
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

macro_rules! mmap_anon {
    ($name:ident, $flags:expr, $prot:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let addr = check_ok!(
                syscall::mmap(0, PAGE, $prot, $flags, -1, 0),
                "mmap"
            );
            check_ok!(syscall::munmap(addr, PAGE), "un");
            Ok(())
        }
    };
}

mmap_anon!(
    d3_mmap_priv_none,
    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
    prot::PROT_NONE
);
mmap_anon!(
    d3_mmap_priv_r,
    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
    prot::PROT_READ
);
mmap_anon!(
    d3_mmap_priv_w,
    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
    prot::PROT_WRITE
);
mmap_anon!(
    d3_mmap_priv_rw,
    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
    prot::PROT_READ | prot::PROT_WRITE
);
mmap_anon!(
    d3_mmap_priv_rx,
    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
    prot::PROT_READ | prot::PROT_EXEC
);
mmap_anon!(
    d3_mmap_shared_none,
    map::MAP_SHARED | map::MAP_ANONYMOUS,
    prot::PROT_NONE
);
mmap_anon!(
    d3_mmap_shared_r,
    map::MAP_SHARED | map::MAP_ANONYMOUS,
    prot::PROT_READ
);
mmap_anon!(
    d3_mmap_shared_rw,
    map::MAP_SHARED | map::MAP_ANONYMOUS,
    prot::PROT_READ | prot::PROT_WRITE
);

#[crate::lctp_test(suite = posix)]
fn d3_mmap_file_private() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"m", 0o644), "c");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "tr");
    let addr = check_ok!(
        syscall::mmap(0, PAGE, prot::PROT_READ, map::MAP_PRIVATE, fd, 0),
        "mmap"
    );
    check_ok!(syscall::munmap(addr, PAGE), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_mmap_file_shared() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let fd = check_ok!(tmp.create_file(b"m", 0o644), "c");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "tr");
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 0x7E;
    }
    check_ok!(syscall::munmap(addr, PAGE), "un");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_mmap_shared_fork_visible() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 0;
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            *(addr as *mut u8) = 0x42;
        }
        syscall::exit(0);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    let v = unsafe { *(addr as *const u8) };
    check_eq!(v, 0x42, "shared");
    check_ok!(syscall::munmap(addr, PAGE), "un");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_mmap_private_fork_cow() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 0x11;
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            *(addr as *mut u8) = 0x22;
        }
        syscall::exit(0);
    }
    let mut st = 0;
    check_ok!(syscall::wait4(pid, &mut st, 0), "wait");
    let v = unsafe { *(addr as *const u8) };
    check_eq!(v, 0x11, "cow");
    check_ok!(syscall::munmap(addr, PAGE), "un");
    Ok(())
}

macro_rules! sig_ign_dfl {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            check_ok!(syscall::signal_ignore($sig), "ign");
            check_ok!(syscall::signal_default($sig), "dfl");
            let _ = SIG_DFL;
            Ok(())
        }
    };
}

sig_ign_dfl!(d3_sig_ign_dfl_usr1, SIGUSR1);
sig_ign_dfl!(d3_sig_ign_dfl_usr2, SIGUSR2);
sig_ign_dfl!(d3_sig_ign_dfl_term, SIGTERM);
sig_ign_dfl!(d3_sig_ign_dfl_int, SIGINT);

macro_rules! sig_block {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let mut old = 0u64;
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask($sig)), Some(&mut old)),
                "block"
            );
            let mut cur = 0u64;
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, None, Some(&mut cur)),
                "q"
            );
            check!(cur & sigmask($sig) != 0, "blocked");
            check_ok!(
                syscall::rt_sigprocmask(SIG_SETMASK, Some(old), None),
                "restore"
            );
            Ok(())
        }
    };
}

sig_block!(d3_blk_usr1, SIGUSR1);
sig_block!(d3_blk_usr2, SIGUSR2);
sig_block!(d3_blk_term, SIGTERM);
sig_block!(d3_blk_int, SIGINT);

macro_rules! sig_pending {
    ($name:ident, $sig:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            check_ok!(syscall::signal_ignore($sig), "ign");
            check_ok!(
                syscall::rt_sigprocmask(SIG_BLOCK, Some(sigmask($sig)), None),
                "block"
            );
            check_ok!(syscall::kill(syscall::getpid(), $sig), "kill");
            let mut pend = 0u64;
            check_ok!(syscall::rt_sigpending(&mut pend), "pend");
            check!(pend & sigmask($sig) != 0, "bit");
            check_ok!(
                syscall::rt_sigprocmask(SIG_UNBLOCK, Some(sigmask($sig)), None),
                "un"
            );
            restore_sig($sig)?;
            Ok(())
        }
    };
}

sig_pending!(d3_pend_usr1, SIGUSR1);
sig_pending!(d3_pend_usr2, SIGUSR2);
sig_pending!(d3_pend_term, SIGTERM);
sig_pending!(d3_pend_int, SIGINT);

#[crate::lctp_test(suite = posix)]
fn d3_sig_block_two() -> TestResult {
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

#[crate::lctp_test(suite = posix)]
fn d3_sig_setmask_empty() -> TestResult {
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

#[crate::lctp_test(suite = posix)]
fn d3_faccessat_dirfd() -> TestResult {
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

#[crate::lctp_test(suite = posix)]
fn d3_access_dir_x() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::access(&d, X_OK), "x");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn d3_write_then_access() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"w")?;
    write_file(&p, b"hi")?;
    check_ok!(syscall::access(&p, R_OK | W_OK), "rw");
    Ok(())
}
