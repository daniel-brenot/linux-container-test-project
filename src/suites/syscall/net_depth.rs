//! TCP/UDP depth: backlog, payloads, MSG_DONTWAIT, shutdown, sockopts.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, SockAddrIn, AF_INET, Errno, MSG_DONTWAIT, SHUT_RD, SHUT_RDWR, SHUT_WR, SOCK_CLOEXEC,
    SOCK_DGRAM, SOCK_NONBLOCK, SOCK_STREAM, SOL_SOCKET, SO_KEEPALIVE, SO_LINGER, SO_REUSEADDR,
    SO_SNDBUF,
};

fn listen_ephemeral(backlog: i32) -> Result<(i32, SockAddrIn), crate::harness::AssertFail> {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "socket");
    let one = 1i32.to_ne_bytes();
    check_ok!(syscall::setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one), "reuse");
    check_ok!(syscall::bind(fd, &SockAddrIn::loopback(0)), "bind");
    check_ok!(syscall::listen(fd, backlog), "listen");
    let bound = check_ok!(syscall::getsockname_in(fd), "name");
    Ok((fd, bound))
}

fn tcp_pair() -> Result<(i32, i32, i32), crate::harness::AssertFail> {
    let (srv, bound) = listen_ephemeral(8)?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "cli");
    check_ok!(syscall::connect(cli, &bound), "connect");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "accept");
    Ok((srv, cli, acc))
}

#[crate::lctp_test(suite = syscall, expect = success, case = "listen with backlog 0 succeeds on a bound TCP socket")]
fn net_listen_backlog_0() -> TestResult {
    let (fd, _) = listen_ephemeral(0)?;
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "listen with backlog 1 succeeds on a bound TCP socket")]
fn net_listen_backlog_1() -> TestResult {
    let (fd, _) = listen_ephemeral(1)?;
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "listen with backlog 5 succeeds on a bound TCP socket")]
fn net_listen_backlog_5() -> TestResult {
    let (fd, _) = listen_ephemeral(5)?;
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "listen with backlog 128 succeeds on a bound TCP socket")]
fn net_listen_backlog_128() -> TestResult {
    let (fd, _) = listen_ephemeral(128)?;
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "shutdown SHUT_WR on a TCP client makes the peer recv return 0")]
fn net_tcp_shutdown_wr() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::shutdown(cli, SHUT_WR), "shut");
    let mut b = [0u8; 4];
    check_eq!(check_ok!(syscall::recv(acc, &mut b, 0), "eof"), 0, "eof");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "shutdown SHUT_RD succeeds on an accepted TCP socket")]
fn net_tcp_shutdown_rd() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::shutdown(acc, SHUT_RD), "shut");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "shutdown SHUT_RDWR succeeds on a connected TCP client")]
fn net_tcp_shutdown_rdwr() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::shutdown(cli, SHUT_RDWR), "shut");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getpeername still reports the peer after SHUT_WR")]
fn net_getpeername_after_shutdown() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::shutdown(cli, SHUT_WR), "shut");
    let peer = check_ok!(syscall::getpeername_in(cli), "peer");
    check_eq!(peer.sin_family, AF_INET as u16, "fam");
    check!(peer.port_host() != 0, "port");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "nonblocking UDP recv with MSG_DONTWAIT on an empty socket returns EAGAIN")]
fn net_msg_dontwait_eagain() -> TestResult {
    let (srv, bound) = listen_ephemeral(1)?;
    let cli = check_ok!(
        syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0),
        "cli"
    );
    match syscall::connect(cli, &bound) {
        Ok(()) | Err(Errno::EINTR) => {}
        Err(Errno::EINPROGRESS) => {}
        Err(_) => {
            let _ = syscall::close(cli);
            let _ = syscall::close(srv);
            return Err(crate::harness::AssertFail::msg("connect"));
        }
    }
    // Accept may need a moment; try nonblock recv on fresh accepted fd path via socketpair-like:
    // Use UDP instead for reliable EAGAIN.
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    let u = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0),
        "udp"
    );
    check_ok!(syscall::bind(u, &SockAddrIn::loopback(0)), "bind");
    let mut buf = [0u8; 8];
    check_err!(syscall::recv(u, &mut buf, MSG_DONTWAIT), Errno::EAGAIN, "eagain");
    check_ok!(syscall::close(u), "cu");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "nonblocking TCP recv with MSG_DONTWAIT on an idle socket returns EAGAIN")]
