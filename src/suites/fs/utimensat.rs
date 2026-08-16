//! utimensat filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{
    copy_child, create_dir, create_empty, nanosleep_secs, timespec_later, write_file,
};
use crate::syscall::{self, oflag, Errno, Timespec, UTIME_NOW, UTIME_OMIT};

fn ts_pair(sec: i64) -> [Timespec; 2] {
    [
        Timespec {
            tv_sec: sec,
            tv_nsec: 0,
        },
        Timespec {
            tv_sec: sec,
            tv_nsec: 0,
        },
    ]
}

#[crate::lctp_test(suite = fs, expect = success, case = "utimensat with UTIME_NOW sets a positive mtime")]
fn utimensat_now() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
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
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &times, 0),
        "utimensat NOW"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.st_mtime > 0, "mtime set");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "utimensat with UTIME_OMIT leaves mtime unchanged")]
fn utimensat_omit() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "stat before");
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
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &times, 0),
        "utimensat OMIT"
    );
    let after = check_ok!(syscall::stat(&path), "stat after");
    check_eq!(after.st_mtime, before.st_mtime, "mtime unchanged");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat sets mtime to an explicit timestamp")]
fn utimensat_explicit() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let t = 1_700_000_000i64;
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &ts_pair(t), 0),
        "utimensat"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_mtime, t, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat with AT_SYMLINK_NOFOLLOW sets the symlink mtime")]
fn utimensat_symlink_nofollow() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    let t = 1_600_000_000i64;
    check_ok!(
        syscall::utimensat(
            syscall::AT_FDCWD,
            &link,
            &ts_pair(t),
            syscall::AT_SYMLINK_NOFOLLOW,
        ),
        "utimensat link"
    );
    let lst = check_ok!(syscall::lstat(&link), "lstat");
    check_eq!(lst.st_mtime, t, "link mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "futimens with UTIME_NOW sets a positive mtime")]
fn futimens_now_via_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
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
    check_ok!(syscall::futimens(fd, &times), "futimens");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.st_mtime > 0, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "futimens sets mtime to an explicit timestamp")]
fn futimens_explicit() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let t = 1_650_000_000i64;
    check_ok!(syscall::futimens(fd, &ts_pair(t)), "futimens");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_mtime, t, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat sets distinct atime and mtime values")]
fn utimensat_atime_mtime_different() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let times = [
        Timespec {
            tv_sec: 1_500_000_000,
            tv_nsec: 0,
        },
        Timespec {
            tv_sec: 1_510_000_000,
            tv_nsec: 0,
        },
    ];
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &times, 0),
        "utimensat"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_atime, 1_500_000_000, "atime");
    check_eq!(st.st_mtime, 1_510_000_000, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat with UTIME_OMIT for atime sets only mtime")]
fn utimensat_omit_atime_set_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "before");
    let times = [
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
        Timespec {
            tv_sec: 1_520_000_000,
            tv_nsec: 0,
        },
    ];
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &times, 0),
        "utimensat"
    );
    let after = check_ok!(syscall::stat(&path), "after");
    check_eq!(after.st_atime, before.st_atime, "atime omit");
    check_eq!(after.st_mtime, 1_520_000_000, "mtime set");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat with UTIME_OMIT for mtime sets only atime")]
fn utimensat_set_atime_omit_mtime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "before");
    let times = [
        Timespec {
            tv_sec: 1_530_000_000,
            tv_nsec: 0,
        },
        Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT,
        },
    ];
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &times, 0),
        "utimensat"
    );
    let after = check_ok!(syscall::stat(&path), "after");
    check_eq!(after.st_atime, 1_530_000_000, "atime set");
    check_eq!(after.st_mtime, before.st_mtime, "mtime omit");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "utimensat on a missing path returns ENOENT")]
fn utimensat_enoent() -> TestResult {
    let times = ts_pair(1_000_000);
    check_err!(
        syscall::utimensat(syscall::AT_FDCWD, b"/tmp/lctp-utime-missing\0", &times, 0),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat sets mtime on a directory")]
fn utimensat_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let t = 1_540_000_000i64;
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &dir, &ts_pair(t), 0),
        "utimensat dir"
    );
    check_eq!(check_ok!(syscall::stat(&dir), "stat").st_mtime, t, "mtime");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat without AT_SYMLINK_NOFOLLOW sets the target file mtime")]
fn utimensat_follow_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"f\0", &link), "symlink");
    let t = 1_550_000_000i64;
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &link, &ts_pair(t), 0),
        "follow"
    );
    check_eq!(check_ok!(syscall::stat(&file), "stat").st_mtime, t, "mtime");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat advances ctime")]
fn utimensat_updates_ctime() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let before = check_ok!(syscall::stat(&path), "before");
    nanosleep_secs(1)?;
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &ts_pair(1_560_000_000), 0),
        "utimensat"
    );
    let after = check_ok!(syscall::stat(&path), "after");
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

#[crate::lctp_test(suite = fs, expect = failure, case = "futimens on fd -1 returns EBADF")]
fn futimens_bad_fd() -> TestResult {
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
    check_err!(syscall::futimens(-1, &times), Errno::EBADF, "ebadf");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat sets nonzero nanosecond atime and mtime")]
fn utimensat_nsec_nonzero() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let times = [
        Timespec {
            tv_sec: 1_570_000_000,
            tv_nsec: 123456789,
        },
        Timespec {
            tv_sec: 1_570_000_000,
            tv_nsec: 987654321,
        },
    ];
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &times, 0),
        "utimensat"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_atime, 1_570_000_000, "atime sec");
    check_eq!(st.st_atime_nsec, 123456789, "atime nsec");
    check_eq!(st.st_mtime_nsec, 987654321, "mtime nsec");
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "utimensat with UTIME_NOW advances mtime after a sleep")]
fn utimensat_both_now_after_sleep() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &ts_pair(1_000_000), 0),
        "old"
    );
    nanosleep_secs(1)?;
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
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &times, 0),
        "now"
    );
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.st_mtime > 1_000_000, "mtime advanced");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "utimensat sets mtime on a file that already has data")]
fn utimensat_on_written_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"x")?;
    let t = 1_580_000_000i64;
    check_ok!(
        syscall::utimensat(syscall::AT_FDCWD, &path, &ts_pair(t), 0),
        "utimensat"
    );
    check_eq!(check_ok!(syscall::stat(&path), "stat").st_mtime, t, "mtime");
    Ok(())
}
