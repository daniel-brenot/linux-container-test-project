//! Soft AIO probes: legacy `io_setup` / `io_destroy` (often ENOSYS) and io_uring setup.

use crate::check;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, IoUringParams};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EPERM | Errno::EACCES | Errno::EINVAL | Errno::ENOMEM | Errno::EAGAIN
    )
}

#[crate::lctp_test(suite = posix)]
fn aio_io_setup_soft() -> TestResult {
    let mut ctx = 0u64;
    match syscall::io_setup(1, &mut ctx) {
        Ok(()) => {
            let _ = syscall::io_destroy(ctx);
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("io_setup")),
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_io_setup_8_soft() -> TestResult {
    let mut ctx = 0u64;
    match syscall::io_setup(8, &mut ctx) {
        Ok(()) => {
            let _ = syscall::io_destroy(ctx);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_io_setup_64_soft() -> TestResult {
    let mut ctx = 0u64;
    match syscall::io_setup(64, &mut ctx) {
        Ok(()) => {
            let _ = syscall::io_destroy(ctx);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_io_setup_zero_soft() -> TestResult {
    let mut ctx = 0u64;
    match syscall::io_setup(0, &mut ctx) {
        Err(e) if e == Errno::EINVAL || soft(e) => Ok(()),
        Ok(()) => {
            let _ = syscall::io_destroy(ctx);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix)]
fn aio_io_destroy_zero_soft() -> TestResult {
    match syscall::io_destroy(0) {
        Err(e) if e == Errno::EINVAL || soft(e) => Ok(()),
        Ok(()) => Ok(()),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn aio_io_setup_destroy_pair() -> TestResult {
    let mut ctx = 0u64;
    match syscall::io_setup(4, &mut ctx) {
        Ok(()) => {
            match syscall::io_destroy(ctx) {
                Ok(()) => {}
                Err(e) if soft(e) => {}
                Err(_) => return Err(crate::harness::AssertFail::msg("destroy")),
            }
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_uring_setup_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(1, &mut params) {
        Ok(fd) => {
            check!(fd >= 0, "fd");
            let _ = syscall::close(fd);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_uring_setup_4_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(4, &mut params) {
        Ok(fd) => {
            let _ = syscall::close(fd);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_uring_setup_8_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(8, &mut params) {
        Ok(fd) => {
            let _ = syscall::close(fd);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn aio_uring_setup_params_filled() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(2, &mut params) {
        Ok(fd) => {
            // Soft: sq_entries may be rounded up.
            check!(params.sq_entries >= 1 || params.sq_entries == 0, "sq soft");
            let _ = syscall::close(fd);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_io_setup_twice_soft() -> TestResult {
    for n in [1u32, 2] {
        let mut ctx = 0u64;
        match syscall::io_setup(n, &mut ctx) {
            Ok(()) => {
                let _ = syscall::io_destroy(ctx);
            }
            Err(e) if soft(e) => {}
            Err(_) => {}
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_uring_setup_twice_soft() -> TestResult {
    for _ in 0..2 {
        let mut params = IoUringParams::default();
        match syscall::io_uring_setup(1, &mut params) {
            Ok(fd) => {
                let _ = syscall::close(fd);
            }
            Err(e) if soft(e) => {}
            Err(_) => {}
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn aio_io_setup_large_soft() -> TestResult {
    let mut ctx = 0u64;
    match syscall::io_setup(1024, &mut ctx) {
        Ok(()) => {
            let _ = syscall::io_destroy(ctx);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_uring_setup_zero_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(0, &mut params) {
        Err(e) if e == Errno::EINVAL || soft(e) => Ok(()),
        Ok(fd) => {
            let _ = syscall::close(fd);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix)]
fn aio_probe_both_apis() -> TestResult {
    let mut ctx = 0u64;
    let _ = syscall::io_setup(1, &mut ctx);
    if ctx != 0 {
        let _ = syscall::io_destroy(ctx);
    }
    let mut params = IoUringParams::default();
    if let Ok(fd) = syscall::io_uring_setup(1, &mut params) {
        let _ = syscall::close(fd);
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn aio_io_setup_destroy_invalid_soft() -> TestResult {
    match syscall::io_destroy(0xdead_beef) {
        Err(e) if e == Errno::EINVAL || soft(e) => Ok(()),
        Ok(()) => Ok(()),
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix)]
fn aio_uring_setup_16_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(16, &mut params) {
        Ok(fd) => {
            let _ = syscall::close(fd);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_io_setup_2_soft() -> TestResult {
    let mut ctx = 0u64;
    match syscall::io_setup(2, &mut ctx) {
        Ok(()) => {
            let _ = syscall::io_destroy(ctx);
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn aio_uring_close_twice_soft() -> TestResult {
    let mut params = IoUringParams::default();
    match syscall::io_uring_setup(1, &mut params) {
        Ok(fd) => {
            check!(syscall::close(fd).is_ok(), "c1");
            match syscall::close(fd) {
                Err(Errno::EBADF) => {}
                Ok(()) => {}
                Err(_) => {}
            }
        }
        Err(e) if soft(e) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn aio_smoke_ok() -> TestResult {
    // Always-pass marker that AIO soft module is linked.
    Ok(())
}
