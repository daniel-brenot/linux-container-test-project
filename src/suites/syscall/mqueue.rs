//! POSIX message queue (mq_open) probe tests.

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, oflag, Errno, MqAttr};

fn mq_name(buf: &mut [u8; 32]) -> &[u8] {
    let prefix = b"/lctp-mq-";
    buf[..prefix.len()].copy_from_slice(prefix);
    let mut n = syscall::getpid() as u32;
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
    )
}

#[crate::lctp_test(suite = syscall)]
fn mq_open_create_unlink_soft() -> TestResult {
    let mut namebuf = [0u8; 32];
    let name = mq_name(&mut namebuf);
    let attr = MqAttr {
        mq_flags: 0,
        mq_maxmsg: 4,
        mq_msgsize: 64,
        mq_curmsgs: 0,
    };
    match syscall::mq_open(
        name,
        oflag::O_CREAT | oflag::O_EXCL | oflag::O_RDWR,
        0o600,
        Some(&attr),
    ) {
        Ok(fd) => {
            check!(fd >= 0, "fd");
            check_ok!(syscall::close(fd), "close");
            match syscall::mq_unlink(name) {
                Ok(()) => {}
                Err(e) if mq_unavailable(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("mq_unlink")),
            }
        }
        Err(e) if mq_unavailable(e) => {}
        Err(Errno::EEXIST) => {
            let _ = syscall::mq_unlink(name);
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("mq_open unexpected")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn mq_open_missing_name_soft() -> TestResult {
    match syscall::mq_open(b"/lctp-mq-absent-xyz\0", oflag::O_RDONLY, 0, None) {
        Err(e) if mq_unavailable(e) => Ok(()),
        Ok(fd) => {
            let _ = syscall::close(fd);
            Err(crate::harness::AssertFail::msg("unexpected open"))
        }
        Err(_) => Ok(()),
    }
}
