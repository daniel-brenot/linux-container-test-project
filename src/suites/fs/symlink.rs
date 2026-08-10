//! symlink filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, write_file};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = fs)]
fn symlink_create_readlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"target")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    let mut buf = [0u8; 64];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(&buf[..n], b"target", "target");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_lstat_vs_stat() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let lst = check_ok!(syscall::lstat(&link), "lstat");
    check!(lst.is_lnk(), "symlink");
    let st = check_ok!(syscall::stat(&link), "stat");
    check!(st.is_reg(), "followed reg");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_stat_follows_size() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    write_file(&file, b"12345")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let st = check_ok!(syscall::stat(&link), "stat");
    check_eq!(st.st_size, 5, "followed size");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_relative_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"./file\0", &link), "symlink rel");
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(&buf[..n], b"./file", "rel target");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_open_follows() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    write_file(&file, b"Z")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let fd = check_ok!(syscall::open(&link, oflag::O_RDONLY, 0), "open link");
    let mut b = [0u8; 1];
    check_ok!(syscall::read(fd, &mut b), "read");
    check_eq!(b[0], b'Z', "content");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_readlink_buffer() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"abcdefghij\0", &link), "symlink");
    let mut small = [0u8; 4];
    let n = check_ok!(syscall::readlink(&link, &mut small), "readlink partial");
    check_eq!(n, 4, "truncated readlink len");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn symlink_chain() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    let l1 = copy_child(&mut tmp, b"l1")?;
    let l2 = copy_child(&mut tmp, b"l2")?;
    check_ok!(syscall::symlink(b"file\0", &l1), "l1");
    check_ok!(syscall::symlink(b"l1\0", &l2), "l2");
    let st = check_ok!(syscall::stat(&l2), "stat chain");
    let ft = check_ok!(syscall::stat(&file), "stat file");
    check_eq!(st.st_ino, ft.st_ino, "same inode");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn symlink_empty_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"link")?;
    match syscall::symlink(b"\0", &link) {
        Err(Errno::EINVAL) | Err(Errno::ENOENT) => Ok(()),
        Ok(()) => {
            let mut buf = [0u8; 4];
            let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
            check_eq!(n, 0, "empty target");
            Ok(())
        }
        Err(_) => Err(crate::harness::AssertFail::msg(
            "symlink empty unexpected errno",
        )),
    }
}

#[crate::lctp_test(suite = fs)]
fn symlink_loop_stat_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::symlink(b"b\0", &a), "a->b");
    check_ok!(syscall::symlink(b"a\0", &b), "b->a");
    check_err!(syscall::stat(&a), Errno::ELOOP, "stat loop");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_loop_open_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"la")?;
    let b = copy_child(&mut tmp, b"lb")?;
    check_ok!(syscall::symlink(b"lb\0", &a), "a->b");
    check_ok!(syscall::symlink(b"la\0", &b), "b->a");
    check_err!(
        syscall::open(&a, oflag::O_RDONLY, 0),
        Errno::ELOOP,
        "open loop"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    check_err!(
        syscall::symlink(b"x\0", &file),
        Errno::EEXIST,
        "eexist"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_dangling_lstat_ok() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"dangle")?;
    check_ok!(syscall::symlink(b"missing\0", &link), "symlink");
    let st = check_ok!(syscall::lstat(&link), "lstat");
    check!(st.is_lnk(), "lnk");
    check_err!(syscall::stat(&link), Errno::ENOENT, "stat follow");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_to_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"d\0", &link), "symlink");
    let st = check_ok!(syscall::stat(&link), "stat");
    check!(st.is_dir(), "dir");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_absolute_style_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"/tmp\0", &link), "symlink");
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(&buf[..n], b"/tmp", "target");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_parent_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_ok!(syscall::chmod(&dir, 0o555), "chmod");
    let mut nested = [0u8; 160];
    let link = join_path(&dir, b"l\0", &mut nested)?;
    check_err!(
        syscall::symlink(b"t\0", link),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&dir, 0o755), "restore");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_long_target() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    let mut tgt = [0u8; 64];
    for i in 0..63 {
        tgt[i] = b'a';
    }
    tgt[63] = 0;
    check_ok!(syscall::symlink(&tgt, &link), "symlink");
    let mut buf = [0u8; 64];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(n, 63, "len");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_nlink_is_one() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    let st = check_ok!(syscall::lstat(&link), "lstat");
    check_eq!(st.st_nlink, 1, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_readlink_enoent() -> TestResult {
    check_err!(
        syscall::readlink(b"/tmp/lctp-no-symlink\0", &mut [0u8; 8]),
        Errno::ENOENT,
        "enoent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_readlink_on_file_einval() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    match syscall::readlink(&file, &mut [0u8; 8]) {
        Err(Errno::EINVAL) => {}
        Ok(_) => return Err(crate::harness::AssertFail::msg("readlink file ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("readlink file errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_chain_three() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    write_file(&file, b"Q")?;
    let l1 = copy_child(&mut tmp, b"s1")?;
    let l2 = copy_child(&mut tmp, b"s2")?;
    let l3 = copy_child(&mut tmp, b"s3")?;
    check_ok!(syscall::symlink(b"f\0", &l1), "s1");
    check_ok!(syscall::symlink(b"s1\0", &l2), "s2");
    check_ok!(syscall::symlink(b"s2\0", &l3), "s3");
    let fd = check_ok!(syscall::open(&l3, oflag::O_RDONLY, 0), "open");
    let mut b = [0u8; 1];
    check_ok!(syscall::read(fd, &mut b), "read");
    check_eq!(b[0], b'Q', "data");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_self_loop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"self")?;
    check_ok!(syscall::symlink(b"self\0", &link), "symlink");
    check_err!(syscall::stat(&link), Errno::ELOOP, "eloop");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_unlink_and_recreate() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"a\0", &link), "first");
    check_ok!(syscall::unlink(&link), "unlink");
    check_ok!(syscall::symlink(b"b\0", &link), "second");
    let mut buf = [0u8; 4];
    let n = check_ok!(syscall::readlink(&link, &mut buf), "readlink");
    check_eq!(&buf[..n], b"b", "new target");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn symlink_eexist_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    check_err!(syscall::symlink(b"x\0", &dir), Errno::EEXIST, "eexist");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}
