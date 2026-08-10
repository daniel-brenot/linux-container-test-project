//! POSIX path resolution and open flag tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty, join_path, truncate_cstr, write_file};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = posix)]
fn open_creat_excl() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"file")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat excl"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        Errno::EEXIST,
        "second excl"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn open_directory_flag() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "O_DIRECTORY dir"
    );
    check_ok!(syscall::close(fd), "close");
    let mut tmp = tmp;
    let file = create_empty(&mut tmp, b"file")?;
    check_err!(
        syscall::open(&file, oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        Errno::ENOTDIR,
        "O_DIRECTORY file"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn open_nofollow_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    check_err!(
        syscall::open(&link, oflag::O_RDONLY | oflag::O_NOFOLLOW, 0),
        Errno::ELOOP,
        "O_NOFOLLOW"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_dot_dotdot() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = copy_child(&mut tmp, b"subdir")?;
    check_ok!(syscall::mkdir(&sub, 0o755), "mkdir");
    let mut parent = [0u8; 160];
    join_path(&sub, b"..", &mut parent)?;
    let st_tmp = check_ok!(syscall::stat(tmp.path()), "stat tmp");
    let st_dotdot = check_ok!(syscall::stat(truncate_cstr(&parent)), "stat ..");
    check_eq!(st_tmp.st_ino, st_dotdot.st_ino, "inode");
    check_eq!(st_tmp.st_dev, st_dotdot.st_dev, "dev");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_dot_current() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = copy_child(&mut tmp, b"sub")?;
    check_ok!(syscall::mkdir(&sub, 0o755), "mkdir");
    let mut dot = [0u8; 160];
    join_path(&sub, b".", &mut dot)?;
    let st_sub = check_ok!(syscall::stat(&sub), "stat sub");
    let st_dot = check_ok!(syscall::stat(truncate_cstr(&dot)), "stat .");
    check_eq!(st_sub.st_ino, st_dot.st_ino, "inode");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn trailing_slash_on_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    let mut with_slash = [0u8; 160];
    let end = file.iter().position(|&c| c == 0).unwrap();
    with_slash[..end].copy_from_slice(&file[..end]);
    with_slash[end] = b'/';
    with_slash[end + 1] = 0;
    check_err!(
        syscall::open(truncate_cstr(&with_slash), oflag::O_RDONLY, 0),
        Errno::ENOTDIR,
        "slash on file"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn trailing_slash_on_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"dir")?;
    check_ok!(syscall::mkdir(&dir, 0o755), "mkdir");
    let mut with_slash = [0u8; 160];
    let end = dir.iter().position(|&c| c == 0).unwrap();
    with_slash[..end + 1].copy_from_slice(&dir[..end + 1]);
    with_slash[end] = b'/';
    with_slash[end + 1] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&with_slash), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "open dir/"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn open_trunc_zeros_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"content")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR | oflag::O_TRUNC, 0), "trunc");
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 0, "size");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn open_append_at_eof() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    write_file(&path, b"AB")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_WRONLY | oflag::O_APPEND, 0), "append");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    check_ok!(syscall::write(fd, b"CD"), "write");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 8];
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "read");
    check_eq!(check_ok!(syscall::read(fd, &mut buf), "read"), 4, "len");
    check_eq!(&buf[..4], b"ABCD", "data");
    check_ok!(syscall::close(fd), "close read");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn open_creat_mode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"modefile")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o755),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&path), "stat");
    check!(st.mode_bits() & 0o400 != 0, "owner read");
    check!(st.is_reg(), "regular");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn open_rdwr_existing() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"f")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "rdwr");
    check_ok!(syscall::write(fd, b"z"), "write");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_multiple_slashes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = copy_child(&mut tmp, b"sub")?;
    check_ok!(syscall::mkdir(&sub, 0o755), "mkdir");
    let mut doubled = [0u8; 160];
    let end = sub.iter().position(|&c| c == 0).unwrap();
    doubled[..end + 1].copy_from_slice(&sub[..end + 1]);
    // Insert extra slash before NUL: "...sub//"
    doubled[end] = b'/';
    doubled[end + 1] = 0;
    let st1 = check_ok!(syscall::stat(&sub), "stat");
    let st2 = check_ok!(syscall::stat(truncate_cstr(&doubled)), "stat doubled");
    check_eq!(st1.st_ino, st2.st_ino, "inode");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}
