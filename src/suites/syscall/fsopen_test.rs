//! fsopen / fsconfig probes (usually privileged; soft ENOSYS/EPERM).

use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, FSCONFIG_CMD_CREATE};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS
            | Errno::EPERM
            | Errno::EACCES
            | Errno::EINVAL
            | Errno::ENOENT
            | Errno::ENODEV
            | Errno::EOPNOTSUPP
            | Errno::EBUSY
    )
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "fsopen of proc succeeds, or is rejected as unsupported or unprivileged")]
fn fsopen_proc_soft() -> TestResult {
    match syscall::fsopen(b"proc\0", 0) {
        Ok(fd) => {
            let _ = syscall::fsconfig(fd, FSCONFIG_CMD_CREATE, 0, 0, 0);
            check_ok!(syscall::close(fd), "close");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("fsopen")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "fsopen of tmpfs succeeds, or is rejected as unsupported or unprivileged")]
fn fsopen_tmpfs_soft() -> TestResult {
    match syscall::fsopen(b"tmpfs\0", 0) {
        Ok(fd) => {
            check_ok!(syscall::close(fd), "close");
        }
        Err(e) if soft(e) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("fsopen tmpfs")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "fsconfig on fd -1 fails, or the syscall is rejected as unsupported")]
fn fsconfig_bad_fd_soft() -> TestResult {
    match syscall::fsconfig(-1, FSCONFIG_CMD_CREATE, 0, 0, 0) {
        Err(Errno::EBADF) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Ok(()) => return Err(crate::harness::AssertFail::msg("fsconfig ok on -1")),
        Err(_) => {}
    }
    Ok(())
}
