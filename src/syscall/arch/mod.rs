//! Architecture-specific syscall invocation and numbers.

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::{clone_thread, nr, syscall};

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::{clone_thread, nr, syscall};

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("linux-container-test only supports x86_64 and aarch64");
