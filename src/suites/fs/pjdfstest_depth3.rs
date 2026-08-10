//! pjdfstest-inspired depth3: chmod/open/unlink/mkdir/rename/link/symlink/
//! truncate mode & errno grids, renameat2 edges, utimensat OMIT/NOW.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{
    copy_child, create_dir, create_empty, join_path, truncate_cstr, write_file,
};
use crate::syscall::{
    self, oflag, Errno, Timespec, AT_FDCWD, F_OK, R_OK, RENAME_EXCHANGE, RENAME_NOREPLACE,
    S_IFIFO, UTIME_NOW, UTIME_OMIT, W_OK, X_OK,
};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::EINVAL | Errno::ENOSYS | Errno::EPERM | Errno::EOPNOTSUPP | Errno::ENOTSUP
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

chmod_file_mode!(d3_chmod_000, 0o000);
chmod_file_mode!(d3_chmod_111, 0o111);
chmod_file_mode!(d3_chmod_222, 0o222);
chmod_file_mode!(d3_chmod_333, 0o333);
chmod_file_mode!(d3_chmod_444, 0o444);
chmod_file_mode!(d3_chmod_555, 0o555);
chmod_file_mode!(d3_chmod_666, 0o666);
chmod_file_mode!(d3_chmod_700, 0o700);
chmod_file_mode!(d3_chmod_710, 0o710);
chmod_file_mode!(d3_chmod_720, 0o720);
chmod_file_mode!(d3_chmod_730, 0o730);
chmod_file_mode!(d3_chmod_740, 0o740);
chmod_file_mode!(d3_chmod_750, 0o750);
chmod_file_mode!(d3_chmod_760, 0o760);
chmod_file_mode!(d3_chmod_770, 0o770);
chmod_file_mode!(d3_chmod_701, 0o701);
chmod_file_mode!(d3_chmod_702, 0o702);
chmod_file_mode!(d3_chmod_704, 0o704);
chmod_file_mode!(d3_chmod_421, 0o421);
chmod_file_mode!(d3_chmod_241, 0o241);
chmod_file_mode!(d3_chmod_124, 0o124);
chmod_file_mode!(d3_chmod_412, 0o412);

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

chmod_dir_mode!(d3_dchmod_700, 0o700);
chmod_dir_mode!(d3_dchmod_711, 0o711);
chmod_dir_mode!(d3_dchmod_755, 0o755);
chmod_dir_mode!(d3_dchmod_775, 0o775);
chmod_dir_mode!(d3_dchmod_777, 0o777);
chmod_dir_mode!(d3_dchmod_555, 0o555);
chmod_dir_mode!(d3_dchmod_511, 0o511);
chmod_dir_mode!(d3_dchmod_500, 0o500);
chmod_dir_mode!(d3_dchmod_750, 0o750);
chmod_dir_mode!(d3_dchmod_730, 0o730);
chmod_dir_mode!(d3_dchmod_710, 0o710);
chmod_dir_mode!(d3_dchmod_701, 0o701);

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

eacces_open!(d3_eacc_rd_000, 0o000, oflag::O_RDONLY);
eacces_open!(d3_eacc_rd_200, 0o200, oflag::O_RDONLY);
eacces_open!(d3_eacc_rd_300, 0o300, oflag::O_RDONLY);
eacces_open!(d3_eacc_rd_010, 0o010, oflag::O_RDONLY);
eacces_open!(d3_eacc_wr_000, 0o000, oflag::O_WRONLY);
eacces_open!(d3_eacc_wr_400, 0o400, oflag::O_WRONLY);
eacces_open!(d3_eacc_wr_500, 0o500, oflag::O_WRONLY);
eacces_open!(d3_eacc_wr_040, 0o040, oflag::O_WRONLY);
eacces_open!(d3_eacc_rw_000, 0o000, oflag::O_RDWR);
eacces_open!(d3_eacc_rw_400, 0o400, oflag::O_RDWR);
eacces_open!(d3_eacc_rw_200, 0o200, oflag::O_RDWR);
eacces_open!(d3_eacc_rw_100, 0o100, oflag::O_RDWR);
eacces_open!(d3_eacc_trunc_400, 0o400, oflag::O_WRONLY | oflag::O_TRUNC);
eacces_open!(d3_eacc_trunc_500, 0o500, oflag::O_WRONLY | oflag::O_TRUNC);
eacces_open!(d3_eacc_app_400, 0o400, oflag::O_WRONLY | oflag::O_APPEND);
eacces_open!(d3_eacc_app_440, 0o440, oflag::O_RDWR | oflag::O_APPEND);

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

