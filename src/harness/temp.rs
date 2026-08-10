//! Per-test temporary directory under `/tmp`.

use crate::runtime::{path_join, u64_to_dec, PidPath};
use crate::syscall::{self, oflag, AT_REMOVEDIR};

pub struct TempDir {
    root: [u8; 96],
    root_len: usize,
    scratch: [u8; 160],
}

impl TempDir {
    pub fn create() -> Result<Self, syscall::Errno> {
        // Unique path: /tmp/lctp-<pid>-<nsec>
        let mut root = [0u8; 96];
        let prefix = b"/tmp/lctp-";
        root[..prefix.len()].copy_from_slice(prefix);
        let mut i = prefix.len();
        let mut num = [0u8; 20];
        let n = u64_to_dec(syscall::getpid() as u64, &mut num);
        root[i..i + n].copy_from_slice(&num[..n]);
        i += n;
        root[i] = b'-';
        i += 1;
        let ts = syscall::clock_gettime(syscall::clock::CLOCK_MONOTONIC)
            .unwrap_or(syscall::Timespec::default());
        let n = u64_to_dec(ts.tv_nsec as u64, &mut num);
        root[i..i + n].copy_from_slice(&num[..n]);
        i += n;
        // Add a tiny uniqueness bump via getrandom if available.
        let mut b = [0u8; 2];
        if syscall::getrandom(&mut b, 0).is_ok() {
            root[i] = b'-';
            i += 1;
            let n = u64_to_dec(u16::from_le_bytes(b) as u64, &mut num);
            root[i..i + n].copy_from_slice(&num[..n]);
            i += n;
        }
        root[i] = 0;
        let root_len = i + 1;

        match syscall::mkdir(&root[..root_len], 0o700) {
            Ok(()) | Err(syscall::Errno::EEXIST) => {}
            Err(e) => return Err(e),
        }
        Ok(Self {
            root,
            root_len,
            scratch: [0u8; 160],
        })
    }

    pub fn path(&self) -> &[u8] {
        &self.root[..self.root_len]
    }

    pub fn child<'a>(&'a mut self, name: &[u8]) -> Result<&'a [u8], syscall::Errno> {
        let root_len = self.root_len;
        let mut root_copy = [0u8; 96];
        root_copy[..root_len].copy_from_slice(&self.root[..root_len]);
        path_join(&root_copy[..root_len], name, &mut self.scratch)
            .ok_or(syscall::Errno::ENAMETOOLONG)
    }

    pub fn create_file(&mut self, name: &[u8], mode: u32) -> Result<i32, syscall::Errno> {
        let p = self.child(name)?;
        syscall::open(p, oflag::O_RDWR | oflag::O_CREAT | oflag::O_TRUNC, mode)
    }

    pub fn remove_file(&mut self, name: &[u8]) -> Result<(), syscall::Errno> {
        let p = self.child(name)?;
        syscall::unlink(p)
    }

    pub fn cleanup(&mut self) {
        let _ = remove_tree(self.path());
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Best-effort recursive remove for our shallow test trees.
fn remove_tree(path: &[u8]) -> Result<(), syscall::Errno> {
    // Try rmdir first (empty).
    if syscall::rmdir(path).is_ok() {
        return Ok(());
    }
    let fd = match syscall::open(path, oflag::O_RDONLY | oflag::O_DIRECTORY, 0) {
        Ok(fd) => fd,
        Err(_) => {
            let _ = syscall::unlink(path);
            return Ok(());
        }
    };
    let mut buf = [0u8; 1024];
    loop {
        let n = match syscall::getdents64(fd, &mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let mut off = 0usize;
        while off + 19 <= n {
            // linux_dirent64: ino u64, off i64, reclen u16, type u8, name...
            let reclen = u16::from_le_bytes([buf[off + 16], buf[off + 17]]) as usize;
            if reclen == 0 || off + reclen > n {
                break;
            }
            let name_start = off + 19;
            let name_end = (name_start..off + reclen)
                .find(|&i| buf[i] == 0)
                .unwrap_or(off + reclen);
            let name = &buf[name_start..name_end];
            if name != b"." && name != b".." {
                let mut child = [0u8; 192];
                if let Some(c) = path_join(path, name, &mut child) {
                    let st = syscall::lstat(c);
                    if let Ok(st) = st {
                        if st.is_dir() {
                            let _ = remove_tree(c);
                        } else {
                            let _ = syscall::unlink(c);
                        }
                    } else {
                        let _ = syscall::unlink(c);
                        let _ = syscall::unlinkat(syscall::AT_FDCWD, c, AT_REMOVEDIR);
                    }
                }
            }
            off += reclen;
        }
    }
    let _ = syscall::close(fd);
    let _ = syscall::rmdir(path);
    Ok(())
}

// Silence unused PidPath import if we stopped using it.
#[allow(unused_imports)]
use PidPath as _;
