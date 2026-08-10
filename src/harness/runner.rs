//! Suite runner and distributed test registry.

use crate::harness::cli::{print_help, Args, Mode, Suite};
use crate::harness::{AssertFail, TestResult};
use crate::suites;
use crate::println;
use linkme::distributed_slice;

/// All tests registered via `#[lctp_test(...)]`.
#[distributed_slice]
pub static ALL_TESTS: [TestCase] = [..];

#[derive(Clone, Copy)]
pub struct TestCase {
    pub name: &'static str,
    pub suite: Suite,
    /// When true, only executed in `--full` mode.
    pub full_only: bool,
    pub func: fn() -> TestResult,
}

#[derive(Default)]
struct Counters {
    passed: usize,
    failed: usize,
    skipped: usize,
}

pub unsafe fn run(argc: usize, argv: *const usize) -> i32 {
    let args = crate::harness::parse_args(argc, argv);
    let prog = prog_name(argc, argv);

    if args.help {
        print_help(prog);
        return 0;
    }

    if args.list {
        list_tests(&args);
        return 0;
    }

    println!("linux-container-test (no_std)");
    println!(
        "mode={} suites={}",
        mode_name(args.mode),
        suites_label(&args)
    );
    println!();

    let mut total = Counters::default();
    let boot = run_suite(&args, Suite::Bootstrap, &mut total);
    if boot.failed > 0 {
        println!();
        println!(
            "bootstrap failed ({}/{} tests) — refusing to run remaining suites",
            boot.failed,
            boot.passed + boot.failed + boot.skipped
        );
        print_summary(&total);
        return 1;
    }

    if args.wants(Suite::Syscall) {
        let _ = run_suite(&args, Suite::Syscall, &mut total);
    }
    if args.wants(Suite::Posix) {
        let _ = run_suite(&args, Suite::Posix, &mut total);
    }
    if args.wants(Suite::Fs) {
        let _ = run_suite(&args, Suite::Fs, &mut total);
    }

    println!();
    print_summary(&total);
    if total.failed > 0 {
        1
    } else {
        0
    }
}

fn run_suite(args: &Args, suite: Suite, total: &mut Counters) -> Counters {
    let mut local = Counters::default();
    println!("==> {}", suite_name(suite));

    suites::for_each_in_suite(suite, |t| {
        if t.full_only && args.mode != Mode::Full {
            print_result("SKIP", t.name, Some("full-only"));
            local.skipped += 1;
            total.skipped += 1;
            return;
        }

        match (t.func)() {
            Ok(()) => {
                print_result("PASS", t.name, None);
                local.passed += 1;
                total.passed += 1;
            }
            Err(AssertFail { message }) => {
                print_result("FAIL", t.name, Some(message));
                local.failed += 1;
                total.failed += 1;
            }
        }
    });

    println!(
        "    {}: {} passed, {} failed, {} skipped",
        suite_name(suite),
        local.passed,
        local.failed,
        local.skipped
    );
    println!();
    local
}

fn list_tests(args: &Args) {
    suites::for_each_test(|t| {
        if t.suite != Suite::Bootstrap && !args.wants(t.suite) {
            return;
        }
        let mode = if t.full_only { "full" } else { "quick" };
        println!("{}\t{}\t{}", suite_name(t.suite), mode, t.name);
    });
}

fn print_result(tag: &str, name: &str, detail: Option<&str>) {
    match detail {
        Some(d) => println!("[{tag}] {name} — {d}"),
        None => println!("[{tag}] {name}"),
    }
}

fn print_summary(c: &Counters) {
    println!(
        "summary: {} passed, {} failed, {} skipped",
        c.passed, c.failed, c.skipped
    );
}

fn suite_name(s: Suite) -> &'static str {
    match s {
        Suite::Bootstrap => "bootstrap",
        Suite::Syscall => "syscall",
        Suite::Posix => "posix",
        Suite::Fs => "fs",
    }
}

fn mode_name(m: Mode) -> &'static str {
    match m {
        Mode::Quick => "quick",
        Mode::Full => "full",
    }
}

fn suites_label(args: &Args) -> &'static str {
    if args.all_suites {
        "all"
    } else {
        "selected"
    }
}

unsafe fn prog_name<'a>(argc: usize, argv: *const usize) -> &'a str {
    if argc == 0 {
        return "linux-container-test";
    }
    let ptr = *argv as *const u8;
    if ptr.is_null() {
        return "linux-container-test";
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let bytes = core::slice::from_raw_parts(ptr, len);
    let start = bytes
        .iter()
        .rposition(|&b| b == b'/')
        .map(|i| i + 1)
        .unwrap_or(0);
    core::str::from_utf8(&bytes[start..]).unwrap_or("linux-container-test")
}
