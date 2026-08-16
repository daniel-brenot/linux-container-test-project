//! Permission EACCES matrices, rename/link/unlink edges,
//! open flag combos, and FIFO modes.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{
    copy_child, create_dir, create_empty, join_path, truncate_cstr, write_file,
};
use crate::syscall::{self, oflag, Errno, AT_FDCWD, F_OK, R_OK, S_IFIFO, W_OK, X_OK};

macro_rules! eacces_open_after_chmod {
    ($name:ident, $mode:expr, $flags:expr) => {
        #[crate::lctp_test(suite = fs, expect = failure, case = "open after chmod without the required permission bits returns EACCES")]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $mode), "chmod");
            check_err!(syscall::open(&path, $flags, 0), Errno::EACCES, "eacces");
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

// No read: reject O_RDONLY / O_RDWR
eacces_open_after_chmod!(d2_open_rd_eacces_200, 0o200, oflag::O_RDONLY);
eacces_open_after_chmod!(d2_open_rd_eacces_000, 0o000, oflag::O_RDONLY);
eacces_open_after_chmod!(d2_open_rd_eacces_100, 0o100, oflag::O_RDONLY);
eacces_open_after_chmod!(d2_open_rdwr_eacces_200, 0o200, oflag::O_RDWR);
eacces_open_after_chmod!(d2_open_rdwr_eacces_400, 0o400, oflag::O_RDWR);
eacces_open_after_chmod!(d2_open_rdwr_eacces_000, 0o000, oflag::O_RDWR);
// No write: reject O_WRONLY / O_RDWR / O_TRUNC append write paths
eacces_open_after_chmod!(d2_open_wr_eacces_400, 0o400, oflag::O_WRONLY);
eacces_open_after_chmod!(d2_open_wr_eacces_500, 0o500, oflag::O_WRONLY);
eacces_open_after_chmod!(d2_open_wr_eacces_000, 0o000, oflag::O_WRONLY);
eacces_open_after_chmod!(d2_open_trunc_eacces_400, 0o400, oflag::O_WRONLY | oflag::O_TRUNC);
eacces_open_after_chmod!(d2_open_append_eacces_400, 0o400, oflag::O_WRONLY | oflag::O_APPEND);
eacces_open_after_chmod!(d2_open_append_eacces_440, 0o440, oflag::O_RDWR | oflag::O_APPEND);

macro_rules! eacces_access_mode {
    ($name:ident, $chmod:expr, $want:expr) => {
        #[crate::lctp_test(suite = fs, expect = failure, case = "access with bits not granted by the file mode returns EACCES")]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $chmod), "chmod");
            check_err!(syscall::access(&path, $want), Errno::EACCES, "eacces");
            check_ok!(syscall::chmod(&path, 0o644), "restore");
            Ok(())
        }
    };
}

eacces_access_mode!(d2_acc_r_vs_200, 0o200, R_OK);
eacces_access_mode!(d2_acc_r_vs_300, 0o300, R_OK);
eacces_access_mode!(d2_acc_r_vs_100, 0o100, R_OK);
eacces_access_mode!(d2_acc_r_vs_000, 0o000, R_OK);
eacces_access_mode!(d2_acc_w_vs_400, 0o400, W_OK);
eacces_access_mode!(d2_acc_w_vs_500, 0o500, W_OK);
eacces_access_mode!(d2_acc_w_vs_100, 0o100, W_OK);
eacces_access_mode!(d2_acc_w_vs_000, 0o000, W_OK);
eacces_access_mode!(d2_acc_x_vs_600, 0o600, X_OK);
eacces_access_mode!(d2_acc_x_vs_640, 0o640, X_OK);
eacces_access_mode!(d2_acc_x_vs_200, 0o200, X_OK);
eacces_access_mode!(d2_acc_x_vs_000, 0o000, X_OK);
eacces_access_mode!(d2_acc_rw_vs_400, 0o400, R_OK | W_OK);
eacces_access_mode!(d2_acc_rw_vs_200, 0o200, R_OK | W_OK);
eacces_access_mode!(d2_acc_rx_vs_600, 0o600, R_OK | X_OK);
eacces_access_mode!(d2_acc_wx_vs_400, 0o400, W_OK | X_OK);
eacces_access_mode!(d2_acc_rwx_vs_600, 0o600, R_OK | W_OK | X_OK);
eacces_access_mode!(d2_acc_rwx_vs_000, 0o000, R_OK | W_OK | X_OK);

macro_rules! eacces_dir_create {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = failure, case = "creating a file in a directory without write permission returns EACCES")]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let dir = create_dir(&mut tmp, b"d", 0o755)?;
            check_ok!(syscall::chmod(&dir, $mode), "chmod");
            let mut child = [0u8; 160];
            let p = join_path(&dir, b"x", &mut child)?;
            check_err!(
                syscall::open(p, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
                Errno::EACCES,
                "creat"
            );
            check_ok!(syscall::chmod(&dir, 0o755), "restore");
            check_ok!(syscall::rmdir(&dir), "rmdir");
            Ok(())
        }
    };
}