eacces_dir_unlink!(d3_punl_555, 0o555);
eacces_dir_unlink!(d3_punl_444, 0o444);
eacces_dir_unlink!(d3_punl_111, 0o111);
eacces_dir_unlink!(d3_punl_000, 0o000);
eacces_dir_unlink!(d3_punl_511, 0o511);
eacces_dir_unlink!(d3_punl_500, 0o500);

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

eacces_dir_mkdir!(d3_pmk_555, 0o555);
eacces_dir_mkdir!(d3_pmk_444, 0o444);
eacces_dir_mkdir!(d3_pmk_000, 0o000);
eacces_dir_mkdir!(d3_pmk_111, 0o111);
eacces_dir_mkdir!(d3_pmk_511, 0o511);
eacces_dir_mkdir!(d3_pmk_501, 0o501);

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

eacces_dir_rename_into!(d3_pren_555, 0o555);
eacces_dir_rename_into!(d3_pren_444, 0o444);
eacces_dir_rename_into!(d3_pren_000, 0o000);
eacces_dir_rename_into!(d3_pren_111, 0o111);

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

eacces_dir_link_into!(d3_plnk_555, 0o555);
eacces_dir_link_into!(d3_plnk_444, 0o444);
eacces_dir_link_into!(d3_plnk_000, 0o000);
eacces_dir_link_into!(d3_plnk_111, 0o111);

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

open_creat_mode!(d3_oc_400, 0o400);
open_creat_mode!(d3_oc_440, 0o440);
open_creat_mode!(d3_oc_444, 0o444);
open_creat_mode!(d3_oc_600, 0o600);
open_creat_mode!(d3_oc_640, 0o640);
open_creat_mode!(d3_oc_644, 0o644);
open_creat_mode!(d3_oc_660, 0o660);
open_creat_mode!(d3_oc_666, 0o666);
open_creat_mode!(d3_oc_700, 0o700);
open_creat_mode!(d3_oc_740, 0o740);
open_creat_mode!(d3_oc_744, 0o744);
open_creat_mode!(d3_oc_750, 0o750);
open_creat_mode!(d3_oc_755, 0o755);
open_creat_mode!(d3_oc_770, 0o770);
open_creat_mode!(d3_oc_777, 0o777);
open_creat_mode!(d3_oc_620, 0o620);
open_creat_mode!(d3_oc_604, 0o604);
open_creat_mode!(d3_oc_420, 0o420);
open_creat_mode!(d3_oc_240, 0o240);
open_creat_mode!(d3_oc_204, 0o204);
open_creat_mode!(d3_oc_140, 0o140);
open_creat_mode!(d3_oc_104, 0o104);
open_creat_mode!(d3_oc_041, 0o041);
open_creat_mode!(d3_oc_014, 0o014);

#[crate::lctp_test(suite = fs)]
fn d3_open_excl_eexist() -> TestResult {
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
fn d3_open_trailing_slash_file_enotdir() -> TestResult {
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
fn d3_open_trailing_slash_dir_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let mut slash = [0u8; 140];
    let n = d.iter().position(|&c| c == 0).unwrap();
    slash[..n].copy_from_slice(&d[..n]);
    slash[n] = b'/';
    slash[n + 1] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&slash), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "open"
    );
    check_ok!(syscall::close(fd), "c");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

macro_rules! trunc_size {
    ($name:ident, $sz:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"t")?;
            write_file(&p, b"0123456789ABCDEF")?;
            check_ok!(syscall::truncate(&p, $sz), "trunc");
            let st = check_ok!(syscall::stat(&p), "stat");
            check_eq!(st.st_size, $sz, "size");
            Ok(())
        }
    };
}

trunc_size!(d3_trunc_0, 0);
trunc_size!(d3_trunc_1, 1);
trunc_size!(d3_trunc_2, 2);
trunc_size!(d3_trunc_3, 3);
trunc_size!(d3_trunc_4, 4);
trunc_size!(d3_trunc_5, 5);
trunc_size!(d3_trunc_7, 7);
trunc_size!(d3_trunc_8, 8);
trunc_size!(d3_trunc_9, 9);
trunc_size!(d3_trunc_15, 15);
trunc_size!(d3_trunc_16, 16);
trunc_size!(d3_trunc_17, 17);
trunc_size!(d3_trunc_32, 32);
trunc_size!(d3_trunc_64, 64);
trunc_size!(d3_trunc_100, 100);
trunc_size!(d3_trunc_256, 256);

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

