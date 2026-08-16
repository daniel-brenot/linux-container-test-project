//! getcpu(2) tests.

use crate::check;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall;

#[crate::lctp_test(suite = syscall, expect = success, case = "getcpu writes a cpu index less than 4096")]
fn getcpu_succeeds() -> TestResult {
    let mut cpu = 0u32;
    check_ok!(syscall::getcpu(Some(&mut cpu), None), "getcpu");
    check!(cpu < 4096, "cpu < 4096");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getcpu with both cpu and node pointers writes values less than 4096")]
fn getcpu_with_node() -> TestResult {
    let mut cpu = 0u32;
    let mut node = 0u32;
    check_ok!(syscall::getcpu(Some(&mut cpu), Some(&mut node)), "getcpu");
    check!(cpu < 4096, "cpu");
    check!(node < 4096, "node");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getcpu with both arguments NULL succeeds")]
fn getcpu_null_args() -> TestResult {
    // Both NULL is allowed and succeeds.
    check_ok!(syscall::getcpu(None, None), "getcpu null");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "two successive getcpu calls each report a cpu index less than 4096")]
fn getcpu_stable_or_valid() -> TestResult {
    let mut a = 0u32;
    let mut b = 0u32;
    check_ok!(syscall::getcpu(Some(&mut a), None), "a");
    check_ok!(syscall::getcpu(Some(&mut b), None), "b");
    check!(a < 4096 && b < 4096, "both valid");
    Ok(())
}
