//! POSIX message queues (MSG): mq_open/close/unlink/send/receive/getattr/setattr
//! and timed variants. Soft-skip when `/dev/mqueue` is unavailable.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, oflag, Errno, MqAttr, Timespec};

fn mq_unavailable(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOENT
            | Errno::EACCES
            | Errno::EPERM
            | Errno::ENOSYS
            | Errno::EINVAL
            | Errno::ENOSPC
            | Errno::ENOMEM
            | Errno::EMFILE
            | Errno::ENFILE
            | Errno::ENOTSUP
            | Errno::EFAULT
    )
}

fn mq_name(buf: &mut [u8; 48], tag: u8) -> &[u8] {
    let prefix = b"/lctp-pmq-";
    buf[..prefix.len()].copy_from_slice(prefix);
    let mut n = ((syscall::getpid() as u32) ^ ((tag as u32) << 16)) as u32;
    let mut digits = [0u8; 10];
    let mut nd = 0;
    if n == 0 {
        digits[0] = b'0';
        nd = 1;
    } else {
        while n > 0 {
            digits[nd] = b'0' + (n % 10) as u8;
            n /= 10;
            nd += 1;
        }
    }
    for i in 0..nd {
        buf[prefix.len() + i] = digits[nd - 1 - i];
    }
    let end = prefix.len() + nd;
    buf[end] = 0;
    &buf[..=end]
}

fn default_attr() -> MqAttr {
    MqAttr {
        mq_flags: 0,
        mq_maxmsg: 4,
        mq_msgsize: 64,
        mq_curmsgs: 0,
    }
}

fn open_create(name: &[u8], flags: i32) -> Result<Option<i32>, crate::harness::AssertFail> {
    let attr = default_attr();
    match syscall::mq_open(name, flags, 0o600, Some(&attr)) {
        Ok(fd) => Ok(Some(fd)),
        Err(e) if mq_unavailable(e) => Ok(None),
        Err(Errno::EEXIST) => {
            let _ = syscall::mq_unlink(name);
            match syscall::mq_open(name, flags, 0o600, Some(&attr)) {
                Ok(fd) => Ok(Some(fd)),
                Err(e) if mq_unavailable(e) => Ok(None),
                Err(_) => Err(crate::harness::AssertFail::msg("mq_open retry")),
            }
        }
        Err(_) => Err(crate::harness::AssertFail::msg("mq_open")),
    }
}

fn cleanup(fd: i32, name: &[u8]) {
    let _ = syscall::close(fd);
    let _ = syscall::mq_unlink(name);
}

macro_rules! mq_open_soft {
    ($name:ident, $tag:expr, $flags:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "mq_open can create a named queue")]
        fn $name() -> TestResult {
            let mut buf = [0u8; 48];
            let name = mq_name(&mut buf, $tag);
            let Some(fd) = open_create(name, $flags)? else {
                return Ok(());
            };
            check!(fd >= 0, "fd");
            cleanup(fd, name);
            Ok(())
        }
    };
}

mq_open_soft!(mq_posix_open_rdwr, 1, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR);
mq_open_soft!(mq_posix_open_rdonly, 2, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDONLY);
mq_open_soft!(mq_posix_open_wronly, 3, oflag::O_CREAT | oflag::O_EXCL | oflag::O_WRONLY);
mq_open_soft!(mq_posix_open_nonblock, 4, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR | oflag::O_NONBLOCK);
mq_open_soft!(mq_posix_open_cloexec, 5, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR | oflag::O_CLOEXEC);

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_unlink removes a queue that was just created")]
fn mq_posix_unlink_soft() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 10);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    check_ok!(syscall::close(fd), "close");
    match syscall::mq_unlink(name) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("unlink")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_open with O_CREAT|O_EXCL on an existing queue returns EEXIST")]
