//! landlock(2) probe tests (soft on older / restricted kernels).

use crate::check;
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

#[crate::lctp_test(suite = syscall)]
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

#[crate::lctp_test(suite = syscall)]
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

#[crate::lctp_test(suite = syscall, full)]
fn landlock_restrict_self_without_nnp_soft() -> TestResult {
    let attr = LandlockRulesetAttr {
        handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE,
    };
    let fd = match syscall::landlock_create_ruleset(
        Some(&attr),
        core::mem::size_of::<LandlockRulesetAttr>(),
        0,
    ) {
        Ok(fd) => fd,
        Err(e) if soft(e) => return Ok(()),
        Err(_) => return Err(crate::harness::AssertFail::msg("create")),
    };
    // Without PR_SET_NO_NEW_PRIVS this typically fails with EPERM — soft-accept.
    match syscall::landlock_restrict_self(fd, 0) {
        Ok(()) => {}
        Err(e) if soft(e) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("restrict_self"));
        }
    }
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
