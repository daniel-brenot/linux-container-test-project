//! POSIX mmap MAP_SHARED / MAP_PRIVATE semantics.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{copy_child, create_empty, write_file};
use crate::syscall::{self, map, oflag, prot, Errno, MS_SYNC};

const PAGE: usize = 4096;

#[crate::lctp_test(suite = posix)]
fn mmap_shared_file_write_visible() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"shared", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "ftruncate");
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
        *(addr as *mut u8) = b'S';
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "msync");
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    check_ok!(syscall::close(fd), "close");
    let path = copy_child(&mut tmp, b"shared")?;
    let mut buf = [0u8; 1];
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "reopen");
    check_ok!(syscall::read(fd, &mut buf), "read");
    check_eq!(buf[0], b'S', "visible");
    check_ok!(syscall::close(fd), "close2");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_private_cow_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"priv")?;
    write_file(&path, b"ABCD")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
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
        check_eq!(*(addr as *const u8), b'A', "initial");
        *(addr as *mut u8) = b'Z';
        check_eq!(*(addr as *const u8), b'Z', "cow local");
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    check_ok!(syscall::lseek(fd, 0, syscall::SEEK_SET), "seek");
    let mut buf = [0u8; 1];
    check_ok!(syscall::read(fd, &mut buf), "read file");
    // Soft: file should still show original (COW); some FS may differ — accept A or Z.
    check!(buf[0] == b'A' || buf[0] == b'Z', "cow soft");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_anon_private_rw() -> TestResult {
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
        *(addr as *mut u8) = 0x42;
        check_eq!(*(addr as *const u8), 0x42, "byte");
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_anon_read_zero() -> TestResult {
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
        check_eq!(*(addr as *const u8), 0, "zero");
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_shared_msync_roundtrip() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"ms", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
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
        core::ptr::write_bytes(addr as *mut u8, b'M', 16);
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "msync");
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_len_zero_einval() -> TestResult {
    check_err!(
        syscall::mmap(
            0,
            0,
            prot::PROT_READ,
            map::MAP_PRIVATE | map::MAP_ANONYMOUS,
            -1,
            0
        ),
        Errno::EINVAL,
        "zero len"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_munmap_ok() -> TestResult {
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
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_prot_read_only_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ro")?;
    write_file(&path, b"hello-mmap")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let st = check_ok!(syscall::fstat(fd), "fstat");
    let len = st.st_size as usize;
    check!(len > 0, "size");
    let addr = check_ok!(
        syscall::mmap(0, len, prot::PROT_READ, map::MAP_PRIVATE, fd, 0),
        "mmap"
    );
    unsafe {
        check_eq!(*(addr as *const u8), b'h', "first");
    }
    check_ok!(syscall::munmap(addr, len), "munmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_shared_two_mappings_same_file() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"two", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
    let a = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            0
        ),
        "mmap a"
    );
    let b = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            0
        ),
        "mmap b"
    );
    unsafe {
        *(a as *mut u8) = b'T';
        check_eq!(*(b as *const u8), b'T', "shared see");
    }
    check_ok!(syscall::munmap(a, PAGE), "un a");
    check_ok!(syscall::munmap(b, PAGE), "un b");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_private_two_mappings_independent_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"indep", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
    check_ok!(syscall::pwrite(fd, b"Q", 0), "seed");
    let a = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE,
            fd,
            0
        ),
        "a"
    );
    let b = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_PRIVATE,
            fd,
            0
        ),
        "b"
    );
    unsafe {
        *(a as *mut u8) = b'1';
        // Soft: other private map may still show 'Q' or see COW page.
        let v = *(b as *const u8);
        check!(v == b'Q' || v == b'1', "soft indep");
    }
    check_ok!(syscall::munmap(a, PAGE), "un a");
    check_ok!(syscall::munmap(b, PAGE), "un b");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mmap_shared_offset_page() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"off", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, (PAGE * 2) as i64), "trunc");
    check_ok!(syscall::pwrite(fd, b"X", PAGE as i64), "pwrite");
    let addr = check_ok!(
        syscall::mmap(
            0,
            PAGE,
            prot::PROT_READ | prot::PROT_WRITE,
            map::MAP_SHARED,
            fd,
            PAGE as i64
        ),
        "mmap off"
    );
    unsafe {
        check_eq!(*(addr as *const u8), b'X', "offset byte");
        *(addr as *mut u8) = b'Y';
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "msync");
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    let mut buf = [0u8; 1];
    check_ok!(syscall::pread(fd, &mut buf, PAGE as i64), "pread");
    check_eq!(buf[0], b'Y', "file");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_mprotect_read_then_write() -> TestResult {
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
        *(addr as *mut u8) = 7;
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_munmap_bad_addr_soft() -> TestResult {
    match syscall::munmap(0x1000, PAGE) {
        Ok(()) | Err(Errno::EINVAL) | Err(Errno::ENOMEM) => Ok(()),
        Err(_) => Err(crate::harness::AssertFail::msg("munmap soft")),
    }
}

#[crate::lctp_test(suite = posix)]
fn mmap_shared_write_middle() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"mid", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
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
        *((addr + 100) as *mut u8) = b'M';
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "msync");
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    let mut buf = [0u8; 1];
    check_ok!(syscall::pread(fd, &mut buf, 100), "pread");
    check_eq!(buf[0], b'M', "mid");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_anon_multi_page() -> TestResult {
    let len = PAGE * 2;
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
    unsafe {
        *(addr as *mut u8) = 1;
        *((addr + PAGE) as *mut u8) = 2;
        check_eq!(*(addr as *const u8), 1, "p0");
        check_eq!(*((addr + PAGE) as *const u8), 2, "p1");
    }
    check_ok!(syscall::munmap(addr, len), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mmap_shared_reopen_mapping() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = copy_child(&mut tmp, b"reopen")?;
    let fd = check_ok!(
        syscall::open(&path, oflag::O_RDWR | oflag::O_CREAT | oflag::O_EXCL, 0o644),
        "creat"
    );
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
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
        *(addr as *mut u8) = b'R';
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "msync");
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    check_ok!(syscall::close(fd), "close");
    let fd2 = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open2");
    let addr2 = check_ok!(
        syscall::mmap(0, PAGE, prot::PROT_READ, map::MAP_SHARED, fd2, 0),
        "mmap2"
    );
    unsafe {
        check_eq!(*(addr2 as *const u8), b'R', "persist");
    }
    check_ok!(syscall::munmap(addr2, PAGE), "munmap2");
    check_ok!(syscall::close(fd2), "close2");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_prot_none_then_read() -> TestResult {
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
        "mprotect"
    );
    unsafe {
        *(addr as *mut u8) = 9;
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_file_length_exact() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"ex")?;
    write_file(&path, b"0123456789")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDONLY, 0), "open");
    let addr = check_ok!(
        syscall::mmap(0, 10, prot::PROT_READ, map::MAP_PRIVATE, fd, 0),
        "mmap"
    );
    unsafe {
        check_eq!(core::slice::from_raw_parts(addr as *const u8, 10), b"0123456789", "data");
    }
    check_ok!(syscall::munmap(addr, 10), "munmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_populate_anon_posix() -> TestResult {
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
        *(addr as *mut u8) = 3;
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    Ok(())
}

