//! rename filesystem tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, truncate_cstr, write_file};
use crate::syscall::{self, oflag, Errno};

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a regular file moves the name and removes the old path")]
fn rename_file_basic() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    let dst = copy_child(&mut tmp, b"dst")?;
    check_ok!(syscall::rename(&src, &dst), "rename");
    check_err!(syscall::stat(&src), Errno::ENOENT, "src gone");
    check_ok!(syscall::stat(&dst), "dst exists");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a directory succeeds and the new path is a directory")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "rename over an existing file replaces its contents")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a file into a subdirectory succeeds")]
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

#[crate::lctp_test(suite = fs, full, expect = success, case = "rename of a regular file preserves the inode")]
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

#[crate::lctp_test(suite = fs, full, expect = success, case = "rename over a symlink replaces the link with a regular file")]
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

#[crate::lctp_test(suite = fs, expect = failure, case = "rename of a missing source path returns ENOENT")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a regular file preserves its contents")]
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

#[crate::lctp_test(suite = fs, full, expect = success, case = "rename of a directory into an empty parent succeeds")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a file between two directories succeeds")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "renameat2 with flags 0 moves a file to a new name")]
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

#[crate::lctp_test(suite = fs, expect = failure, case = "renameat2 with RENAME_NOREPLACE over an existing path returns EEXIST")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "renameat2 with RENAME_NOREPLACE onto a missing path succeeds")]
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

#[crate::lctp_test(suite = fs, full, expect = success, case = "renameat2 of a regular file preserves its contents")]
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

#[crate::lctp_test(suite = fs, expect = success, case = "renameat2 with RENAME_EXCHANGE swaps the contents of two files")]
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

#[crate::lctp_test(suite = fs, full, expect = success, case = "renameat2 with RENAME_EXCHANGE swaps two directories")]
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

#[crate::lctp_test(suite = fs, expect = soft, case = "renameat2 with RENAME_WHITEOUT succeeds when the filesystem supports it")]
fn renameat2_whiteout_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"wo_src")?;
    let dst = copy_child(&mut tmp, b"wo_dst")?;
    match syscall::renameat2(
        syscall::AT_FDCWD,
        &src,
        syscall::AT_FDCWD,
        &dst,
        syscall::RENAME_WHITEOUT,
    ) {
        Ok(()) => {
            check_ok!(syscall::stat(&dst), "dst");
        }
        Err(Errno::EINVAL)
        | Err(Errno::EOPNOTSUPP)
        | Err(Errno::ENOTSUP)
        | Err(Errno::EPERM)
        | Err(Errno::EACCES)
        | Err(Errno::ENOSYS) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("whiteout errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a file onto itself succeeds and the path remains")]
fn rename_to_self() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"self")?;
    write_file(&a, b"x")?;
    check_ok!(syscall::rename(&a, &a), "rename self");
    check_ok!(syscall::stat(&a), "still there");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename into a missing destination directory returns ENOENT")]
fn rename_empty_dst_component_enoent() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    let mut dest = [0u8; 160];
    let base = tmp.path();
    let blen = base.iter().position(|&c| c == 0).unwrap();
    dest[..blen].copy_from_slice(&base[..blen]);
    dest[blen..blen + 12].copy_from_slice(b"/missing/dst");
    dest[blen + 12] = 0;
    check_err!(
        syscall::rename(&src, truncate_cstr(&dest)),
        Errno::ENOENT,
        "missing parent"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, full, expect = success, case = "rename over an existing file replaces it with the source contents")]
fn rename_replace_same_content() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"ra")?;
    let b = create_empty(&mut tmp, b"rb")?;
    write_file(&a, b"same")?;
    write_file(&b, b"old")?;
    check_ok!(syscall::rename(&a, &b), "rename");
    let mut buf = [0u8; 8];
    check_eq!(crate::suites::common::read_file(&b, &mut buf)?, 4, "len");
    check_eq!(&buf[..4], b"same", "data");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename of a directory into its own subdirectory returns EINVAL, ENOTEMPTY, or EBUSY")]
