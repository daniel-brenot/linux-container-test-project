//! Test harness: registration, assertions, CLI, and runner.

mod assert;
mod cli;
mod runner;
mod temp;

pub use assert::{AssertFail, TestResult};
pub use cli::{parse_args, Suite};
pub use runner::{run, TestCase, ALL_TESTS};
pub use temp::TempDir;
