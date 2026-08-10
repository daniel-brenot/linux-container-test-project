//! userfaultfd(2) probe tests.

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, UFFD_CLOEXEC};

#[crate::lctp_test(suite = syscall)]
fn userfaultfd_probe_soft() -> TestResult {
    match syscall::userfaultfd(UFFD_CLOEXEC) {
        Ok(fd) => {
            check!(fd >= 0, "fd");
            check_ok!(syscall::close(fd), "close");
            Ok(())
        }
        Err(Errno::EPERM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EACCES) => {
            Ok(())
        }
        Err(_) => Err(crate::harness::AssertFail::msg("userfaultfd unexpected")),
    }
}

#[crate::lctp_test(suite = syscall, full)]
fn userfaultfd_nonblock_soft() -> TestResult {
    match syscall::userfaultfd(UFFD_CLOEXEC | crate::syscall::UFFD_NONBLOCK) {
        Ok(fd) => {
            check_ok!(syscall::close(fd), "close");
            Ok(())
        }
        Err(Errno::EPERM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EACCES) => {
            Ok(())
        }
        Err(_) => Err(crate::harness::AssertFail::msg("userfaultfd nb")),
    }
}
