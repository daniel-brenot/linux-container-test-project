//! open/creat filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{
    copy_child, create_dir, create_empty, join_path, nanosleep_secs, read_file, timespec_later,
    truncate_cstr, write_file,
};
use crate::syscall::{self, oflag, Errno, S_IFIFO};

macro_rules! open_creat_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = concat!("open O_CREAT creates a regular file with mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let path = copy_child(&mut tmp, b"m")?;
            let fd = check_ok!(
                syscall::open(
                    &path,
                    oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL,
                    $mode
                ),
                "creat"
            );
            check_ok!(syscall::close(fd), "close");
            // Force exact mode (umask may have cleared bits at creat).
            check_ok!(syscall::chmod(&path, $mode & 0o777), "chmod");
            let st = check_ok!(syscall::stat(&path), "stat");
            check_eq!(st.mode_bits() & 0o777, $mode & 0o777, "mode");
            Ok(())
        }
    };
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT|O_EXCL creates a regular file")]
fn open_creat_new() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"new")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_reg(), "regular");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open with O_CREAT|O_EXCL on an existing file returns EEXIST")]
fn open_excl_fails_if_exists() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "excl"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_TRUNC sets the file size to 0")]
fn open_trunc_zeroes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"long data here")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_APPEND writes after existing data")]
fn open_append_mode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"X")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0), "append");
    check_ok!(syscall::write(fd, b"Y"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "read");
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 2, "len");
    check_eq!(&buf[..2], b"XY", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open of a directory with O_DIRECTORY succeeds")]
fn open_directory_on_dir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "O_DIRECTORY"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a regular file with O_DIRECTORY returns ENOTDIR")]
fn open_directory_on_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        Errno::ENOTDIR,
        "ENOTDIR"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "write on a file opened O_RDONLY returns EBADF")]
fn open_readonly_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "rdonly");
    check_err!(syscall::write(fd, b"x"), Errno::EBADF, "no write");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "read on a file opened O_WRONLY returns EBADF")]
fn open_wronly_no_read() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "wronly");
    check_err!(syscall::read(fd, &mut [0u8; 1]), Errno::EBADF, "no read");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT|O_EXCL creates a file in a subdirectory")]
fn creat_in_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"sub", 0o755)?;
    let mut nested = [0u8; 160];
    let path = join_path(&dir, b"x\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(path), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open without O_TRUNC leaves existing file contents")]
fn open_existing_no_trunc() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"keep")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 8];
    check_eq!(read_file(&path, &mut buf)?, 4, "len");
    check_eq!(&buf[..4], b"keep", "preserved");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_PATH allows fstat and rejects read and write with EBADF")]
fn open_opath_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"opath")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_PATH, 0), "O_PATH");
    check_err!(syscall::read(fd, &mut [0u8; 1]), Errno::EBADF, "no read");
    check_err!(syscall::write(fd, b"x"), Errno::EBADF, "no write");
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check!(st.is_reg(), "reg");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open of a directory with O_PATH reports a directory via fstat")]
fn open_opath_directory() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(tmp.path(), oflag::O_PATH | oflag::O_DIRECTORY, 0),
        "O_PATH dir"
    );
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check!(st.is_dir(), "dir");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "open with O_PATH|O_NOFOLLOW on a symlink reports a symlink via fstat")]
fn open_opath_symlink_nofollow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"tgt")?;
    let link = copy_child(&mut tmp, b"lnk")?;
    check_ok!(syscall::symlink(b"tgt\0", &link), "symlink");
    let fd = check_ok!(
        syscall::open(&link, oflag::O_PATH | oflag::O_NOFOLLOW, 0),
        "O_PATH|O_NOFOLLOW"
    );
    let st = check_ok!(syscall::fstat(fd), "fstat");
    check!(st.is_lnk(), "symlink");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = soft, case = "open with O_TMPFILE succeeds when the filesystem supports it")]
fn open_tmpfile_soft() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    match syscall::open(tmp.path(), oflag::O_TMPFILE | oflag::O_RDWR, 0o600) {
        Ok(fd) => {
            check_ok!(syscall::write(fd, b"tmp"), "write");
            check_ok!(syscall::close(fd), "close");
        }
        Err(Errno::EOPNOTSUPP)
        | Err(Errno::ENOTSUP)
        | Err(Errno::EISDIR)
        | Err(Errno::ENOENT)
        | Err(Errno::EPERM)
        | Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("O_TMPFILE errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT creates a file with mode 0600")]
fn open_creat_mode_bits() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"mode")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o600),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.mode_bits() & 0o777, 0o600, "mode");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_NONBLOCK sets O_NONBLOCK in F_GETFL")]
fn open_nonblock_regular() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"nb")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_NONBLOCK, 0),
        "nonblock"
    );
    let fl = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & oflag::O_NONBLOCK != 0, "O_NONBLOCK");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CLOEXEC sets FD_CLOEXEC in F_GETFD")]
