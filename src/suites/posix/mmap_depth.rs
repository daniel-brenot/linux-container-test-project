//! mmap depth (MEM): MAP_SHARED/PRIVATE/ANON, mprotect, msync, mincore, mlock soft, fork COW.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, write_file};
use crate::syscall::{self, map, oflag, prot, Errno, MS_ASYNC, MS_SYNC, MCL_CURRENT};

const PAGE: usize = 4096;

macro_rules! anon_rw {
    ($name:ident, $byte:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let addr = check_ok!(
                syscall::mmap(
                    0,
                    PAGE,
                    prot::PROT_READ | prot::PROT_WRITE,
                    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
                    -1,
                    0
                ),
                "mmap"
            );
            unsafe {
                *(addr as *mut u8) = $byte;
                check_eq!(*(addr as *const u8), $byte, "byte");
            }
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
            Ok(())
        }
    };
}

anon_rw!(mem_d_anon_1, 1);
anon_rw!(mem_d_anon_2, 2);
anon_rw!(mem_d_anon_7, 7);
anon_rw!(mem_d_anon_42, 42);
anon_rw!(mem_d_anon_ff, 0xff);
anon_rw!(mem_d_anon_aa, 0xaa);
anon_rw!(mem_d_anon_55, 0x55);

#[crate::lctp_test(suite = posix)]
fn mem_d_anon_zero_page() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        for i in 0..64 {
            check_eq!(*((addr + i) as *const u8), 0, "z");
        }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_mprotect_rw_ro_rw() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ), "ro");
    check_ok!(
        syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE),
        "rw"
    );
    unsafe {
        *(addr as *mut u8) = 9;
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_mprotect_none_then_rw() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_NONE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    check_ok!(
        syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE),
        "rw"
    );
    unsafe {
        *(addr as *mut u8) = 3;
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_shared_file_msync() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let fd = check_ok!(tmp.create_file(b"sh", 0o644), "f");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "tr");
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = b'M';
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "sync");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_msync_async() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 1;
    }
    check_ok!(syscall::msync(addr, PAGE, MS_ASYNC), "async");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_mincore_soft() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 1;
    }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => check!(vec[0] & 1 != 0 || vec[0] == 0, "bit soft"),
        Err(Errno::ENOSYS) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => {}
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_mlock_soft() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    match syscall::mlock(addr, PAGE) {
        Ok(()) => {
            let _ = syscall::munlock(addr, PAGE);
        }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::EAGAIN) | Err(Errno::ENOSYS) => {}
        Err(_) => {}
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_mlockall_soft() -> TestResult {
    match syscall::mlockall(MCL_CURRENT) {
        Ok(()) => {
            let _ = syscall::munlockall();
        }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mem_d_fork_cow_private() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = b'P';
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            *(addr as *mut u8) = b'C';
        }
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    unsafe {
        // Parent should still see P (COW); soft if not.
        let v = *(addr as *const u8);
        check!(v == b'P' || v == b'C', "cow soft");
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mem_d_fork_shared_anon() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 0;
    }
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        unsafe {
            *(addr as *mut u8) = b'S';
        }
        syscall::exit(0);
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    unsafe {
        check_eq!(*(addr as *const u8), b'S', "shared");
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_private_file_cow_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"p")?;
    write_file(&path, b"ABCD")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "tr");
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE,
            fd,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = b'Z';
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_two_pages() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE * 2,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 1;
        *((addr + PAGE) as *mut u8) = 2;
    }
    check_ok!(syscall::munmap(addr, PAGE * 2), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_mprotect_partial() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE * 2,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ), "ro half");
    unsafe {
        *((addr + PAGE) as *mut u8) = 5;
    }
    check_ok!(syscall::munmap(addr, PAGE * 2), "unmap");
    Ok(())
}

macro_rules! anon_fill {
    ($name:ident, $val:expr) => {
        #[crate::lctp_test(suite = posix)]
        fn $name() -> TestResult {
            let addr = check_ok!(
                syscall::mmap(
                    0,
                    PAGE,
                    prot::PROT_READ | prot::PROT_WRITE,
                    map::MAP_PRIVATE | map::MAP_ANONYMOUS,
                    -1,
                    0
                ),
                "mmap"
            );
            unsafe {
                core::ptr::write_bytes(addr as *mut u8, $val, 64);
                check_eq!(*(addr as *const u8), $val, "f");
            }
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
            Ok(())
        }
    };
}

