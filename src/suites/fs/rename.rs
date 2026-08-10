//! rename filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, truncate_cstr, write_file};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = fs)]
fn rename_file_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    let dst = copy_child(&mut tmp, b"dst")?;
    check_ok!(syscall::rename(&src, &dst), "rename");
    check_err!(syscall::stat(&src), Errno::ENOENT, "src gone");
    check_ok!(syscall::stat(&dst), "dst exists");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rename_directory() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_dir(&mut tmp, b"src", 0o755)?;
    let dst = copy_child(&mut tmp, b"dst")?;
    check_ok!(syscall::rename(&src, &dst), "rename");
    let st = check_ok!(syscall::stat(&dst), "stat");
    check!(st.is_dir(), "dir");
    check_ok!(syscall::rmdir(&dst), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rename_replace_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    write_file(&a, b"A")?;
    write_file(&b, b"B")?;
    check_ok!(syscall::rename(&a, &b), "rename");
    let fd = check_ok!(syscall::open(&b, oflag::O_RDONLY, 0), "open");
    let mut buf = [0u8; 1];
    check_ok!(syscall::read(fd, &mut buf), "read");
    check_eq!(buf[0], b'A', "replaced content");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rename_into_subdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = create_dir(&mut tmp, b"sub", 0o755)?;
    let file = create_empty(&mut tmp, b"file")?;
    let mut dest = [0u8; 160];
    let slen = sub.iter().position(|&c| c == 0).unwrap();
    dest[..slen].copy_from_slice(&sub[..slen]);
    dest[slen..slen + 5].copy_from_slice(b"/file");
    dest[slen + 5] = 0;
    check_ok!(syscall::rename(&file, truncate_cstr(&dest)), "rename");
    check_err!(syscall::stat(&file), Errno::ENOENT, "old gone");
    check_ok!(syscall::stat(truncate_cstr(&dest)), "new exists");
    check_ok!(syscall::unlink(truncate_cstr(&dest)), "unlink");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn rename_same_inode() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    let ino = check_ok!(syscall::stat(&a), "stat").st_ino;
    check_ok!(syscall::rename(&a, &b), "rename");
    let st = check_ok!(syscall::stat(&b), "stat b");
    check_eq!(st.st_ino, ino, "inode preserved");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn rename_over_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"file")?;
    let link = copy_child(&mut tmp, b"link")?;
    check_ok!(syscall::symlink(b"file\0", &link), "symlink");
    let other = create_empty(&mut tmp, b"other")?;
    check_ok!(syscall::rename(&other, &link), "rename over link");
    let st = check_ok!(syscall::lstat(&link), "lstat");
    check!(st.is_reg(), "now regular file");
    check_ok!(syscall::stat(&file), "original target");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rename_missing_src() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dst = copy_child(&mut tmp, b"dst")?;
    check_err!(
        syscall::rename(b"/tmp/lctp-no-src-rename\0", &dst),
        Errno::ENOENT,
        "missing src"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rename_preserves_content() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    write_file(&src, b"payload")?;
    let dst = copy_child(&mut tmp, b"dst")?;
    check_ok!(syscall::rename(&src, &dst), "rename");
    let mut buf = [0u8; 8];
    check_eq!(
        crate::suites::common::read_file(&dst, &mut buf)?,
        7,
        "len"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn rename_dir_into_empty_parent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let parent = create_dir(&mut tmp, b"parent", 0o755)?;
    let child = create_dir(&mut tmp, b"child", 0o755)?;
    let mut nested = [0u8; 160];
    let plen = parent.iter().position(|&c| c == 0).unwrap();
    nested[..plen].copy_from_slice(&parent[..plen]);
    nested[plen..plen + 6].copy_from_slice(b"/child");
    nested[plen + 6] = 0;
    check_ok!(syscall::rename(&child, truncate_cstr(&nested)), "rename dir");
    check_ok!(syscall::rmdir(truncate_cstr(&nested)), "rmdir");
    check_ok!(syscall::rmdir(&parent), "rmdir parent");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn rename_file_cross_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let d1 = create_dir(&mut tmp, b"d1", 0o755)?;
    let d2 = create_dir(&mut tmp, b"d2", 0o755)?;
    let mut f1 = [0u8; 160];
    let d1len = d1.iter().position(|&c| c == 0).unwrap();
    f1[..d1len].copy_from_slice(&d1[..d1len]);
    f1[d1len..d1len + 5].copy_from_slice(b"/file");
    f1[d1len + 5] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&f1), oflag::O_CREAT | oflag::O_RDWR, 0o644),
        "create"
    );
    check_ok!(syscall::close(fd), "close");
    let mut f2 = [0u8; 160];
    let d2len = d2.iter().position(|&c| c == 0).unwrap();
    f2[..d2len].copy_from_slice(&d2[..d2len]);
    f2[d2len..d2len + 5].copy_from_slice(b"/file");
    f2[d2len + 5] = 0;
    check_ok!(syscall::rename(truncate_cstr(&f1), truncate_cstr(&f2)), "rename cross");
    check_ok!(syscall::unlink(truncate_cstr(&f2)), "unlink");
    check_ok!(syscall::rmdir(&d1), "rmdir d1");
    check_ok!(syscall::rmdir(&d2), "rmdir d2");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn renameat2_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src2")?;
    let dst = copy_child(&mut tmp, b"dst2")?;
    check_ok!(
        syscall::renameat2(syscall::AT_FDCWD, &src, syscall::AT_FDCWD, &dst, 0),
        "renameat2"
    );
    check_err!(syscall::stat(&src), Errno::ENOENT, "src gone");
    check_ok!(syscall::stat(&dst), "dst exists");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn renameat2_noreplace_eexist() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = create_empty(&mut tmp, b"b")?;
    write_file(&a, b"A")?;
    write_file(&b, b"B")?;
    check_err!(
        syscall::renameat2(
            syscall::AT_FDCWD,
            &a,
            syscall::AT_FDCWD,
            &b,
            syscall::RENAME_NOREPLACE
        ),
        Errno::EEXIST,
        "NOREPLACE"
    );
    // Both paths still present with original content.
    let mut buf = [0u8; 1];
    check_eq!(crate::suites::common::read_file(&a, &mut buf)?, 1, "a len");
    check_eq!(buf[0], b'A', "a data");
    check_eq!(crate::suites::common::read_file(&b, &mut buf)?, 1, "b len");
    check_eq!(buf[0], b'B', "b data");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn renameat2_noreplace_success() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    let dst = copy_child(&mut tmp, b"fresh")?;
    check_ok!(
        syscall::renameat2(
            syscall::AT_FDCWD,
            &src,
            syscall::AT_FDCWD,
            &dst,
            syscall::RENAME_NOREPLACE
        ),
        "noreplace ok"
    );
    check_ok!(syscall::stat(&dst), "dst");
    check_err!(syscall::stat(&src), Errno::ENOENT, "src");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn renameat2_preserves_content() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    write_file(&src, b"payload")?;
    let dst = copy_child(&mut tmp, b"dst")?;
    check_ok!(
        syscall::renameat2(syscall::AT_FDCWD, &src, syscall::AT_FDCWD, &dst, 0),
        "renameat2"
    );
    let mut buf = [0u8; 8];
    check_eq!(crate::suites::common::read_file(&dst, &mut buf)?, 7, "len");
    check_eq!(&buf[..7], b"payload", "data");
    Ok(())
}

