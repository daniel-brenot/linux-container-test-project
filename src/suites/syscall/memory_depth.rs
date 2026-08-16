//! mmap/mprotect/madvise/mlock/mremap/mincore/brk depth.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::syscall::{self, madvise, map, prot, Errno, MREMAP_MAYMOVE};

fn soft_mlock(e: Errno) -> bool {
    matches!(
        e,
        Errno::EPERM | Errno::ENOMEM | Errno::EAGAIN | Errno::EINVAL | Errno::ENOSYS
    )
}

fn map_anon(len: usize, p: i32, flags: i32) -> Result<usize, crate::harness::AssertFail> {
    Ok(check_ok!(
        syscall::mmap(0, len, p, flags | map::MAP_ANONYMOUS, -1, 0),
        "mmap"
    ))
}

#[crate::lctp_test(suite = syscall, expect = success, case = "private anonymous mmap is writable then munmap succeeds")]
fn mmap_private_anon() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    unsafe { *(addr as *mut u8) = 1 };
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "MAP_SHARED anonymous mmap succeeds or is rejected with EINVAL/ENOSYS")]
fn mmap_shared_anon_soft() -> TestResult {
    // MAP_SHARED|ANON is supported on Linux.
    match syscall::mmap(
        0,
        4096,
        prot::PROT_READ | prot::PROT_WRITE,
        map::MAP_SHARED | map::MAP_ANONYMOUS,
        -1,
        0,
    ) {
        Ok(addr) => check_ok!(syscall::munmap(addr, 4096), "munmap"),
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("shared anon")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "anonymous mmap with PROT_NONE succeeds")]
fn mmap_prot_none() -> TestResult {
    let addr = map_anon(4096, prot::PROT_NONE, map::MAP_PRIVATE)?;
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "anonymous mmap with PROT_READ succeeds")]
fn mmap_prot_read_only() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ, map::MAP_PRIVATE)?;
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "MAP_FIXED onto an existing mapping succeeds or returns EINVAL/ENOMEM")]
fn mmap_fixed_soft() -> TestResult {
    let base = map_anon(8192, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    let target = base + 4096;
    match syscall::mmap(
        target,
        4096,
        prot::PROT_READ | prot::PROT_WRITE,
        map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_FIXED,
        -1,
        0,
    ) {
        Ok(addr) => {
            check_eq!(addr, target, "fixed");
            check_ok!(syscall::munmap(base, 8192), "munmap");
        }
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {
            check_ok!(syscall::munmap(base, 8192), "munmap");
        }
        Err(_) => {
            let _ = syscall::munmap(base, 8192);
            return Err(crate::harness::AssertFail::msg("MAP_FIXED"));
        }
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a private file mmap sees the file's first byte")]
fn mmap_file_private() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"mp", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 4096), "trunc");
    check_ok!(syscall::pwrite(fd, b"Z", 0), "pw");
    let addr = check_ok!(
        syscall::mmap(0, 4096, prot::PROT_READ, map::MAP_PRIVATE, fd, 0),
        "mmap"
    );
    unsafe {
        check_eq!(*(addr as *const u8), b'Z', "byte");
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mprotect can set PROT_NONE then restore PROT_READ")]
fn mprotect_none() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    check_ok!(syscall::mprotect(addr, 4096, prot::PROT_NONE), "none");
    check_ok!(syscall::mprotect(addr, 4096, prot::PROT_READ), "read");
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mprotect can add PROT_WRITE to a read-only mapping")]
fn mprotect_write() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ, map::MAP_PRIVATE)?;
    check_ok!(
        syscall::mprotect(addr, 4096, prot::PROT_READ | prot::PROT_WRITE),
        "rw"
    );
    unsafe { *(addr as *mut u8) = 7 };
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "mprotect PROT_EXEC succeeds or is rejected with EPERM/EINVAL/EACCES")]
fn mprotect_exec_soft() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    match syscall::mprotect(addr, 4096, prot::PROT_READ | prot::PROT_EXEC) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EINVAL) | Err(Errno::EACCES) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("mprotect exec"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "madvise MADV_NORMAL succeeds on an anonymous mapping")]
fn madvise_normal() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    check_ok!(syscall::madvise(addr, 4096, madvise::MADV_NORMAL), "normal");
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "madvise MADV_RANDOM succeeds on an anonymous mapping")]
fn madvise_random() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    check_ok!(syscall::madvise(addr, 4096, madvise::MADV_RANDOM), "random");
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "madvise MADV_SEQUENTIAL succeeds on an anonymous mapping")]
fn madvise_sequential() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    check_ok!(
        syscall::madvise(addr, 4096, madvise::MADV_SEQUENTIAL),
        "seq"
    );
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "madvise MADV_WILLNEED succeeds on an anonymous mapping")]
fn madvise_willneed() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    check_ok!(
        syscall::madvise(addr, 4096, madvise::MADV_WILLNEED),
        "willneed"
    );
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "madvise MADV_DONTNEED succeeds after a write to the mapping")]
fn madvise_dontneed_again() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    unsafe { *(addr as *mut u8) = 9 };
    check_ok!(
        syscall::madvise(addr, 4096, madvise::MADV_DONTNEED),
        "dontneed"
    );
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "madvise MADV_FREE succeeds or is rejected with EINVAL/ENOSYS")]
fn madvise_free_soft() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    match syscall::madvise(addr, 4096, madvise::MADV_FREE) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("MADV_FREE"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "madvise MADV_HUGEPAGE succeeds or is rejected with EINVAL/ENOSYS")]
fn madvise_hugepage_soft() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    match syscall::madvise(addr, 4096, madvise::MADV_HUGEPAGE) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("HUGEPAGE"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "madvise MADV_NOHUGEPAGE succeeds or is rejected with EINVAL/ENOSYS")]
fn madvise_nohugepage_soft() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    match syscall::madvise(addr, 4096, madvise::MADV_NOHUGEPAGE) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("NOHUGEPAGE"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "mlock of one page succeeds or is rejected with EPERM/ENOMEM/EAGAIN/EINVAL/ENOSYS")]
fn mlock_soft_eperm() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    match syscall::mlock(addr, 4096) {
        Ok(()) => {
            check_ok!(syscall::munlock(addr, 4096), "munlock");
        }
        Err(e) if soft_mlock(e) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("mlock"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "mlock of two pages succeeds or is rejected with EPERM/ENOMEM/EAGAIN/EINVAL/ENOSYS")]
