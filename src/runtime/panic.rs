use crate::syscall;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let _ = syscall::write_all(syscall::STDERR_FILENO, b"panic: ");
    if let Some(loc) = info.location() {
        let _ = syscall::write_all(syscall::STDERR_FILENO, loc.file().as_bytes());
        let _ = syscall::write_all(syscall::STDERR_FILENO, b":");
        let mut buf = [0u8; 16];
        let n = crate::runtime::u64_to_dec(loc.line() as u64, &mut buf);
        let _ = syscall::write_all(syscall::STDERR_FILENO, &buf[..n]);
    }
    let _ = syscall::write_all(syscall::STDERR_FILENO, b"\n");
    syscall::exit(101);
}