#[crate::lctp_test(suite = fs)]
fn renameat2_exchange() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"ex_a")?;
    let b = create_empty(&mut tmp, b"ex_b")?;
    write_file(&a, b"AAA")?;
    write_file(&b, b"BBB")?;
    check_ok!(
        syscall::renameat2(
            syscall::AT_FDCWD,
            &a,
            syscall::AT_FDCWD,
            &b,
            syscall::RENAME_EXCHANGE
        ),
        "EXCHANGE"
    );
    let mut buf = [0u8; 4];
    check_eq!(crate::suites::common::read_file(&a, &mut buf)?, 3, "a len");
    check_eq!(&buf[..3], b"BBB", "a now B");
    check_eq!(crate::suites::common::read_file(&b, &mut buf)?, 3, "b len");
    check_eq!(&buf[..3], b"AAA", "b now A");
    Ok(())
}

#[crate::lctp_test(suite = fs, full)]
fn renameat2_exchange_dirs() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let d1 = create_dir(&mut tmp, b"d1", 0o755)?;
    let d2 = create_dir(&mut tmp, b"d2", 0o755)?;
    // Place a marker file in d1.
    let mut marker = [0u8; 160];
    let d1len = d1.iter().position(|&c| c == 0).unwrap();
    marker[..d1len].copy_from_slice(&d1[..d1len]);
    marker[d1len..d1len + 2].copy_from_slice(b"/m");
    marker[d1len + 2] = 0;
    let fd = check_ok!(
        syscall::open(
            truncate_cstr(&marker),
            oflag::O_CREAT | oflag::O_WRONLY,
            0o644
        ),
        "marker"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(
        syscall::renameat2(
            syscall::AT_FDCWD,
            &d1,
            syscall::AT_FDCWD,
            &d2,
            syscall::RENAME_EXCHANGE
        ),
        "exchange dirs"
    );
    // Marker should now live under the path still named d2 (exchanged).
    let mut marker2 = [0u8; 160];
    let d2len = d2.iter().position(|&c| c == 0).unwrap();
    marker2[..d2len].copy_from_slice(&d2[..d2len]);
    marker2[d2len..d2len + 2].copy_from_slice(b"/m");
    marker2[d2len + 2] = 0;
    check_ok!(syscall::stat(truncate_cstr(&marker2)), "marker under d2");
    check_ok!(syscall::unlink(truncate_cstr(&marker2)), "unlink");
    check_ok!(syscall::rmdir(&d1), "rmdir d1");
    check_ok!(syscall::rmdir(&d2), "rmdir d2");
    Ok(())
}
