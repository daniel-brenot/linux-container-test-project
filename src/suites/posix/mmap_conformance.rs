//! mmap conformance deepeners (MEM).

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::{TempDir, TestResult};
use crate::suites::common::{create_empty, write_file};
use crate::syscall::{self, map, oflag, prot, Errno, MS_ASYNC, MS_SYNC, MCL_CURRENT, MCL_FUTURE};

const PAGE: usize = 4096;

#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 0; check_eq!(*(addr as *const u8), 0, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 1; check_eq!(*(addr as *const u8), 1, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 2; check_eq!(*(addr as *const u8), 2, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 3; check_eq!(*(addr as *const u8), 3, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_5() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 4; check_eq!(*(addr as *const u8), 4, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_6() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 5; check_eq!(*(addr as *const u8), 5, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_7() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 6; check_eq!(*(addr as *const u8), 6, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_8() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; check_eq!(*(addr as *const u8), 7, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_9() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 8; check_eq!(*(addr as *const u8), 8, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_10() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 9; check_eq!(*(addr as *const u8), 9, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_11() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 10; check_eq!(*(addr as *const u8), 10, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_12() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 11; check_eq!(*(addr as *const u8), 11, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_13() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 12; check_eq!(*(addr as *const u8), 12, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_14() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 13; check_eq!(*(addr as *const u8), 13, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_15() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 14; check_eq!(*(addr as *const u8), 14, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_16() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 15; check_eq!(*(addr as *const u8), 15, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_17() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 16; check_eq!(*(addr as *const u8), 16, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_18() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 32; check_eq!(*(addr as *const u8), 32, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_19() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 64; check_eq!(*(addr as *const u8), 64, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_20() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 85; check_eq!(*(addr as *const u8), 85, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_21() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 127; check_eq!(*(addr as *const u8), 127, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_22() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 128; check_eq!(*(addr as *const u8), 128, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_23() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 170; check_eq!(*(addr as *const u8), 170, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_24() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 200; check_eq!(*(addr as *const u8), 200, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_25() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 220; check_eq!(*(addr as *const u8), 220, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_26() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 240; check_eq!(*(addr as *const u8), 240, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_27() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 254; check_eq!(*(addr as *const u8), 254, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous private page can be written and read back")]
fn mmapc_anon_28() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 255; check_eq!(*(addr as *const u8), 255, "byte"); }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_1() -> TestResult {
    let len = PAGE * 1;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..1 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_2() -> TestResult {
    let len = PAGE * 2;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..2 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_3() -> TestResult {
    let len = PAGE * 3;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..3 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_4() -> TestResult {
    let len = PAGE * 4;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..4 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_5() -> TestResult {
    let len = PAGE * 5;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..5 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_6() -> TestResult {
    let len = PAGE * 6;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..6 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_7() -> TestResult {
    let len = PAGE * 7;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..7 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_8() -> TestResult {
    let len = PAGE * 8;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..8 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "an anonymous mapping spanning several pages can be written at each page start")]
fn mmapc_pages_16() -> TestResult {
    let len = PAGE * 16;
    let addr = check_ok!(syscall::mmap(0, len, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe {
        for i in 0..16 {
            *((addr + i * PAGE) as *mut u8) = (i as u8).wrapping_add(1);
            check_eq!(*((addr + i * PAGE) as *const u8), (i as u8).wrapping_add(1), "p");
        }
    }
    check_ok!(syscall::munmap(addr, len), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_NONE then restore read-write")]
fn mmapc_mprotect_none_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_NONE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_NONE then restore read-write")]
fn mmapc_mprotect_none_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_NONE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_NONE then restore read-write")]
fn mmapc_mprotect_none_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_NONE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_NONE then restore read-write")]
fn mmapc_mprotect_none_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_NONE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ then restore read-write")]
fn mmapc_mprotect_r_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ then restore read-write")]
fn mmapc_mprotect_r_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ then restore read-write")]
fn mmapc_mprotect_r_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ then restore read-write")]
fn mmapc_mprotect_r_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_WRITE on an anonymous page")]
fn mmapc_mprotect_rw_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_WRITE on an anonymous page")]
fn mmapc_mprotect_rw_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_WRITE on an anonymous page")]
fn mmapc_mprotect_rw_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_WRITE on an anonymous page")]
fn mmapc_mprotect_rw_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_EXEC then restore read-write")]
fn mmapc_mprotect_rx_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_EXEC), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_EXEC then restore read-write")]
fn mmapc_mprotect_rx_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_EXEC), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_EXEC then restore read-write")]
fn mmapc_mprotect_rx_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_EXEC), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = success, case = "mprotect can set PROT_READ|PROT_EXEC then restore read-write")]
fn mmapc_mprotect_rx_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_EXEC), "prot");
    check_ok!(syscall::mprotect(addr, PAGE, prot::PROT_READ | prot::PROT_WRITE), "rw");
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_NORMAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_normal_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_NORMAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_NORMAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_normal_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_NORMAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_NORMAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_normal_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_NORMAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_NORMAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_normal_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_NORMAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_RANDOM succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_random_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_RANDOM) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_RANDOM succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_random_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_RANDOM) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_RANDOM succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_random_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_RANDOM) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_RANDOM succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_random_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_RANDOM) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_SEQUENTIAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_seq_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_SEQUENTIAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_SEQUENTIAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_seq_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_SEQUENTIAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_SEQUENTIAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_seq_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_SEQUENTIAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_SEQUENTIAL succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_seq_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_SEQUENTIAL) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_WILLNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_willneed_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_WILLNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_WILLNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_willneed_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_WILLNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_WILLNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_willneed_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_WILLNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_WILLNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_willneed_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_WILLNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_DONTNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_dontneed_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_DONTNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_DONTNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_dontneed_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_DONTNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_DONTNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_dontneed_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_DONTNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "madvise MADV_DONTNEED succeeds or is rejected on an anonymous page")]
fn mmapc_madvise_dontneed_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::madvise(addr, PAGE, syscall::madvise::MADV_DONTNEED) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("madvise")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_1() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 1 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_2() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 2 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_3() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 3 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_4() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 4 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_5() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 5 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_6() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 6 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_7() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_SYNC on a MAP_SHARED file mapping succeeds or is rejected")]
fn mmapc_msync_8() -> TestResult {
    let mut tmp = check_ok!(TempDir::create(), "tmp");
    let path = create_empty(&mut tmp, b"msync.bin")?;
    write_file(&path, &[0u8; 4096])?;
    let fd = check_ok!(syscall::open(&path, oflag::O_RDWR, 0), "open");
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_SHARED, fd, 0), "mmap");
    unsafe { *(addr as *mut u8) = 8 as u8; }
    match syscall::msync(addr, PAGE, MS_SYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); let _ = syscall::close(fd); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_ASYNC on an anonymous mapping succeeds or is rejected")]
