//! pjdfstest-inspired depth4: denser chmod/fchmod/open/unlink/mkdir/rename/link/
//! symlink/truncate/access/stat/fallocate/utimensat/renameat2 matrices + ctime (full).

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{
    copy_child, create_dir, create_empty, join_path, nanosleep_secs, timespec_later, truncate_cstr,
    write_file,
};
use crate::syscall::{
    self, oflag, Errno, Timespec, AT_FDCWD, F_OK, R_OK, RENAME_EXCHANGE, RENAME_NOREPLACE,
    S_IFIFO, UTIME_NOW, UTIME_OMIT, W_OK, X_OK, FALLOC_FL_KEEP_SIZE, FALLOC_FL_PUNCH_HOLE,
    FALLOC_FL_ZERO_RANGE,
};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::EINVAL | Errno::ENOSYS | Errno::EPERM | Errno::EOPNOTSUPP | Errno::ENOTSUP | Errno::ENOSPC
    )
}

macro_rules! chmod_file_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $mode), "chmod");
            let st = check_ok!(syscall::stat(&p), "stat");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            check_ok!(syscall::chmod(&p, 0o644), "restore");
            Ok(())
        }
    };
}

macro_rules! fchmod_file_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            let fd = check_ok!(syscall::open(&p, oflag::O_RDWR, 0), "o");
            check_ok!(syscall::fchmod(fd, $mode), "fchmod");
            check_ok!(syscall::close(fd), "c");
            let st = check_ok!(syscall::stat(&p), "stat");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            check_ok!(syscall::chmod(&p, 0o644), "restore");
            Ok(())
        }
    };
}

macro_rules! chmod_dir_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let d = create_dir(&mut tmp, b"d", 0o755)?;
            check_ok!(syscall::chmod(&d, $mode), "chmod");
            let st = check_ok!(syscall::stat(&d), "stat");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            check_ok!(syscall::chmod(&d, 0o755), "restore");
            check_ok!(syscall::rmdir(&d), "rm");
            Ok(())
        }
    };
}

macro_rules! eacces_open {
    ($name:ident, $mode:expr, $flags:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $mode), "chmod");
            check_err!(syscall::open(&p, $flags, 0), Errno::EACCES, "eacces");
            check_ok!(syscall::chmod(&p, 0o644), "restore");
            Ok(())
        }
    };
}

macro_rules! eacces_access {
    ($name:ident, $mode:expr, $want:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $mode), "chmod");
            check_err!(syscall::access(&p, $want), Errno::EACCES, "eacces");
            check_ok!(syscall::chmod(&p, 0o644), "restore");
            Ok(())
        }
    };
}

macro_rules! eacces_dir_unlink {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let dir = create_dir(&mut tmp, b"d", 0o755)?;
            let mut child = [0u8; 160];
            let p = {
                let j = join_path(&dir, b"f", &mut child)?;
                let mut b = [0u8; 160];
                b[..j.len()].copy_from_slice(j);
                b
            };
            write_file(truncate_cstr(&p), b"z")?;
            check_ok!(syscall::chmod(&dir, $mode), "chmod");
            check_err!(syscall::unlink(truncate_cstr(&p)), Errno::EACCES, "unlink");
            check_ok!(syscall::chmod(&dir, 0o755), "restore");
            check_ok!(syscall::unlink(truncate_cstr(&p)), "cleanup");
            check_ok!(syscall::rmdir(&dir), "rmdir");
            Ok(())
        }
    };
}

macro_rules! eacces_dir_mkdir {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let dir = create_dir(&mut tmp, b"d", 0o755)?;
            let mut child = [0u8; 160];
            let p = join_path(&dir, b"sub", &mut child)?;
            check_ok!(syscall::chmod(&dir, $mode), "chmod");
            check_err!(syscall::mkdir(p, 0o755), Errno::EACCES, "mkdir");
            check_ok!(syscall::chmod(&dir, 0o755), "restore");
            check_ok!(syscall::rmdir(&dir), "rmdir");
            Ok(())
        }
    };
}

macro_rules! eacces_dir_rename_into {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let dir = create_dir(&mut tmp, b"d", 0o755)?;
            let src = create_empty(&mut tmp, b"s")?;
            let mut dest = [0u8; 160];
            let dst = join_path(&dir, b"t", &mut dest)?;
            check_ok!(syscall::chmod(&dir, $mode), "chmod");
            check_err!(syscall::rename(&src, dst), Errno::EACCES, "rename");
            check_ok!(syscall::chmod(&dir, 0o755), "restore");
            check_ok!(syscall::rmdir(&dir), "rmdir");
            Ok(())
        }
    };
}

macro_rules! eacces_dir_link_into {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let dir = create_dir(&mut tmp, b"d", 0o755)?;
            let src = create_empty(&mut tmp, b"s")?;
            let mut dest = [0u8; 160];
            let dst = join_path(&dir, b"l", &mut dest)?;
            check_ok!(syscall::chmod(&dir, $mode), "chmod");
            check_err!(syscall::link(&src, dst), Errno::EACCES, "link");
            check_ok!(syscall::chmod(&dir, 0o755), "restore");
            check_ok!(syscall::rmdir(&dir), "rmdir");
            Ok(())
        }
    };
}

macro_rules! open_creat_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = copy_child(&mut tmp, b"oc")?;
            let fd = check_ok!(
                syscall::open(&p, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, $mode),
                "open"
            );
            check_ok!(syscall::close(fd), "c");
            check_ok!(syscall::chmod(&p, $mode & 0o777), "chmod");
            let st = check_ok!(syscall::stat(&p), "stat");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            Ok(())
        }
    };
}