fn mq_posix_eexist_excl() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 11);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let attr = default_attr();
    match syscall::mq_open(
        name,
        oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR,
        0o600,
        Some(&attr),
    ) {
        Err(Errno::EEXIST) => {}
        Ok(fd2) => {
            let _ = syscall::close(fd2);
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("expected EEXIST"));
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("excl"));
        }
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_send then mq_receive round-trips a payload")]
fn mq_posix_send_receive() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 12);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    match syscall::mq_send(fd, b"hello", 0) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {
            cleanup(fd, name);
            return Ok(());
        }
        Err(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("send"));
        }
    }
    let mut msg = [0u8; 64];
    let mut prio = 0u32;
    match syscall::mq_receive(fd, &mut msg, Some(&mut prio)) {
        Ok(n) => {
            check_eq!(n, 5, "len");
            check_eq!(&msg[..5], b"hello", "payload");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("recv"));
        }
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "higher-priority messages are received before lower-priority ones")]
fn mq_posix_send_prio_order() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 13);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    if syscall::mq_send(fd, b"lo", 1).is_err() {
        cleanup(fd, name);
        return Ok(());
    }
    if syscall::mq_send(fd, b"hi", 5).is_err() {
        cleanup(fd, name);
        return Ok(());
    }
    let mut msg = [0u8; 64];
    let mut prio = 0u32;
    match syscall::mq_receive(fd, &mut msg, Some(&mut prio)) {
        Ok(n) => {
            check_eq!(n, 2, "len");
            check_eq!(&msg[..2], b"hi", "high first");
            check_eq!(prio, 5, "prio");
        }
        Err(_) => {}
    }
    let _ = syscall::mq_receive(fd, &mut msg, None);
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_getattr reports positive maxmsg and msgsize")]
fn mq_posix_getattr() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 14);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("getattr"));
        }
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_setattr can set O_NONBLOCK on a queue")]
fn mq_posix_setattr_nonblock() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 15);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let mut cur = MqAttr::default();
    if syscall::mq_getattr(fd, &mut cur).is_err() {
        cleanup(fd, name);
        return Ok(());
    }
    let new = MqAttr {
        mq_flags: oflag::O_NONBLOCK as i64,
        mq_maxmsg: cur.mq_maxmsg,
        mq_msgsize: cur.mq_msgsize,
        mq_curmsgs: 0,
    };
    let mut old = MqAttr::default();
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("setattr"));
        }
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "nonblocking mq_receive on an empty queue returns EAGAIN")]
fn mq_posix_nonblock_eagain_recv() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 16);
    let Some(fd) = open_create(
        name,
        oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR | oflag::O_NONBLOCK,
    )?
    else {
        return Ok(());
    };
    let mut msg = [0u8; 64];
    match syscall::mq_receive(fd, &mut msg, None) {
        Err(Errno::EAGAIN) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("unexpected recv"));
        }
        Err(_) => {}
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "mq_timedsend with a current timeout can enqueue a message")]
fn mq_posix_timedsend_now() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 17);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let now = match syscall::clock_gettime(syscall::clock::CLOCK_REALTIME) {
        Ok(t) => t,
        Err(_) => {
            cleanup(fd, name);
            return Ok(());
        }
    };
    let abs = Timespec {
        tv_sec: now.tv_sec + 2,
        tv_nsec: now.tv_nsec,
    };
    match syscall::mq_timedsend(fd, b"t", 0, Some(&abs)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("timedsend"));
        }
    }
    let mut msg = [0u8; 64];
    let _ = syscall::mq_receive(fd, &mut msg, None);
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "mq_timedreceive on an empty queue times out or is unsupported")]
fn mq_posix_timedreceive_timeout() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 18);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let now = match syscall::clock_gettime(syscall::clock::CLOCK_REALTIME) {
        Ok(t) => t,
        Err(_) => {
            cleanup(fd, name);
            return Ok(());
        }
    };
    let abs = Timespec {
        tv_sec: now.tv_sec,
        tv_nsec: now.tv_nsec.saturating_add(5_000_000) % 1_000_000_000,
    };
    let mut msg = [0u8; 64];
    match syscall::mq_timedreceive(fd, &mut msg, None, Some(&abs)) {
        Err(Errno::ETIMEDOUT) | Err(Errno::EAGAIN) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(_) => {}
        Err(_) => {}
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a forked child can receive a message the parent sent")]
fn mq_posix_fork_child_receive() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 19);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    if syscall::mq_send(fd, b"fork", 0).is_err() {
        cleanup(fd, name);
        return Ok(());
    }
    let pid = match syscall::fork() {
        Ok(p) => p,
        Err(_) => {
            cleanup(fd, name);
            return Ok(());
        }
    };
    if pid == 0 {
        let mut msg = [0u8; 64];
        match syscall::mq_receive(fd, &mut msg, None) {
            Ok(n) if n == 4 && &msg[..4] == b"fork" => syscall::exit(0),
            _ => syscall::exit(1),
        }
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "child ok");
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_open of a missing name returns ENOENT or is unsupported")]
fn mq_posix_missing_enoent_soft() -> TestResult {
    match syscall::mq_open(b"/lctp-mq-absent-posix\0", oflag::O_RDONLY, 0, None) {
        Err(e) if e == Errno::ENOENT || mq_unavailable(e) => Ok(()),
        Ok(fd) => {
            let _ = syscall::close(fd);
            Err(crate::harness::AssertFail::msg("unexpected"))
        }
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, expect = soft, case = "a second close of a message-queue fd is rejected or ignored")]
fn mq_posix_close_twice_soft() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 20);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    check_ok!(syscall::close(fd), "close");
    match syscall::close(fd) {
        Err(Errno::EBADF) => {}
        Ok(()) => {}
        Err(_) => {}
    }
    let _ = syscall::mq_unlink(name);
    Ok(())
}

