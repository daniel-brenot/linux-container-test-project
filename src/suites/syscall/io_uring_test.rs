//! io_uring_setup / enter / register probes (soft on restricted kernels).

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, IoUringParams};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::EINVAL | Errno::ENOMEM | Errno::EOPNOTSUPP
    )
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "io_uring_setup succeeds, or is rejected as unsupported")]
fn io_uring_setup_probe_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(1, &mut params) {
        Ok(fd) => {
            check!(fd >= 0, "ring fd");
            check_ok!(syscall::close(fd), "close");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("io_uring_setup")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "io_uring_setup with four entries reports sq_entries, or is rejected as unsupported")]
fn io_uring_setup_entries_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(4, &mut params) {
        Ok(fd) => {
            check!(params.sq_entries >= 1, "sq_entries");
            check_ok!(syscall::close(fd), "close");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("io_uring_setup 4")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "io_uring_enter with zero submit and complete counts succeeds or is rejected as unsupported")]
fn io_uring_enter_nop_soft() -> TestResult {
    let mut params = IoUringParams::default();
    let fd = match syscall::io_uring_setup(1, &mut params) {
        Ok(fd) => fd,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("setup")),
    };
    // Submit/complete nothing — may succeed with 0 or fail EINVAL without mapped rings.
    match syscall::io_uring_enter(fd, 0, 0, 0, 0) {
        Ok(_) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("io_uring_enter"));
        }
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "io_uring_register of buffers with a null argument succeeds or is rejected as unsupported")]
fn io_uring_register_probe_soft() -> TestResult {
    let mut params = IoUringParams::default();
    let fd = match syscall::io_uring_setup(1, &mut params) {
        Ok(fd) => fd,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("setup")),
    };
    // Opcode 0 is IORING_REGISTER_BUFFERS with null — expect soft failure.
    match syscall::io_uring_register(fd, 0, 0, 0) {
        Ok(_) => {}
        Err(e) if soft(e) || e == Errno::EBADF || e == Errno::EFAULT => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("io_uring_register"));
        }
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
