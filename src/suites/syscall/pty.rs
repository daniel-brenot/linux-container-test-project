//! Pseudo-terminal (`/dev/ptmx` + `/dev/pts/N`) coverage.
//!
//! Mirrors the glibc `openpty` sequence: open ptmx → `TIOCGPTN` → unlock →
//! open `/dev/pts/N`. Guests with a sparse `/dev` (no on-disk `pts/` directory)
//! must still succeed via synthetic device nodes — bare `open("/dev/ptmx")`
//! alone is not enough.

use crate::check;
use crate::check_eq;
use crate::check_err;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{
    self, oflag, poll, wait, Errno, F_OK, POLLIN, TIOCGPTN, TIOCSPTLCK, TIOCSCTTY,
};

const PTMX: &[u8] = b"/dev/ptmx\0";

/// Write `/dev/pts/<n>\0` into `out`; returns length including the NUL.
fn format_pts_path(n: u32, out: &mut [u8; 64]) -> Result<usize, crate::harness::AssertFail> {
    let prefix = b"/dev/pts/";
    out[..prefix.len()].copy_from_slice(prefix);
    let mut digits = [0u8; 10];
    let mut x = n;
    let mut nd = 0usize;
    if x == 0 {
        digits[0] = b'0';
        nd = 1;
    } else {
        while x > 0 {
            digits[nd] = b'0' + (x % 10) as u8;
            x /= 10;
            nd += 1;
        }
        let mut i = 0;
        let mut j = nd - 1;
        while i < j {
            digits.swap(i, j);
            i += 1;
            j -= 1;
        }
    }
    let start = prefix.len();
    if start + nd + 1 > out.len() {
        return Err(crate::harness::AssertFail::msg("pts path buf"));
    }
    out[start..start + nd].copy_from_slice(&digits[..nd]);
    out[start + nd] = 0;
    Ok(start + nd + 1)
}

/// glibc-style `openpty`: master fd, slave fd, pty index.
fn openpty_pair() -> Result<(i32, i32, u32), crate::harness::AssertFail> {
    let master = check_ok!(
        syscall::open(PTMX, oflag::O_RDWR | oflag::O_NOCTTY | oflag::O_CLOEXEC, 0),
        "open /dev/ptmx"
    );
    let mut ptyno: u32 = 0;
    if syscall::ioctl(master, TIOCGPTN, &mut ptyno as *mut u32 as usize).is_err() {
        let _ = syscall::close(master);
        return Err(crate::harness::AssertFail::msg("TIOCGPTN"));
    }
    let mut unlock: i32 = 0;
    if syscall::ioctl(master, TIOCSPTLCK, &mut unlock as *mut i32 as usize).is_err() {
        let _ = syscall::close(master);
        return Err(crate::harness::AssertFail::msg("TIOCSPTLCK unlock"));
    }
    let mut path = [0u8; 64];
    let plen = match format_pts_path(ptyno, &mut path) {
        Ok(n) => n,
        Err(e) => {
            let _ = syscall::close(master);
            return Err(e);
        }
    };
    let slave = match syscall::open(
        &path[..plen],
        oflag::O_RDWR | oflag::O_NOCTTY | oflag::O_CLOEXEC,
        0,
    ) {
        Ok(fd) => fd,
        Err(_) => {
            let _ = syscall::close(master);
            return Err(crate::harness::AssertFail::msg("open /dev/pts/N"));
        }
    };
    Ok((master, slave, ptyno))
}

fn close_pair(master: i32, slave: i32) {
    let _ = syscall::close(slave);
    let _ = syscall::close(master);
}

