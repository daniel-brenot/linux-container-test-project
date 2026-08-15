//! Freestanding Linux container verification suite.
//!
//! Speaks the Linux syscall ABI directly (`#![no_std]` + `#![no_main]`), so a
//! single static binary covers kernel behaviour without depending on glibc vs
//! musl libc differences.

#![no_std]
#![no_main]

pub use lctp_macros::lctp_test;

mod harness;
mod runtime;
mod suites;
mod syscall;

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".text",
    ".global _start",
    ".type _start, @function",
    "_start:",
    "    mov rdi, rsp",
    "    call {entry}",
    "    ud2",
    entry = sym runtime::rust_entry,
);

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".text",
    ".global _start",
    ".type _start, @function",
    "_start:",
    "    mov x0, sp",
    "    bl {entry}",
    "    brk #0",
    entry = sym runtime::rust_entry,
);