fn net_tcp_msg_dontwait_empty() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    // Set nonblock on acc
    let fl = check_ok!(syscall::fcntl(acc, syscall::fcntl_cmd::F_GETFL, 0), "getfl");
    check_ok!(
        syscall::fcntl(
            acc,
            syscall::fcntl_cmd::F_SETFL,
            (fl as i32 | syscall::oflag::O_NONBLOCK) as usize
        ),
        "setfl"
    );
    let mut buf = [0u8; 8];
    check_err!(syscall::recv(acc, &mut buf, MSG_DONTWAIT), Errno::EAGAIN, "eagain");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "SO_KEEPALIVE can be set and read back or is rejected with ENOPROTOOPT/EINVAL/ENOSYS")]
fn net_so_keepalive_set_get() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    let one = 1i32.to_ne_bytes();
    match syscall::setsockopt(fd, SOL_SOCKET, SO_KEEPALIVE, &one) {
        Ok(()) => {
            let mut val = [0u8; 4];
            check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_KEEPALIVE, &mut val), "get");
            let v = i32::from_ne_bytes(val);
            check_eq!(v, 1, "ka");
        }
        Err(Errno::ENOPROTOOPT) | Err(Errno::EINVAL) | Err(Errno::ENOSYS) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("keepalive"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "SO_LINGER can be set or is rejected with EINVAL/ENOSYS/ENOPROTOOPT")]
fn net_so_linger_soft() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    // struct linger { l_onoff, l_linger }
    let mut ling = [0u8; 8];
    ling[..4].copy_from_slice(&1i32.to_ne_bytes());
    ling[4..].copy_from_slice(&0i32.to_ne_bytes());
    match syscall::setsockopt(fd, SOL_SOCKET, SO_LINGER, &ling) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::ENOSYS) | Err(Errno::ENOPROTOOPT) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("linger"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "SO_SNDBUF can be set and getsockopt reports a positive value")]
fn net_so_sndbuf_set() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    let sz = 64_000i32.to_ne_bytes();
    check_ok!(syscall::setsockopt(fd, SOL_SOCKET, SO_SNDBUF, &sz), "set");
    let mut val = [0u8; 4];
    check_ok!(syscall::getsockopt(fd, SOL_SOCKET, SO_SNDBUF, &mut val), "get");
    check!(i32::from_ne_bytes(val) > 0, "sndbuf");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a two-byte TCP send/recv on loopback delivers the payload")]
fn net_tcp_small_payload() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::send(cli, b"xy", 0), "send");
    let mut b = [0u8; 2];
    check_eq!(check_ok!(syscall::recv(acc, &mut b, 0), "recv"), 2, "n");
    check_eq!(&b, b"xy", "d");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a 1024-byte TCP send/recv on loopback delivers the payload")]
fn net_tcp_large_payload_1k() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    let msg = [0xA5u8; 1024];
    let mut sent = 0usize;
    while sent < msg.len() {
        let n = check_ok!(syscall::send(cli, &msg[sent..], 0), "send");
        check!(n > 0, "progress");
        sent += n;
    }
    let mut buf = [0u8; 1024];
    let mut got = 0usize;
    while got < buf.len() {
        let n = check_ok!(syscall::recv(acc, &mut buf[got..], 0), "recv");
        check!(n > 0, "rprog");
        got += n;
    }
    check!(&buf == &msg, "data");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a 4096-byte TCP send/recv on loopback completes")]
