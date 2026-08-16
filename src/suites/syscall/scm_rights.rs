//! sendmsg/recvmsg with SCM_RIGHTS fd passing.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, cmsg_align, cmsg_len, cmsg_space, CmsgHdr, IoVec, MsgHdr, AF_UNIX, SCM_RIGHTS,
    SOCK_CLOEXEC, SOCK_STREAM, SOL_SOCKET,
};

fn put_scm_rights(cbuf: &mut [u8], fd: i32) -> usize {
    let hdr_size = cmsg_align(core::mem::size_of::<CmsgHdr>());
    let needed = cmsg_space(core::mem::size_of::<i32>());
    for b in &mut cbuf[..needed] {
        *b = 0;
    }
    unsafe {
        let cmsg = cbuf.as_mut_ptr() as *mut CmsgHdr;
        (*cmsg).cmsg_len = cmsg_len(core::mem::size_of::<i32>());
        (*cmsg).cmsg_level = SOL_SOCKET;
        (*cmsg).cmsg_type = SCM_RIGHTS;
        let data = cbuf.as_mut_ptr().add(hdr_size) as *mut i32;
        *data = fd;
    }
    needed
}

fn take_scm_rights(cbuf: &[u8], controllen: usize) -> Option<i32> {
    if controllen < core::mem::size_of::<CmsgHdr>() {
        return None;
    }
    unsafe {
        let cmsg = cbuf.as_ptr() as *const CmsgHdr;
        if (*cmsg).cmsg_level != SOL_SOCKET || (*cmsg).cmsg_type != SCM_RIGHTS {
            return None;
        }
        let hdr_size = cmsg_align(core::mem::size_of::<CmsgHdr>());
        if controllen < hdr_size + core::mem::size_of::<i32>() {
            return None;
        }
        let data = cbuf.as_ptr().add(hdr_size) as *const i32;
        Some(*data)
    }
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendmsg SCM_RIGHTS passes a pipe fd whose payload is readable on the received fd")]
fn scm_rights_pass_pipe_fd() -> TestResult {
    let (a, b) = check_ok!(
        syscall::socketpair(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "socketpair"
    );
    let (pr, pw) = check_ok!(syscall::pipe2(syscall::oflag::O_CLOEXEC), "pipe2");
    check_ok!(syscall::write(pw, b"via-fd"), "write pipe");
    check_ok!(syscall::close(pw), "close pw");

    let mut dummy = [b'x'];
    let mut iov = [IoVec {
        iov_base: dummy.as_mut_ptr(),
        iov_len: 1,
    }];
    let mut cbuf = [0u8; 64];
    let clen = put_scm_rights(&mut cbuf, pr);
    let msg = MsgHdr {
        msg_name: core::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: iov.as_mut_ptr(),
        msg_iovlen: 1,
        msg_control: cbuf.as_mut_ptr(),
        msg_controllen: clen,
        msg_flags: 0,
    };
    check_eq!(check_ok!(syscall::sendmsg(a, &msg, 0), "sendmsg"), 1, "sent");
    check_ok!(syscall::close(pr), "close original pr");

    let mut rbuf = [0u8; 1];
    let mut riov = [IoVec {
        iov_base: rbuf.as_mut_ptr(),
        iov_len: 1,
    }];
    let mut rcbuf = [0u8; 64];
    let mut rmsg = MsgHdr {
        msg_name: core::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: riov.as_mut_ptr(),
        msg_iovlen: 1,
        msg_control: rcbuf.as_mut_ptr(),
        msg_controllen: rcbuf.len(),
        msg_flags: 0,
    };
    check_eq!(check_ok!(syscall::recvmsg(b, &mut rmsg, 0), "recvmsg"), 1, "recv");
    check_eq!(rbuf[0], b'x', "dummy");
    check!(rmsg.msg_controllen > 0, "got control");
    let received = take_scm_rights(&rcbuf, rmsg.msg_controllen)
        .ok_or_else(|| crate::harness::AssertFail::msg("no SCM_RIGHTS"))?;
    check!(received >= 0, "fd");

    let mut payload = [0u8; 16];
    let n = check_ok!(syscall::read(received, &mut payload), "read passed fd");
    check_eq!(n, 6, "payload len");
    check_eq!(&payload[..6], b"via-fd", "payload");

    check_ok!(syscall::close(received), "close received");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "sendmsg SCM_RIGHTS passes a memfd whose contents are readable on the received fd")]
fn scm_rights_pass_memfd() -> TestResult {
    let (a, b) = check_ok!(
        syscall::socketpair(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "socketpair"
    );
    let mfd = check_ok!(syscall::memfd_create(b"scm\0", syscall::MFD_CLOEXEC as u32), "memfd");
    check_ok!(syscall::write(mfd, b"mem"), "write memfd");
    check_ok!(syscall::lseek(mfd, 0, syscall::SEEK_SET), "lseek");

    let mut dummy = [b'y'];
    let mut iov = [IoVec {
        iov_base: dummy.as_mut_ptr(),
        iov_len: 1,
    }];
    let mut cbuf = [0u8; 64];
    let clen = put_scm_rights(&mut cbuf, mfd);
    let msg = MsgHdr {
        msg_name: core::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: iov.as_mut_ptr(),
        msg_iovlen: 1,
        msg_control: cbuf.as_mut_ptr(),
        msg_controllen: clen,
        msg_flags: 0,
    };
    check_ok!(syscall::sendmsg(a, &msg, 0), "sendmsg");
    check_ok!(syscall::close(mfd), "close mfd");

    let mut rbuf = [0u8; 1];
    let mut riov = [IoVec {
        iov_base: rbuf.as_mut_ptr(),
        iov_len: 1,
    }];
    let mut rcbuf = [0u8; 64];
    let mut rmsg = MsgHdr {
        msg_name: core::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: riov.as_mut_ptr(),
        msg_iovlen: 1,
        msg_control: rcbuf.as_mut_ptr(),
        msg_controllen: rcbuf.len(),
        msg_flags: 0,
    };
    check_ok!(syscall::recvmsg(b, &mut rmsg, 0), "recvmsg");
    let received = take_scm_rights(&rcbuf, rmsg.msg_controllen)
        .ok_or_else(|| crate::harness::AssertFail::msg("no SCM_RIGHTS"))?;
    let mut payload = [0u8; 8];
    check_eq!(check_ok!(syscall::read(received, &mut payload), "read"), 3, "len");
    check_eq!(&payload[..3], b"mem", "data");
    check_ok!(syscall::close(received), "close");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "sendmsg and recvmsg without control data transfer a plain payload")]
fn scm_rights_sendmsg_no_control() -> TestResult {
    let (a, b) = check_ok!(
        syscall::socketpair(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0),
        "socketpair"
    );
    let mut data = *b"plain";
    let mut iov = [IoVec {
        iov_base: data.as_mut_ptr(),
        iov_len: data.len(),
    }];
    let msg = MsgHdr {
        msg_name: core::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: iov.as_mut_ptr(),
        msg_iovlen: 1,
        msg_control: core::ptr::null_mut(),
        msg_controllen: 0,
        msg_flags: 0,
    };
    check_eq!(check_ok!(syscall::sendmsg(a, &msg, 0), "sendmsg"), 5, "sent");
    let mut rbuf = [0u8; 8];
    let mut riov = [IoVec {
        iov_base: rbuf.as_mut_ptr(),
        iov_len: rbuf.len(),
    }];
    let mut rmsg = MsgHdr {
        msg_name: core::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: riov.as_mut_ptr(),
        msg_iovlen: 1,
        msg_control: core::ptr::null_mut(),
        msg_controllen: 0,
        msg_flags: 0,
    };
    check_eq!(check_ok!(syscall::recvmsg(b, &mut rmsg, 0), "recvmsg"), 5, "recv");
    check_eq!(&rbuf[..5], b"plain", "data");
    check_ok!(syscall::close(a), "close a");
    check_ok!(syscall::close(b), "close b");
    Ok(())
}