eacces_dir_create!(d2_dir_creat_eacces_555, 0o555);
eacces_dir_create!(d2_dir_creat_eacces_444, 0o444);
eacces_dir_create!(d2_dir_creat_eacces_111, 0o111);
eacces_dir_create!(d2_dir_creat_eacces_000, 0o000);

macro_rules! eacces_dir_unlink {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = failure, case = "unlink in a directory without write permission returns EACCES")]
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

eacces_dir_unlink!(d2_dir_unlink_eacces_555, 0o555);
eacces_dir_unlink!(d2_dir_unlink_eacces_555x, 0o555);
eacces_dir_unlink!(d2_dir_unlink_eacces_444, 0o444);
eacces_dir_unlink!(d2_dir_unlink_eacces_111, 0o111);

macro_rules! eacces_dir_rename_into {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = failure, case = "rename into a directory without write permission returns EACCES")]
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

eacces_dir_rename_into!(d2_dir_rename_eacces_555, 0o555);
eacces_dir_rename_into!(d2_dir_rename_eacces_444, 0o444);
eacces_dir_rename_into!(d2_dir_rename_eacces_000, 0o000);

macro_rules! eacces_dir_link_into {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = failure, case = "link into a directory without write permission returns EACCES")]
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

eacces_dir_link_into!(d2_dir_link_eacces_555, 0o555);
eacces_dir_link_into!(d2_dir_link_eacces_444, 0o444);
eacces_dir_link_into!(d2_dir_link_eacces_111, 0o111);

macro_rules! eacces_dir_mkdir {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = failure, case = "mkdir in a directory without write permission returns EACCES")]
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

eacces_dir_mkdir!(d2_dir_mkdir_eacces_555, 0o555);
eacces_dir_mkdir!(d2_dir_mkdir_eacces_444, 0o444);
eacces_dir_mkdir!(d2_dir_mkdir_eacces_000, 0o000);

macro_rules! rename_edge {
    ($name:ident, $src_name:expr, $dst_name:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = "rename of a regular file to a new name succeeds and removes the old path")]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let src = create_empty(&mut tmp, $src_name)?;
            write_file(&src, b"data")?;
            let dst = copy_child(&mut tmp, $dst_name)?;
            check_ok!(syscall::rename(&src, &dst), "rename");
            check_err!(syscall::stat(&src), Errno::ENOENT, "gone");
            check_ok!(syscall::stat(&dst), "there");
            Ok(())
        }
    };
}

rename_edge!(d2_ren_a_to_b, b"a", b"b");
rename_edge!(d2_ren_foo_to_bar, b"foo", b"bar");
rename_edge!(d2_ren_x1_to_x2, b"x1", b"x2");
rename_edge!(d2_ren_longish_names, b"srcname01", b"dstname02");
rename_edge!(d2_ren_dot_prefix, b".hidden", b".moved");
rename_edge!(d2_ren_num, b"n1", b"n2");
rename_edge!(d2_ren_u_to_v, b"u", b"v");
rename_edge!(d2_ren_p_to_q, b"p", b"q");

#[crate::lctp_test(suite = fs, expect = success, case = "rename over an existing file replaces it with the source contents")]
fn d2_rename_replace_same_content_check() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"ra")?;
    let b = create_empty(&mut tmp, b"rb")?;
    write_file(&a, b"AAA")?;
    write_file(&b, b"BBB")?;
    check_ok!(syscall::rename(&a, &b), "ren");
    let mut buf = [0u8; 3];
    let fd = check_ok!(syscall::open(&b, oflag::O_RDONLY, 0), "o");
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "r"), 3, "n");
    check_eq!(&buf, b"AAA", "data");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a directory onto an empty directory succeeds")]
