//! chmod/fchmod filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{
    chmod_path, copy_child, create_dir, create_empty, join_path, nanosleep_secs, timespec_later,
    write_file,
};
use crate::syscall::{self, oflag, Errno, AT_SYMLINK_NOFOLLOW, S_IFIFO};

fn assert_mode(path: &[u8], mode: u32) -> TestResult {
    let st = check_ok!(syscall::stat(path), "stat");
    check_eq!(st.mode_bits() & 0o777, mode, "mode");
    Ok(())
}

fn assert_mode7777(path: &[u8], mode: u32) -> TestResult {
    let st = check_ok!(syscall::stat(path), "stat");
    check_eq!(st.mode_bits() & 0o7777, mode, "mode7777");
    Ok(())
}

macro_rules! chmod_reg_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = concat!("chmod on a regular file sets mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let path = create_empty(&mut tmp, b"f")?;
            check_ok!(syscall::chmod(&path, $mode), "chmod");
            assert_mode(&path, $mode & 0o777)
        }
    };
}

macro_rules! chmod_dir_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = concat!("chmod on a directory sets mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let dir = create_dir(&mut tmp, b"d", 0o755)?;
            check_ok!(syscall::chmod(&dir, $mode), "chmod");
            assert_mode(&dir, $mode & 0o777)?;
            check_ok!(syscall::chmod(&dir, 0o755), "restore");
            check_ok!(syscall::rmdir(&dir), "rmdir");
            Ok(())
        }
    };
}

macro_rules! fchmod_reg_mode {
    ($name:ident, $mode:expr) => {
        #[crate::lctp_test(suite = fs, expect = success, case = concat!("fchmod on a regular file sets mode ", stringify!($mode)))]
        fn $name() -> TestResult {
            let mut tmp = check_ok!(TempDir::create(), "tempdir");
            let path = create_empty(&mut tmp, b"f")?;
            let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
            check_ok!(syscall::fchmod(fd, $mode), "fchmod");
            check_ok!(syscall::close(fd), "close");
            assert_mode(&path, $mode & 0o777)
        }
    };
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0644")]
fn chmod_file_644() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    chmod_path(&path, 0o644)?;
    assert_mode(&path, 0o644)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0600")]
fn chmod_file_600() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    assert_mode(&path, 0o600)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0755")]
fn chmod_file_755() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o755), "chmod");
    assert_mode(&path, 0o755)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0444")]
fn chmod_file_444() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o444), "chmod");
    assert_mode(&path, 0o444)
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "chmod on a regular file sets mode 0777")]
fn chmod_file_777() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o777), "chmod");
    assert_mode(&path, 0o777)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a directory sets mode 0700")]
fn chmod_dir_700() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o700), "chmod");
    let st = check_ok!(syscall::stat(&dir), "stat");
    check!(st.is_dir(), "dir");
    check_eq!(st.mode_bits() & 0o777, 0o700, "mode");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a directory sets mode 0755")]
fn chmod_dir_755() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o700)?;
    check_ok!(syscall::chmod(&dir, 0o755), "chmod");
    assert_mode(&dir, 0o755)?;
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "fchmod sets the same mode bits that chmod would")]
fn fchmod_matches_chmod() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fchmod(fd, 0o640), "fchmod");
    check_ok!(syscall::close(fd), "close");
    assert_mode(&path, 0o640)
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "chmod on a symlink follows it and sets the target file mode")]
fn chmod_symlink_follow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    check_ok!(syscall::chmod(&link, 0o644), "chmod link");
    // Linux chmod follows symlinks — target mode changes.
    let st = check_ok!(syscall::stat(&file), "stat target");
    check_eq!(st.mode_bits() & 0o777, 0o644, "target mode");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = soft, case = "fchmodat with AT_SYMLINK_NOFOLLOW succeeds or returns EOPNOTSUPP")]
