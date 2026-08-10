//! POSIX path resolution and open flag tests.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_dir, create_empty, join_path, truncate_cstr, write_file};
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

#[crate::lctp_test(suite = posix)]
fn path_triple_slashes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = copy_child(&mut tmp, b"t")?;
    check_ok!(syscall::mkdir(&sub, 0o755), "mkdir");
    let mut path = [0u8; 160];
    let end = sub.iter().position(|&c| c == 0).unwrap();
    path[..end].copy_from_slice(&sub[..end]);
    path[end] = b'/';
    path[end + 1] = b'/';
    path[end + 2] = b'/';
    path[end + 3] = 0;
    let st1 = check_ok!(syscall::stat(&sub), "stat");
    let st2 = check_ok!(syscall::stat(truncate_cstr(&path)), "stat ///");
    check_eq!(st1.st_ino, st2.st_ino, "inode");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_dot_slash_chains() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"f")?;
    let mut p = [0u8; 160];
    join_path(tmp.path(), b"./././f", &mut p)?;
    let st1 = check_ok!(syscall::stat(&file), "stat f");
    let st2 = check_ok!(syscall::stat(truncate_cstr(&p)), "stat ./");
    check_eq!(st1.st_ino, st2.st_ino, "inode");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_dotdot_chains() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = create_dir(&mut tmp, b"s", 0o755)?;
    let file = create_empty(&mut tmp, b"f")?;
    let mut p = [0u8; 160];
    join_path(&sub, b"../f", &mut p)?;
    let st1 = check_ok!(syscall::stat(&file), "stat");
    let st2 = check_ok!(syscall::stat(truncate_cstr(&p)), "via ..");
    check_eq!(st1.st_ino, st2.st_ino, "inode");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_symlink_loop_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"a")?;
    let b = copy_child(&mut tmp, b"b")?;
    check_ok!(syscall::symlink(b"b\0", &a), "a->b");
    check_ok!(syscall::symlink(b"a\0", &b), "b->a");
    check_err!(
        syscall::open(&a, oflag::O_RDONLY, 0),
        Errno::ELOOP,
        "loop"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_symlink_self_loop_eloop() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = copy_child(&mut tmp, b"self")?;
    check_ok!(syscall::symlink(b"self\0", &a), "self");
    check_err!(
        syscall::open(&a, oflag::O_RDONLY, 0),
        Errno::ELOOP,
        "self loop"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_openat_at_fdcwd_relative() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"rel")?;
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let fd = check_ok!(
        syscall::openat(syscall::AT_FDCWD, b"rel\0", oflag::O_RDONLY, 0),
        "openat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_openat_dirfd_relative() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"inside")?;
    let dirfd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "dirfd"
    );
    let fd = check_ok!(
        syscall::openat(dirfd, b"inside\0", oflag::O_RDONLY, 0),
        "openat"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::close(dirfd), "close dir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_openat_at_fdcwd_missing() -> TestResult {
    check_err!(
        syscall::openat(syscall::AT_FDCWD, b"lctp-no-such-rel-path\0", oflag::O_RDONLY, 0),
        Errno::ENOENT,
        "missing"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_trailing_slash_mkdir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = copy_child(&mut tmp, b"ts")?;
    let mut with = [0u8; 160];
    let end = dir.iter().position(|&c| c == 0).unwrap();
    with[..end].copy_from_slice(&dir[..end]);
    with[end] = b'/';
    with[end + 1] = 0;
    check_ok!(syscall::mkdir(truncate_cstr(&with), 0o755), "mkdir/");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_trailing_double_slash_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"dd", 0o755)?;
    let mut with = [0u8; 160];
    let end = dir.iter().position(|&c| c == 0).unwrap();
    with[..end].copy_from_slice(&dir[..end]);
    with[end] = b'/';
    with[end + 1] = b'/';
    with[end + 2] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&with), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "open //"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::rmdir(&dir), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_long_relative_name() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let mut name = [b'a'; 120];
    name[119] = 0;
    let fd = check_ok!(
        syscall::open(&name, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "long creat"
    );
    check_ok!(syscall::close(fd), "close");
    let st = check_ok!(syscall::stat(&name), "stat");
    check!(st.is_reg(), "reg");
    check_ok!(syscall::unlink(&name), "unlink");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn path_long_relative_nested_dots() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"t")?;
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let fd = check_ok!(
        syscall::open(b"./././././t\0", oflag::O_RDONLY, 0),
        "dots"
    );
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_empty_component_slashes() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"g")?;
    let mut p = [0u8; 160];
    // tmp//g
    let end = tmp.path().iter().position(|&c| c == 0).unwrap();
    p[..end].copy_from_slice(&tmp.path()[..end]);
    p[end] = b'/';
    p[end + 1] = b'/';
    p[end + 2] = b'g';
    p[end + 3] = 0;
    let st1 = check_ok!(syscall::stat(&file), "stat");
    let st2 = check_ok!(syscall::stat(truncate_cstr(&p)), "stat //");
    check_eq!(st1.st_ino, st2.st_ino, "inode");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_dotdot_to_same_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let sub = create_dir(&mut tmp, b"x", 0o755)?;
    let mut p = [0u8; 160];
    join_path(&sub, b"../x", &mut p)?;
    let st1 = check_ok!(syscall::stat(&sub), "stat");
    let st2 = check_ok!(syscall::stat(truncate_cstr(&p)), "via ../x");
    check_eq!(st1.st_ino, st2.st_ino, "inode");
    check_ok!(syscall::rmdir(&sub), "rmdir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_openat_creat_excl() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let dirfd = check_ok!(
        syscall::open(tmp.path(), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "dirfd"
    );
    let fd = check_ok!(
        syscall::openat(
            dirfd,
            b"new\0",
            oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL,
            0o644
        ),
        "creat"
    );
    check_ok!(syscall::close(fd), "close");
    check_err!(
        syscall::openat(
            dirfd,
            b"new\0",
            oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL,
            0o644
        ),
        Errno::EEXIST,
        "excl"
    );
    check_ok!(syscall::unlinkat(dirfd, b"new\0", 0), "unlink");
    check_ok!(syscall::close(dirfd), "close dir");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_stat_dot_equals_cwd_dir() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let st_dot = check_ok!(syscall::stat(b".\0"), "stat .");
    let st_abs = check_ok!(syscall::stat(tmp.path()), "stat abs");
    check_eq!(st_dot.st_ino, st_abs.st_ino, "inode");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_lstat_symlink_is_lnk() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    let st = check_ok!(syscall::lstat(&link), "lstat");
    check!(st.is_lnk(), "lnk");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_stat_follows_symlink() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"t")?;
    let link = copy_child(&mut tmp, b"l")?;
    check_ok!(syscall::symlink(b"t\0", &link), "symlink");
    let st_f = check_ok!(syscall::stat(&file), "stat f");
    let st_l = check_ok!(syscall::stat(&link), "stat l");
    check_eq!(st_f.st_ino, st_l.st_ino, "follow");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_open_cloexec_directory() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(
        syscall::open(
            tmp.path(),
            oflag::O_RDONLY | oflag::O_DIRECTORY | oflag::O_CLOEXEC,
            0
        ),
        "open"
    );
    let flags = check_ok!(syscall::fcntl(fd, syscall::fcntl_cmd::F_GETFD, 0), "getfd");
    check!(flags as i32 & syscall::FD_CLOEXEC != 0, "cloexec");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_trailing_slash_symlink_to_dir() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let dir = create_dir(&mut tmp, b"d", 0o755)?;
    let link = copy_child(&mut tmp, b"ld")?;
    check_ok!(syscall::symlink(b"d\0", &link), "symlink");
    let mut with = [0u8; 160];
    let end = link.iter().position(|&c| c == 0).unwrap();
    with[..end].copy_from_slice(&link[..end]);
    with[end] = b'/';
    with[end + 1] = 0;
    let fd = check_ok!(
        syscall::open(truncate_cstr(&with), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "open link/"
    );
    check_ok!(syscall::close(fd), "close");
    let _ = dir;
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_trailing_slash_symlink_to_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"f")?;
    let link = copy_child(&mut tmp, b"lf")?;
    check_ok!(syscall::symlink(b"f\0", &link), "symlink");
    let mut with = [0u8; 160];
    let end = link.iter().position(|&c| c == 0).unwrap();
    with[..end].copy_from_slice(&link[..end]);
    with[end] = b'/';
    with[end + 1] = 0;
    check_err!(
        syscall::open(truncate_cstr(&with), oflag::O_RDONLY, 0),
        Errno::ENOTDIR,
        "slash on file link"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn path_deep_dotdot_chain() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let a = create_dir(&mut tmp, b"a", 0o755)?;
    let mut b = [0u8; 160];
    join_path(&a, b"b", &mut b)?;
    check_ok!(syscall::mkdir(truncate_cstr(&b), 0o755), "mkdir b");
    let file = create_empty(&mut tmp, b"rootf")?;
    let mut p = [0u8; 160];
    join_path(truncate_cstr(&b), b"../../rootf", &mut p)?;
    let st1 = check_ok!(syscall::stat(&file), "stat");
    let st2 = check_ok!(syscall::stat(truncate_cstr(&p)), "deep ..");
    check_eq!(st1.st_ino, st2.st_ino, "inode");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_openat_bad_dirfd_ebadf() -> TestResult {
    check_err!(
        syscall::openat(-1, b"x\0", oflag::O_RDONLY, 0),
        Errno::EBADF,
        "bad dirfd"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_relative_open_after_chdir() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let _ = create_empty(&mut tmp, b"m")?;
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let fd = check_ok!(syscall::open(b"m\0", oflag::O_RDONLY, 0), "open");
    check_ok!(syscall::close(fd), "close");
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_nametoolong_soft() -> TestResult {
    let mut saved = [0u8; 256];
    let n = check_ok!(syscall::getcwd(&mut saved), "save");
    let tmp = check_ok!(TempDir::create(), "tempdir");
    check_ok!(syscall::chdir(tmp.path()), "chdir");
    let mut name = [b'x'; 300];
    name[299] = 0;
    match syscall::open(&name, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644) {
        Err(Errno::ENAMETOOLONG) => {}
        Ok(fd) => {
            let _ = syscall::close(fd);
            let _ = syscall::unlink(&name);
        }
        Err(Errno::EINVAL) | Err(Errno::ENOENT) => {}
        Err(_) => {
            let _ = syscall::chdir(&saved[..n]);
            return Err(crate::harness::AssertFail::msg("nametoolong"));
        }
    }
    check_ok!(syscall::chdir(&saved[..n]), "restore");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn path_dot_component_open() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let file = create_empty(&mut tmp, b"z")?;
    let mut p = [0u8; 160];
    join_path(tmp.path(), b".", &mut p)?;
    // open dir via .
    let fd = check_ok!(
        syscall::open(truncate_cstr(&p), oflag::O_RDONLY | oflag::O_DIRECTORY, 0),
        "open ."
    );
    check_ok!(syscall::close(fd), "close");
    let _ = file;
    Ok(())
}
