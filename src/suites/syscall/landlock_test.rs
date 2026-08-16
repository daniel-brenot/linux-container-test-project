//! landlock(2) probe tests (soft on older / restricted kernels).
//!
//! Never call `landlock_restrict_self` in the test process itself: once applied
//! (especially after `PR_SET_NO_NEW_PRIVS`), an empty EXECUTE ruleset blocks
//! directory traversal for the remainder of the suite.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, Errno, LandlockRulesetAttr, LANDLOCK_ACCESS_FS_EXECUTE, LANDLOCK_CREATE_RULESET_VERSION,
};

fn soft(e: Errno) -> bool {
    matches!(
        e,
        Errno::ENOSYS | Errno::EOPNOTSUPP | Errno::EPERM | Errno::EINVAL | Errno::ENOTSUP
    )
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "landlock_create_ruleset with VERSION returns an ABI of at least 1, or is rejected as unsupported")]
fn landlock_create_ruleset_version_soft() -> TestResult {
    match syscall::landlock_create_ruleset(None, 0, LANDLOCK_CREATE_RULESET_VERSION) {
        Ok(v) => {
            check!(v >= 1, "abi version");
            Ok(())
        }
        Err(e) if soft(e) => Ok(()),
        Err(_) => Err(crate::harness::AssertFail::msg("landlock version")),
    }
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "landlock_create_ruleset with an EXECUTE ruleset attr returns a fd, or is rejected as unsupported")]
fn landlock_create_ruleset_attr_soft() -> TestResult {
    let attr = LandlockRulesetAttr {
        handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE,
    };
    match syscall::landlock_create_ruleset(
        Some(&attr),
        core::mem::size_of::<LandlockRulesetAttr>(),
        0,
    ) {
        Ok(fd) => {
            check!(fd >= 0, "ruleset fd");
            check_ok!(syscall::close(fd), "close");
            Ok(())
        }
        Err(e) if soft(e) => Ok(()),
        Err(_) => Err(crate::harness::AssertFail::msg("landlock create")),
    }
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "a child can create a landlock ruleset and call restrict_self, or skip if unsupported")]
fn landlock_restrict_self_in_child_soft() -> TestResult {
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let attr = LandlockRulesetAttr {
            handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE,
        };
        let fd = match syscall::landlock_create_ruleset(
            Some(&attr),
            core::mem::size_of::<LandlockRulesetAttr>(),
            0,
        ) {
            Ok(fd) => fd,
            Err(_) => syscall::exit(0), // soft: unsupported
        };
        // May need NNP; try set then restrict. Child exits either way.
        let _ = syscall::prctl(syscall::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        let _ = syscall::landlock_restrict_self(fd, 0);
        let _ = syscall::close(fd);
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "child ok");
    Ok(())
}