fn rename_into_self_subdir_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let outer = create_dir(&mut tmp, b"outer", 0o755)?;
    let mut inner = [0u8; 160];
    let slen = outer.iter().position(|&c| c == 0).unwrap();
    inner[..slen].copy_from_slice(&outer[..slen]);
    inner[slen..slen + 6].copy_from_slice(b"/inner");
    inner[slen + 6] = 0;
    check_ok!(syscall::mkdir(truncate_cstr(&inner), 0o755), "mkdir");
    // rename outer into its own subdirectory must fail.
    match syscall::rename(&outer, truncate_cstr(&inner)) {
        Err(Errno::EINVAL) | Err(Errno::ENOTEMPTY) | Err(Errno::EBUSY) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("rename into self ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("rename into self errno")),
    }
    check_ok!(syscall::rmdir(truncate_cstr(&inner)), "rmdir inner");
    check_ok!(syscall::rmdir(&outer), "rmdir outer");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename of a file onto a directory returns EISDIR or ENOTDIR")]
fn rename_file_over_dir_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    match syscall::rename(&file, &dir) {
        Err(Errno::EISDIR) | Err(Errno::ENOTDIR) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("file over dir ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("file over dir errno")),
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename of a directory onto a file returns ENOTDIR or EISDIR")]
fn rename_dir_over_file_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let file = create_empty(&mut tmp, b"f")?;
    match syscall::rename(&dir, &file) {
        Err(Errno::ENOTDIR) | Err(Errno::EISDIR) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("dir over file ok")),
        Err(_) => return Err(crate::harness::AssertFail::msg("dir over file errno")),
    }
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename of a directory onto a nonempty directory returns ENOTEMPTY")]
fn rename_dir_over_nonempty_fails() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_dir(&mut tmp, b"a", 0o755)?;
    let b = create_dir(&mut tmp, b"b", 0o755)?;
    let mut nested = [0u8; 160];
    let blen = b.iter().position(|&c| c == 0).unwrap();
    nested[..blen].copy_from_slice(&b[..blen]);
    nested[blen..blen + 2].copy_from_slice(b"/x");
    nested[blen + 2] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&nested), oflag::O_CREAT | oflag::O_RDWR, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(syscall::rename(&a, &b), Errno::ENOTEMPTY, "notempty");
    check_ok!(syscall::unlink(truncate_cstr(&nested)), "unlink");
    check_ok!(syscall::rmdir(&b), "rmdir b");
    check_ok!(syscall::rmdir(&a), "rmdir a");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a directory onto an empty directory succeeds")]
fn rename_replace_empty_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_dir(&mut tmp, b"a", 0o755)?;
    let b = create_dir(&mut tmp, b"b", 0o755)?;
    check_ok!(syscall::rename(&a, &b), "rename");
    check_err!(syscall::stat(&a), Errno::ENOENT, "a gone");
    check!(check_ok!(syscall::stat(&b), "stat").is_dir(), "dir");
    check_ok!(syscall::rmdir(&b), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a symlink moves the link and lstat still reports a symlink")]
fn rename_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"target\0", &link), "symlink");
    let dst = copy_child(&mut tmp, b"l2")?;
    check_ok!(syscall::rename(&link, &dst), "rename");
    let st = check_ok!(syscall::lstat(&dst), "lstat");
    check!(st.is_lnk(), "lnk");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename out of a directory without write permission returns EACCES")]
fn rename_parent_src_no_write() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let d1 = create_dir(&mut tmp, b"d1", 0o755)?;
    let d2 = create_dir(&mut tmp, b"d2", 0o755)?;
    let mut f1 = [0u8; 160];
    let d1len = d1.iter().position(|&c| c == 0).unwrap();
    f1[..d1len].copy_from_slice(&d1[..d1len]);
    f1[d1len..d1len + 2].copy_from_slice(b"/f");
    f1[d1len + 2] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&f1), oflag::O_CREAT | oflag::O_RDWR, 0o644),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chmod(&d1, 0o555), "chmod");
    let mut f2 = [0u8; 160];
    let d2len = d2.iter().position(|&c| c == 0).unwrap();
    f2[..d2len].copy_from_slice(&d2[..d2len]);
    f2[d2len..d2len + 2].copy_from_slice(b"/f");
    f2[d2len + 2] = 0;
    check_err!(
        syscall::rename(truncate_cstr(&f1), truncate_cstr(&f2)),
        Errno::EACCES,
        "eacces"
    );
    check_ok!(syscall::chmod(&d1, 0o755), "restore");
    check_ok!(syscall::unlink(truncate_cstr(&f1)), "unlink");
    check_ok!(syscall::rmdir(&d1), "rmdir d1");
    check_ok!(syscall::rmdir(&d2), "rmdir d2");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of one hard-link name leaves nlink at 2")]
fn rename_hardlink_preserves_nlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_empty(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::link(&a, &b), "link");
    let c = copy_child(&mut tmp, b"c")?;
    check_ok!(syscall::rename(&b, &c), "rename");
    check_eq!(check_ok!(syscall::stat(&a), "stat").st_nlink, 2, "nlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a FIFO succeeds and the new path is a FIFO")]
fn rename_fifo() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = copy_child(&mut tmp, b"fifo")?;
    check_ok!(
        syscall::mknodat(syscall::AT_FDCWD, &src, crate::syscall::S_IFIFO | 0o644, 0),
        "mkfifo"
    );
    let dst = copy_child(&mut tmp, b"fifo2")?;
    check_ok!(syscall::rename(&src, &dst), "rename");
    check!(check_ok!(syscall::stat(&dst), "stat").is_fifo(), "fifo");
    check_ok!(syscall::unlink(&dst), "unlink");
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = failure, case = "rename through a non-directory destination component returns ENOTDIR")]
fn rename_enotdir_dst_component() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let src = create_empty(&mut tmp, b"src")?;
    let file = create_empty(&mut tmp, b"f")?;
    let mut dest = [0u8; 160];
    let flen = file.iter().position(|&c| c == 0).unwrap();
    dest[..flen].copy_from_slice(&file[..flen]);
    dest[flen..flen + 2].copy_from_slice(b"/x");
    dest[flen + 2] = 0;
    check_err!(
        syscall::rename(&src, truncate_cstr(&dest)),
        Errno::ENOTDIR,
        "enotdir"
    );
    Ok(())
}

#[crate::lctp_test(suite = fs, expect = success, case = "rename of a looping symlink moves the symlink itself")]
fn rename_loop_symlink_src() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::symlink(b"b\0", &a), "a");
    check_ok!(syscall::symlink(b"a\0", &b), "b");
    let dst = copy_child(&mut tmp, b"dst")?;
    // rename of the symlink itself should succeed (does not follow).
    check_ok!(syscall::rename(&a, &dst), "rename symlink");
    check!(check_ok!(syscall::lstat(&dst), "lstat").is_lnk(), "lnk");
    Ok(())
}
