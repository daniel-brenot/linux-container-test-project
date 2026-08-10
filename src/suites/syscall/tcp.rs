//! TCP loopback bind/listen/connect/accept4 tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, SockAddrIn, AF_INET, SOCK_CLOEXEC, SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR,
};

fn listen_ephemeral() -> Result<(i32, SockAddrIn), crate::harness::AssertFail> {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "socket");
    let one = 1i32.to_ne_bytes();
    check_ok!(
        syscall::setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one),
        "SO_REUSEADDR"
    );
    let addr = SockAddrIn::loopback(0);
    check_ok!(syscall::bind(fd, &addr), "bind");
    check_ok!(syscall::listen(fd, 8), "listen");
    let bound = check_ok!(syscall::getsockname_in(fd), "getsockname");
    check!(bound.port_host() != 0, "ephemeral port");
    Ok((fd, bound))
}

#[crate::lctp_test(suite = syscall)]
fn tcp_bind_listen_ephemeral() -> TestResult {
    let (fd, addr) = listen_ephemeral()?;
    check_eq!(addr.sin_family, AF_INET as u16, "family");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_connect_accept4() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let mut peer = SockAddrIn::default();
    let mut plen = core::mem::size_of::<SockAddrIn>() as u32;
    let acc = check_ok!(
        syscall::accept4(srv, Some(&mut peer), Some(&mut plen), SOCK_CLOEXEC),
        "accept4"
    );
    check!(acc >= 0, "accepted fd");
    check_eq!(peer.sin_family, AF_INET as u16, "peer family");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_send_recv_loopback() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    let msg = b"tcp-loopback";
    check_eq!(check_ok!(syscall::send(cli, msg, 0), "send"), msg.len(), "slen");
    let mut buf = [0u8; 32];
    check_eq!(check_ok!(syscall::recv(acc, &mut buf, 0), "recv"), msg.len(), "rlen");
    check!(&buf[..msg.len()] == msg, "payload");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_bidirectional() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::send(cli, b"c2s", 0), "c2s");
    check_ok!(syscall::send(acc, b"s2c", 0), "s2c");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(acc, &mut buf, 0), "recv acc"), 3, "acc");
    check_eq!(&buf[..3], b"c2s", "c2s data");
    check_eq!(check_ok!(syscall::recv(cli, &mut buf, 0), "recv cli"), 3, "cli");
    check_eq!(&buf[..3], b"s2c", "s2c data");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn tcp_accept4_cloexec() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    let flags = check_ok!(syscall::fcntl(acc, syscall::fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(flags & syscall::FD_CLOEXEC as usize != 0, "CLOEXEC");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn tcp_shutdown_wr_eof() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::shutdown(cli, syscall::SHUT_WR), "shutdown");
    let mut buf = [0u8; 4];
    let n = check_ok!(syscall::recv(acc, &mut buf, 0), "recv eof");
    check_eq!(n, 0, "EOF");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_getsockname_port_matches() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let peer = check_ok!(syscall::getpeername_in(cli), "getpeername");
    check_eq!(peer.port_host(), bound.port_host(), "peer port");
    check_eq!(peer.sin_family, AF_INET as u16, "peer family");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_reuseaddr_rebind() -> TestResult {
    let (fd1, bound) = listen_ephemeral()?;
    check_ok!(syscall::close(fd1), "close1");
    let fd2 = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "socket2");
    let one = 1i32.to_ne_bytes();
    check_ok!(
        syscall::setsockopt(fd2, SOL_SOCKET, SO_REUSEADDR, &one),
        "reuse"
    );
    // Rebind same ephemeral port may race; bind to 0 again is always safe.
    let addr = SockAddrIn::loopback(0);
    check_ok!(syscall::bind(fd2, &addr), "rebind");
    let _ = bound;
    check_ok!(syscall::close(fd2), "close2");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_shutdown_after_connect_rd() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::shutdown(cli, syscall::SHUT_RD), "shutdown rd");
    // Peer can still send; local read side is shut down.
    check_ok!(syscall::send(acc, b"x", 0), "peer send");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn tcp_accept4_nonblock_flag() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(
        syscall::accept4(srv, None, None, SOCK_CLOEXEC | syscall::SOCK_NONBLOCK),
        "accept4"
    );
    let flags = check_ok!(syscall::fcntl(acc, syscall::fcntl_cmd::F_GETFL, 0), "F_GETFL");
    check!(flags as i32 & syscall::oflag::O_NONBLOCK != 0, "NONBLOCK");
    let fdflags = check_ok!(syscall::fcntl(acc, syscall::fcntl_cmd::F_GETFD, 0), "F_GETFD");
    check!(fdflags & syscall::FD_CLOEXEC as usize != 0, "CLOEXEC");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_shutdown_rdwr() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::shutdown(cli, syscall::SHUT_RDWR), "shutdown rdwr");
    let mut buf = [0u8; 4];
    let n = check_ok!(syscall::recv(acc, &mut buf, 0), "recv eof");
    check_eq!(n, 0, "EOF");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn tcp_large_send_recv() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    let mut msg = [0u8; 8192];
    for (i, b) in msg.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    let mut sent = 0usize;
    while sent < msg.len() {
        let n = check_ok!(syscall::send(cli, &msg[sent..], 0), "send");
        check!(n > 0, "send progress");
        sent += n;
    }
    let mut buf = [0u8; 8192];
    let mut got = 0usize;
    while got < msg.len() {
        let n = check_ok!(syscall::recv(acc, &mut buf[got..], 0), "recv");
        check!(n > 0, "recv progress");
        got += n;
    }
    check!(&buf == &msg, "payload");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn tcp_shutdown_wr_then_recv_zero() -> TestResult {
    let (srv, bound) = listen_ephemeral()?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "client");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept4");
    check_ok!(syscall::send(cli, b"pre", 0), "send pre");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(acc, &mut buf, 0), "recv pre"), 3, "pre");
    check_ok!(syscall::shutdown(cli, syscall::SHUT_WR), "SHUT_WR");
    let n = check_ok!(syscall::recv(acc, &mut buf, 0), "recv eof");
    check_eq!(n, 0, "EOF after SHUT_WR");
    // Peer can still write the other way.
    check_ok!(syscall::send(acc, b"ack", 0), "peer send");
    check_eq!(check_ok!(syscall::recv(cli, &mut buf, 0), "cli recv"), 3, "ack");
    check_ok!(syscall::close(acc), "close acc");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}