fn open_cloexec_flag() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ce")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_CLOEXEC, 0),
        "cloexec"
    );
    let fl = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFD, 0), "getfd");
    check!(fl & crate::syscall::FD_CLOEXEC as usize != 0, "CLOEXEC");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = failure, case = "open of a nested missing path returns ENOENT")]
fn open_enoent_nested() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let mut path = [0u8; 160];
    let base = tmp.path();
    let blen = base.iter().position(|&c| c == 0).unwrap();
    let suffix = b"/no/such/file";
    check!(blen + suffix.len() + 1 < path.len(), "path fits");
    path[..blen].copy_from_slice(&base[..blen]);
    path[blen..blen + suffix.len()].copy_from_slice(suffix);
    path[blen + suffix.len()] = 0;
    check_err!(
        syscall::open(truncate_cstr(&path), oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "nested missing"
    );
    Ok(())
}

open_creat_mode!(open_creat_mode_000, 0o000);
open_creat_mode!(open_creat_mode_001, 0o001);
open_creat_mode!(open_creat_mode_010, 0o010);
open_creat_mode!(open_creat_mode_100, 0o100);
open_creat_mode!(open_creat_mode_111, 0o111);
open_creat_mode!(open_creat_mode_222, 0o222);
open_creat_mode!(open_creat_mode_444, 0o444);
open_creat_mode!(open_creat_mode_555, 0o555);
open_creat_mode!(open_creat_mode_666, 0o666);
open_creat_mode!(open_creat_mode_700, 0o700);
open_creat_mode!(open_creat_mode_711, 0o711);
open_creat_mode!(open_creat_mode_750, 0o750);
open_creat_mode!(open_creat_mode_755, 0o755);
open_creat_mode!(open_creat_mode_764, 0o764);
open_creat_mode!(open_creat_mode_775, 0o775);
open_creat_mode!(open_creat_mode_777, 0o777);
open_creat_mode!(open_creat_mode_640, 0o640);
open_creat_mode!(open_creat_mode_620, 0o620);
open_creat_mode!(open_creat_mode_400, 0o400);
open_creat_mode!(open_creat_mode_200, 0o200);
open_creat_mode!(open_creat_mode_124, 0o124);
open_creat_mode!(open_creat_mode_421, 0o421);
open_creat_mode!(open_creat_mode_070, 0o070);
open_creat_mode!(open_creat_mode_007, 0o007);
open_creat_mode!(open_creat_mode_505, 0o505);

#[crate::lctp_test(suite = fs, expect = failure, case = "open with O_CREAT|O_EXCL on a directory returns EEXIST")]
fn open_excl_dir_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(
        syscall::open(&dir, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "excl dir"
    );
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT without O_EXCL on an existing file preserves contents")]
fn open_creat_without_excl_existing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"old")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT, 0o644),
        "creat existing"
    );
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    check_eq!(read_file(&path, &mut buf)?, 3, "preserved");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "open with O_TRUNC sets size 0 and advances mtime")]
fn open_trunc_updates_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"data")?;
    let before = check_ok!(syscall::stat(&path), "stat");
    nanosleep_secs(1)?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    let after = check_ok!(syscall::stat(&path), "stat after");
    check_eq!(after.st_size, 0, "size");
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

#[crate::lctp_test(suite = fs, expect = success, case = "multiple writes on an O_APPEND fd append in order")]
fn open_append_multiple() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0),
        "append"
    );
    check_ok!(syscall::write(fd, b"A"), "w1");
    check_ok!(syscall::write(fd, b"B"), "w2");
    check_ok!(syscall::write(fd, b"C"), "w3");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    check_eq!(read_file(&path, &mut buf)?, 3, "len");
    check_eq!(&buf[..3], b"ABC", "data");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "write on an O_APPEND fd appends even after lseek to 0")]
