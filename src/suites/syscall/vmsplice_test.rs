//! vmsplice(2) tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, IoVec};

#[crate::lctp_test(suite = syscall)]
fn vmsplice_into_pipe() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut data = *b"vmsplice-payload";
    let iov = [IoVec {
        iov_base: data.as_mut_ptr(),
        iov_len: data.len(),
    }];
    let n = check_ok!(syscall::vmsplice(w, &iov, 0), "vmsplice");
    check_eq!(n, data.len(), "spliced");
    check_ok!(syscall::close(w), "close w");
    let mut buf = [0u8; 32];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), data.len(), "rlen");
    check!(&buf[..data.len()] == b"vmsplice-payload", "data");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn vmsplice_multi_iovec() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut a = *b"hello";
    let mut b = *b"-world";
    let iov = [
        IoVec {
            iov_base: a.as_mut_ptr(),
            iov_len: a.len(),
        },
        IoVec {
            iov_base: b.as_mut_ptr(),
            iov_len: b.len(),
        },
    ];
    let n = check_ok!(syscall::vmsplice(w, &iov, 0), "vmsplice");
    check_eq!(n, 11, "total");
    check_ok!(syscall::close(w), "close w");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 11, "rlen");
    check_eq!(&buf[..11], b"hello-world", "concat");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn vmsplice_empty_iov() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let iov: [IoVec; 0] = [];
    let n = check_ok!(syscall::vmsplice(w, &iov, 0), "vmsplice empty");
    check_eq!(n, 0, "zero");
    check_ok!(syscall::close(w), "close w");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn vmsplice_partial_then_read() -> TestResult {
    let (r, w) = check_ok!(syscall::pipe2(0), "pipe2");
    let mut data = [0u8; 64];
    for (i, b) in data.iter_mut().enumerate() {
        *b = i as u8;
    }
    let iov = [IoVec {
        iov_base: data.as_mut_ptr(),
        iov_len: data.len(),
    }];
    let n = check_ok!(syscall::vmsplice(w, &iov, 0), "vmsplice");
    check_eq!(n, 64, "n");
    check_ok!(syscall::close(w), "close w");
    let mut buf = [0u8; 64];
    check_eq!(check_ok!(syscall::read(r, &mut buf), "read"), 64, "rlen");
    check_eq!(buf, data, "bytes");
    check_ok!(syscall::close(r), "close r");
    Ok(())
}