fn fchmodat_symlink_nofollow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _file = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    // Linux does not implement fchmodat(AT_SYMLINK_NOFOLLOW); expect ENOTSUP.
    match syscall::fchmodat(syscall::AT_FDCWD, &link, 0o600, AT_SYMLINK_NOFOLLOW) {
        Err(Errno::EOPNOTSUPP) | Err(Errno::ENOSYS) => Ok(()),
        Err(e) if e.0 == 95 => Ok(()), // EOPNOTSUPP alternate
        Ok(()) => Ok(()),               // some FS may allow it
        Err(_) => Err(crate::harness::AssertFail::msg(
            "unexpected fchmodat(AT_SYMLINK_NOFOLLOW) errno",
        )),
    }
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod from 0777 to 0600 clears group and other bits")]
fn chmod_clear_group_other() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o777), "chmod wide");
    check_ok!(syscall::chmod(&path, 0o600), "chmod narrow");
    assert_mode(&path, 0o600)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod 0755 sets execute bits on a regular file")]
fn chmod_set_executable() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o755), "chmod +x");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.mode_bits() & 0o111 != 0, "execute bits");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0640")]
fn chmod_file_640() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o640), "chmod");
    assert_mode(&path, 0o640)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0400")]
fn chmod_file_400() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o400), "chmod");
    assert_mode(&path, 0o400)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0200")]
fn chmod_file_200() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o200), "chmod");
    assert_mode(&path, 0o200)
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "chmod on a regular file sets mode 0711")]
fn chmod_file_711() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o711), "chmod");
    assert_mode(&path, 0o711)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a directory sets mode 0555")]
fn chmod_dir_555() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o555), "chmod");
    assert_mode(&dir, 0o555)?;
    // Restore write so cleanup can rmdir.
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "fchmod on a regular file sets mode 0700")]
fn fchmod_0700() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fchmod(fd, 0o700), "fchmod");
    check_ok!(syscall::close(fd), "close");
    assert_mode(&path, 0o700)
}

// ---- mode matrix (permission key bits) ----

chmod_reg_mode!(chmod_reg_000, 0o000);
chmod_reg_mode!(chmod_reg_001, 0o001);
chmod_reg_mode!(chmod_reg_010, 0o010);
chmod_reg_mode!(chmod_reg_100, 0o100);
chmod_reg_mode!(chmod_reg_111, 0o111);
chmod_reg_mode!(chmod_reg_222, 0o222);
chmod_reg_mode!(chmod_reg_444, 0o444);
chmod_reg_mode!(chmod_reg_555, 0o555);
chmod_reg_mode!(chmod_reg_666, 0o666);
chmod_reg_mode!(chmod_reg_700, 0o700);
chmod_reg_mode!(chmod_reg_711, 0o711);
chmod_reg_mode!(chmod_reg_750, 0o750);
chmod_reg_mode!(chmod_reg_755, 0o755);
chmod_reg_mode!(chmod_reg_764, 0o764);
chmod_reg_mode!(chmod_reg_775, 0o775);
chmod_reg_mode!(chmod_reg_777, 0o777);
chmod_reg_mode!(chmod_reg_620, 0o620);
chmod_reg_mode!(chmod_reg_460, 0o460);
chmod_reg_mode!(chmod_reg_240, 0o240);
chmod_reg_mode!(chmod_reg_124, 0o124);
chmod_reg_mode!(chmod_reg_421, 0o421);
chmod_reg_mode!(chmod_reg_321, 0o321);
chmod_reg_mode!(chmod_reg_123, 0o123);
chmod_reg_mode!(chmod_reg_070, 0o070);
chmod_reg_mode!(chmod_reg_007, 0o007);
chmod_reg_mode!(chmod_reg_505, 0o505);
chmod_reg_mode!(chmod_reg_050, 0o050);
chmod_reg_mode!(chmod_reg_005, 0o005);
chmod_reg_mode!(chmod_reg_303, 0o303);
chmod_reg_mode!(chmod_reg_606, 0o606);

