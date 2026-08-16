//! Socket option and unix socket tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, AF_UNIX, SOCK_DGRAM, SOCK_STREAM, SOL_SOCKET, SO_RCVBUF, SO_REUSEADDR, SO_TYPE,
};

#[crate::lctp_test(suite = syscall, expect = success, case = "getsockopt SO_TYPE on a unix stream socketpair reports SOCK_STREAM")]
fn socketpair_so_type_stream() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let mut val = [0u8; 4];
    let n = check_ok!(syscall::getsockopt(a, SOL_SOCKET, SO_TYPE, &mut val), "getsockopt");
    check_eq!(n, 4, "opt len");
    let ty = i32::from_ne_bytes(val[..4].try_into().unwrap());
    check_eq!(ty, SOCK_STREAM, "SO_TYPE");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsockopt SO_TYPE on both ends of a unix stream socketpair reports SOCK_STREAM")]
fn socketpair_so_type_both_ends() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    for fd in [a, b] {
        let mut val = [0u8; 4];
        check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_TYPE, &mut val), "getsockopt");
        let ty = i32::from_ne_bytes(val[..4].try_into().unwrap());
        check_eq!(ty, SOCK_STREAM, "stream");
    }
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "send and recv on a unix dgram socketpair transfer a datagram payload")]
fn unix_dgram_socketpair_send_recv() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_DGRAM, 0), "socketpair");
    let msg = b"dgram-msg";
    check_eq!(check_ok!(syscall::send(a, msg, 0), "send"), msg.len(), "slen");
    let mut buf = [0u8; 16];
    check_eq!(check_ok!(syscall::recv(b, &mut buf, 0), "recv"), msg.len(), "rlen");
    check!(&buf[..msg.len()] == msg, "data");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setsockopt SO_RCVBUF raises the receive buffer to at least half the requested size")]
fn setsockopt_so_rcvbuf() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let newbuf = 64_000i32;
    let bytes = newbuf.to_ne_bytes();
    check_ok!(syscall::setsockopt(a, SOL_SOCKET, SO_RCVBUF, &bytes), "setsockopt");
    let mut val = [0u8; 4];
    check_ok!(syscall::getsockopt(a, SOL_SOCKET, SO_RCVBUF, &mut val), "getsockopt");
    let got = i32::from_ne_bytes(val[..4].try_into().unwrap());
    check!(got >= newbuf / 2, "rcvbuf raised");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setsockopt SO_REUSEADDR on a unix stream socket succeeds")]
fn setsockopt_so_reuseaddr() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_UNIX, SOCK_STREAM, 0), "socket");
    let one = 1i32.to_ne_bytes();
    check_ok!(syscall::setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one), "reuseaddr");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsockname on a unix socketpair writes a nonzero address length")]
fn getsockname_socketpair() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let mut addr = [0u8; 128];
    let mut len = addr.len() as u32;
    check_ok!(syscall::getsockname(a, &mut addr, &mut len), "getsockname");
    check!(len > 0, "addr len");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getpeername on a unix socketpair writes a nonzero address length")]
fn getpeername_socketpair() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let mut addr = [0u8; 128];
    let mut len = addr.len() as u32;
    check_ok!(syscall::getpeername(a, &mut addr, &mut len), "getpeername");
    check!(len > 0, "peer len");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "unix dgram socketpair send and recv work in both directions")]
fn dgram_socketpair_bidirectional() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_DGRAM, 0), "socketpair");
    check_ok!(syscall::send(a, b"a->b", 0), "send ab");
    check_ok!(syscall::send(b, b"b->a", 0), "send ba");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(b, &mut buf, 0), "recv b"), 4, "rlen b");
    check_eq!(&buf[..4], b"a->b", "ab");
    check_eq!(check_ok!(syscall::recv(a, &mut buf, 0), "recv a"), 4, "rlen a");
    check_eq!(&buf[..4], b"b->a", "ba");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsockopt SO_TYPE on a unix dgram socket reports SOCK_DGRAM")]
fn socket_dgram_so_type() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_UNIX, SOCK_DGRAM, 0), "socket");
    let mut val = [0u8; 4];
    check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_TYPE, &mut val), "getsockopt");
    let ty = i32::from_ne_bytes(val[..4].try_into().unwrap());
    check_eq!(ty, SOCK_DGRAM, "dgram");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "send and recv of 1024 bytes on a unix stream socketpair transfer the payload")]
fn stream_socketpair_large_send() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let msg = [0x5Cu8; 1024];
    check_eq!(check_ok!(syscall::send(a, &msg, 0), "send"), msg.len(), "slen");
    let mut buf = [0u8; 1024];
    check_eq!(check_ok!(syscall::recv(b, &mut buf, 0), "recv"), msg.len(), "rlen");
    check_eq!(&buf, &msg, "payload");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsockopt SO_RCVBUF on a unix stream socketpair reports a positive buffer size")]
fn getsockopt_so_rcvbuf_default() -> TestResult {
    let (a, b) = check_ok!(syscall::socketpair(AF_UNIX, SOCK_STREAM, 0), "socketpair");
    let mut val = [0u8; 4];
    check_ok!(syscall::getsockopt(a, SOL_SOCKET, SO_RCVBUF, &mut val), "getsockopt");
    let got = i32::from_ne_bytes(val[..4].try_into().unwrap());
    check!(got > 0, "default rcvbuf");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}