fn d2_rename_dir_over_empty_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_dir(&mut tmp, b"da", 0o755)?;
    let b = create_dir(&mut tmp, b"db", 0o755)?;
    check_ok!(syscall::rename(&a, &b), "ren");
    check_err!(syscall::stat(&a), Errno::ENOENT, "gone");
    let st = check_ok!(syscall::stat(&b), "stat");
    check!(st.is_dir(), "dir");
    check_ok!(syscall::rmdir(&b), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename of a missing source path returns ENOENT")]
fn d2_rename_enoent_src() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let dst = copy_child(&mut tmp, b"dst")?;
    check_err!(
        syscall::rename(b"/tmp/lctp-no-ren-src\0", &dst),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a file onto itself succeeds and the path remains")]
fn d2_rename_to_self() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = create_empty(&mut tmp, b"self")?;
    write_file(&p, b"x")?;
    check_ok!(syscall::rename(&p, &p), "self");
    check_ok!(syscall::stat(&p), "still");
    Ok(())
}

macro_rules! link_edge {
    ($name:ident, $old:expr, $new:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = "link creates a second name that shares the source inode")]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let src = create_empty(&mut tmp, $old)?;
            write_file(&src, b"L")?;
            let dst = copy_child(&mut tmp, $new)?;
            check_ok!(syscall::link(&src, &dst), "link");
            let sa = check_ok!(syscall::stat(&src), "sa");
            let sb = check_ok!(syscall::stat(&dst), "sb");
            check_eq!(sa.st_ino, sb.st_ino, "ino");
            check!(sa.st_nlink >= 2, "nlink");
            check_ok!(syscall::unlink(&dst), "ul");
            Ok(())
        }
    };
}

link_edge!(d2_link_a_b, b"la", b"lb");
link_edge!(d2_link_f1_f2, b"f1", b"f2");
link_edge!(d2_link_one_two, b"one", b"two");
link_edge!(d2_link_alpha_beta, b"alpha", b"beta");
link_edge!(d2_link_s_t, b"s", b"t");
link_edge!(d2_link_m_n, b"m", b"n");
link_edge!(d2_link_i_j, b"i", b"j");
link_edge!(d2_link_c_d, b"c", b"d");

#[crate::lctp_test(suite = fs, expect = failure, case = "link onto an existing path returns EEXIST")]
fn d2_link_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    check_err!(syscall::link(&a, &b), Errno::EEXIST, "eexist");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "link with a missing source path returns ENOENT")]
fn d2_link_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let dst = copy_child(&mut tmp, b"dst")?;
    check_err!(
        syscall::link(b"/tmp/lctp-no-link-src\0", &dst),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "two extra hard links raise nlink to at least 3")]
fn d2_link_three_names() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    write_file(&a, b"Z")?;
    let b = copy_child(&mut tmp, b"b")?;
    let c = copy_child(&mut tmp, b"c")?;
    check_ok!(syscall::link(&a, &b), "b");
    check_ok!(syscall::link(&a, &c), "c");
    let st = check_ok!(syscall::stat(&a), "st");
    check!(st.st_nlink >= 3, "nlink3");
    check_ok!(syscall::unlink(&b), "ub");
    check_ok!(syscall::unlink(&c), "uc");
    Ok(())
}

macro_rules! unlink_edge {
    ($name:ident, $nm:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = "unlink of a regular file succeeds and the path is gone")]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let p = create_empty(&mut tmp, $nm)?;
            write_file(&p, b"u")?;
            check_ok!(syscall::unlink(&p), "ul");
            check_err!(syscall::stat(&p), Errno::ENOENT, "gone");
            Ok(())
        }
    };
}

unlink_edge!(d2_ul_a, b"a");
unlink_edge!(d2_ul_file, b"file");
unlink_edge!(d2_ul_tmp1, b"tmp1");
unlink_edge!(d2_ul_x, b"x");
unlink_edge!(d2_ul_y, b"y");
unlink_edge!(d2_ul_z, b"z");
unlink_edge!(d2_ul_name01, b"name01");
unlink_edge!(d2_ul_dotfile, b".u");

#[crate::lctp_test(suite = fs, expect = failure, case = "unlink of a missing path returns ENOENT")]
fn d2_unlink_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let p = copy_child(&mut tmp, b"missing")?;
    check_err!(syscall::unlink(&p), Errno::ENOENT, "enoent");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "unlink of a directory returns EISDIR or EPERM")]
fn d2_unlink_dir_eisdir_or_eperm() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    match syscall::unlink(&d) {
        Err(Errno::EISDIR) | Err(Errno::EPERM) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("unexpected ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("unlink dir")),
    }
    check_ok!(syscall::rmdir(&d), "rmdir");
    Ok(())
}

macro_rules! open_flag_combo {
    ($name:ident, $flags:expr, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT|O_EXCL and the requested flags succeeds")]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = copy_child(&mut tmp, b"of")?;
            let fd = check_ok!(syscall::open(&path, $flags, $mode), "open");
            check_ok!(syscall::close(fd), "c");
            Ok(())
        }
    };
}