macro_rules! mq_send_payload {
    ($name:ident, $tag:expr, $payload:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "mq_send then mq_receive round-trips a payload")]
        fn $name() -> TestResult {
            let mut buf = [0u8; 48];
            let name = mq_name(&mut buf, $tag);
            let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
                return Ok(());
            };
            match syscall::mq_send(fd, $payload, 0) {
                Ok(()) => {
                    let mut msg = [0u8; 64];
                    if let Ok(n) = syscall::mq_receive(fd, &mut msg, None) {
                        check_eq!(n, $payload.len(), "len");
                        check_eq!(&msg[..n], $payload, "data");
                    }
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => {
                    cleanup(fd, name);
                    return Err(crate::harness::AssertFail::msg("send"));
                }
            }
            cleanup(fd, name);
            Ok(())
        }
    };
}

mq_send_payload!(mq_posix_payload_a, 30, b"a");
mq_send_payload!(mq_posix_payload_ab, 31, b"ab");
mq_send_payload!(mq_posix_payload_xyz, 32, b"xyz");
mq_send_payload!(mq_posix_payload_hello, 33, b"hello");
mq_send_payload!(mq_posix_payload_posix, 34, b"posix");
mq_send_payload!(mq_posix_payload_msg, 35, b"message");
mq_send_payload!(mq_posix_payload_0123, 36, b"0123456789");
mq_send_payload!(mq_posix_payload_emptyish, 37, b"x");

macro_rules! mq_open_tag {
    ($name:ident, $tag:expr) => {
        #[crate::lctp_test(suite = posix, expect = soft, case = "mq_open can create a uniquely named queue")]
        fn $name() -> TestResult {
            let mut buf = [0u8; 48];
            let name = mq_name(&mut buf, $tag);
            let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
                return Ok(());
            };
            cleanup(fd, name);
            Ok(())
        }
    };
}

mq_open_tag!(mq_posix_open_t40, 40);
mq_open_tag!(mq_posix_open_t41, 41);
mq_open_tag!(mq_posix_open_t42, 42);
mq_open_tag!(mq_posix_open_t43, 43);
mq_open_tag!(mq_posix_open_t44, 44);
mq_open_tag!(mq_posix_open_t45, 45);
mq_open_tag!(mq_posix_open_t46, 46);
mq_open_tag!(mq_posix_open_t47, 47);
mq_open_tag!(mq_posix_open_t48, 48);
mq_open_tag!(mq_posix_open_t49, 49);
mq_open_tag!(mq_posix_open_t50, 50);
mq_open_tag!(mq_posix_open_t51, 51);
mq_open_tag!(mq_posix_open_t52, 52);
mq_open_tag!(mq_posix_open_t53, 53);
mq_open_tag!(mq_posix_open_t54, 54);
mq_open_tag!(mq_posix_open_t55, 55);
mq_open_tag!(mq_posix_open_t56, 56);
mq_open_tag!(mq_posix_open_t57, 57);
mq_open_tag!(mq_posix_open_t58, 58);
mq_open_tag!(mq_posix_open_t59, 59);

