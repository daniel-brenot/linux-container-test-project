//! inotify tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, write_file};
use crate::syscall::{self, oflag, InotifyEvent, IN_CLOEXEC, IN_CREATE, IN_MODIFY};

fn read_events(fd: i32, buf: &mut [u8]) -> Result<usize, crate::harness::AssertFail> {
    Ok(check_ok!(syscall::read(fd, buf), "inotify read"))
}

fn event_at(buf: &[u8], off: usize) -> InotifyEvent {
    let mut ev = InotifyEvent::default();
    let size = core::mem::size_of::<InotifyEvent>();
    unsafe {
        core::ptr::copy_nonoverlapping(
            buf[off..].as_ptr(),
            &mut ev as *mut InotifyEvent as *mut u8,
            size,
        );
    }
    ev
}

#[crate::lctp_test(suite = syscall)]
fn inotify_init1_cloexec() -> TestResult {
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init1");
    let flags = check_ok!(syscall::fcntl(fd, syscall::fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(flags & syscall::FD_CLOEXEC as usize != 0, "CLOEXEC");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn inotify_watch_create() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init1");
    let wd = check_ok!(
        syscall::inotify_add_watch(fd, tmp.path(), IN_CREATE),
        "add_watch"
    );
    check!(wd >= 0, "wd");
    let path = copy_child(&mut tmp, b"newfile")?;
    let created = check_ok!(
        syscall::open(&path, oflag::O_CREAT | oflag::O_WRONLY, 0o644),
        "create"
    );
    check_ok!(syscall::close(created), "close created");
    let mut buf = [0u8; 512];
    let n = read_events(fd, &mut buf)?;
    check!(n >= core::mem::size_of::<InotifyEvent>(), "event bytes");
    let ev = event_at(&buf, 0);
    check_eq!(ev.wd, wd, "wd match");
    check!(ev.mask & IN_CREATE != 0, "IN_CREATE");
    check_ok!(syscall::inotify_rm_watch(fd, wd), "rm_watch");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn inotify_watch_modify() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"m")?;
    write_file(&path, b"start")?;
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init1");
    let wd = check_ok!(
        syscall::inotify_add_watch(fd, &path, IN_MODIFY),
        "add_watch"
    );
    write_file(&path, b"changed")?;
    let mut buf = [0u8; 512];
    let n = read_events(fd, &mut buf)?;
    check!(n >= core::mem::size_of::<InotifyEvent>(), "event");
    let ev = event_at(&buf, 0);
    check_eq!(ev.wd, wd, "wd");
    check!(ev.mask & IN_MODIFY != 0, "IN_MODIFY");
    check_ok!(syscall::inotify_rm_watch(fd, wd), "rm");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn inotify_create_and_modify() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init1");
    let wd = check_ok!(
        syscall::inotify_add_watch(fd, tmp.path(), IN_CREATE | IN_MODIFY),
        "add_watch"
    );
    let path = copy_child(&mut tmp, b"cm")?;
    write_file(&path, b"v1")?;
    write_file(&path, b"v2")?;
    let mut buf = [0u8; 1024];
    let n = read_events(fd, &mut buf)?;
    check!(n >= core::mem::size_of::<InotifyEvent>(), "events");
    let mut saw_create = false;
    let mut saw_modify = false;
    let mut off = 0usize;
    let hdr = core::mem::size_of::<InotifyEvent>();
    while off + hdr <= n {
        let ev = event_at(&buf, off);
        if ev.mask & IN_CREATE != 0 {
            saw_create = true;
        }
        if ev.mask & IN_MODIFY != 0 {
            saw_modify = true;
        }
        off += hdr + ev.len as usize;
    }
    check!(saw_create, "saw create");
    check!(saw_modify, "saw modify");
    check_ok!(syscall::inotify_rm_watch(fd, wd), "rm");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn inotify_rm_watch_ok() -> TestResult {
    let tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(syscall::inotify_init1(IN_CLOEXEC), "init1");
    let wd = check_ok!(
        syscall::inotify_add_watch(fd, tmp.path(), IN_CREATE),
        "add"
    );
    check_ok!(syscall::inotify_rm_watch(fd, wd), "rm");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