fn open_append_ignores_seek() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"XY")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_APPEND, 0),
        "append"
    );
    check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
    check_ok!(syscall::write(fd, b"Z"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    check_eq!(read_file(&path, &mut buf)?, 3, "len");
    check_eq!(&buf[..3], b"XYZ", "append not overwrite");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a symlink with O_NOFOLLOW returns ELOOP")]
fn open_nofollow_symlink_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    check_err!(
        syscall::open(&link, oflag::O_RDONLY | oflag::O_NOFOLLOW, 0),
        Errno::ELOOP,
        "nofollow"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a directory O_WRONLY returns EISDIR")]
fn open_directory_wronly_eisdir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_err!(
        syscall::open(tmp.path(), oflag::O_WRONLY, 0),
        Errno::EISDIR,
        "wronly dir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a directory O_RDWR returns EISDIR")]
fn open_directory_rdwr_eisdir() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_err!(
        syscall::open(tmp.path(), oflag::O_RDWR, 0),
        Errno::EISDIR,
        "rdwr dir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open through a non-directory path component returns ENOTDIR")]
fn open_enotdir_trailing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"x\0", &mut nested)?;
    check_err!(
        syscall::open(path, oflag::O_RDONLY, 0),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a missing path returns ENOENT")]
fn open_enoent_plain() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"missing")?;
    check_err!(
        syscall::open(&path, oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT through a dangling symlink creates the target")]
fn open_creat_through_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let target = copy_child(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    let fd = check_ok!(
        syscall::open(&link, oflag::O_RDWR | oflag::O_CREAT, 0o644),
        "creat via link"
    );
    check_ok!(syscall::write(fd, b"via"), "write");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&target), "stat target");
    check!(st.is_reg(), "created target");
    check_eq!(st.st_size, 3, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "write on an O_RDONLY fd returns EBADF")]
fn open_rdonly_then_write_ebadf() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "rdonly");
    check_err!(syscall::write(fd, b"no"), Errno::EBADF, "ebadf");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open through a parent with mode 0000 returns EACCES")]
fn open_parent_chmod0_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chmod(&dir, 0o000), "chmod 0");
    check_err!(
        syscall::open(child, oflag::O_RDONLY, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::unlink(child), "unlink");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a mode 0000 file returns EACCES")]
fn open_file_chmod0_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o000), "chmod");
    check_err!(
        syscall::open(&path, oflag::O_RDONLY, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "open of a FIFO O_RDONLY|O_NONBLOCK succeeds")]
fn open_fifo_nonblock_rdonly() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_NONBLOCK, 0),
        "rdonly nb"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = failure, case = "open of a FIFO O_WRONLY|O_NONBLOCK with no reader returns ENXIO")]
fn open_fifo_nonblock_wronly_enxio() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_err!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_NONBLOCK, 0),
        Errno::ENXIO,
        "enxio"
    );
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "fcntl F_SETFD sets FD_CLOEXEC on an open fd")]
fn open_cloexec_via_fcntl_setfd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let before = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFD, 0), "getfd");
    check!(before & crate::syscall::FD_CLOEXEC as usize == 0, "no cloexec");
    check_ok!(
        syscall::fcntl(
            fd,
            crate::syscall::fcntl_cmd::F_SETFD,
            crate::syscall::FD_CLOEXEC as usize
        ),
        "setfd"
    );
    let after = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFD, 0), "getfd2");
    check!(after & crate::syscall::FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "write then read on an O_RDWR fd round-trips the bytes")]
fn open_rdwr_roundtrip() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::write(fd, b"hi"), "write");
    check_ok!(syscall::lseek(fd, 0, crate::syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 2, "len");
    check_eq!(&buf, b"hi", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "a second open with O_CREAT|O_EXCL returns EEXIST")]
fn open_creat_eexist_twice() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"x")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "first"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "second"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_TRUNC preserves the inode")]
fn open_trunc_preserves_inode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"abc")?;
    let ino = check_ok!(syscall::stat(&path), "stat").st_ino;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    check_eq!(check_ok!(syscall::stat(&path), "stat2").st_ino, ino, "ino");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_DIRECTORY follows a symlink to a directory")]
fn open_directory_on_symlink_to_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"d\0", &link), "symlink");
    let fd = check_ok!(
        syscall::open(&link, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "O_DIRECTORY follow"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "fchmod on an O_PATH fd returns EBADF")]
fn open_opath_fchmod_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_PATH, 0), "opath");
    check_err!(syscall::fchmod(fd, 0o600), Errno::EBADF, "fchmod opath");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "write on an O_WRONLY fd succeeds")]
fn open_wronly_write_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "wronly");
    check_ok!(syscall::write(fd, b"ok"), "write");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT|O_TRUNC on an existing file sets size 0")]
fn open_creat_trunc_combo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"olddata")?;
    let fd = check_ok!(
        syscall::open(
            &path,
            oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC,
            0o644
        ),
        "creat trunc"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 0, "truncated");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_EXCL and without O_CREAT opens an existing file")]
fn open_excl_without_creat_ignored() -> TestResult {
    // O_EXCL without O_CREAT is ignored on Linux.
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_EXCL, 0),
        "excl alone"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "write on an O_APPEND fd of an empty file stores the bytes")]
fn open_append_empty_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0),
        "append"
    );
    check_ok!(syscall::write(fd, b"Z"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 1];
    check_eq!(read_file(&path, &mut buf)?, 1, "len");
    check_eq!(buf[0], b'Z', "data");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open of a directory with O_CLOEXEC sets FD_CLOEXEC")]
fn open_directory_cloexec() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(
            tmp.path(),
            oflag::O_RDONLY | oflag::O_DIRECTORY | oflag::O_CLOEXEC,
            0
        ),
        "dir cloexec"
    );
    let fl = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFD, 0), "getfd");
    check!(fl & crate::syscall::FD_CLOEXEC as usize != 0, "CLOEXEC");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CREAT creates a previously missing file")]