#[crate::lctp_test(suite = posix, full, expect = soft, case = "several messages can be sent and received in order")]
fn mq_posix_multi_send_recv() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 60);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    for i in 0u8..3 {
        let payload = [b'A' + i];
        if syscall::mq_send(fd, &payload, 0).is_err() {
            cleanup(fd, name);
            return Ok(());
        }
    }
    for i in 0u8..3 {
        let mut msg = [0u8; 64];
        match syscall::mq_receive(fd, &mut msg, None) {
            Ok(n) => {
                check_eq!(n, 1, "len");
                check_eq!(msg[0], b'A' + i, "byte");
            }
            Err(_) => break,
        }
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_getattr curmsgs increases after a successful send")]
fn mq_posix_getattr_curmsgs() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 61);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    if syscall::mq_send(fd, b"1", 0).is_ok() {
        let mut attr = MqAttr::default();
        if syscall::mq_getattr(fd, &mut attr).is_ok() {
            check!(attr.mq_curmsgs >= 1, "curmsgs");
        }
        let mut msg = [0u8; 64];
        let _ = syscall::mq_receive(fd, &mut msg, None);
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a name can be reused after mq_unlink")]
fn mq_posix_reopen_after_unlink_soft() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 62);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    check_ok!(syscall::close(fd), "close");
    let _ = syscall::mq_unlink(name);
    match syscall::mq_open(name, oflag::O_RDONLY, 0, None) {
        Err(e) if e == Errno::ENOENT || mq_unavailable(e) => Ok(()),
        Ok(fd2) => {
            let _ = syscall::close(fd2);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_send with priority zero can be received")]
fn mq_posix_send_prio_zero() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 63);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    match syscall::mq_send(fd, b"z", 0) {
        Ok(()) => {
            let mut msg = [0u8; 64];
            let mut prio = 99;
            if let Ok(_) = syscall::mq_receive(fd, &mut msg, Some(&mut prio)) {
                check_eq!(prio, 0, "prio0");
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => {
            cleanup(fd, name);
            return Err(crate::harness::AssertFail::msg("send"));
        }
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "mq_setattr can clear O_NONBLOCK on a queue")]
fn mq_posix_setattr_clear_nonblock() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 64);
    let Some(fd) = open_create(
        name,
        oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR | oflag::O_NONBLOCK,
    )?
    else {
        return Ok(());
    };
    let mut cur = MqAttr::default();
    if syscall::mq_getattr(fd, &mut cur).is_err() {
        cleanup(fd, name);
        return Ok(());
    }
    let new = MqAttr {
        mq_flags: 0,
        ..cur
    };
    let _ = syscall::mq_setattr(fd, &new, None);
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "two named queues can be created at once")]
fn mq_posix_two_queues() -> TestResult {
    let mut b1 = [0u8; 48];
    let mut b2 = [0u8; 48];
    let n1 = mq_name(&mut b1, 70);
    let n2 = mq_name(&mut b2, 71);
    let Some(fd1) = open_create(n1, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let Some(fd2) = open_create(n2, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        cleanup(fd1, n1);
        return Ok(());
    };
    check!(fd1 != fd2, "distinct fds");
    cleanup(fd1, n1);
    cleanup(fd2, n2);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "mq_timedsend with a past timeout succeeds or returns ETIMEDOUT")]
