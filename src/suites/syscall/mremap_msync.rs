//! mremap, msync, and mincore tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::syscall::{self, map, madvise, MS_ASYNC, MS_SYNC, MREMAP_MAYMOVE, prot};

const PAGE: usize = 4096;

#[crate::lctp_test(suite = syscall)]
fn mremap_grow_maymove() -> TestResult {
    let old_len = PAGE;
    let new_len = PAGE * 2;
    let addr = check_ok!(
        syscall::mmap(0, old_len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 0x42;
    }
    let new_addr = check_ok!(
        syscall::mremap(addr, old_len, new_len, MREMAP_MAYMOVE, 0),
        "mremap"
    );
    unsafe {
        check_eq!(*(new_addr as *mut u8), 0x42, "preserved byte");
        let p = (new_addr as *mut u8).add(old_len);
        *p = 0x99;
        check_eq!(*p, 0x99, "new page");
    }
    check_ok!(syscall::munmap(new_addr, new_len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn mremap_same_size() -> TestResult {
    let len = PAGE;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    let new_addr = check_ok!(syscall::mremap(addr, len, len, 0, 0), "mremap");
    check_eq!(new_addr, addr, "same addr");
    check_ok!(syscall::munmap(new_addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn msync_async_anon() -> TestResult {
    let len = PAGE;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 1;
    }
    check_ok!(syscall::msync(addr, len, MS_ASYNC), "msync async");
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn msync_sync_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"ms", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "truncate");
    let addr = check_ok!(
        syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = b'M';
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "msync sync");
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn mincore_anon_page() -> TestResult {
    let len = PAGE;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    unsafe {
        *(addr as *mut u8) = 0xAA;
    }
    let mut vec = [0u8; 1];
    check_ok!(syscall::mincore(addr, len, &mut vec), "mincore");
    // Page may or may not be resident yet; mincore should succeed.
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn mincore_two_pages() -> TestResult {
    let len = PAGE * 2;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    unsafe {
        core::ptr::write_bytes(addr as *mut u8, 0, len);
    }
    let mut vec = [0u8; 2];
    check_ok!(syscall::mincore(addr, len, &mut vec), "mincore");
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn mremap_shrink() -> TestResult {
    let old_len = PAGE * 2;
    let new_len = PAGE;
    let addr = check_ok!(
        syscall::mmap(0, old_len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    let new_addr = check_ok!(
        syscall::mremap(addr, old_len, new_len, MREMAP_MAYMOVE, 0),
        "mremap shrink"
    );
    unsafe {
        *(new_addr as *mut u8) = 7;
    }
    check_ok!(syscall::munmap(new_addr, new_len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn msync_partial_page() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"msp", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64 * 2), "truncate");
    let addr = check_ok!(
        syscall::mmap(0, PAGE * 2, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0),
        "mmap"
    );
    check_ok!(syscall::msync(addr, PAGE, MS_ASYNC), "msync one page");
    check_ok!(syscall::munmap(addr, PAGE * 2), "munmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn mremap_grow_preserves_data() -> TestResult {
    let old_len = PAGE;
    let new_len = PAGE * 4;
    let addr = check_ok!(
        syscall::mmap(0, old_len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    unsafe {
        let s = core::slice::from_raw_parts_mut(addr as *mut u8, old_len);
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i & 0xFF) as u8;
        }
    }
    let new_addr = check_ok!(
        syscall::mremap(addr, old_len, new_len, MREMAP_MAYMOVE, 0),
        "mremap"
    );
    unsafe {
        let s = core::slice::from_raw_parts(addr as *const u8, old_len);
        let t = core::slice::from_raw_parts(new_addr as *const u8, old_len);
        check!(s == t, "data preserved");
    }
    check_ok!(syscall::munmap(new_addr, new_len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn mincore_after_madvise() -> TestResult {
    let len = PAGE;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    check_ok!(syscall::madvise(addr, len, madvise::MADV_WILLNEED), "madvise");
    let mut vec = [0u8; 1];
    check_ok!(syscall::mincore(addr, len, &mut vec), "mincore");
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn clock_getres_monotonic() -> TestResult {
    let res = check_ok!(syscall::clock_getres(crate::syscall::clock::CLOCK_MONOTONIC), "getres");
    check!(res.tv_sec >= 0, "sec");
    check!(res.tv_nsec > 0, "nsec");
    Ok(())
}
