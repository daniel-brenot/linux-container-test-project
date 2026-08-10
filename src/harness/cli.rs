//! Command-line argument parsing without `alloc`.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Quick,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    Bootstrap,
    Syscall,
    Posix,
    Fs,
}

#[derive(Clone, Copy)]
pub struct Args {
    pub mode: Mode,
    pub help: bool,
    pub list: bool,
    /// When false, omit ANSI colors from pass/fail/skip tags.
    pub color: bool,
    /// If none of the suite flags were set, run all suites.
    pub all_suites: bool,
    pub bootstrap: bool,
    pub syscall: bool,
    pub posix: bool,
    pub fs: bool,
}

impl Args {
    pub fn wants(&self, suite: Suite) -> bool {
        if self.all_suites {
            return true;
        }
        match suite {
            Suite::Bootstrap => self.bootstrap,
            Suite::Syscall => self.syscall,
            Suite::Posix => self.posix,
            Suite::Fs => self.fs,
        }
    }
}

fn arg_eq(arg: &[u8], lit: &str) -> bool {
    arg == lit.as_bytes()
}

/// Parse argv pointers from the ELF stack.
///
/// # Safety
/// `argv` must point at `argc` NUL-terminated C string pointers.
pub unsafe fn parse_args(argc: usize, argv: *const usize) -> Args {
    let mut args = Args {
        mode: Mode::Quick,
        help: false,
        list: false,
        color: true,
        all_suites: true,
        bootstrap: false,
        syscall: false,
        posix: false,
        fs: false,
    };

    let mut suite_selected = false;

    // Skip argv[0].
    for i in 1..argc {
        let ptr = *argv.add(i) as *const u8;
        if ptr.is_null() {
            break;
        }
        let arg = cstr_bytes(ptr);
        if arg_eq(arg, "-h") || arg_eq(arg, "--help") {
            args.help = true;
        } else if arg_eq(arg, "-q") || arg_eq(arg, "--quick") {
            args.mode = Mode::Quick;
        } else if arg_eq(arg, "-f") || arg_eq(arg, "--full") {
            args.mode = Mode::Full;
        } else if arg_eq(arg, "--list") {
            args.list = true;
        } else if arg_eq(arg, "--no-color") {
            args.color = false;
        } else if arg_eq(arg, "--bootstrap") {
            args.bootstrap = true;
            suite_selected = true;
        } else if arg_eq(arg, "--syscall") || arg_eq(arg, "--syscalls") {
            args.syscall = true;
            suite_selected = true;
        } else if arg_eq(arg, "--posix") {
            args.posix = true;
            suite_selected = true;
        } else if arg_eq(arg, "--fs") {
            args.fs = true;
            suite_selected = true;
        }
    }

    args.all_suites = !suite_selected;
    // Bootstrap always runs unless the user asked only for --list/--help.
    if suite_selected && !args.bootstrap {
        // Still run bootstrap as a gate when other suites are selected.
        args.bootstrap = true;
    }
    args
}

unsafe fn cstr_bytes<'a>(ptr: *const u8) -> &'a [u8] {
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    core::slice::from_raw_parts(ptr, len)
}

pub fn print_help(prog: &str) {
    crate::println!("{prog} — no_std Linux container verification suite");
    crate::println!();
    crate::println!("Usage: {prog} [OPTIONS]");
    crate::println!();
    crate::println!("Modes:");
    crate::println!("  -q, --quick       Quick smoke pass (default)");
    crate::println!("  -f, --full        Full suite pass");
    crate::println!();
    crate::println!("Suites (default: all). Bootstrap always runs first as a gate:");
    crate::println!("  --bootstrap       Prerequisite / self-hosting tests");
    crate::println!("  --syscall         Linux syscall behaviour tests");
    crate::println!("  --posix           POSIX semantics tests");
    crate::println!("  --fs              Filesystem semantics (pjdfstest-like)");
    crate::println!();
    crate::println!("Other:");
    crate::println!("  --list            List tests that would run, then exit");
    crate::println!("  --no-color        Disable ANSI colors on pass/fail/skip");
    crate::println!("  -h, --help        Show this help");
}