open_flag_combo!(
    d2_open_creat_rdwr,
    oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL,
    0o644
);
open_flag_combo!(
    d2_open_creat_wronly,
    oflag::O_WRONLY | oflag::O_CREAT | oflag::O_EXCL,
    0o600
);
open_flag_combo!(
    d2_open_creat_rdonly,
    oflag::O_RDONLY | oflag::O_CREAT | oflag::O_EXCL,
    0o444
);
open_flag_combo!(
    d2_open_creat_cloexec,
    oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL | oflag::O_CLOEXEC,
    0o644
);
open_flag_combo!(
    d2_open_creat_nonblock,
    oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL | oflag::O_NONBLOCK,
    0o644
);
open_flag_combo!(
    d2_open_creat_noctty,
    oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL | oflag::O_NOCTTY,
    0o644
);
open_flag_combo!(
    d2_open_creat_append,
    oflag::O_WRONLY | oflag::O_CREAT | oflag::O_EXCL | oflag::O_APPEND,
    0o644
);

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_TRUNC preserves the inode and sets size 0")]
fn d2_open_trunc_preserves_inode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"t")?;
    write_file(&path, b"abcdef")?;
    let ino = check_ok!(syscall::stat(&path), "s").st_ino;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR | oflag::O_TRUNC, 0), "o");
    check_ok!(syscall::close(fd), "c");
    let st = check_ok!(syscall::stat(&path), "s2");
    check_eq!(st.st_ino, ino, "ino");
    check_eq!(st.st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open with O_CREAT|O_EXCL on an existing file returns EEXIST")]
fn d2_open_excl_eexist_modes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"e")?;
    for mode in [0o644u32, 0o600, 0o666, 0o700] {
        check_err!(
            syscall::open(
                &path,
                oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL,
                mode
            ),
            Errno::EEXIST,
            "eexist"
        );
    }
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open of a directory with O_DIRECTORY succeeds")]
fn d2_open_directory_flag() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let d = create_dir(&mut tmp, b"d", 0o755)?;
    let fd = check_ok!(syscall::open(&d, oflag::O_RDONLY | oflag::O_DIRECTORY, 0), "o");
    check_ok!(syscall::close(fd), "c");
    check_ok!(syscall::rmdir(&d), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a regular file with O_DIRECTORY returns ENOTDIR")]
fn d2_open_directory_on_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = soft, case = "open with O_PATH succeeds when the interface is supported")]
fn d2_open_path_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"f")?;
    match syscall::open(&path, oflag::O_PATH, 0) {
        Ok(fd) => check_ok!(syscall::close(fd), "c"),
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("opath")),
    }
    Ok(())
}

macro_rules! mkfifo_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = concat!("mknodat creates a FIFO with mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "t");
            let path = copy_child(&mut tmp, b"fifo")?;
            check_ok!(
                syscall::mknodat(AT_FDCWD, &path, S_IFIFO | ($mode & 0o777), 0),
                "mknod"
            );
            check_ok!(syscall::chmod(&path, $mode & 0o777), "chmod");
            let st = check_ok!(syscall::stat(&path), "stat");
            check!(st.is_fifo(), "fifo");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            check_ok!(syscall::unlink(&path), "ul");
            Ok(())
        }
    };
}

mkfifo_mode!(d2_fifo_000, 0o000);
mkfifo_mode!(d2_fifo_100, 0o100);
mkfifo_mode!(d2_fifo_200, 0o200);
mkfifo_mode!(d2_fifo_300, 0o300);
mkfifo_mode!(d2_fifo_400, 0o400);
mkfifo_mode!(d2_fifo_500, 0o500);
mkfifo_mode!(d2_fifo_600, 0o600);
mkfifo_mode!(d2_fifo_700, 0o700);
mkfifo_mode!(d2_fifo_644, 0o644);
mkfifo_mode!(d2_fifo_664, 0o664);
mkfifo_mode!(d2_fifo_666, 0o666);
mkfifo_mode!(d2_fifo_755, 0o755);
mkfifo_mode!(d2_fifo_775, 0o775);
mkfifo_mode!(d2_fifo_777, 0o777);
mkfifo_mode!(d2_fifo_440, 0o440);
mkfifo_mode!(d2_fifo_220, 0o220);
mkfifo_mode!(d2_fifo_111, 0o111);
mkfifo_mode!(d2_fifo_555, 0o555);
mkfifo_mode!(d2_fifo_711, 0o711);
mkfifo_mode!(d2_fifo_733, 0o733);