fn mmapc_msync_async_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::msync(addr, PAGE, MS_ASYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_ASYNC on an anonymous mapping succeeds or is rejected")]
fn mmapc_msync_async_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::msync(addr, PAGE, MS_ASYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_ASYNC on an anonymous mapping succeeds or is rejected")]
fn mmapc_msync_async_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::msync(addr, PAGE, MS_ASYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "msync MS_ASYNC on an anonymous mapping succeeds or is rejected")]
fn mmapc_msync_async_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::msync(addr, PAGE, MS_ASYNC) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("msync")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_5() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_6() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_7() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mincore reports residency of a touched page when supported")]
fn mmapc_mincore_8() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    unsafe { *(addr as *mut u8) = 7; }
    let mut vec = [0u8; 1];
    match syscall::mincore(addr, PAGE, &mut vec) {
        Ok(()) => { check!(vec[0] & 1 == 1, "resident"); }
        Err(Errno::ENOSYS) | Err(Errno::EINVAL) | Err(Errno::EAGAIN) => {}
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mincore")); }
    }
    check_ok!(syscall::munmap(addr, PAGE), "unmap");
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_1() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_2() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_3() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_4() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_5() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_6() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_7() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlock of an anonymous page succeeds or is rejected as unsupported")]
fn mmapc_mlock_soft_8() -> TestResult {
    let addr = check_ok!(syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS, -1, 0), "mmap");
    match syscall::mlock(addr, PAGE) {
        Ok(()) => { let _ = syscall::munmap(addr, PAGE); }
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EAGAIN) | Err(Errno::EINVAL) => {
            check_ok!(syscall::munmap(addr, PAGE), "unmap");
        }
        Err(_) => { let _ = syscall::munmap(addr, PAGE); return Err(crate::harness::AssertFail::msg("mlock")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_CURRENT succeeds or is rejected as unsupported")]
fn mmapc_mlockall_cur_1() -> TestResult {
    match syscall::mlockall(MCL_CURRENT) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_CURRENT succeeds or is rejected as unsupported")]
fn mmapc_mlockall_cur_2() -> TestResult {
    match syscall::mlockall(MCL_CURRENT) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_CURRENT succeeds or is rejected as unsupported")]
fn mmapc_mlockall_cur_3() -> TestResult {
    match syscall::mlockall(MCL_CURRENT) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_CURRENT succeeds or is rejected as unsupported")]
fn mmapc_mlockall_cur_4() -> TestResult {
    match syscall::mlockall(MCL_CURRENT) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_CURRENT succeeds or is rejected as unsupported")]
fn mmapc_mlockall_cur_5() -> TestResult {
    match syscall::mlockall(MCL_CURRENT) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_CURRENT succeeds or is rejected as unsupported")]
fn mmapc_mlockall_cur_6() -> TestResult {
    match syscall::mlockall(MCL_CURRENT) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_FUTURE succeeds or is rejected as unsupported")]
fn mmapc_mlockall_fut_1() -> TestResult {
    match syscall::mlockall(MCL_FUTURE) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_FUTURE succeeds or is rejected as unsupported")]
fn mmapc_mlockall_fut_2() -> TestResult {
    match syscall::mlockall(MCL_FUTURE) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_FUTURE succeeds or is rejected as unsupported")]
fn mmapc_mlockall_fut_3() -> TestResult {
    match syscall::mlockall(MCL_FUTURE) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mlockall with MCL_FUTURE succeeds or is rejected as unsupported")]
fn mmapc_mlockall_fut_4() -> TestResult {
    match syscall::mlockall(MCL_FUTURE) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::ENOMEM) | Err(Errno::ENOSYS) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("mlockall")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mmap with MAP_POPULATE succeeds or is rejected")]
fn mmapc_populate_1() -> TestResult {
    match syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE, -1, 0) {
        Ok(addr) => { check_ok!(syscall::munmap(addr, PAGE), "unmap"); }
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("populate")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mmap with MAP_POPULATE succeeds or is rejected")]
fn mmapc_populate_2() -> TestResult {
    match syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE, -1, 0) {
        Ok(addr) => { check_ok!(syscall::munmap(addr, PAGE), "unmap"); }
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("populate")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mmap with MAP_POPULATE succeeds or is rejected")]
fn mmapc_populate_3() -> TestResult {
    match syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE, -1, 0) {
        Ok(addr) => { check_ok!(syscall::munmap(addr, PAGE), "unmap"); }
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("populate")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mmap with MAP_POPULATE succeeds or is rejected")]
fn mmapc_populate_4() -> TestResult {
    match syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE, -1, 0) {
        Ok(addr) => { check_ok!(syscall::munmap(addr, PAGE), "unmap"); }
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("populate")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mmap with MAP_POPULATE succeeds or is rejected")]
fn mmapc_populate_5() -> TestResult {
    match syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE, -1, 0) {
        Ok(addr) => { check_ok!(syscall::munmap(addr, PAGE), "unmap"); }
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("populate")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, expect = soft, case = "mmap with MAP_POPULATE succeeds or is rejected")]
fn mmapc_populate_6() -> TestResult {
    match syscall::mmap(0, PAGE, prot::PROT_READ | prot::PROT_WRITE, map::MAP_PRIVATE | map::MAP_ANONYMOUS | map::MAP_POPULATE, -1, 0) {
        Ok(addr) => { check_ok!(syscall::munmap(addr, PAGE), "unmap"); }
        Err(Errno::EINVAL) | Err(Errno::ENOMEM) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("populate")); }
    }
    Ok(())
}