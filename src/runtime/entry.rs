//! Program entry from the ELF `_start` stub.

use super::ipc_echo;
use crate::harness;
use crate::syscall;

/// Called by architecture `_start` with a pointer to the initial stack
/// (`argc`, `argv…`, `NULL`, `envp…`).
#[no_mangle]
pub unsafe extern "C" fn rust_entry(stack: *const usize) -> ! {
    let argc = *stack;
    let argv = stack.add(1);
    if ipc_echo::dispatch_helper(argc, argv) {
        // helpers never return
    }
    let code = harness::run(argc, argv);
    syscall::exit(code);
}
