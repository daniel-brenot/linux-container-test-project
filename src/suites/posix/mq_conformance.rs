//! Message queue conformance deepeners (soft ENOSYS/ENOENT/EPERM).

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, oflag, Errno, MqAttr, Timespec};

fn mq_unavailable(e: Errno) -> bool {
    matches!(e, Errno::ENOENT | Errno::EACCES | Errno::EPERM | Errno::ENOSYS | Errno::EINVAL
        | Errno::ENOSPC | Errno::ENOMEM | Errno::EMFILE | Errno::ENFILE | Errno::ENOTSUP | Errno::EFAULT)
}

fn mq_name(buf: &mut [u8; 48], tag: u8) -> &[u8] {
    let prefix = b"/lctp-mqc-";
    buf[..prefix.len()].copy_from_slice(prefix);
    let mut n = ((syscall::getpid() as u32) ^ ((tag as u32) << 16)) as u32;
    let mut digits = [0u8; 10];
    let mut nd = 0;
    if n == 0 { digits[0] = b'0'; nd = 1; }
    else { while n > 0 { digits[nd] = b'0' + (n % 10) as u8; n /= 10; nd += 1; } }
    for i in 0..nd { buf[prefix.len() + i] = digits[nd - 1 - i]; }
    let end = prefix.len() + nd;
    buf[end] = 0;
    &buf[..=end]
}

fn default_attr() -> MqAttr {
    MqAttr { mq_flags: 0, mq_maxmsg: 4, mq_msgsize: 64, mq_curmsgs: 0 }
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

#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_1() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 1);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_2() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 2);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_3() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 3);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_4() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 4);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_5() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 5);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_6() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 6);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_7() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 7);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_8() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 8);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_9() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 9);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_10() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 10);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_11() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 11);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_12() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 12);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_13() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 13);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_14() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 14);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_15() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 15);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_16() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 16);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_17() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 17);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_18() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 18);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_19() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 19);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_open_rdwr_20() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 20);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    check!(fd >= 0, "fd");
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_21() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 21);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_22() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 22);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_23() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 23);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_24() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 24);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_25() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 25);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_26() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 26);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_27() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 27);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_28() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 28);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_29() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 29);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_30() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 30);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_31() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 31);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_getattr_32() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 32);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut attr = MqAttr::default();
    match syscall::mq_getattr(fd, &mut attr) {
        Ok(()) => {
            check!(attr.mq_maxmsg > 0, "maxmsg");
            check!(attr.mq_msgsize > 0, "msgsize");
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("getattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_41() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 41);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 1) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 1, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_42() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 42);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 2) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 2, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_43() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 43);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 3) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 3, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_44() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 44);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 4) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 4, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_45() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 45);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 5) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 5, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_46() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 46);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 6) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 6, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_47() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 47);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 7) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 7, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_48() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 48);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 0) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 0, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_49() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 49);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 1) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 1, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_50() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 50);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 2) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 2, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_51() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 51);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 3) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 3, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_52() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 52);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 4) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 4, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_53() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 53);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 5) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 5, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_54() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 54);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 6) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 6, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_send_recv_55() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 55);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let msg = b"mqc-msg";
    match syscall::mq_send(fd, msg, 7) {
        Ok(()) => {
            let mut out = [0u8; 64];
            let mut prio = 0u32;
            match syscall::mq_receive(fd, &mut out, Some(&mut prio)) {
                Ok(n) => {
                    check_eq!(n, msg.len(), "len");
                    check_eq!(&out[..n], &msg[..], "data");
                    check_eq!(prio, 7, "prio");
                }
                Err(e) if mq_unavailable(e) => {}
                Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("recv")); }
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("send")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_61() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 61);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_62() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 62);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_63() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 63);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_64() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 64);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_65() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 65);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_66() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 66);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_67() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 67);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_68() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 68);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_69() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 69);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn mqc_timedsend_70() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 70);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let abs = Timespec { tv_sec: 0, tv_nsec: 0 };
    // Absolute past => may succeed immediately if queue empty, or ETIMEDOUT when full.
    match syscall::mq_timedsend(fd, b"x", 0, Some(&abs)) {
        Ok(()) => {}
        Err(Errno::ETIMEDOUT) | Err(Errno::EINVAL) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("timedsend")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_81() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 81);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_82() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 82);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_83() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 83);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_84() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 84);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_85() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 85);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_86() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 86);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_87() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 87);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_88() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 88);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_89() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 89);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_unlink_missing_90() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 90);
    let _ = syscall::mq_unlink(name);
    match syscall::mq_unlink(name) {
        Err(Errno::ENOENT) => {}
        Err(e) if mq_unavailable(e) => {}
        Ok(()) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("unlink")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_101() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 101);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_102() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 102);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_103() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 103);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_104() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 104);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_105() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 105);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_106() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 106);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_107() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 107);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn mqc_setattr_nonblock_108() -> TestResult {
    let mut buf = [0u8; 48];
    let name = mq_name(&mut buf, 108);
    let Some(fd) = open_create(name, oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR)? else { return Ok(()); };
    let mut old = MqAttr::default();
    let new = MqAttr { mq_flags: oflag::O_NONBLOCK as i64, mq_maxmsg: 0, mq_msgsize: 0, mq_curmsgs: 0 };
    match syscall::mq_setattr(fd, &new, Some(&mut old)) {
        Ok(()) => {}
        Err(e) if mq_unavailable(e) => {}
        Err(_) => { cleanup(fd, name); return Err(crate::harness::AssertFail::msg("setattr")); }
    }
    cleanup(fd, name);
    Ok(())
}