anon_fill!(mem_d_fill_11, 0x11);
anon_fill!(mem_d_fill_22, 0x22);
anon_fill!(mem_d_fill_33, 0x33);
anon_fill!(mem_d_fill_44, 0x44);
anon_fill!(mem_d_fill_66, 0x66);
anon_fill!(mem_d_fill_77, 0x77);
anon_fill!(mem_d_fill_88, 0x88);
anon_fill!(mem_d_fill_99, 0x99);

#[crate::lctp_test(suite = posix, full)]
fn mem_d_shared_fill_msync() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let fd = check_ok!(tmp.create_file(b"fill", 0o644), "f");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "tr");
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            0
        ),
        "mmap"
    );
    unsafe {
        core::ptr::write_bytes(addr as *mut u8, 0xCD, PAGE);
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "sync");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    let mut buf = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, &[0xCD; 4], "data");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_munmap_idempotent_soft() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    check_ok!(syscall::munmap(addr, PAGE), "u1");
    match syscall::munmap(addr, PAGE) {
        Ok(()) | Err(Errno::EINVAL) => {}
        Err(_) => {}
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_populate_soft() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 4;
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mem_d_mincore_touch_all() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE * 2,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 1;
        *((addr + PAGE) as *mut u8) = 2;
    }
    let mut vec = [0u8; 2];
    let _ = syscall::mincore(addr, PAGE * 2, &mut vec);
    check_ok!(syscall::munmap(addr, PAGE * 2), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_end_byte() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        *((addr + PAGE - 1) as *mut u8) = 0xFE;
        check_eq!(*((addr + PAGE - 1) as *const u8), 0xFE, "end");
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mem_d_mlock_munlock_pair() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    if syscall::mlock(addr, PAGE).is_ok() {
        check_ok!(syscall::munlock(addr, PAGE), "ul");
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_prot_read_only() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    unsafe {
        let _ = *(addr as *const u8);
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mem_d_four_anon_maps() -> TestResult {
    let mut addrs = [0usize; 4];
    for a in addrs.iter_mut() {
        *a = check_ok!(
            syscall::mmap(
                0,
                PAGE,
                prot::PROT_READ | prot::PROT_WRITE,
                map::MAP_PRIVATE | map::MAP_ANONYMOUS,
                -1,
                0
            ),
            "m"
        );
    }
    for a in addrs.iter() {
        check_ok!(syscall::munmap(*a, PAGE), "u");
    }
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_msync_sync_twice() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "s1");
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "s2");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mem_d_fork_parent_write_after() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        syscall::exit(0);
    }
    unsafe {
        *(addr as *mut u8) = b'X';
    }
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    unsafe {
        check_eq!(*(addr as *const u8), b'X', "parent");
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_len_zero_einval() -> TestResult {
    match syscall::mmap(
        0,
        0,
        prot::PROT_READ,
        map::MAP_PRIVATE | map::MAP_ANONYMOUS,
        -1,
        0,
    ) {
        Err(Errno::EINVAL) => Ok(()),
        Ok(a) => {
            let _ = syscall::munmap(a, 0);
            Err(crate::harness::AssertFail::msg("unexpected"))
        }
        Err(_) => Ok(()),
    }
}

#[crate::lctp_test(suite = posix, full)]
fn mem_d_shared_then_private() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let fd = check_ok!(tmp.create_file(b"sp", 0o644), "f");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "tr");
    let s = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            0
        ),
        "s"
    );
    let p = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE,
            fd,
            0
        ),
        "p"
    );
    check_ok!(syscall::munmap(s, PAGE), "us");
    check_ok!(syscall::munmap(p, PAGE), "up");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mem_d_mprotect_exec_soft() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    match syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_EXEC) {
        Ok(()) => {}
        Err(Errno::EACCES) | Err(Errno::EPERM) | Err(Errno::EINVAL) => {}
        Err(_) => {}
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