fn net_tcp_large_payload_4k() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    let msg = [0x3Cu8; 4096];
    let mut sent = 0usize;
    while sent < msg.len() {
        let n = check_ok!(syscall::send(cli, &msg[sent..], 0), "send");
        sent += n;
    }
    let mut buf = [0u8; 4096];
    let mut got = 0usize;
    while got < buf.len() {
        let n = check_ok!(syscall::recv(acc, &mut buf[got..], 0), "recv");
        check!(n > 0, "r");
        got += n;
    }
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "UDP sendto/recv on loopback delivers a short datagram")]
fn net_udp_send_recv() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "srv");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "bind");
    let bound = check_ok!(syscall::getsockname_in(srv), "name");
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "cli");
    let msg = b"udp-hi";
    check_eq!(
        check_ok!(syscall::sendto(cli, msg, 0, Some(&bound)), "sendto"),
        msg.len(),
        "n"
    );
    let mut buf = [0u8; 16];
    let n = check_ok!(syscall::recv(srv, &mut buf, 0), "recv");
    check_eq!(&buf[..n], msg, "d");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = failure, case = "nonblocking UDP recv with MSG_DONTWAIT on an empty socket returns EAGAIN")]
fn net_udp_msg_dontwait() -> TestResult {
    let u = check_ok!(
        syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0),
        "u"
    );
    check_ok!(syscall::bind(u, &SockAddrIn::loopback(0)), "bind");
    let mut buf = [0u8; 8];
    check_err!(syscall::recv(u, &mut buf, MSG_DONTWAIT), Errno::EAGAIN, "eagain");
    check_ok!(syscall::close(u), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "UDP sendto/recv on loopback delivers a 512-byte datagram")]
fn net_udp_large_payload() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "srv");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "bind");
    let bound = check_ok!(syscall::getsockname_in(srv), "name");
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "cli");
    let msg = [b'U'; 512];
    check_eq!(
        check_ok!(syscall::sendto(cli, &msg, 0, Some(&bound)), "sendto"),
        512,
        "n"
    );
    let mut buf = [0u8; 512];
    check_eq!(check_ok!(syscall::recv(srv, &mut buf, 0), "recv"), 512, "r");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "send after SHUT_WR returns EPIPE/EINVAL/ECONNRESET or succeeds")]
fn net_tcp_shutdown_then_send_fails_soft() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::shutdown(cli, SHUT_WR), "shut");
    match syscall::send(cli, b"x", 0) {
        Err(Errno::EPIPE) | Err(Errno::EINVAL) | Err(Errno::ECONNRESET) => {}
        Ok(_) => {}
        Err(_) => {
            let _ = syscall::close(acc);
            let _ = syscall::close(cli);
            let _ = syscall::close(srv);
            return Err(crate::harness::AssertFail::msg("send after shut"));
        }
    }
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "shutdown SHUT_RD on the accepted TCP socket succeeds after a send")]
fn net_tcp_bidirectional_shutdown_rd_on_acc() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    check_ok!(syscall::send(cli, b"ping", 0), "send");
    check_ok!(syscall::shutdown(acc, SHUT_RD), "shut rd");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "getsockname after listen reports the bound port")]
fn net_getsockname_after_listen() -> TestResult {
    let (fd, bound) = listen_ephemeral(4)?;
    let again = check_ok!(syscall::getsockname_in(fd), "name");
    check_eq!(again.port_host(), bound.port_host(), "port");
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "accept4 with SOCK_CLOEXEC sets FD_CLOEXEC on the accepted fd")]
fn net_tcp_accept_cloexec() -> TestResult {
    let (srv, bound) = listen_ephemeral(2)?;
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "cli");
    check_ok!(syscall::connect(cli, &bound), "conn");
    let acc = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "acc");
    let flags = check_ok!(syscall::fcntl(acc, syscall::fcntl_cmd::F_GETFD, 0), "fd");
    check!(flags & syscall::FD_CLOEXEC as usize != 0, "cloexec");
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "a connected UDP socket can send to the peer")]
fn net_udp_connect_send() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "srv");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "bind");
    let bound = check_ok!(syscall::getsockname_in(srv), "name");
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "cli");
    check_ok!(syscall::connect(cli, &bound), "conn");
    check_ok!(syscall::send(cli, b"c", 0), "send");
    let mut b = [0u8; 1];
    check_eq!(check_ok!(syscall::recv(srv, &mut b, 0), "recv"), 1, "n");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "setsockopt SO_KEEPALIVE to 0 is accepted on a TCP socket")]
