//! utimensat filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty};
use crate::syscall::{self, Timespec, UTIME_NOW, UTIME_OMIT};

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

#[crate::lctp_test(suite = fs)]
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

#[crate::lctp_test(suite = fs)]
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

#[crate::lctp_test(suite = fs, full)]
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

#[crate::lctp_test(suite = fs, full)]
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

#[crate::lctp_test(suite = fs)]
fn futimens_now_via_fd() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(
        syscall::open(&path, crate::syscall::oflag::O_RDWR, 0),
        "open"
    );
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

#[crate::lctp_test(suite = fs, full)]
fn futimens_explicit() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(
        syscall::open(&path, crate::syscall::oflag::O_RDWR, 0),
        "open"
    );
    let t = 1_650_000_000i64;
    check_ok!(syscall::futimens(fd, &ts_pair(t)), "futimens");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_mtime, t, "mtime");
    Ok(())
}