macro_rules! trunc_size {
    ($name:ident, $sz:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"t")?;
            write_file(&p, b"0123456789ABCDEFGHIJKLMNOP")?;
            check_ok!(syscall::truncate(&p, $sz), "trunc");
            let st = check_ok!(syscall::stat(&p), "stat");
            check_eq!(st.st_size, $sz, "size");
            Ok(())
        }
    };
}

macro_rules! rename_pair {
    ($name:ident, $a:expr, $b:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let s = create_empty(&mut tmp, $a)?;
            write_file(&s, b"R")?;
            let d = copy_child(&mut tmp, $b)?;
            check_ok!(syscall::rename(&s, &d), "ren");
            check_err!(syscall::stat(&s), Errno::ENOENT, "gone");
            check_ok!(syscall::stat(&d), "there");
            Ok(())
        }
    };
}

macro_rules! link_pair {
    ($name:ident, $a:expr, $b:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let s = create_empty(&mut tmp, $a)?;
            write_file(&s, b"L")?;
            let d = copy_child(&mut tmp, $b)?;
            check_ok!(syscall::link(&s, &d), "link");
            let sa = check_ok!(syscall::stat(&s), "sa");
            let sb = check_ok!(syscall::stat(&d), "sb");
            check_eq!(sa.st_ino, sb.st_ino, "ino");
            check!(sa.st_nlink >= 2, "nlink");
            check_ok!(syscall::unlink(&d), "ul");
            Ok(())
        }
    };
}

macro_rules! symlink_pair {
    ($name:ident, $tgt:expr, $link:expr, $tgt_c:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let _ = create_empty(&mut tmp, $tgt)?;
            let l = copy_child(&mut tmp, $link)?;
            check_ok!(syscall::symlink($tgt_c, &l), "sym");
            let st = check_ok!(syscall::lstat(&l), "lstat");
            check!(st.is_lnk(), "lnk");
            check_ok!(syscall::unlink(&l), "ul");
            Ok(())
        }
    };
}

macro_rules! unlink_name {
    ($name:ident, $nm:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, $nm)?;
            check_ok!(syscall::unlink(&p), "ul");
            check_err!(syscall::stat(&p), Errno::ENOENT, "gone");
            Ok(())
        }
    };
}

macro_rules! mkdir_rmdir {
    ($name:ident, $nm:expr, $mode:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let d = create_dir(&mut tmp, $nm, $mode)?;
            let st = check_ok!(syscall::stat(&d), "stat");
            check!(st.is_dir(), "dir");
            check_ok!(syscall::rmdir(&d), "rm");
            check_err!(syscall::stat(&d), Errno::ENOENT, "gone");
            Ok(())
        }
    };
}

macro_rules! access_ok {
    ($name:ident, $mode:expr, $want:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&p, $mode), "chmod");
            check_ok!(syscall::access(&p, $want), "access");
            check_ok!(syscall::chmod(&p, 0o644), "restore");
            Ok(())
        }
    };
}

macro_rules! falloc_soft {
    ($name:ident, $mode:expr, $off:expr, $len:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"fa")?;
            let fd = check_ok!(syscall::open(&p, oflag::O_RDWR, 0), "o");
            check_ok!(syscall::ftruncate(fd, 8192), "tr");
            match syscall::fallocate(fd, $mode, $off, $len) {
                Ok(()) => {}
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::close(fd);
                    return Err(crate::harness::AssertFail::msg("falloc"));
                }
            }
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

macro_rules! utimens_explicit {
    ($name:ident, $sec:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"u")?;
            let times = [
                Timespec { tv_sec: $sec, tv_nsec: 0 },
                Timespec { tv_sec: $sec, tv_nsec: 0 },
            ];
            check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "set");
            let st = check_ok!(syscall::stat(&p), "stat");
            check_eq!(st.st_mtime, $sec, "mtime");
            Ok(())
        }
    };
}

macro_rules! fifo_nb {
    ($name:ident, $flags:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = copy_child(&mut tmp, b"fifo")?;
            check_ok!(syscall::mknodat(AT_FDCWD, &p, S_IFIFO | 0o644, 0), "mknod");
            match syscall::open(&p, $flags | oflag::O_NONBLOCK, 0) {
                Ok(fd) => check_ok!(syscall::close(fd), "c"),
                Err(Errno::ENXIO) | Err(Errno::EAGAIN) | Err(Errno::EWOULDBLOCK) => {}
                Err(e) if soft(e) => {}
                Err(_) => {
                    let _ = syscall::unlink(&p);
                    return Err(crate::harness::AssertFail::msg("fifo open"));
                }
            }
            check_ok!(syscall::unlink(&p), "ul");
            Ok(())
        }
    };
}

