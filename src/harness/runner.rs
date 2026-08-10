//! Suite runner and distributed test registry.

use crate::harness::cli::{print_help, Args, Mode, Suite};
use crate::harness::{AssertFail, TestResult};
use crate::print;
use crate::println;
use crate::suites;
use crate::syscall::{self, clock, Timespec};
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
    // Bootstrap exercises clock/write primitives; skip per-test timing there.
    let time_tests = suite != Suite::Bootstrap;
    println!("==> {}", suite_name(suite));

    suites::for_each_in_suite(suite, |t| {
        if t.full_only && args.mode != Mode::Full {
            print_result(args.color, Status::Skip, t.name, Some("full-only"), None);
            local.skipped += 1;
            total.skipped += 1;
            return;
        }

        let start = if time_tests { monotonic_now() } else { None };
        let result = (t.func)();
        let elapsed_ns = if time_tests {
            elapsed_ns(start, monotonic_now())
        } else {
            None
        };

        match result {
            Ok(()) => {
                print_result(args.color, Status::Pass, t.name, None, elapsed_ns);
                local.passed += 1;
                total.passed += 1;
            }
            Err(AssertFail { message }) => {
                print_result(args.color, Status::Fail, t.name, Some(message), elapsed_ns);
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

#[derive(Clone, Copy)]
enum Status {
    Pass,
    Fail,
    Skip,
}

impl Status {
    fn tag(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::Skip => "SKIP",
        }
    }

    fn ansi(self) -> &'static str {
        match self {
            Status::Pass => "\x1b[32m",
            Status::Fail => "\x1b[31m",
            Status::Skip => "\x1b[33m",
        }
    }
}

fn print_result(
    color: bool,
    status: Status,
    name: &str,
    detail: Option<&str>,
    elapsed_ns: Option<u64>,
) {
    let tag = status.tag();
    if color {
        print!("[{}{tag}\x1b[0m] {name}", status.ansi());
    } else {
        print!("[{tag}] {name}");
    }
    if let Some(d) = detail {
        print!(" — {d}");
    }
    if let Some(ns) = elapsed_ns {
        print_duration(ns);
    }
    println!();
}

/// Print ` (Nunit)` using the smallest unit that stays a readable integer.
fn print_duration(ns: u64) {
    if ns < 1_000 {
        print!(" ({ns}ns)");
    } else if ns < 1_000_000 {
        print!(" ({}us)", ns / 1_000);
    } else if ns < 1_000_000_000 {
        print!(" ({}ms)", ns / 1_000_000);
    } else {
        let secs = ns / 1_000_000_000;
        let tenths = (ns % 1_000_000_000) / 100_000_000;
        print!(" ({secs}.{tenths}s)");
    }
}

fn monotonic_now() -> Option<Timespec> {
    syscall::clock_gettime(clock::CLOCK_MONOTONIC).ok()
}

fn elapsed_ns(start: Option<Timespec>, end: Option<Timespec>) -> Option<u64> {
    let (s, e) = (start?, end?);
    let start_ns = (s.tv_sec as i128) * 1_000_000_000 + (s.tv_nsec as i128);
    let end_ns = (e.tv_sec as i128) * 1_000_000_000 + (e.tv_nsec as i128);
    let delta = end_ns - start_ns;
    if delta < 0 {
        Some(0)
    } else {
        Some(delta as u64)
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