chmod_dir_mode!(chmod_dmode_000, 0o000);
chmod_dir_mode!(chmod_dmode_001, 0o001);
chmod_dir_mode!(chmod_dmode_010, 0o010);
chmod_dir_mode!(chmod_dmode_100, 0o100);
chmod_dir_mode!(chmod_dmode_111, 0o111);
chmod_dir_mode!(chmod_dmode_222, 0o222);
chmod_dir_mode!(chmod_dmode_444, 0o444);
chmod_dir_mode!(chmod_dmode_555, 0o555);
chmod_dir_mode!(chmod_dmode_666, 0o666);
chmod_dir_mode!(chmod_dmode_700, 0o700);
chmod_dir_mode!(chmod_dmode_711, 0o711);
chmod_dir_mode!(chmod_dmode_750, 0o750);
chmod_dir_mode!(chmod_dmode_755, 0o755);
chmod_dir_mode!(chmod_dmode_764, 0o764);
chmod_dir_mode!(chmod_dmode_775, 0o775);
chmod_dir_mode!(chmod_dmode_777, 0o777);
chmod_dir_mode!(chmod_dmode_731, 0o731);
chmod_dir_mode!(chmod_dmode_713, 0o713);
chmod_dir_mode!(chmod_dmode_517, 0o517);
chmod_dir_mode!(chmod_dmode_175, 0o175);

fchmod_reg_mode!(fchmod_reg_000, 0o000);
fchmod_reg_mode!(fchmod_reg_111, 0o111);
fchmod_reg_mode!(fchmod_reg_222, 0o222);
fchmod_reg_mode!(fchmod_reg_444, 0o444);
fchmod_reg_mode!(fchmod_reg_555, 0o555);
fchmod_reg_mode!(fchmod_reg_666, 0o666);
fchmod_reg_mode!(fchmod_reg_700, 0o700);
fchmod_reg_mode!(fchmod_reg_755, 0o755);
fchmod_reg_mode!(fchmod_reg_777, 0o777);
fchmod_reg_mode!(fchmod_reg_640, 0o640);
fchmod_reg_mode!(fchmod_reg_620, 0o620);
fchmod_reg_mode!(fchmod_reg_400, 0o400);
fchmod_reg_mode!(fchmod_reg_200, 0o200);
fchmod_reg_mode!(fchmod_reg_100, 0o100);
fchmod_reg_mode!(fchmod_reg_001, 0o001);

#[crate::lctp_test(suite = fs, expect = success, case = "chmod sets the setuid bit on a regular file")]
fn chmod_setuid_bit() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o4644), "chmod setuid");
    assert_mode7777(&path, 0o4644)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod sets the setgid bit on a regular file")]
fn chmod_setgid_bit() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o2644), "chmod setgid");
    assert_mode7777(&path, 0o2644)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod sets the sticky bit on a regular file")]
fn chmod_sticky_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o1644), "chmod sticky");
    assert_mode7777(&path, 0o1644)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod sets the sticky bit on a directory")]
fn chmod_sticky_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o1755), "chmod sticky dir");
    assert_mode7777(&dir, 0o1755)?;
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod 07777 sets setuid, setgid, and sticky bits")]
fn chmod_setuid_setgid_sticky() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o7777), "chmod all special");
    assert_mode7777(&path, 0o7777)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod clears the setuid bit on a regular file")]
fn chmod_clear_setuid() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o4644), "set");
    check_ok!(syscall::chmod(&path, 0o644), "clear");
    assert_mode7777(&path, 0o644)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod clears the setgid bit on a regular file")]
fn chmod_clear_setgid() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o2644), "set");
    check_ok!(syscall::chmod(&path, 0o644), "clear");
    assert_mode7777(&path, 0o644)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod clears the sticky bit on a regular file")]
fn chmod_clear_sticky() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o1644), "set");
    check_ok!(syscall::chmod(&path, 0o644), "clear");
    assert_mode7777(&path, 0o644)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a FIFO sets mode 0600")]
fn chmod_fifo_mode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_ok!(syscall::chmod(&path, 0o600), "chmod fifo");
    assert_mode(&path, 0o600)?;
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a FIFO sets mode 0555")]
fn chmod_fifo_555() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_ok!(syscall::chmod(&path, 0o555), "chmod");
    assert_mode(&path, 0o555)?;
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a FIFO sets mode 0000")]
fn chmod_fifo_000() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    check_ok!(syscall::chmod(&path, 0o000), "chmod");
    assert_mode(&path, 0o000)?;
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "fchmod on a FIFO sets mode 0640")]
fn fchmod_fifo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &path, S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_NONBLOCK, 0),
        "open"
    );
    check_ok!(syscall::fchmod(fd, 0o640), "fchmod");
    check_ok!(syscall::close(fd), "close");
    assert_mode(&path, 0o640)?;
    check_ok!(syscall::unlink(&path), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "chmod on a missing path returns ENOENT")]