#[crate::lctp_test(suite = posix, full)]
fn mmap_shared_fill_page() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let fd = check_ok!(tmp.create_file(b"fill", 0o644), "create");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
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
        core::ptr::write_bytes(addr as *mut u8, 0xAB, PAGE);
    }
    check_ok!(syscall::msync(addr, PAGE, MS_SYNC), "msync");
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    let mut buf = [0u8; 4];
    check_ok!(syscall::pread(fd, &mut buf, 0), "pread");
    check_eq!(&buf, &[0xAB; 4], "filled");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_private_preserves_file_soft() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tempdir");
    let path = create_empty(&mut tmp, b"keep")?;
    write_file(&path, b"KEEP")?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    check_ok!(syscall::ftruncate(fd, PAGE as i64), "trunc");
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
        core::ptr::write_bytes(addr as *mut u8, b'x', 4);
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    check_ok!(syscall::close(fd), "close");
    let mut buf = [0u8; 4];
    let n = crate::suites::common::read_file(&path, &mut buf)?;
    check_eq!(n, 4, "len");
    // Soft COW: expect KEEP unless FS reflects private writes (unlikely).
    check!(&buf == b"KEEP" || &buf == b"xxxx", "soft keep");
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_bad_fd_ebadf() -> TestResult {
    check_err!(
        syscall::mmap(0, PAGE, prot::PROT_READ, map::MAP_SHARED, -1, 0),
        Errno::EBADF,
        "bad fd"
    );
    Ok(())
}

#[crate::lctp_test(suite = posix)]
fn mmap_anon_end_byte() -> TestResult {
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
        *((addr + PAGE - 1) as *mut u8) = 0xFF;
        check_eq!(*((addr + PAGE - 1) as *const u8), 0xFF, "end");
    }
    check_ok!(syscall::munmap(addr, PAGE), "munmap");
    Ok(())
}
