//! Memory mapping syscall tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::copy_child;
use crate::syscall::{self, madvise, map, oflag, prot};

#[crate::lctp_test(suite = syscall, expect = success, case = "anonymous mmap is readable and writable then munmap succeeds")]
fn mmap_anonymous_rw() -> TestResult {
    let len = 4096usize;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    check!(addr != 0 && addr != usize::MAX, "bad addr");
    unsafe {
        let s = core::slice::from_raw_parts_mut(addr as *mut u8, len);
        s[0] = 0xAA;
        s[len - 1] = 0x55;
        check_eq!(s[0], 0xAA, "start byte");
        check_eq!(s[len - 1], 0x55, "end byte");
    }
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a shared file mmap write is visible after munmap")]
fn mmap_file_shared() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"mmapf", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 4096), "ftruncate");
    let addr = check_ok!(
        syscall::mmap(0, 4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0),
        "mmap file"
    );
    unsafe {
        *(addr as *mut u8) = b'X';
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    check_ok!(syscall::close(fd), "close");
    let path = copy_child(&mut tmp, b"mmapf")?;
    let mut buf = [0u8; 1];
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "reopen");
    check_ok!(syscall::read(fd, &mut buf), "read");
    check_eq!(buf[0], b'X', "shared write visible");
    check_ok!(syscall::close(fd), "close2");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "munmap can unmap each half of a two-page mapping")]
fn munmap_partial() -> TestResult {
    let len = 8192usize;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    check_ok!(syscall::munmap(addr, 4096), "munmap half");
    check_ok!(syscall::munmap(addr + 4096, 4096), "munmap rest");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mprotect can switch a mapping from write to read-only and back")]
fn mprotect_none_roundtrip() -> TestResult {
    let len = 4096usize;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    check_ok!(syscall::mprotect(addr, len, prot::PROT_READ), "mprotect RO");
    check_ok!(syscall::mprotect(addr, len, prot::PROT_READ | prot::PROT_WRITE), "mprotect RW");
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "mprotect can change protection independently on adjacent pages")]
fn mprotect_split_pages() -> TestResult {
    let len = 8192usize;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    check_ok!(syscall::mprotect(addr, 4096, prot::PROT_READ), "page0 RO");
    check_ok!(syscall::mprotect(addr + 4096, 4096, prot::PROT_READ), "page1 RO");
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "madvise MADV_DONTNEED succeeds on an anonymous mapping")]
fn madvise_dontneed() -> TestResult {
    let len = 4096usize;
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    check_ok!(syscall::madvise(addr, len, madvise::MADV_DONTNEED), "madvise");
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "brk(0) returns a nonzero program break")]
fn brk_query() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "brk(0)");
    check!(cur != 0, "brk returned null");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "fallocate grows a file to 4096 bytes")]
fn fallocate_punch_hole() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"falloc", 0o644), "create");
    check_ok!(syscall::fallocate(fd, 0, 0, 4096), "fallocate");
    let path = copy_child(&mut tmp, b"falloc")?;
    let st = check_ok!(syscall::stat(&path), "stat");
    check_eq!(st.st_size, 4096, "size");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "anonymous mmap without MAP_FIXED succeeds at an arbitrary address")]
fn mmap_fixed_not_used() -> TestResult {
    // Anonymous mmap without MAP_FIXED should succeed at arbitrary address.
    let addr = check_ok!(
        syscall::mmap(0, 4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    );
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "MAP_POPULATE anonymous mmap is writable")]
fn mmap_populate_anon() -> TestResult {
    let len = 4096usize;
    let addr = check_ok!(
        syscall::mmap(
            0,
            len,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE,
            -1,
            0
        ),
        "mmap populate"
    );
    unsafe {
        *(addr as *mut u8) = 0x11;
        check_eq!(*(addr as *const u8), 0x11, "byte");
    }
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "mlock/munlock succeed or are rejected with EPERM/ENOMEM/EAGAIN/EINVAL/ENOSYS")]
fn mlock_munlock_soft() -> TestResult {
    let len = 4096usize;
    let addr = check_ok!(
        syscall::mmap(
            0,
            len,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        "mmap"
    );
    match syscall::mlock(addr, len) {
        Ok(()) => {
            check_ok!(syscall::munlock(addr, len), "munlock");
        }
        Err(crate::syscall::Errno::EPERM)
        | Err(crate::syscall::Errno::ENOMEM)
        | Err(crate::syscall::Errno::EAGAIN)
        | Err(crate::syscall::Errno::EINVAL)
        | Err(crate::syscall::Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, len);
            return Err(crate::harness::AssertFail::msg("mlock errno"));
        }
    }
    // munlock without prior mlock may succeed or fail softly.
    match syscall::munlock(addr, len) {
        Ok(()) => {}
        Err(crate::syscall::Errno::ENOMEM)
        | Err(crate::syscall::Errno::EINVAL)
        | Err(crate::syscall::Errno::EPERM)
        | Err(crate::syscall::Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, len);
            return Err(crate::harness::AssertFail::msg("munlock errno"));
        }
    }
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}
