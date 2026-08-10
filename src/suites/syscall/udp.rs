//! UDP loopback sendto/recvfrom tests.

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, SockAddrIn, AF_INET, SOCK_CLOEXEC, SOCK_DGRAM};

fn bind_udp_ephemeral() -> Result<(i32, SockAddrIn), crate::harness::AssertFail> {
    let fd = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "socket"
    );
    let addr = SockAddrIn::loopback(0);
    check_ok!(syscall::bind(fd, &addr), "bind");
    let bound = check_ok!(syscall::getsockname_in(fd), "getsockname");
    check!(bound.port_host() != 0, "ephemeral port");
    Ok((fd, bound))
}

#[crate::lctp_test(suite = syscall)]
fn udp_bind_ephemeral() -> TestResult {
    let (fd, addr) = bind_udp_ephemeral()?;
    check_eq!(addr.sin_family, AF_INET as u16, "family");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn udp_sendto_recvfrom_loopback() -> TestResult {
    let (srv, bound) = bind_udp_ephemeral()?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "client"
    );
    let msg = b"udp-ping";
    check_eq!(
        check_ok!(syscall::sendto(cli, msg, 0, Some(&bound)), "sendto"),
        msg.len(),
        "slen"
    );
    let mut buf = [0u8; 32];
    let mut peer = SockAddrIn::default();
    let mut plen = core::mem::size_of::<SockAddrIn>() as u32;
    let n = check_ok!(
        syscall::recvfrom(srv, &mut buf, 0, Some(&mut peer), Some(&mut plen)),
        "recvfrom"
    );
    check_eq!(n, msg.len(), "rlen");
    check!(&buf[..msg.len()] == msg, "payload");
    check_eq!(peer.sin_family, AF_INET as u16, "peer family");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn udp_echo_roundtrip() -> TestResult {
    let (srv, bound) = bind_udp_ephemeral()?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "client"
    );
    // Bind client so server can reply to a known address.
    check_ok!(syscall::bind(cli, &SockAddrIn::loopback(0)), "bind cli");
    let cli_addr = check_ok!(syscall::getsockname_in(cli), "cli getsockname");

    check_ok!(syscall::sendto(cli, b"ping", 0, Some(&bound)), "sendto");
    let mut buf = [0u8; 16];
    let mut peer = SockAddrIn::default();
    let mut plen = core::mem::size_of::<SockAddrIn>() as u32;
    let n = check_ok!(
        syscall::recvfrom(srv, &mut buf, 0, Some(&mut peer), Some(&mut plen)),
        "recv"
    );
    check_eq!(n, 4, "ping len");
    check_eq!(peer.port_host(), cli_addr.port_host(), "peer port");
    check_ok!(syscall::sendto(srv, b"pong", 0, Some(&peer)), "reply");
    let mut rbuf = [0u8; 16];
    let rn = check_ok!(syscall::recvfrom(cli, &mut rbuf, 0, None, None), "recv reply");
    check_eq!(rn, 4, "pong len");
    check_eq!(&rbuf[..4], b"pong", "pong");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn udp_sendto_zero_len() -> TestResult {
    let (srv, bound) = bind_udp_ephemeral()?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_eq!(
        check_ok!(syscall::sendto(cli, b"", 0, Some(&bound)), "sendto empty"),
        0,
        "slen"
    );
    let mut buf = [0u8; 8];
    let n = check_ok!(syscall::recvfrom(srv, &mut buf, 0, None, None), "recv");
    check_eq!(n, 0, "empty datagram");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn udp_connected_send_recv() -> TestResult {
    let (srv, bound) = bind_udp_ephemeral()?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::connect(cli, &bound), "connect");
    let msg = b"connected-udp";
    check_eq!(check_ok!(syscall::send(cli, msg, 0), "send"), msg.len(), "slen");
    let mut buf = [0u8; 32];
    check_eq!(
        check_ok!(syscall::recvfrom(srv, &mut buf, 0, None, None), "recv"),
        msg.len(),
        "rlen"
    );
    check!(&buf[..msg.len()] == msg, "data");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn udp_largeish_datagram() -> TestResult {
    let (srv, bound) = bind_udp_ephemeral()?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "client"
    );
    let mut msg = [0u8; 1200];
    for (i, b) in msg.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    check_eq!(
        check_ok!(syscall::sendto(cli, &msg, 0, Some(&bound)), "sendto"),
        msg.len(),
        "slen"
    );
    let mut buf = [0u8; 1400];
    let n = check_ok!(syscall::recvfrom(srv, &mut buf, 0, None, None), "recv");
    check_eq!(n, msg.len(), "rlen");
    check!(&buf[..msg.len()] == msg, "payload");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn udp_connected_socket_bidirectional() -> TestResult {
    let (srv, bound) = bind_udp_ephemeral()?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0),
        "client"
    );
    check_ok!(syscall::bind(cli, &SockAddrIn::loopback(0)), "bind cli");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let cli_addr = check_ok!(syscall::getsockname_in(cli), "cli addr");
    check_ok!(syscall::connect(srv, &cli_addr), "connect srv");
    check_ok!(syscall::send(cli, b"up", 0), "cli send");
    let mut buf = [0u8; 8];
    check_eq!(check_ok!(syscall::recv(srv, &mut buf, 0), "srv recv"), 2, "up");
    check_eq!(&buf[..2], b"up", "up data");
    check_ok!(syscall::send(srv, b"dn", 0), "srv send");
    check_eq!(check_ok!(syscall::recv(cli, &mut buf, 0), "cli recv"), 2, "dn");
    check_eq!(&buf[..2], b"dn", "dn data");
    check_ok!(syscall::close(cli), "close cli");
    check_ok!(syscall::close(srv), "close srv");
    Ok(())
}