fn open_missing_creat_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fresh")?;
    check_err!(syscall::stat(&path), Errno::ENOENT, "missing");
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_CREAT, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::stat(&path), "exists");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_RDONLY|O_TRUNC succeeds and leaves size 0 or unchanged")]
fn open_trunc_readonly_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"data")?;
    // Linux may truncate even with O_RDONLY|O_TRUNC, or ignore O_TRUNC — accept either.
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_TRUNC, 0),
        "rdonly trunc"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.st_size == 0 || st.st_size == 4, "size ok");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "openat with a directory fd creates a relative child file")]
fn openat_relative() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let dirfd = check_ok!(
        syscall::open(&dir, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    let fd = check_ok!(
        syscall::openat(dirfd, b"f\0", oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "openat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::unlinkat(dirfd, b"f\0", 0), "unlinkat");
    check_ok!(syscall::close(dirfd), "close dir");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open through a symlink loop returns ELOOP")]
fn open_symlink_loop_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::symlink(b"b\0", &a), "a");
    check_ok!(syscall::symlink(b"a\0", &b), "b");
    check_err!(
        syscall::open(&a, oflag::O_RDONLY, 0),
        Errno::ELOOP,
        "eloop"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open with O_CREAT|O_EXCL on a FIFO returns EEXIST")]
fn open_creat_excl_fifo_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_err!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "eexist"
    );
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_TRUNC on an empty file leaves size 0")]
fn open_trunc_empty_stays_empty() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_APPEND sets O_APPEND in F_GETFL")]
fn open_append_getfl() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0),
        "append"
    );
    let fl = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & oflag::O_APPEND != 0, "O_APPEND");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open of a directory with O_DIRECTORY|O_NONBLOCK succeeds")]
fn open_directory_nonblock() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(
            tmp.path(),
            oflag::O_RDONLY | oflag::O_DIRECTORY | oflag::O_NONBLOCK,
            0
        ),
        "dir nb"
    );
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open with O_CREAT|O_EXCL on a symlink returns EEXIST")]
fn open_creat_then_excl_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    check_err!(
        syscall::open(&link, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "eexist"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open O_WRONLY of a mode 0000 file returns EACCES")]
fn open_wronly_chmod0_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o000), "chmod");
    check_err!(
        syscall::open(&path, oflag::O_WRONLY, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open O_RDWR of a mode 0400 file returns EACCES")]
fn open_rdwr_chmod400_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o400), "chmod");
    check_err!(
        syscall::open(&path, oflag::O_RDWR, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open with O_CREAT through a non-directory component returns ENOTDIR")]
fn open_path_component_file_enotdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"y\0", &mut nested)?;
    check_err!(
        syscall::open(path, oflag::O_CREAT | oflag::O_WRONLY, 0o644),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_CLOEXEC|O_NONBLOCK sets both flags")]
fn open_cloexec_and_nonblock() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(
        syscall::open(
            &path,
            oflag::O_RDONLY | oflag::O_CLOEXEC | oflag::O_NONBLOCK,
            0
        ),
        "open"
    );
    let fdfl = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFD, 0), "getfd");
    check!(fdfl & crate::syscall::FD_CLOEXEC as usize != 0, "cloexec");
    let fl = check_ok!(syscall::fcntl(fd, crate::syscall::fcntl_cmd::F_GETFL, 0), "getfl");
    check!(fl as i32 & oflag::O_NONBLOCK != 0, "nonblock");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "open with O_TRUNC after a large write sets size 0")]
fn open_large_write_then_trunc() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY, 0), "open");
    let buf = [b'A'; 128];
    for _ in 0..8 {
        check_ok!(syscall::write(fd, &buf), "write");
    }
    check_ok!(syscall::close(fd), "close");
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "openat of a missing relative name returns ENOENT")]
fn openat_enoent() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let dirfd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    check_err!(
        syscall::openat(dirfd, b"nope\0", oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "enoent"
    );
    check_ok!(syscall::close(dirfd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "close of an O_PATH fd succeeds")]
fn open_opath_close() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_PATH, 0), "opath");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open O_RDONLY of a mode 0200 file returns EACCES")]
fn open_rdonly_chmod200_eacces() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o200), "chmod");
    check_err!(
        syscall::open(&path, oflag::O_RDONLY, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a FIFO with O_DIRECTORY returns ENOTDIR")]
fn open_directory_on_fifo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_err!(
        syscall::open(&path, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        Errno::ENOTDIR,
        "enotdir"
    );
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}
