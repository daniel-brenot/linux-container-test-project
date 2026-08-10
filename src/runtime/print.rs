//! Stack-based formatting to stdout without `alloc`.

use crate::syscall;
use core::fmt::{self, Write};

struct FdWriter(i32);

impl Write for FdWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall::write_all(self.0, s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub fn print_str(s: &str) {
    let _ = syscall::write_all(syscall::STDOUT_FILENO, s.as_bytes());
}

pub fn print_fmt(args: fmt::Arguments<'_>) {
    let mut w = FdWriter(syscall::STDOUT_FILENO);
    let _ = w.write_fmt(args);
}

pub fn println_fmt(args: fmt::Arguments<'_>) {
    print_fmt(args);
    print_str("\n");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::runtime::print_fmt(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => { $crate::runtime::print_str("\n") };
    ($($arg:tt)*) => {
        $crate::runtime::println_fmt(core::format_args!($($arg)*))
    };
}
