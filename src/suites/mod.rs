//! Test suite modules.
//!
//! Individual tests are registered with `#[lctp_test(suite = ...)]` into
//! [`crate::harness::ALL_TESTS`]. Submodules are declared so their
//! annotated tests are linked into the binary.

pub mod bootstrap;
pub mod common;
pub mod fs;
pub mod posix;
pub mod syscall;

use crate::harness::{Suite, TestCase, ALL_TESTS};

/// Visit every registered test case across all suites.
pub fn for_each_test(mut f: impl FnMut(&'static TestCase)) {
    for t in ALL_TESTS {
        f(t);
    }
}

pub fn for_each_in_suite(suite: Suite, mut f: impl FnMut(&'static TestCase)) {
    for t in ALL_TESTS {
        if t.suite == suite {
            f(t);
        }
    }
}