fn mlock_two_pages_soft() -> TestResult {
    let addr = map_anon(8192, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    match syscall::mlock(addr, 8192) {
        Ok(()) => {
            let _ = syscall::munlock(addr, 8192);
        }
        Err(e) if soft_mlock(e) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 8192);
            return Err(crate::harness::AssertFail::msg("mlock2"));
        }
    }
    check_ok!(syscall::munmap(addr, 8192), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mremap with MREMAP_MAYMOVE can grow a mapping")]
fn mremap_maymove_grow() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    let new = check_ok!(
        syscall::mremap(addr, 4096, 8192, MREMAP_MAYMOVE, 0),
        "mremap"
    );
    check!(new != 0, "addr");
    check_ok!(syscall::munmap(new, 8192), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mremap with MREMAP_MAYMOVE can shrink a mapping")]
fn mremap_maymove_shrink() -> TestResult {
    let addr = map_anon(8192, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    let new = check_ok!(
        syscall::mremap(addr, 8192, 4096, MREMAP_MAYMOVE, 0),
        "mremap"
    );
    check_ok!(syscall::munmap(new, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mremap with the same size succeeds")]
fn mremap_same_size() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    let new = check_ok!(
        syscall::mremap(addr, 4096, 4096, MREMAP_MAYMOVE, 0),
        "mremap"
    );
    check_ok!(syscall::munmap(new, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mincore reports a touched anonymous page as resident")]
fn mincore_anon_page() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    unsafe { *(addr as *mut u8) = 1 };
    let mut vec = [0u8; 1];
    check_ok!(syscall::mincore(addr, 4096, &mut vec), "mincore");
    check!(vec[0] & 1 != 0, "resident");
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mincore reports two touched pages as resident")]
fn mincore_two_pages() -> TestResult {
    let addr = map_anon(8192, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    unsafe {
        *(addr as *mut u8) = 1;
        *((addr + 4096) as *mut u8) = 2;
    }
    let mut vec = [0u8; 2];
    check_ok!(syscall::mincore(addr, 8192, &mut vec), "mincore");
    check!(vec[0] & 1 != 0, "p0");
    check!(vec[1] & 1 != 0, "p1");
    check_ok!(syscall::munmap(addr, 8192), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "brk(0) returns a nonzero program break")]
fn brk_query_nonzero() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "brk0");
    check!(cur != 0, "null");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "brk can grow by one page or is rejected with ENOMEM/EINVAL/EPERM")]
fn brk_grow_shrink_soft() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "cur");
    let grow = cur + 4096;
    match syscall::brk(grow) {
        Ok(n) => {
            check!(n >= grow || n == grow, "grew");
            let _ = syscall::brk(cur);
        }
        Err(Errno::ENOMEM) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("brk grow")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = soft, case = "brk can grow by two pages or is rejected with ENOMEM/EINVAL/EPERM")]
fn brk_grow_two_pages_soft() -> TestResult {
    let cur = check_ok!(syscall::brk(0), "cur");
    match syscall::brk(cur + 8192) {
        Ok(_) => {
            let _ = syscall::brk(cur);
        }
        Err(Errno::ENOMEM) | Err(Errno::EINVAL) | Err(Errno::EPERM) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("brk 8k")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "an 8192-byte anonymous mmap is writable at both ends")]
fn mmap_len_8192() -> TestResult {
    let addr = map_anon(8192, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    unsafe {
        *(addr as *mut u8) = 1;
        *((addr + 8191) as *mut u8) = 2;
    }
    check_ok!(syscall::munmap(addr, 8192), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "MAP_POPULATE anonymous mmap can be queried with mincore")]
fn mmap_populate_touch() -> TestResult {
    let addr = check_ok!(
        syscall::mmap(
            0,
            4096,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE,
            -1,
            0
        ),
        "mmap"
    );
    let mut vec = [0u8; 1];
    check_ok!(syscall::mincore(addr, 4096, &mut vec), "mincore");
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mprotect can set different protections on two adjacent pages")]
fn mprotect_split_two_pages() -> TestResult {
    let addr = map_anon(8192, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    check_ok!(syscall::mprotect(addr, 4096, prot::PROT_READ), "p0");
    check_ok!(
        syscall::mprotect(addr + 4096, 4096, prot::PROT_NONE),
        "p1"
    );
    check_ok!(syscall::munmap(addr, 8192), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "munmap of a low non-mapped address succeeds or returns EINVAL")]
fn munmap_bad_addr_soft() -> TestResult {
    match syscall::munmap(0x1000, 4096) {
        Ok(()) => {}
        Err(Errno::EINVAL) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("munmap bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "madvise with an invalid advice value returns EINVAL or succeeds")]
fn madvise_bad_advice_einval() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    match syscall::madvise(addr, 4096, 9999) {
        Err(Errno::EINVAL) => {}
        Ok(()) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("bad advice"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "mincore succeeds on an untouched anonymous page")]
fn mincore_unfaulted_page() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    let mut vec = [0xffu8; 1];
    check_ok!(syscall::mincore(addr, 4096, &mut vec), "mincore");
    // May or may not be resident without touch; just ensure syscall works.
    let _ = vec[0];
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "mincore succeeds on a mapping grown with mremap")]
fn mremap_grow_then_mincore() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    unsafe { *(addr as *mut u8) = 3 };
    let new = check_ok!(
        syscall::mremap(addr, 4096, 16384, MREMAP_MAYMOVE, 0),
        "mremap"
    );
    let mut vec = [0u8; 4];
    check_ok!(syscall::mincore(new, 16384, &mut vec), "mincore");
    check_ok!(syscall::munmap(new, 16384), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "mmap with length 0 returns EINVAL")]
fn mmap_zero_len_einval() -> TestResult {
    match syscall::mmap(
        0,
        0,
        prot::PROT_READ,
        map::MAP_PRIVATE | map::MAP_ANONYMOUS,
        -1,
        0,
    ) {
        Err(Errno::EINVAL) => {}
        Ok(a) => {
            let _ = syscall::munmap(a, 4096);
            return Err(crate::harness::AssertFail::msg("mmap 0"));
        }
        Err(_) => return Err(crate::harness::AssertFail::msg("mmap 0 errno")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "mprotect of a low non-mapped address returns ENOMEM/EINVAL/EACCES or succeeds")]
fn mprotect_bad_addr_soft() -> TestResult {
    match syscall::mprotect(0x1000, 4096, prot::PROT_READ) {
        Err(Errno::ENOMEM) | Err(Errno::EINVAL) | Err(Errno::EACCES) => {}
        Ok(()) => {}
        Err(_) => return Err(crate::harness::AssertFail::msg("mprotect bad")),
    }
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a shared file mmap write is visible via pread after msync")]
fn mmap_shared_file_writeback() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"sw", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, 4096), "trunc");
    let addr = check_ok!(
        syscall::mmap(
            0,
            4096,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            0
        ),
        "mmap"
    );
    unsafe { *(addr as *mut u8) = b'W' };
    check_ok!(syscall::msync(addr, 4096, syscall::MS_SYNC), "msync");
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    let mut b = [0u8; 1];
    check_ok!(syscall::pread(fd, &mut b, 0), "pread");
    check_eq!(b[0], b'W', "wb");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "mlock of a sub-page length succeeds or is rejected with EPERM/ENOMEM/EAGAIN/EINVAL/ENOSYS")]
fn mlock_partial_page_soft() -> TestResult {
    let addr = map_anon(4096, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE)?;
    // Length need not be page-aligned on Linux for mlock in modern kernels; soft-accept.
    match syscall::mlock(addr, 100) {
        Ok(()) => {
            let _ = syscall::munlock(addr, 100);
        }
        Err(e) if soft_mlock(e) => {}
        Err(_) => {
            let _ = syscall::munmap(addr, 4096);
            return Err(crate::harness::AssertFail::msg("mlock partial"));
        }
    }
    check_ok!(syscall::munmap(addr, 4096), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "two brk(0) queries return the same program break")]
fn brk_idempotent_query() -> TestResult {
    let a = check_ok!(syscall::brk(0), "a");
    let b = check_ok!(syscall::brk(0), "b");
    check_eq!(a, b, "stable");
    Ok(())
}