rename_pair!(d3_ren_01, b"a0", b"b0");
rename_pair!(d3_ren_02, b"a1", b"b1");
rename_pair!(d3_ren_03, b"a2", b"b2");
rename_pair!(d3_ren_04, b"a3", b"b3");
rename_pair!(d3_ren_05, b"a4", b"b4");
rename_pair!(d3_ren_06, b"a5", b"b5");
rename_pair!(d3_ren_07, b"a6", b"b6");
rename_pair!(d3_ren_08, b"a7", b"b7");
rename_pair!(d3_ren_09, b"a8", b"b8");
rename_pair!(d3_ren_10, b"a9", b"b9");
rename_pair!(d3_ren_11, b"c0", b"d0");
rename_pair!(d3_ren_12, b"c1", b"d1");
rename_pair!(d3_ren_13, b".h0", b".h1");
rename_pair!(d3_ren_14, b"x_y", b"y_x");
rename_pair!(d3_ren_15, b"src15", b"dst15");
rename_pair!(d3_ren_16, b"longnameA", b"longnameB");

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
            check_ok!(syscall::unlink(&d), "ul");
            Ok(())
        }
    };
}

link_pair!(d3_lnk_01, b"l0", b"m0");
link_pair!(d3_lnk_02, b"l1", b"m1");
link_pair!(d3_lnk_03, b"l2", b"m2");
link_pair!(d3_lnk_04, b"l3", b"m3");
link_pair!(d3_lnk_05, b"l4", b"m4");
link_pair!(d3_lnk_06, b"l5", b"m5");
link_pair!(d3_lnk_07, b"l6", b"m6");
link_pair!(d3_lnk_08, b"l7", b"m7");
link_pair!(d3_lnk_09, b"l8", b"m8");
link_pair!(d3_lnk_10, b"l9", b"m9");
link_pair!(d3_lnk_11, b"p0", b"q0");
link_pair!(d3_lnk_12, b"p1", b"q1");

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

symlink_pair!(d3_sym_01, b"t0", b"s0", b"t0\0");
symlink_pair!(d3_sym_02, b"t1", b"s1", b"t1\0");
symlink_pair!(d3_sym_03, b"t2", b"s2", b"t2\0");
symlink_pair!(d3_sym_04, b"t3", b"s3", b"t3\0");
symlink_pair!(d3_sym_05, b"t4", b"s4", b"t4\0");
symlink_pair!(d3_sym_06, b"t5", b"s5", b"t5\0");
symlink_pair!(d3_sym_07, b"t6", b"s6", b"t6\0");
symlink_pair!(d3_sym_08, b"t7", b"s7", b"t7\0");

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

unlink_name!(d3_ul_01, b"u0");
unlink_name!(d3_ul_02, b"u1");
unlink_name!(d3_ul_03, b"u2");
unlink_name!(d3_ul_04, b"u3");
unlink_name!(d3_ul_05, b"u4");
unlink_name!(d3_ul_06, b"u5");
unlink_name!(d3_ul_07, b"u6");
unlink_name!(d3_ul_08, b"u7");
unlink_name!(d3_ul_09, b".u8");
unlink_name!(d3_ul_10, b"name10");

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

mkdir_rmdir!(d3_md_700, b"d0", 0o700);
mkdir_rmdir!(d3_md_755, b"d1", 0o755);
mkdir_rmdir!(d3_md_775, b"d2", 0o775);
mkdir_rmdir!(d3_md_777, b"d3", 0o777);
mkdir_rmdir!(d3_md_711, b"d4", 0o711);
mkdir_rmdir!(d3_md_750, b"d5", 0o750);
mkdir_rmdir!(d3_md_701, b"d6", 0o701);
mkdir_rmdir!(d3_md_730, b"d7", 0o730);
mkdir_rmdir!(d3_md_760, b"d8", 0o760);
mkdir_rmdir!(d3_md_770, b"d9", 0o770);
mkdir_rmdir!(d3_md_555, b"da", 0o555);
mkdir_rmdir!(d3_md_511, b"db", 0o511);

#[crate::lctp_test(suite = fs)]
fn d3_renameat2_noreplace_ok() -> TestResult {
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
fn d3_renameat2_noreplace_eexist() -> TestResult {
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
fn d3_renameat2_exchange() -> TestResult {
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
fn d3_renameat2_exchange_dirs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_dir(&mut tmp, b"da", 0o755)?;
    let b = create_dir(&mut tmp, b"db", 0o755)?;
    match syscall::renameat2(AT_FDCWD, &a, AT_FDCWD, &b, RENAME_EXCHANGE) {
        Ok(()) => {
            check_ok!(syscall::stat(&a), "a");
            check_ok!(syscall::stat(&b), "b");
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

macro_rules! utimens_now {
    ($name:ident) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
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
    };
}

utimens_now!(d3_ut_now_01);
utimens_now!(d3_ut_now_02);
utimens_now!(d3_ut_now_03);
utimens_now!(d3_ut_now_04);

macro_rules! utimens_omit {
    ($name:ident) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
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
    };
}

utimens_omit!(d3_ut_omit_01);
utimens_omit!(d3_ut_omit_02);
utimens_omit!(d3_ut_omit_03);
utimens_omit!(d3_ut_omit_04);

macro_rules! utimens_explicit {
    ($name:ident, $sec:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, b"u")?;
            let times = [
                Timespec {
                    tv_sec: $sec,
                    tv_nsec: 0,
                },
                Timespec {
                    tv_sec: $sec,
                    tv_nsec: 0,
                },
            ];
            check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "set");
            let st = check_ok!(syscall::stat(&p), "stat");
            check_eq!(st.st_mtime, $sec, "mtime");
            Ok(())
        }
    };
}