#[crate::lctp_test(suite = fs, expect = failure, case = "mknodat of an existing FIFO returns EEXIST")]
fn d2_fifo_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "m1"
    );
    check_err!(
        syscall::mknodat(AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        Errno::EEXIST,
        "eexist"
    );
    check_ok!(syscall::unlink(&path), "ul");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "access F_OK on a newly created FIFO succeeds")]
fn d2_fifo_stat_f_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(AT_FDCWD, &path, S_IFIFO | 0o600, 0),
        "m"
    );
    check_ok!(syscall::access(&path, F_OK), "fok");
    check_ok!(syscall::unlink(&path), "ul");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "unlinking and recreating a FIFO three times succeeds")]
fn d2_fifo_unlink_then_recreate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = copy_child(&mut tmp, b"fifo")?;
    for _ in 0..3 {
        check_ok!(
            syscall::mknodat(AT_FDCWD, &path, S_IFIFO | 0o644, 0),
            "m"
        );
        check_ok!(syscall::unlink(&path), "ul");
    }
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open succeeds after chmod grants the matching access bits")]
fn d2_chmod_then_open_ok_matrix() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"f")?;
    for (mode, flags) in [
        (0o400u32, oflag::O_RDONLY),
        (0o200, oflag::O_WRONLY),
        (0o600, oflag::O_RDWR),
        (0o644, oflag::O_RDONLY),
        (0o666, oflag::O_RDWR),
    ] {
        check_ok!(syscall::chmod(&path, mode), "chmod");
        let fd = check_ok!(syscall::open(&path, flags, 0), "open");
        check_ok!(syscall::close(fd), "c");
    }
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "faccessat with bits not granted by the file mode returns EACCES")]
fn d2_faccessat_eacces_matrix() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let path = create_empty(&mut tmp, b"f")?;
    for (mode, want) in [
        (0o200u32, R_OK),
        (0o400, W_OK),
        (0o600, X_OK),
        (0o000, R_OK | W_OK),
    ] {
        check_ok!(syscall::chmod(&path, mode), "chmod");
        check_err!(
            syscall::faccessat(AT_FDCWD, &path, want, 0),
            Errno::EACCES,
            "eacces"
        );
    }
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a file into a subdirectory succeeds")]
fn d2_rename_across_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let sub = create_dir(&mut tmp, b"sub", 0o755)?;
    let file = create_empty(&mut tmp, b"f")?;
    write_file(&file, b"M")?;
    let mut dest = [0u8; 160];
    let dst = join_path(&sub, b"f", &mut dest)?;
    check_ok!(syscall::rename(&file, dst), "ren");
    check_ok!(syscall::unlink(dst), "ul");
    check_ok!(syscall::rmdir(&sub), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "link of a file into a subdirectory succeeds")]
fn d2_link_across_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let sub = create_dir(&mut tmp, b"sub", 0o755)?;
    let file = create_empty(&mut tmp, b"f")?;
    write_file(&file, b"L")?;
    let mut dest = [0u8; 160];
    let dst = join_path(&sub, b"l", &mut dest)?;
    check_ok!(syscall::link(&file, dst), "link");
    check_ok!(syscall::unlink(dst), "ul");
    check_ok!(syscall::rmdir(&sub), "rm");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "unlink of the last remaining hard-link name removes the path")]
fn d2_unlink_last_link_removes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    check_ok!(syscall::unlink(&a), "ua");
    check_ok!(syscall::stat(&b), "b remains");
    check_ok!(syscall::unlink(&b), "ub");
    check_err!(syscall::stat(&b), Errno::ENOENT, "gone");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open O_CREAT then chmod sets each requested file mode")]
fn d2_open_creat_umask_force_mode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "t");
    for mode in [0o640u32, 0o620, 0o604, 0o444, 0o222, 0o111, 0o750, 0o740] {
        let mut name = [b'm', b'0' + ((mode >> 6) & 7) as u8, b'0' + ((mode >> 3) & 7) as u8, b'0' + (mode & 7) as u8, 0];
        // unique-ish name bytes
        name[1] = b'a' + ((mode >> 6) & 7) as u8;
        let path = copy_child(&mut tmp, &name)?;
        let fd = check_ok!(
            syscall::open(
                &path,
                oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL,
                mode
            ),
            "creat"
        );
        check_ok!(syscall::close(fd), "c");
        check_ok!(syscall::chmod(&path, mode & 0o777), "chmod");
        let st = check_ok!(syscall::stat(&path), "st");
        check_eq!(st.mode_bits() & 0o777, mode & 0o777, "mode");
    }
    Ok(())
}