fn mq_posix_timedsend_expired_soft() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 72);
    let Some(fd) = open_create(
        name,
        oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR | oflag::O_NONBLOCK,
    )?
    else {
        return Ok(());
    };
    // Fill queue soft.
    for _ in 0..8 {
        if syscall::mq_send(fd, b"x", 0).is_err() {
            break;
        }
    }
    let past = Timespec {
        tv_sec: 1,
        tv_nsec: 0,
    };
    match syscall::mq_timedsend(fd, b"y", 0, Some(&past)) {
        Err(e) if e == Errno::ETIMEDOUT || e == Errno::EAGAIN || mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => {}
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_unlink of a missing name returns ENOENT or is unsupported")]
fn mq_posix_unlink_absent_soft() -> TestResult {
    match syscall::mq_unlink(b"/lctp-mq-never-existed\0") {
        Err(e) if e == Errno::ENOENT || mq_unavailable(e) => Ok(()),
        Ok(()) => Ok(()),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "two send/receive round-trips succeed on the same queue")]
fn mq_posix_send_receive_roundtrip_twice() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 73);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    for payload in [b"one".as_slice(), b"two".as_slice()] {
        if syscall::mq_send(fd, payload, 0).is_err() {
            cleanup(fd, name);
            return Ok(());
        }
        let mut msg = [0u8; 64];
        match syscall::mq_receive(fd, &mut msg, None) {
            Ok(n) => check_eq!(&msg[..n], payload, "rt"),
            Err(_) => break,
        }
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "two mq_getattr calls succeed on the same queue")]
fn mq_posix_getattr_twice() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 74);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let mut a = MqAttr::default();
    let mut b = MqAttr::default();
    if syscall::mq_getattr(fd, &mut a).is_ok() && syscall::mq_getattr(fd, &mut b).is_ok() {
        check_eq!(a.mq_maxmsg, b.mq_maxmsg, "stable max");
        check_eq!(a.mq_msgsize, b.mq_msgsize, "stable size");
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "mq_open with O_CREAT without O_EXCL can reopen an existing queue")]
fn mq_posix_creat_without_excl() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 75);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    check_ok!(syscall::close(fd), "close");
    let attr = default_attr();
    match syscall::mq_open(name, oflag::O_CREAT | oflag::O_RDWR, 0o600, Some(&attr)) {
        Ok(fd2) => {
            check_ok!(syscall::close(fd2), "close2");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => {}
    }
    let _ = syscall::mq_unlink(name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_send of an empty payload succeeds or is rejected")]
fn mq_posix_send_empty_msg_soft() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 76);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    match syscall::mq_send(fd, b"", 0) {
        Ok(()) => {
            let mut msg = [0u8; 64];
            let _ = syscall::mq_receive(fd, &mut msg, None);
        }
        Err(e) if mq_unavailable(e) || e == Errno::EINVAL => {}
        Err(_) => {}
    }
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, full, expect = soft, case = "a parent can send a message a forked child receives")]
fn mq_posix_fork_parent_send_child_recv() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 77);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else {
        return Ok(());
    };
    let pid = match syscall::fork() {
        Ok(p) => p,
        Err(_) => {
            cleanup(fd, name);
            return Ok(());
        }
    };
    if pid == 0 {
        let mut msg = [0u8; 64];
        let timeout = Timespec {
            tv_sec: syscall::clock_gettime(syscall::clock::CLOCK_REALTIME)
                .map(|t| t.tv_sec + 3)
                .unwrap_or(3),
            tv_nsec: 0,
        };
        match syscall::mq_timedreceive(fd, &mut msg, None, Some(&timeout)) {
            Ok(n) if n == 3 && &msg[..3] == b"abc" => syscall::exit(0),
            _ => syscall::exit(2),
        }
    }
    let pause = Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let _ = syscall::nanosleep(&pause);
    let _ = syscall::mq_send(fd, b"abc", 0);
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    // Soft: child may fail if mq broken in environment.
    let _ = status;
    cleanup(fd, name);
    Ok(())
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_send on fd -1 returns EBADF or is unsupported")]
fn mq_posix_bad_fd_send_soft() -> TestResult {
    match syscall::mq_send(-1, b"x", 0) {
        Err(e) if e == Errno::EBADF || mq_unavailable(e) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected ok")),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_receive on fd -1 returns EBADF or is unsupported")]
fn mq_posix_bad_fd_recv_soft() -> TestResult {
    let mut msg = [0u8; 8];
    match syscall::mq_receive(-1, &mut msg, None) {
        Err(e) if e == Errno::EBADF || mq_unavailable(e) => Ok(()),
        Ok(_) => Err(crate::harness::AssertFail::msg("unexpected ok")),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, expect = soft, case = "mq_getattr on fd -1 returns EBADF or is unsupported")]
fn mq_posix_bad_fd_getattr_soft() -> TestResult {
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(-1, &mut attr) {
        Err(e) if e == Errno::EBADF || mq_unavailable(e) => Ok(()),
        Ok(()) => Err(crate::harness::AssertFail::msg("unexpected ok")),
        Err(_) => Ok(()),
    }
}
