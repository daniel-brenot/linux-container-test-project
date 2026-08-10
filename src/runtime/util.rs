//! Small helpers used by tests (paths, parsing, comparisons).

/// Write an unsigned integer in decimal into `buf`. Returns byte length.
pub fn u64_to_dec(mut n: u64, buf: &mut [u8]) -> usize {
    if buf.is_empty() {
        return 0;
    }
    if n == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    let len = i;
    for j in 0..len {
        buf[j] = tmp[len - 1 - j];
    }
    len
}

/// Join `dir` (NUL-terminated) and `name` (without NUL) into `out`, NUL-terminating.
/// Returns the filled slice including NUL, or `None` if it does not fit.
pub fn path_join<'a>(dir: &[u8], name: &[u8], out: &'a mut [u8]) -> Option<&'a [u8]> {
    let dir = strip_nul(dir);
    let need = dir.len() + 1 + name.len() + 1;
    if out.len() < need {
        return None;
    }
    let mut i = 0;
    out[i..i + dir.len()].copy_from_slice(dir);
    i += dir.len();
    out[i] = b'/';
    i += 1;
    out[i..i + name.len()].copy_from_slice(name);
    i += name.len();
    out[i] = 0;
    Some(&out[..i + 1])
}

fn strip_nul(s: &[u8]) -> &[u8] {
    if let Some(0) = s.last() {
        &s[..s.len() - 1]
    } else {
        s
    }
}

/// Build `/tmp/lctp-<pid>` into a fixed buffer.
pub struct PidPath {
    pub buf: [u8; 64],
    pub len: usize,
}

impl PidPath {
    pub fn workdir() -> Self {
        let mut buf = [0u8; 64];
        let prefix = b"/tmp/lctp-";
        buf[..prefix.len()].copy_from_slice(prefix);
        let mut num = [0u8; 16];
        let nlen = u64_to_dec(crate::syscall::getpid() as u64, &mut num);
        let start = prefix.len();
        buf[start..start + nlen].copy_from_slice(&num[..nlen]);
        let len = start + nlen;
        buf[len] = 0;
        Self { buf, len: len + 1 }
    }

    pub fn as_cstr(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}
