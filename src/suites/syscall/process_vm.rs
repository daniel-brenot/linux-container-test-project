//! process_vm_readv / process_vm_writev tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, IoVec};

#[crate::lctp_test(suite = syscall, expect = success, case = "process_vm_readv from the calling process copies a local buffer into another local buffer")]
fn process_vm_readv_self() -> TestResult {
    let src = b"vm-readv-payload";
    let mut dst = [0u8; 16];
    let remote = [IoVec {
        iov_base: src.as_ptr() as *mut u8,
        iov_len: src.len(),
    }];
    let mut local = [IoVec {
        iov_base: dst.as_mut_ptr(),
        iov_len: src.len(),
    }];
    let n = check_ok!(
        syscall::process_vm_readv(syscall::getpid(), &mut local, &remote, 0),
        "readv self"
    );
    check_eq!(n, src.len(), "len");
    check_eq!(&dst[..src.len()], src.as_slice(), "data");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "process_vm_writev into the calling process copies a local buffer into another local buffer")]
fn process_vm_writev_self() -> TestResult {
    let local_data = b"vm-writev-out";
    let mut remote_buf = [0u8; 16];
    let local = [IoVec {
        iov_base: local_data.as_ptr() as *mut u8,
        iov_len: local_data.len(),
    }];
    let remote = [IoVec {
        iov_base: remote_buf.as_mut_ptr(),
        iov_len: local_data.len(),
    }];
    let n = check_ok!(
        syscall::process_vm_writev(syscall::getpid(), &local, &remote, 0),
        "writev self"
    );
    check_eq!(n, local_data.len(), "len");
    check_eq!(&remote_buf[..local_data.len()], local_data.as_slice(), "data");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "process_vm_readv from a child copies the child's payload into the parent")]
fn process_vm_readv_parent_child() -> TestResult {
    let (addr_r, addr_w) = check_ok!(syscall::pipe2(0), "addr pipe");
    let (hold_r, hold_w) = check_ok!(syscall::pipe2(0), "hold pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(addr_r);
        let _ = syscall::close(hold_w);
        let payload = b"child-vm-bytes!";
        let meta = (payload.as_ptr() as u64).to_ne_bytes();
        let _ = syscall::write(addr_w, &meta);
        let _ = syscall::close(addr_w);
        // Park until parent closes hold_w / writes (SIGKILL also fine).
        let mut tok = [0u8; 1];
        let _ = syscall::read(hold_r, &mut tok);
        syscall::exit(0);
    }
    let _ = syscall::close(addr_w);
    let _ = syscall::close(hold_r);
    let mut meta = [0u8; 8];
    check_eq!(check_ok!(syscall::read(addr_r, &mut meta), "addr"), 8, "addr len");
    let addr = u64::from_ne_bytes(meta) as usize;
    let len = 15usize;
    let mut dst = [0u8; 32];
    let remote = [IoVec {
        iov_base: addr as *mut u8,
        iov_len: len,
    }];
    let mut local = [IoVec {
        iov_base: dst.as_mut_ptr(),
        iov_len: len,
    }];
    let n = check_ok!(
        syscall::process_vm_readv(pid, &mut local, &remote, 0),
        "readv child"
    );
    check_eq!(n, len, "copied");
    check_eq!(&dst[..len], b"child-vm-bytes!", "payload");
    check_ok!(syscall::close(hold_w), "release child");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check_ok!(syscall::close(addr_r), "close addr_r");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "process_vm_writev into a child buffer is observed by the child")]
fn process_vm_writev_to_child_buf() -> TestResult {
    let (addr_r, addr_w) = check_ok!(syscall::pipe2(0), "addr pipe");
    let (done_r, done_w) = check_ok!(syscall::pipe2(0), "done pipe");
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(addr_r);
        let _ = syscall::close(done_w);
        let mut buf = [0u8; 8];
        let meta = (buf.as_mut_ptr() as u64).to_ne_bytes();
        let _ = syscall::write(addr_w, &meta);
        let _ = syscall::close(addr_w);
        let mut tok = [0u8; 1];
        let _ = syscall::read(done_r, &mut tok);
        if &buf == b"PARENT!!" {
            syscall::exit(0);
        }
        syscall::exit(1);
    }
    let _ = syscall::close(addr_w);
    let _ = syscall::close(done_r);
    let mut meta = [0u8; 8];
    check_eq!(check_ok!(syscall::read(addr_r, &mut meta), "addr"), 8, "addr len");
    let addr = u64::from_ne_bytes(meta) as usize;
    let src = b"PARENT!!";
    let local = [IoVec {
        iov_base: src.as_ptr() as *mut u8,
        iov_len: src.len(),
    }];
    let remote = [IoVec {
        iov_base: addr as *mut u8,
        iov_len: src.len(),
    }];
    let n = check_ok!(
        syscall::process_vm_writev(pid, &local, &remote, 0),
        "writev child"
    );
    check_eq!(n, src.len(), "wrote");
    check_ok!(syscall::write(done_w, b"x"), "signal");
    let mut status = 0;
    check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "child verified");
    check_ok!(syscall::close(addr_r), "close addr_r");
    check_ok!(syscall::close(done_w), "close done_w");
    Ok(())
}