fn chmod_enoent() -> TestResult {
    check_err!(
        syscall::chmod(b"/tmp/lctp-chmod-missing\0", 0o644),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "chmod through a non-directory path component returns ENOTDIR")]
fn chmod_enotdir_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"notdir")?;
    let mut nested = [0u8; 160];
    let path = join_path(&file, b"child\0", &mut nested)?;
    check_err!(syscall::chmod(path, 0o644), Errno::ENOTDIR, "enotdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "fchmod on a closed fd returns EBADF")]
fn fchmod_bad_fd() -> TestResult {
    check_err!(syscall::fchmod(-1, 0o644), Errno::EBADF, "bad fd");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "chmod advances ctime on a regular file")]
fn chmod_updates_ctime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "stat before");
    nanosleep_secs(1)?;
    check_ok!(syscall::chmod(&path, 0o600), "chmod");
    let after = check_ok!(syscall::stat(&path), "stat after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime advanced"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "fchmod advances ctime on a regular file")]
fn fchmod_updates_ctime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "stat before");
    nanosleep_secs(1)?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::fchmod(fd, 0o640), "fchmod");
    check_ok!(syscall::close(fd), "close");
    let after = check_ok!(syscall::stat(&path), "stat after");
    check!(
        timespec_later(
            after.st_ctime,
            after.st_ctime_nsec,
            before.st_ctime,
            before.st_ctime_nsec
        ),
        "ctime advanced"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open after chmod 0000 returns EACCES")]
fn chmod_then_open_denied() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"x")?;
    check_ok!(syscall::chmod(&path, 0o000), "chmod 0");
    check_err!(
        syscall::open(&path, oflag::O_RDONLY, 0),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&path, 0o644), "restore");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "open of a child after chmod 0000 on the parent returns EACCES")]
fn chmod_parent_denies_lookup() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let mut nested = [0u8; 160];
    let child = join_path(&dir, b"f\0", &mut nested)?;
    let fd = check_ok!(
        syscall::open(child, oflag::O_CREAT | oflag::O_RDWR | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chmod(&dir, 0o000), "chmod parent 0");
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

#[crate::lctp_test(suite = fs, expect = success, case = "chmod with the same mode twice leaves mode 0640")]
fn chmod_idempotent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(syscall::chmod(&path, 0o640), "chmod1");
    check_ok!(syscall::chmod(&path, 0o640), "chmod2");
    assert_mode(&path, 0o640)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod through a sequence of modes sets each requested mode")]
fn chmod_cycle_modes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    for m in [0o400u32, 0o200, 0o100, 0o700, 0o644] {
        check_ok!(syscall::chmod(&path, m), "chmod cycle");
        assert_mode(&path, m)?;
    }
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "chmod on a symlink leaves the link mode unchanged and sets the target mode")]
fn chmod_symlink_does_not_change_link_mode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    let before = check_ok!(syscall::lstat(&link), "lstat before");
    check_ok!(syscall::chmod(&link, 0o600), "chmod follow");
    let after = check_ok!(syscall::lstat(&link), "lstat after");
    check_eq!(
        after.mode_bits() & 0o7777,
        before.mode_bits() & 0o7777,
        "link mode unchanged"
    );
    assert_mode(&file, 0o600)
}

#[crate::lctp_test(suite = fs, expect = success, case = "fchmodat with AT_FDCWD sets mode 0611")]
fn fchmodat_cwd_path() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(
        syscall::fchmodat(syscall::AT_FDCWD, &path, 0o611, 0),
        "fchmodat"
    );
    assert_mode(&path, 0o611)
}

#[crate::lctp_test(suite = fs, expect = success, case = "chmod on a directory sets sticky and setgid bits")]
fn chmod_dir_sticky_setgid() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o3775), "chmod");
    assert_mode7777(&dir, 0o3775)?;
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "chmod on an empty path returns ENOENT or EINVAL")]
fn chmod_empty_path_enoent() -> TestResult {
    // Empty path is invalid / ENOENT depending on kernel path.
    match syscall::chmod(b"\0", 0o644) {
        Err(Errno::ENOENT) | Err(Errno::EINVAL) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("chmod empty ok")),
        Err(_) => Err(crate::harness::AssertFail::msg("chmod empty errno")),
    }
}