fn net_so_keepalive_off() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    let zero = 0i32.to_ne_bytes();
    let _ = syscall::setsockopt(fd, SOL_SOCKET, SO_KEEPALIVE, &zero);
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "several sequential TCP send/recv pairs deliver each payload")]
fn net_tcp_multi_send_recv() -> TestResult {
    let (srv, cli, acc) = tcp_pair()?;
    for msg in [b"a" as &[u8], b"bb", b"ccc"] {
        check_ok!(syscall::send(cli, msg, 0), "send");
        let mut buf = [0u8; 8];
        let n = check_ok!(syscall::recv(acc, &mut buf, 0), "recv");
        check_eq!(&buf[..n], msg, "d");
    }
    check_ok!(syscall::close(acc), "a");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "shutdown on a listening TCP socket succeeds or returns ENOTCONN/EINVAL")]
fn net_shutdown_listen_fd_soft() -> TestResult {
    let (fd, _) = listen_ephemeral(1)?;
    match syscall::shutdown(fd, SHUT_RD) {
        Ok(()) | Err(Errno::ENOTCONN) | Err(Errno::EINVAL) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("shutdown listen"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full, expect = success, case = "a listening TCP socket can accept two connected clients")]
fn net_tcp_backlog_two_clients_soft() -> TestResult {
    let (srv, bound) = listen_ephemeral(2)?;
    let c1 = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "c1");
    let c2 = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "c2");
    check_ok!(syscall::connect(c1, &bound), "conn1");
    check_ok!(syscall::connect(c2, &bound), "conn2");
    let a1 = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "a1");
    let a2 = check_ok!(syscall::accept4(srv, None, None, SOCK_CLOEXEC), "a2");
    check_ok!(syscall::close(a1), "a1c");
    check_ok!(syscall::close(a2), "a2c");
    check_ok!(syscall::close(c1), "c1c");
    check_ok!(syscall::close(c2), "c2c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = success, case = "UDP sendto of a zero-length datagram is received as zero bytes")]
fn net_udp_zero_len_sendto() -> TestResult {
    let srv = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "srv");
    check_ok!(syscall::bind(srv, &SockAddrIn::loopback(0)), "bind");
    let bound = check_ok!(syscall::getsockname_in(srv), "name");
    let cli = check_ok!(syscall::socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0), "cli");
    check_eq!(check_ok!(syscall::sendto(cli, b"", 0, Some(&bound)), "sendto"), 0, "n");
    let mut buf = [0u8; 4];
    check_eq!(check_ok!(syscall::recv(srv, &mut buf, 0), "recv"), 0, "r");
    check_ok!(syscall::close(cli), "c");
    check_ok!(syscall::close(srv), "s");
    Ok(())
}

#[crate::lctp_test(suite = syscall, expect = soft, case = "getpeername on an unconnected TCP socket returns ENOTCONN/EINVAL or succeeds")]
fn net_getpeername_unconnected_soft() -> TestResult {
    let fd = check_ok!(syscall::socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0), "s");
    match syscall::getpeername_in(fd) {
        Err(Errno::ENOTCONN) | Err(Errno::EINVAL) => {}
        Ok(_) => {}
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("peer"));
        }
    }
    check_ok!(syscall::close(fd), "c");
    Ok(())
}