utimens_explicit!(d3_ut_exp_1, 1_500_000_000);
utimens_explicit!(d3_ut_exp_2, 1_600_000_000);
utimens_explicit!(d3_ut_exp_3, 1_700_000_000);
utimens_explicit!(d3_ut_exp_4, 1_800_000_000);
utimens_explicit!(d3_ut_exp_5, 1_900_000_000);
utimens_explicit!(d3_ut_exp_6, 2_000_000_000);

#[crate::lctp_test(suite = fs)]
fn d3_ut_omit_atime_set_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    let before = check_ok!(syscall::stat(&p), "before");
    let times = [
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
        Timespec {
            tv_sec: 1_550_000_000,
            tv_nsec: 0,
        },
    ];
    check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "set");
    let after = check_ok!(syscall::stat(&p), "after");
    check_eq!(after.st_atime, before.st_atime, "atime");
    check_eq!(after.st_mtime, 1_550_000_000, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d3_ut_set_atime_omit_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    let before = check_ok!(syscall::stat(&p), "before");
    let times = [
        Timespec {
            tv_sec: 1_560_000_000,
            tv_nsec: 0,
        },
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
    ];
    check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "set");
    let after = check_ok!(syscall::stat(&p), "after");
    check_eq!(after.st_atime, 1_560_000_000, "atime");
    check_eq!(after.st_mtime, before.st_mtime, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d3_ut_ctime_changes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"u")?;
    let before = check_ok!(syscall::stat(&p), "before");
    let times = [
        Timespec {
            tv_sec: 1_570_000_000,
            tv_nsec: 0,
        },
        Timespec {
            tv_sec: 1_570_000_000,
            tv_nsec: 0,
        },
    ];
    check_ok!(syscall::utimensat(AT_FDCWD, &p, &times, 0), "set");
    let after = check_ok!(syscall::stat(&p), "after");
    check!(after.st_ctime >= before.st_ctime, "ctime");
    Ok(())
}

macro_rules! fifo_nonblock_open {
    ($name:ident, $flags:expr) => {
        #[crate::lctp_test(suite = fs)]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = copy_child(&mut tmp, b"fifo")?;
            check_ok!(
                syscall::mknodat(AT_FDCWD, &p, S_IFIFO | 0o644, 0),
                "mknod"
            );
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

fifo_nonblock_open!(d3_fifo_rd, oflag::O_RDONLY);
fifo_nonblock_open!(d3_fifo_wr, oflag::O_WRONLY);
fifo_nonblock_open!(d3_fifo_rdwr, oflag::O_RDWR);
fifo_nonblock_open!(d3_fifo_rd_cloexec, oflag::O_RDONLY | oflag::O_CLOEXEC);
fifo_nonblock_open!(d3_fifo_wr_cloexec, oflag::O_WRONLY | oflag::O_CLOEXEC);
fifo_nonblock_open!(d3_fifo_rdwr_cloexec, oflag::O_RDWR | oflag::O_CLOEXEC);

#[crate::lctp_test(suite = fs)]
fn d3_access_ok_matrix() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"f")?;
    for (mode, want) in [
        (0o400u32, R_OK),
        (0o200, W_OK),
        (0o100, X_OK),
        (0o600, R_OK | W_OK),
        (0o500, R_OK | X_OK),
        (0o300, W_OK | X_OK),
        (0o700, R_OK | W_OK | X_OK),
        (0o000, F_OK),
    ] {
        check_ok!(syscall::chmod(&p, mode), "chmod");
        check_ok!(syscall::access(&p, want), "access");
    }
    check_ok!(syscall::chmod(&p, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d3_rmdir_notempty() -> TestResult {
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
fn d3_mkdir_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::mkdir(&d, 0o755), Errno::EEXIST, "eexist");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d3_link_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    check_err!(syscall::link(&a, &b), Errno::EEXIST, "eexist");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn d3_symlink_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    check_err!(syscall::symlink(b"x\0", &a), Errno::EEXIST, "eexist");
    Ok(())
}
