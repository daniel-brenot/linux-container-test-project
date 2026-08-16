//! openat2(2) tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, write_file};
use crate::syscall::{self, oflag, Errno, OpenHow, RESOLVE_BENEATH, RESOLVE_NO_SYMLINKS};

#[crate::lctp_test(suite = syscall, expect = success, case = "openat2 can open an existing file read-only")]
fn openat2_rdonly() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"ok")?;
    let how = OpenHow {
        flags: oflag::O_RDONLY as u64,
        mode: 0,
        resolve: 0,
    };
    let fd = check_ok!(syscall::openat2(syscall::AT_FDCWD, &path, &how), "openat2");
    let mut buf = [0u8; 2];
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 2, "len");
    check_eq!(&buf, b"ok", "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "openat2 with O_CREAT|O_EXCL creates a new regular file")]
fn openat2_creat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"new")?;
    let how = OpenHow {
        flags: (oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL) as u64,
        mode: 0o644,
        resolve: 0,
    };
    let fd = check_ok!(syscall::openat2(syscall::AT_FDCWD, &path, &how), "creat");
    check_ok!(syscall::write(fd, b"n"), "write");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.is_reg(), "reg");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "openat2 with RESOLVE_NO_SYMLINKS on a symlink returns ELOOP")]
fn openat2_resolve_no_symlinks() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let how = OpenHow {
        flags: oflag::O_RDONLY as u64,
        mode: 0,
        resolve: RESOLVE_NO_SYMLINKS,
    };
    check_err!(
        syscall::openat2(syscall::AT_FDCWD, &link, &how),
        Errno::ELOOP,
        "no symlinks"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "openat2 with RESOLVE_BENEATH on .. returns EXDEV, ENOENT, or EPERM")]
fn openat2_resolve_beneath_escape() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = create_dir(&mut tmp, b"sub", 0o755)?;
    let dirfd = check_ok!(
        syscall::open(&sub, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    let how = OpenHow {
        flags: oflag::O_RDONLY as u64,
        mode: 0,
        resolve: RESOLVE_BENEATH,
    };
    // Escape via .. should fail under RESOLVE_BENEATH.
    match syscall::openat2(dirfd, b"../\0", &how) {
        Err(Errno::EXDEV) | Err(Errno::ENOENT) | Err(Errno::EPERM) => {}
        Ok(fd) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg(
                "RESOLVE_BENEATH allowed escape",
            ));
        }
        Err(_) => {
            return Err(crate::harness::AssertFail::msg(
                "unexpected beneath errno",
            ));
        }
    }
    check_ok!(syscall::close(dirfd), "close");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "openat2 with RESOLVE_BENEATH can open a file in the same directory")]
fn openat2_beneath_same_dir_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = create_dir(&mut tmp, b"sub", 0o755)?;
    let file = copy_child(&mut tmp, b"sub/f")?;
    // create file under sub
    let fd_create = check_ok!(
        syscall::open(&file, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::write(fd_create, b"z"), "write");
    check_ok!(syscall::close(fd_create), "close create");

    let dirfd = check_ok!(
        syscall::open(&sub, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "opendir"
    );
    let how = OpenHow {
        flags: oflag::O_RDONLY as u64,
        mode: 0,
        resolve: RESOLVE_BENEATH,
    };
    let fd = check_ok!(syscall::openat2(dirfd, b"f\0", &how), "beneath open");
    let mut b = [0u8; 1];
    check_eq!(check_ok!(syscall::read(fd, &mut b), "read"), 1, "len");
    check_eq!(b[0], b'z', "byte");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(dirfd), "close dir");
    check_ok!(syscall::unlink(&file), "unlink");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}