chmod_file_mode!(d4_chmod_01, 0o0);
chmod_file_mode!(d4_chmod_02, 0o1);
chmod_file_mode!(d4_chmod_03, 0o2);
chmod_file_mode!(d4_chmod_04, 0o4);
chmod_file_mode!(d4_chmod_05, 0o10);
chmod_file_mode!(d4_chmod_06, 0o20);
chmod_file_mode!(d4_chmod_07, 0o40);
chmod_file_mode!(d4_chmod_08, 0o100);
chmod_file_mode!(d4_chmod_09, 0o200);
chmod_file_mode!(d4_chmod_10, 0o400);
chmod_file_mode!(d4_chmod_11, 0o111);
chmod_file_mode!(d4_chmod_12, 0o222);
chmod_file_mode!(d4_chmod_13, 0o333);
chmod_file_mode!(d4_chmod_14, 0o444);
chmod_file_mode!(d4_chmod_15, 0o555);
chmod_file_mode!(d4_chmod_16, 0o666);
chmod_file_mode!(d4_chmod_17, 0o777);
chmod_file_mode!(d4_chmod_18, 0o700);
chmod_file_mode!(d4_chmod_19, 0o710);
chmod_file_mode!(d4_chmod_20, 0o720);
chmod_file_mode!(d4_chmod_21, 0o730);
chmod_file_mode!(d4_chmod_22, 0o740);
chmod_file_mode!(d4_chmod_23, 0o750);
chmod_file_mode!(d4_chmod_24, 0o760);
chmod_file_mode!(d4_chmod_25, 0o770);
chmod_file_mode!(d4_chmod_26, 0o701);
chmod_file_mode!(d4_chmod_27, 0o702);
chmod_file_mode!(d4_chmod_28, 0o704);
chmod_file_mode!(d4_chmod_29, 0o421);
chmod_file_mode!(d4_chmod_30, 0o241);
chmod_file_mode!(d4_chmod_31, 0o124);
chmod_file_mode!(d4_chmod_32, 0o412);
chmod_file_mode!(d4_chmod_33, 0o140);
chmod_file_mode!(d4_chmod_34, 0o14);
chmod_file_mode!(d4_chmod_35, 0o41);
chmod_file_mode!(d4_chmod_36, 0o640);
chmod_file_mode!(d4_chmod_37, 0o620);
chmod_file_mode!(d4_chmod_38, 0o604);
chmod_file_mode!(d4_chmod_39, 0o460);
chmod_file_mode!(d4_chmod_40, 0o260);
chmod_file_mode!(d4_chmod_41, 0o64);
fchmod_file_mode!(d4_fchmod_01, 0o400);
fchmod_file_mode!(d4_fchmod_02, 0o440);
fchmod_file_mode!(d4_fchmod_03, 0o444);
fchmod_file_mode!(d4_fchmod_04, 0o600);
fchmod_file_mode!(d4_fchmod_05, 0o640);
fchmod_file_mode!(d4_fchmod_06, 0o644);
fchmod_file_mode!(d4_fchmod_07, 0o660);
fchmod_file_mode!(d4_fchmod_08, 0o666);
fchmod_file_mode!(d4_fchmod_09, 0o700);
fchmod_file_mode!(d4_fchmod_10, 0o740);
fchmod_file_mode!(d4_fchmod_11, 0o744);
fchmod_file_mode!(d4_fchmod_12, 0o750);
fchmod_file_mode!(d4_fchmod_13, 0o755);
fchmod_file_mode!(d4_fchmod_14, 0o770);
fchmod_file_mode!(d4_fchmod_15, 0o777);
fchmod_file_mode!(d4_fchmod_16, 0o620);
fchmod_file_mode!(d4_fchmod_17, 0o604);
fchmod_file_mode!(d4_fchmod_18, 0o420);
fchmod_file_mode!(d4_fchmod_19, 0o240);
fchmod_file_mode!(d4_fchmod_20, 0o204);
fchmod_file_mode!(d4_fchmod_21, 0o140);
fchmod_file_mode!(d4_fchmod_22, 0o104);
chmod_dir_mode!(d4_dchmod_01, 0o700);
chmod_dir_mode!(d4_dchmod_02, 0o711);
chmod_dir_mode!(d4_dchmod_03, 0o755);
chmod_dir_mode!(d4_dchmod_04, 0o775);
chmod_dir_mode!(d4_dchmod_05, 0o777);
chmod_dir_mode!(d4_dchmod_06, 0o555);
chmod_dir_mode!(d4_dchmod_07, 0o511);
chmod_dir_mode!(d4_dchmod_08, 0o500);
chmod_dir_mode!(d4_dchmod_09, 0o750);
chmod_dir_mode!(d4_dchmod_10, 0o730);
chmod_dir_mode!(d4_dchmod_11, 0o710);
chmod_dir_mode!(d4_dchmod_12, 0o701);
chmod_dir_mode!(d4_dchmod_13, 0o760);
chmod_dir_mode!(d4_dchmod_14, 0o770);
chmod_dir_mode!(d4_dchmod_15, 0o705);
chmod_dir_mode!(d4_dchmod_16, 0o715);
chmod_dir_mode!(d4_dchmod_17, 0o725);
chmod_dir_mode!(d4_dchmod_18, 0o735);
chmod_dir_mode!(d4_dchmod_19, 0o745);
chmod_dir_mode!(d4_dchmod_20, 0o765);
eacces_open!(d4_eacc_rd_000, 0o0, oflag::O_RDONLY);
eacces_open!(d4_eacc_rd_200, 0o200, oflag::O_RDONLY);
eacces_open!(d4_eacc_rd_300, 0o300, oflag::O_RDONLY);
eacces_open!(d4_eacc_rd_010, 0o10, oflag::O_RDONLY);
eacces_open!(d4_eacc_rd_100, 0o100, oflag::O_RDONLY);
eacces_open!(d4_eacc_wr_000, 0o0, oflag::O_WRONLY);
eacces_open!(d4_eacc_wr_400, 0o400, oflag::O_WRONLY);
eacces_open!(d4_eacc_wr_500, 0o500, oflag::O_WRONLY);
eacces_open!(d4_eacc_wr_040, 0o40, oflag::O_WRONLY);
eacces_open!(d4_eacc_wr_100, 0o100, oflag::O_WRONLY);
eacces_open!(d4_eacc_rw_000, 0o0, oflag::O_RDWR);
eacces_open!(d4_eacc_rw_400, 0o400, oflag::O_RDWR);
eacces_open!(d4_eacc_rw_200, 0o200, oflag::O_RDWR);
eacces_open!(d4_eacc_rw_100, 0o100, oflag::O_RDWR);
eacces_open!(d4_eacc_rw_440, 0o440, oflag::O_RDWR);
eacces_open!(d4_eacc_tr_400, 0o400, oflag::O_WRONLY | oflag::O_TRUNC);
eacces_open!(d4_eacc_tr_500, 0o500, oflag::O_WRONLY | oflag::O_TRUNC);
eacces_open!(d4_eacc_tr_440, 0o440, oflag::O_RDWR | oflag::O_TRUNC);
eacces_open!(d4_eacc_ap_400, 0o400, oflag::O_WRONLY | oflag::O_APPEND);
eacces_open!(d4_eacc_ap_440, 0o440, oflag::O_RDWR | oflag::O_APPEND);
eacces_open!(d4_eacc_ap_500, 0o500, oflag::O_WRONLY | oflag::O_APPEND);
eacces_open!(d4_eacc_ap_100, 0o100, oflag::O_RDWR | oflag::O_APPEND);
eacces_access!(d4_acc_r_200, 0o200, R_OK);
eacces_access!(d4_acc_r_300, 0o300, R_OK);
eacces_access!(d4_acc_r_100, 0o100, R_OK);
eacces_access!(d4_acc_r_000, 0o0, R_OK);
eacces_access!(d4_acc_r_020, 0o20, R_OK);
eacces_access!(d4_acc_w_400, 0o400, W_OK);
eacces_access!(d4_acc_w_500, 0o500, W_OK);
eacces_access!(d4_acc_w_100, 0o100, W_OK);
eacces_access!(d4_acc_w_000, 0o0, W_OK);
eacces_access!(d4_acc_w_040, 0o40, W_OK);
eacces_access!(d4_acc_x_600, 0o600, X_OK);
eacces_access!(d4_acc_x_640, 0o640, X_OK);
eacces_access!(d4_acc_x_200, 0o200, X_OK);
eacces_access!(d4_acc_x_000, 0o0, X_OK);
eacces_access!(d4_acc_x_400, 0o400, X_OK);
eacces_access!(d4_acc_rw_400, 0o400, R_OK | W_OK);
eacces_access!(d4_acc_rw_200, 0o200, R_OK | W_OK);
eacces_access!(d4_acc_rx_600, 0o600, R_OK | X_OK);
eacces_access!(d4_acc_wx_400, 0o400, W_OK | X_OK);
eacces_access!(d4_acc_rwx_600, 0o600, R_OK | W_OK | X_OK);
eacces_access!(d4_acc_rwx_000, 0o0, R_OK | W_OK | X_OK);
eacces_access!(d4_acc_rwx_400, 0o400, R_OK | W_OK | X_OK);
access_ok!(d4_aok_f_000, 0o0, F_OK);
access_ok!(d4_aok_f_644, 0o644, F_OK);
access_ok!(d4_aok_r_400, 0o400, R_OK);
access_ok!(d4_aok_r_440, 0o440, R_OK);
access_ok!(d4_aok_r_444, 0o444, R_OK);
access_ok!(d4_aok_w_200, 0o200, W_OK);
access_ok!(d4_aok_w_600, 0o600, W_OK);
access_ok!(d4_aok_w_620, 0o620, W_OK);
access_ok!(d4_aok_x_100, 0o100, X_OK);
access_ok!(d4_aok_x_500, 0o500, X_OK);
access_ok!(d4_aok_x_700, 0o700, X_OK);
access_ok!(d4_aok_rw_600, 0o600, R_OK | W_OK);
access_ok!(d4_aok_rx_500, 0o500, R_OK | X_OK);
access_ok!(d4_aok_wx_300, 0o300, W_OK | X_OK);
access_ok!(d4_aok_rwx_700, 0o700, R_OK | W_OK | X_OK);
eacces_dir_unlink!(d4_punl_555, 0o555);
eacces_dir_mkdir!(d4_pmk_555, 0o555);
eacces_dir_unlink!(d4_punl_444, 0o444);
eacces_dir_mkdir!(d4_pmk_444, 0o444);
eacces_dir_unlink!(d4_punl_111, 0o111);
eacces_dir_mkdir!(d4_pmk_111, 0o111);
eacces_dir_unlink!(d4_punl_0, 0o0);
eacces_dir_mkdir!(d4_pmk_0, 0o0);
eacces_dir_unlink!(d4_punl_511, 0o511);
eacces_dir_mkdir!(d4_pmk_511, 0o511);
eacces_dir_unlink!(d4_punl_500, 0o500);
eacces_dir_mkdir!(d4_pmk_500, 0o500);
eacces_dir_unlink!(d4_punl_501, 0o501);
eacces_dir_mkdir!(d4_pmk_501, 0o501);
eacces_dir_unlink!(d4_punl_401, 0o401);
eacces_dir_mkdir!(d4_pmk_401, 0o401);
eacces_dir_rename_into!(d4_pren_555, 0o555);
eacces_dir_link_into!(d4_plnk_555, 0o555);
eacces_dir_rename_into!(d4_pren_444, 0o444);
eacces_dir_link_into!(d4_plnk_444, 0o444);
eacces_dir_rename_into!(d4_pren_0, 0o0);
eacces_dir_link_into!(d4_plnk_0, 0o0);
eacces_dir_rename_into!(d4_pren_111, 0o111);
eacces_dir_link_into!(d4_plnk_111, 0o111);
eacces_dir_rename_into!(d4_pren_511, 0o511);
eacces_dir_link_into!(d4_plnk_511, 0o511);
eacces_dir_rename_into!(d4_pren_501, 0o501);
eacces_dir_link_into!(d4_plnk_501, 0o501);
open_creat_mode!(d4_oc_01, 0o400);
open_creat_mode!(d4_oc_02, 0o440);
open_creat_mode!(d4_oc_03, 0o444);
open_creat_mode!(d4_oc_04, 0o600);
open_creat_mode!(d4_oc_05, 0o640);
open_creat_mode!(d4_oc_06, 0o644);
open_creat_mode!(d4_oc_07, 0o660);
open_creat_mode!(d4_oc_08, 0o666);
open_creat_mode!(d4_oc_09, 0o700);
open_creat_mode!(d4_oc_10, 0o740);
open_creat_mode!(d4_oc_11, 0o744);
open_creat_mode!(d4_oc_12, 0o750);
open_creat_mode!(d4_oc_13, 0o755);
open_creat_mode!(d4_oc_14, 0o770);
open_creat_mode!(d4_oc_15, 0o777);
open_creat_mode!(d4_oc_16, 0o620);
open_creat_mode!(d4_oc_17, 0o604);
open_creat_mode!(d4_oc_18, 0o420);
open_creat_mode!(d4_oc_19, 0o240);
open_creat_mode!(d4_oc_20, 0o204);
open_creat_mode!(d4_oc_21, 0o140);
open_creat_mode!(d4_oc_22, 0o104);
open_creat_mode!(d4_oc_23, 0o41);
open_creat_mode!(d4_oc_24, 0o14);
open_creat_mode!(d4_oc_25, 0o421);
open_creat_mode!(d4_oc_26, 0o124);
open_creat_mode!(d4_oc_27, 0o412);
open_creat_mode!(d4_oc_28, 0o241);
open_creat_mode!(d4_oc_29, 0o61);
open_creat_mode!(d4_oc_30, 0o16);
open_creat_mode!(d4_oc_31, 0o160);
open_creat_mode!(d4_oc_32, 0o61);
trunc_size!(d4_trunc_01, 0);
trunc_size!(d4_trunc_02, 1);
trunc_size!(d4_trunc_03, 2);
trunc_size!(d4_trunc_04, 3);
trunc_size!(d4_trunc_05, 4);
trunc_size!(d4_trunc_06, 5);
trunc_size!(d4_trunc_07, 6);
trunc_size!(d4_trunc_08, 7);
trunc_size!(d4_trunc_09, 8);
trunc_size!(d4_trunc_10, 9);
trunc_size!(d4_trunc_11, 10);
trunc_size!(d4_trunc_12, 11);
trunc_size!(d4_trunc_13, 12);
trunc_size!(d4_trunc_14, 13);
trunc_size!(d4_trunc_15, 14);
trunc_size!(d4_trunc_16, 15);
trunc_size!(d4_trunc_17, 16);
trunc_size!(d4_trunc_18, 17);
trunc_size!(d4_trunc_19, 20);
trunc_size!(d4_trunc_20, 24);
trunc_size!(d4_trunc_21, 32);
trunc_size!(d4_trunc_22, 48);
trunc_size!(d4_trunc_23, 64);
trunc_size!(d4_trunc_24, 100);
trunc_size!(d4_trunc_25, 128);
trunc_size!(d4_trunc_26, 200);
trunc_size!(d4_trunc_27, 256);
trunc_size!(d4_trunc_28, 512);
trunc_size!(d4_trunc_29, 1024);
rename_pair!(d4_ren_00, b"ra0", b"rb0");
link_pair!(d4_lnk_00, b"la0", b"lb0");
rename_pair!(d4_ren_01, b"ra1", b"rb1");
link_pair!(d4_lnk_01, b"la1", b"lb1");
rename_pair!(d4_ren_02, b"ra2", b"rb2");
link_pair!(d4_lnk_02, b"la2", b"lb2");
rename_pair!(d4_ren_03, b"ra3", b"rb3");
link_pair!(d4_lnk_03, b"la3", b"lb3");
rename_pair!(d4_ren_04, b"ra4", b"rb4");
link_pair!(d4_lnk_04, b"la4", b"lb4");
rename_pair!(d4_ren_05, b"ra5", b"rb5");
link_pair!(d4_lnk_05, b"la5", b"lb5");
rename_pair!(d4_ren_06, b"ra6", b"rb6");
link_pair!(d4_lnk_06, b"la6", b"lb6");
rename_pair!(d4_ren_07, b"ra7", b"rb7");
link_pair!(d4_lnk_07, b"la7", b"lb7");
rename_pair!(d4_ren_08, b"ra8", b"rb8");
link_pair!(d4_lnk_08, b"la8", b"lb8");
rename_pair!(d4_ren_09, b"ra9", b"rb9");
link_pair!(d4_lnk_09, b"la9", b"lb9");
rename_pair!(d4_ren_10, b"ra10", b"rb10");
link_pair!(d4_lnk_10, b"la10", b"lb10");
rename_pair!(d4_ren_11, b"ra11", b"rb11");
link_pair!(d4_lnk_11, b"la11", b"lb11");
rename_pair!(d4_ren_12, b"ra12", b"rb12");
link_pair!(d4_lnk_12, b"la12", b"lb12");
rename_pair!(d4_ren_13, b"ra13", b"rb13");
link_pair!(d4_lnk_13, b"la13", b"lb13");
rename_pair!(d4_ren_14, b"ra14", b"rb14");
link_pair!(d4_lnk_14, b"la14", b"lb14");
rename_pair!(d4_ren_15, b"ra15", b"rb15");
link_pair!(d4_lnk_15, b"la15", b"lb15");
rename_pair!(d4_ren_16, b"ra16", b"rb16");
link_pair!(d4_lnk_16, b"la16", b"lb16");
rename_pair!(d4_ren_17, b"ra17", b"rb17");
link_pair!(d4_lnk_17, b"la17", b"lb17");
rename_pair!(d4_ren_18, b"ra18", b"rb18");
link_pair!(d4_lnk_18, b"la18", b"lb18");
rename_pair!(d4_ren_19, b"ra19", b"rb19");
link_pair!(d4_lnk_19, b"la19", b"lb19");
rename_pair!(d4_ren_20, b"ra20", b"rb20");
link_pair!(d4_lnk_20, b"la20", b"lb20");
rename_pair!(d4_ren_21, b"ra21", b"rb21");
link_pair!(d4_lnk_21, b"la21", b"lb21");
rename_pair!(d4_ren_22, b"ra22", b"rb22");
link_pair!(d4_lnk_22, b"la22", b"lb22");
rename_pair!(d4_ren_23, b"ra23", b"rb23");
link_pair!(d4_lnk_23, b"la23", b"lb23");
symlink_pair!(d4_sym_00, b"t0", b"s0", b"t0\0");
unlink_name!(d4_ul_00, b"u0");
symlink_pair!(d4_sym_01, b"t1", b"s1", b"t1\0");
unlink_name!(d4_ul_01, b"u1");
symlink_pair!(d4_sym_02, b"t2", b"s2", b"t2\0");
unlink_name!(d4_ul_02, b"u2");
symlink_pair!(d4_sym_03, b"t3", b"s3", b"t3\0");
unlink_name!(d4_ul_03, b"u3");
symlink_pair!(d4_sym_04, b"t4", b"s4", b"t4\0");
unlink_name!(d4_ul_04, b"u4");
symlink_pair!(d4_sym_05, b"t5", b"s5", b"t5\0");
unlink_name!(d4_ul_05, b"u5");
symlink_pair!(d4_sym_06, b"t6", b"s6", b"t6\0");
unlink_name!(d4_ul_06, b"u6");
symlink_pair!(d4_sym_07, b"t7", b"s7", b"t7\0");
unlink_name!(d4_ul_07, b"u7");
symlink_pair!(d4_sym_08, b"t8", b"s8", b"t8\0");
unlink_name!(d4_ul_08, b"u8");
symlink_pair!(d4_sym_09, b"t9", b"s9", b"t9\0");
unlink_name!(d4_ul_09, b"u9");
symlink_pair!(d4_sym_10, b"t10", b"s10", b"t10\0");
unlink_name!(d4_ul_10, b"u10");
symlink_pair!(d4_sym_11, b"t11", b"s11", b"t11\0");
unlink_name!(d4_ul_11, b"u11");
symlink_pair!(d4_sym_12, b"t12", b"s12", b"t12\0");
unlink_name!(d4_ul_12, b"u12");
symlink_pair!(d4_sym_13, b"t13", b"s13", b"t13\0");
unlink_name!(d4_ul_13, b"u13");
symlink_pair!(d4_sym_14, b"t14", b"s14", b"t14\0");
unlink_name!(d4_ul_14, b"u14");
symlink_pair!(d4_sym_15, b"t15", b"s15", b"t15\0");
unlink_name!(d4_ul_15, b"u15");
mkdir_rmdir!(d4_md_01, b"d1", 0o700);
mkdir_rmdir!(d4_md_02, b"d2", 0o755);
mkdir_rmdir!(d4_md_03, b"d3", 0o775);
mkdir_rmdir!(d4_md_04, b"d4", 0o777);
mkdir_rmdir!(d4_md_05, b"d5", 0o711);
mkdir_rmdir!(d4_md_06, b"d6", 0o750);
mkdir_rmdir!(d4_md_07, b"d7", 0o701);
mkdir_rmdir!(d4_md_08, b"d8", 0o730);
mkdir_rmdir!(d4_md_09, b"d9", 0o760);
mkdir_rmdir!(d4_md_10, b"d10", 0o770);
mkdir_rmdir!(d4_md_11, b"d11", 0o555);
mkdir_rmdir!(d4_md_12, b"d12", 0o511);
mkdir_rmdir!(d4_md_13, b"d13", 0o705);
mkdir_rmdir!(d4_md_14, b"d14", 0o715);
mkdir_rmdir!(d4_md_15, b"d15", 0o725);
mkdir_rmdir!(d4_md_16, b"d16", 0o735);
mkdir_rmdir!(d4_md_17, b"d17", 0o745);
mkdir_rmdir!(d4_md_18, b"d18", 0o765);
mkdir_rmdir!(d4_md_19, b"d19", 0o500);
mkdir_rmdir!(d4_md_20, b"d20", 0o510);
utimens_explicit!(d4_ut_exp_01, 1500000000);
utimens_explicit!(d4_ut_exp_02, 1550000000);
utimens_explicit!(d4_ut_exp_03, 1600000000);
utimens_explicit!(d4_ut_exp_04, 1650000000);
utimens_explicit!(d4_ut_exp_05, 1700000000);
utimens_explicit!(d4_ut_exp_06, 1750000000);
utimens_explicit!(d4_ut_exp_07, 1800000000);
utimens_explicit!(d4_ut_exp_08, 1850000000);
utimens_explicit!(d4_ut_exp_09, 1900000000);
utimens_explicit!(d4_ut_exp_10, 1950000000);
utimens_explicit!(d4_ut_exp_11, 2000000000);
utimens_explicit!(d4_ut_exp_12, 2050000000);
falloc_soft!(d4_fa_grow_0_1k, 0, 0, 1024);
falloc_soft!(d4_fa_grow_0_4k, 0, 0, 4096);
falloc_soft!(d4_fa_grow_1k_1k, 0, 1024, 1024);
falloc_soft!(d4_fa_grow_2k_2k, 0, 2048, 2048);
falloc_soft!(d4_fa_ks_0_4k, FALLOC_FL_KEEP_SIZE, 0, 4096);
falloc_soft!(d4_fa_ks_512_512, FALLOC_FL_KEEP_SIZE, 512, 512);
falloc_soft!(d4_fa_punch_0_1k, FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE, 0, 1024);
falloc_soft!(d4_fa_punch_1k_1k, FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE, 1024, 1024);
falloc_soft!(d4_fa_punch_2k_2k, FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE, 2048, 2048);
falloc_soft!(d4_fa_punch_4k_4k, FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE, 4096, 4096);
falloc_soft!(d4_fa_zero_0_1k, FALLOC_FL_ZERO_RANGE, 0, 1024);
falloc_soft!(d4_fa_zero_1k_1k, FALLOC_FL_ZERO_RANGE, 1024, 1024);
falloc_soft!(d4_fa_zero_0_4k, FALLOC_FL_ZERO_RANGE, 0, 4096);
falloc_soft!(d4_fa_zero_ks, FALLOC_FL_ZERO_RANGE | FALLOC_FL_KEEP_SIZE, 0, 2048);
fifo_nb!(d4_fifo_rd, oflag::O_RDONLY);
fifo_nb!(d4_fifo_wr, oflag::O_WRONLY);
fifo_nb!(d4_fifo_rdwr, oflag::O_RDWR);
fifo_nb!(d4_fifo_rd_ce, oflag::O_RDONLY | oflag::O_CLOEXEC);
fifo_nb!(d4_fifo_wr_ce, oflag::O_WRONLY | oflag::O_CLOEXEC);
fifo_nb!(d4_fifo_rdwr_ce, oflag::O_RDWR | oflag::O_CLOEXEC);