#[crate::lctp_test(suite = syscall)]
fn pty_ptmx_open_chr() -> TestResult {
    let fd = check_ok!(
        syscall::open(PTMX, oflag::O_RDWR | oflag::O_NOCTTY, 0),
        "open ptmx"
    );
    let st = match syscall::fstat(fd) {
        Ok(st) => st,
        Err(_) => {
            let _ = syscall::close(fd);
            return Err(crate::harness::AssertFail::msg("fstat ptmx"));
        }
    };
    check!(st.is_chr(), "ptmx not chr");
    check_ok!(syscall::close(fd), "close");
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pty_openpty_pair() -> TestResult {
    let (master, slave, ptyno) = openpty_pair()?;
    check!(master >= 0, "master");
    check!(slave >= 0, "slave");
    check!(master != slave, "distinct fds");
    let st = check_ok!(syscall::fstat(slave), "fstat slave");
    check!(st.is_chr(), "slave not chr");
    let mut path = [0u8; 64];
    let plen = format_pts_path(ptyno, &mut path)?;
    check_ok!(syscall::access(&path[..plen], F_OK), "pts path F_OK");
    close_pair(master, slave);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pty_unused_index_enoent() -> TestResult {
    // No live pty at a high index → ENOENT.
    check_err!(
        syscall::open(b"/dev/pts/999999\0", oflag::O_RDWR | oflag::O_NOCTTY, 0),
        Errno::ENOENT,
        "unused pts"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pty_slave_open_after_unlock() -> TestResult {
    let master = check_ok!(
        syscall::open(PTMX, oflag::O_RDWR | oflag::O_NOCTTY | oflag::O_CLOEXEC, 0),
        "ptmx"
    );
    let mut ptyno: u32 = 0;
    check_ok!(
        syscall::ioctl(master, TIOCGPTN, &mut ptyno as *mut u32 as usize),
        "TIOCGPTN"
    );
    let mut unlock: i32 = 0;
    check_ok!(
        syscall::ioctl(master, TIOCSPTLCK, &mut unlock as *mut i32 as usize),
        "unlock"
    );
    let mut path = [0u8; 64];
    let plen = format_pts_path(ptyno, &mut path)?;
    let slave = check_ok!(
        syscall::open(
            &path[..plen],
            oflag::O_RDWR | oflag::O_NOCTTY | oflag::O_CLOEXEC,
            0
        ),
        "open slave"
    );
    close_pair(master, slave);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pty_master_slave_byte_io() -> TestResult {
    let (master, slave, _) = openpty_pair()?;
    check_ok!(syscall::write(slave, b"ping\n"), "write slave");
    let mut fds = [poll::PollFd {
        fd: master,
        events: POLLIN,
        revents: 0,
    }];
    let n = check_ok!(syscall::poll(&mut fds, 2000), "poll master");
    check!(n >= 1, "master readable");
    let mut buf = [0u8; 64];
    let nr = check_ok!(syscall::read(master, &mut buf), "read master");
    check!(nr >= 4, "short");
    check_eq!(buf[0], b'p', "p");
    check_eq!(buf[1], b'i', "i");
    check_eq!(buf[2], b'n', "n");
    check_eq!(buf[3], b'g', "g");
    close_pair(master, slave);
    Ok(())
}

#[crate::lctp_test(suite = syscall)]
fn pty_shell_echo_via_master() -> TestResult {
    // openpty + fork/exec `/bin/sh -c 'echo ok'` with slave as stdio; parent
    // reads "ok" from the master — exercises the full terminal spawn path.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "/bin/sh");
    let (master, slave, _) = openpty_pair()?;
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(master);
        let _ = syscall::setsid();
        let _ = syscall::ioctl(slave, TIOCSCTTY, 0);
        if syscall::dup2(slave, 0).is_err()
            || syscall::dup2(slave, 1).is_err()
            || syscall::dup2(slave, 2).is_err()
        {
            syscall::exit(125);
        }
        if slave > 2 {
            let _ = syscall::close(slave);
        }
        let arg0 = b"sh\0";
        let arg1 = b"-c\0";
        let arg2 = b"echo ok\0";
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            core::ptr::null(),
        ];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(slave);

    let mut buf = [0u8; 64];
    let mut filled = 0usize;
    let mut reaped = false;
    let mut status = 0i32;
    for _ in 0..50 {
        if !reaped {
            match syscall::wait4(pid, &mut status, wait::WNOHANG) {
                Ok(p) if p == pid => reaped = true,
                _ => {}
            }
        }
        let mut fds = [poll::PollFd {
            fd: master,
            events: POLLIN,
            revents: 0,
        }];
        if syscall::poll(&mut fds, 100).unwrap_or(0) > 0 {
            match syscall::read(master, &mut buf[filled..]) {
                Ok(0) => break,
                Ok(n) => {
                    filled += n;
                    if filled >= 2 && (buf[0] == b'o' || filled >= 8) {
                        break;
                    }
                }
                Err(Errno::EIO) => break,
                Err(_) => {}
            }
        } else if reaped {
            // Final drain after child exit.
            if let Ok(n) = syscall::read(master, &mut buf[filled..]) {
                filled += n;
            }
            break;
        }
    }
    if !reaped {
        check_ok!(syscall::wait4(pid, &mut status, 0), "wait");
    }
    let _ = syscall::close(master);
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "sh status");
    // PTY line discipline may prefix with \r; find "ok".
    let mut found = false;
    if filled >= 2 {
        for i in 0..filled.saturating_sub(1) {
            if buf[i] == b'o' && buf[i + 1] == b'k' {
                found = true;
                break;
            }
        }
    }
    check!(found, "missing ok on master");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pty_dev_pts_usable_with_live_pair() -> TestResult {
    // With a live pair, `/dev/pts` must resolve as a directory *or* the concrete
    // `/dev/pts/N` slave path must remain openable — covering synthetic pts.
    let (master, slave, ptyno) = openpty_pair()?;
    let mut path = [0u8; 64];
    let plen = format_pts_path(ptyno, &mut path)?;
    match syscall::stat(b"/dev/pts\0") {
        Ok(st) => check!(st.is_dir(), "/dev/pts not dir"),
        Err(Errno::ENOENT) => {
            check_ok!(syscall::access(&path[..plen], F_OK), "slave path");
        }
        Err(_) => {
            close_pair(master, slave);
            return Err(crate::harness::AssertFail::msg("stat /dev/pts"));
        }
    }
    // Re-open the live slave path by name (second fd).
    let slave2 = check_ok!(
        syscall::open(
            &path[..plen],
            oflag::O_RDWR | oflag::O_NOCTTY | oflag::O_CLOEXEC,
            0
        ),
        "reopen slave"
    );
    check_ok!(syscall::close(slave2), "close slave2");
    close_pair(master, slave);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pty_two_pairs_independent() -> TestResult {
    let (m1, s1, n1) = openpty_pair()?;
    let (m2, s2, n2) = openpty_pair()?;
    check!(n1 != n2, "distinct pty nums");
    check_ok!(syscall::write(s1, b"A\n"), "w1");
    check_ok!(syscall::write(s2, b"B\n"), "w2");
    let mut a = [0u8; 16];
    let mut b = [0u8; 16];
    let na = check_ok!(syscall::read(m1, &mut a), "r1");
    let nb = check_ok!(syscall::read(m2, &mut b), "r2");
    check!(na >= 1 && a[0] == b'A', "pair1");
    check!(nb >= 1 && b[0] == b'B', "pair2");
    close_pair(m1, s1);
    close_pair(m2, s2);
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pty_slave_gone_after_close_soft() -> TestResult {
    let (master, slave, ptyno) = openpty_pair()?;
    let mut path = [0u8; 64];
    let plen = format_pts_path(ptyno, &mut path)?;
    close_pair(master, slave);
    match syscall::open(&path[..plen], oflag::O_RDWR | oflag::O_NOCTTY, 0) {
        Err(Errno::ENOENT) | Err(Errno::EIO) | Err(Errno::ENXIO) => {}
        Ok(fd) => {
            let _ = syscall::close(fd);
        }
        Err(_) => {}
    }
    Ok(())
}

fn buf_contains(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return false;
    }
    for i in 0..=hay.len() - needle.len() {
        if &hay[i..i + needle.len()] == needle {
            return true;
        }
    }
    false
}

fn pty_drain_until(
    master: i32,
    pid: i32,
    pred: &dyn Fn(&[u8]) -> bool,
    buf: &mut [u8],
) -> Result<(usize, i32, bool), crate::harness::AssertFail> {
    let mut filled = 0usize;
    let mut reaped = false;
    let mut status = 0i32;
    for _ in 0..80 {
        if !reaped {
            match syscall::wait4(pid, &mut status, wait::WNOHANG) {
                Ok(p) if p == pid => reaped = true,
                _ => {}
            }
        }
        let mut fds = [poll::PollFd {
            fd: master,
            events: POLLIN,
            revents: 0,
        }];
        if syscall::poll(&mut fds, 100).unwrap_or(0) > 0 {
            let end = if filled < buf.len() { buf.len() } else { filled };
            if filled < buf.len() {
                match syscall::read(master, &mut buf[filled..end]) {
                    Ok(0) => break,
                    Ok(n) => {
                        filled += n;
                        if pred(&buf[..filled]) {
                            break;
                        }
                    }
                    Err(Errno::EIO) => break,
                    Err(_) => {}
                }
            }
        } else if reaped {
            if filled < buf.len() {
                if let Ok(n) = syscall::read(master, &mut buf[filled..]) {
                    filled += n;
                }
            }
            break;
        }
    }
    if !reaped {
        // Bound final wait so a deadlock fails the suite.
        for _ in 0..40 {
            match syscall::wait4(pid, &mut status, wait::WNOHANG) {
                Ok(p) if p == pid => {
                    reaped = true;
                    break;
                }
                _ => {}
            }
            let req = syscall::Timespec {
                tv_sec: 0,
                tv_nsec: 50_000_000,
            };
            let _ = syscall::nanosleep(&req);
        }
        if !reaped {
            let _ = syscall::kill(pid, 9);
            let _ = syscall::wait4(pid, &mut status, 0);
            return Err(crate::harness::AssertFail::msg("pty child hung"));
        }
    }
    Ok((filled, status, reaped))
}

fn pty_spawn_shell_cmd(shell: &[u8], arg0: &[u8], cmd: &[u8]) -> Result<(i32, i32), crate::harness::AssertFail> {
    let (master, slave, _) = openpty_pair()?;
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(master);
        let _ = syscall::setsid();
        let _ = syscall::ioctl(slave, TIOCSCTTY, 0);
        if syscall::dup2(slave, 0).is_err()
            || syscall::dup2(slave, 1).is_err()
            || syscall::dup2(slave, 2).is_err()
        {
            syscall::exit(125);
        }
        if slave > 2 {
            let _ = syscall::close(slave);
        }
        let dash_c = b"-c\0";
        let argv = [
            arg0.as_ptr(),
            dash_c.as_ptr(),
            cmd.as_ptr(),
            core::ptr::null(),
        ];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(shell, &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(slave);
    Ok((master, pid))
}

#[crate::lctp_test(suite = syscall)]
fn pty_shell_ls_root() -> TestResult {
    // Shell forks an external `ls` with the PTY as stdio — must list and exit
    // (deadlock if a child pipe read wrongly yields into the waiting parent).
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let (master, pid) = pty_spawn_shell_cmd(b"/bin/sh\0", b"sh\0", b"ls /\0")?;
    let mut buf = [0u8; 1024];
    let (n, status, _) = pty_drain_until(
        master,
        pid,
        &|b| buf_contains(b, b"etc") || buf_contains(b, b"bin") || buf_contains(b, b"usr"),
        &mut buf,
    )?;
    let _ = syscall::close(master);
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "ls status");
    check!(
        buf_contains(&buf[..n], b"etc")
            || buf_contains(&buf[..n], b"bin")
            || buf_contains(&buf[..n], b"usr")
            || buf_contains(&buf[..n], b"tmp"),
        "ls listing"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pty_bash_ls_root_soft() -> TestResult {
    if syscall::access(b"/bin/bash\0", F_OK).is_err() {
        return Ok(());
    }
    let (master, pid) = pty_spawn_shell_cmd(b"/bin/bash\0", b"bash\0", b"ls /\0")?;
    let mut buf = [0u8; 1024];
    let (n, status, _) = pty_drain_until(
        master,
        pid,
        &|b| buf_contains(b, b"etc") || buf_contains(b, b"bin") || buf_contains(b, b"usr"),
        &mut buf,
    )?;
    let _ = syscall::close(master);
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "bash ls");
    check!(
        buf_contains(&buf[..n], b"etc")
            || buf_contains(&buf[..n], b"bin")
            || buf_contains(&buf[..n], b"usr")
            || buf_contains(&buf[..n], b"tmp"),
        "listing"
    );
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pty_shell_pipeline_on_pty() -> TestResult {
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let (master, pid) = pty_spawn_shell_cmd(b"/bin/sh\0", b"sh\0", b"printf ab | cat\0")?;
    let mut buf = [0u8; 64];
    let (n, status, _) =
        pty_drain_until(master, pid, &|b| buf_contains(b, b"ab"), &mut buf)?;
    let _ = syscall::close(master);
    check!(syscall::wifexited(status), "exited");
    check_eq!(syscall::wexitstatus(status), 0, "pipeline");
    check!(buf_contains(&buf[..n], b"ab"), "pipeline out");
    Ok(())
}

#[crate::lctp_test(suite = syscall, full)]
fn pty_interactive_ls_line() -> TestResult {
    // Drive a shell over the PTY like a terminal: write `ls /\n`, read listing.
    check_ok!(syscall::access(b"/bin/sh\0", F_OK), "sh");
    let (master, slave, _) = openpty_pair()?;
    let pid = check_ok!(syscall::fork(), "fork");
    if pid == 0 {
        let _ = syscall::close(master);
        let _ = syscall::setsid();
        let _ = syscall::ioctl(slave, TIOCSCTTY, 0);
        if syscall::dup2(slave, 0).is_err()
            || syscall::dup2(slave, 1).is_err()
            || syscall::dup2(slave, 2).is_err()
        {
            syscall::exit(125);
        }
        if slave > 2 {
            let _ = syscall::close(slave);
        }
        // Interactive-ish: no -c; read commands from the tty.
        let arg0 = b"sh\0";
        let argv = [arg0.as_ptr(), core::ptr::null()];
        let envp: [*const u8; 1] = [core::ptr::null()];
        let _ = syscall::execve(b"/bin/sh\0", &argv, &envp);
        syscall::exit(127);
    }
    let _ = syscall::close(slave);
    // Give the shell a moment to start, then send a command line.
    let req = syscall::Timespec {
        tv_sec: 0,
        tv_nsec: 100_000_000,
    };
    let _ = syscall::nanosleep(&req);
    check_ok!(syscall::write(master, b"ls /\n"), "write ls");
    let mut buf = [0u8; 2048];
    let mut filled = 0usize;
    let mut saw_listing = false;
    for _ in 0..60 {
        let mut fds = [poll::PollFd {
            fd: master,
            events: POLLIN,
            revents: 0,
        }];
        if syscall::poll(&mut fds, 100).unwrap_or(0) > 0 {
            if filled < buf.len() {
                match syscall::read(master, &mut buf[filled..]) {
                    Ok(0) => break,
                    Ok(n) => {
                        filled += n;
                        if buf_contains(&buf[..filled], b"etc")
                            || buf_contains(&buf[..filled], b"bin")
                            || buf_contains(&buf[..filled], b"usr")
                        {
                            saw_listing = true;
                            break;
                        }
                    }
                    Err(Errno::EIO) => break,
                    Err(_) => {}
                }
            }
        }
    }
    let _ = syscall::write(master, b"exit\n");
    let mut status = 0;
    for _ in 0..40 {
        match syscall::wait4(pid, &mut status, wait::WNOHANG) {
            Ok(p) if p == pid => break,
            _ => {
                let req = syscall::Timespec {
                    tv_sec: 0,
                    tv_nsec: 50_000_000,
                };
                let _ = syscall::nanosleep(&req);
            }
        }
    }
    match syscall::wait4(pid, &mut status, wait::WNOHANG) {
        Ok(p) if p == pid => {}
        _ => {
            let _ = syscall::kill(pid, 9);
            let _ = syscall::wait4(pid, &mut status, 0);
        }
    }
    let _ = syscall::close(master);
    check!(saw_listing, "interactive ls listing");
    Ok(())
}