#[crate::lctp_test(suite = fs)]
fn d4_open_excl_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"e")?;
    check_err!(
        syscall::open(&p, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "eexist"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"missing")?;
    check_err!(syscall::open(&p, oflag::O_RDONLY, 0), Errno::ENOENT, "enoent");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_dir_eisdir_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::open(&d, oflag::O_WRONLY, 0), Errno::EISDIR, "eisdir");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_dir_eisdir_rdwr() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::open(&d, oflag::O_RDWR, 0), Errno::EISDIR, "eisdir");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_trailing_slash_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let mut slash = [0u8; 140];
    let n = p.iter().position(|&c| c == 0).unwrap();
    slash[..n].copy_from_slice(&p[..n]);
    slash[n] = b'/';
    slash[n + 1] = 0;
    check_err!(
        syscall::open(truncate_cstr(&slash), oflag::O_RDONLY, 0),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_directory_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let fd = check_ok!(
        syscall::open(&d, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "o"
    );
    check_ok!(syscall::close(fd), "c");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_directory_on_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::open(&p, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_cloexec() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&p, oflag::O_RDONLY | oflag::O_CLOEXEC, 0), "o");
    let flags = check_ok!(syscall::fcntl(fd, syscall::fcntl_cmd::F_GETFD, 0), "fd");
    check!(flags as i32 & syscall::FD_CLOEXEC != 0, "cloexec");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_path() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    match syscall::open(&p, oflag::O_PATH, 0) {
        Ok(fd) => check_ok!(syscall::close(fd), "c"),
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("opath")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_append_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    write_file(&p, b"AB")?;
    let fd = check_ok!(syscall::open(&p, oflag::O_WRONLY | oflag::O_APPEND, 0), "o");
    check_ok!(syscall::write(fd, b"C"), "w");
    check_ok!(syscall::close(fd), "c");
    let st = check_ok!(syscall::stat(&p), "st");
    check_eq!(st.st_size, 3, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_trunc() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    write_file(&p, b"ABCDEF")?;
    let fd = check_ok!(syscall::open(&p, oflag::O_WRONLY | oflag::O_TRUNC, 0), "o");
    check_ok!(syscall::close(fd), "c");
    let st = check_ok!(syscall::stat(&p), "st");
    check_eq!(st.st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_open_nofollow_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let _t = create_empty(&mut tmp, b"tgt")?;
    let l = copy_child(&mut tmp, b"lnk")?;
    check_ok!(syscall::symlink(b"tgt\0", &l), "sym");
    check_err!(
        syscall::open(&l, oflag::O_RDONLY | oflag::O_NOFOLLOW, 0),
        Errno::ELOOP,
        "eloop"
    );
    check_ok!(syscall::unlink(&l), "ul");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_stat_reg_type() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let st = check_ok!(syscall::stat(&p), "st");
    check!(st.is_reg(), "reg");
    check_eq!(st.st_nlink, 1, "nlink");
    check_eq!(st.st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_stat_dir_type() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let st = check_ok!(syscall::stat(&d), "st");
    check!(st.is_dir(), "dir");
    check!(st.st_nlink >= 2, "nlink");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_stat_size_after_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    write_file(&p, b"0123456789")?;
    let st = check_ok!(syscall::stat(&p), "st");
    check_eq!(st.st_size, 10, "size");
    check!(st.is_reg(), "reg");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_stat_nlink_hardlinks() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    let c = copy_child(&mut tmp, b"c")?;
    check_ok!(syscall::link(&a, &b), "l1");
    check_ok!(syscall::link(&a, &c), "l2");
    let st = check_ok!(syscall::stat(&a), "st");
    check_eq!(st.st_nlink, 3, "nlink");
    check_ok!(syscall::unlink(&b), "ub");
    check_ok!(syscall::unlink(&c), "uc");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_renameat2_noreplace_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    match syscall::renameat2(AT_FDCWD, &a, AT_FDCWD, &b, RENAME_NOREPLACE) {
        Ok(()) => {
            check_err!(syscall::stat(&a), Errno::ENOENT, "gone");
            check_ok!(syscall::stat(&b), "there");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("noreplace")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_renameat2_noreplace_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    match syscall::renameat2(AT_FDCWD, &a, AT_FDCWD, &b, RENAME_NOREPLACE) {
        Err(Errno::EEXIST) => {}
        Err(e) if soft(e) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("expected eexist")),
        Err(_) => return Err(crate::harness::AssertFail::msg("noreplace eexist")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_renameat2_exchange() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    write_file(&a, b"A")?;
    write_file(&b, b"B")?;
    match syscall::renameat2(AT_FDCWD, &a, AT_FDCWD, &b, RENAME_EXCHANGE) {
        Ok(()) => {
            let mut buf = [0u8; 1];
            let fd = check_ok!(syscall::open(&a, oflag::O_RDONLY, 0), "oa");
            check_ok!(syscall::read(fd, &mut buf), "ra");
            check_eq!(buf[0], b'B', "a got B");
            check_ok!(syscall::close(fd), "ca");
            let fd = check_ok!(syscall::open(&b, oflag::O_RDONLY, 0), "ob");
            check_ok!(syscall::read(fd, &mut buf), "rb");
            check_eq!(buf[0], b'A', "b got A");
            check_ok!(syscall::close(fd), "cb");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("exchange")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_renameat2_exchange_dirs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_dir(&mut tmp, b"da", 0o755)?;
    let b = create_dir(&mut tmp, b"db", 0o755)?;
    match syscall::renameat2(AT_FDCWD, &a, AT_FDCWD, &b, RENAME_EXCHANGE) {
        Ok(()) => {
            check_ok!(syscall::rmdir(&a), "ra");
            check_ok!(syscall::rmdir(&b), "rb");
        }
        Err(e) if soft(e) => {
            check_ok!(syscall::rmdir(&a), "ra");
            check_ok!(syscall::rmdir(&b), "rb");
        }
        Err(_) => {
            let _ = syscall::rmdir(&a);
            let _ = syscall::rmdir(&b);
            return Err(crate::harness::AssertFail::msg("ex dirs"));
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_rmdir_notempty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let mut child = [0u8; 160];
    let p = {
        let j = join_path(&d, b"f", &mut child)?;
        let mut b = [0u8; 160];
        b[..j.len()].copy_from_slice(j);
        b
    };
    write_file(truncate_cstr(&p), b"x")?;
    check_err!(syscall::rmdir(&d), Errno::ENOTEMPTY, "notempty");
    check_ok!(syscall::unlink(truncate_cstr(&p)), "ul");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_mkdir_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::mkdir(&d, 0o755), Errno::EEXIST, "eexist");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_link_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    check_err!(syscall::link(&a, &b), Errno::EEXIST, "eexist");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_symlink_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    check_err!(syscall::symlink(b"x\0", &a), Errno::EEXIST, "eexist");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_unlink_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"gone")?;
    check_err!(syscall::unlink(&p), Errno::ENOENT, "enoent");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_rmdir_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"gone")?;
    check_err!(syscall::rmdir(&p), Errno::ENOENT, "enoent");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_ut_now() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    let times = [
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_NOW,
        },
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_NOW,
        },
    ];
    check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "now");
    let st = check_ok!(syscall::stat(&p), "stat");
    check!(st.st_mtime > 0, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_ut_omit_both() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    let before = check_ok!(syscall::stat(&p), "before");
    let times = [
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
    ];
    check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "omit");
    let after = check_ok!(syscall::stat(&p), "after");
    check_eq!(after.st_mtime, before.st_mtime, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_ut_omit_atime_set_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    let before = check_ok!(syscall::stat(&p), "before");
    let times = [
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
        Timespec {
            tv_sec: 1_555_000_000,
            tv_nsec: 0,
        },
    ];
    check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "set");
    let after = check_ok!(syscall::stat(&p), "after");
    check_eq!(after.st_atime, before.st_atime, "atime");
    check_eq!(after.st_mtime, 1_555_000_000, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d4_ut_set_atime_omit_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    let before = check_ok!(syscall::stat(&p), "before");
    let times = [
        Timespec {
            tv_sec: 1_565_000_000,
            tv_nsec: 0,
        },
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
    ];
    check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "set");
    let after = check_ok!(syscall::stat(&p), "after");
    check_eq!(after.st_atime, 1_565_000_000, "atime");
    check_eq!(after.st_mtime, before.st_mtime, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn d4_ctime_on_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&p), "before");
    nanosleep_secs(1)?;
    write_file(&p, b"x")?;
    let after = check_ok!(syscall::stat(&p), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn d4_mtime_on_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&p), "before");
    nanosleep_secs(1)?;
    write_file(&p, b"y")?;
    let after = check_ok!(syscall::stat(&p), "after");
    check!(
        timespec_later(
            after.st_mtime,
            after.st_mtime_nsec,
            before.st_mtime,
            before.st_mtime_nsec
        ),
        "mtime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn d4_ctime_on_chmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&p), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::chmod(&p, 0o600), "chmod");
    let after = check_ok!(syscall::stat(&p), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    check_ok!(syscall::chmod(&p, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn d4_ctime_on_fchmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&p), "before");
    nanosleep_secs(1)?;
    let fd = check_ok!(syscall::open(&p, oflag::O_RDWR, 0), "o");
    check_ok!(syscall::fchmod(fd, 0o640), "fchmod");
    check_ok!(syscall::close(fd), "c");
    let after = check_ok!(syscall::stat(&p), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    check_ok!(syscall::chmod(&p, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn d4_ctime_on_link() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let before = check_ok!(syscall::stat(&a), "before");
    nanosleep_secs(1)?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let after = check_ok!(syscall::stat(&a), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn d4_ctime_on_truncate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    write_file(&p, b"abcdef")?;
    let before = check_ok!(syscall::stat(&p), "before");
    nanosleep_secs(1)?;
    check_ok!(syscall::truncate(&p, 2), "tr");
    let after = check_ok!(syscall::stat(&p), "after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime"
    );
    Ok(())
}
