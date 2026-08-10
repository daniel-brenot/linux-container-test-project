//! Filesystem semantics tests (pjdfstest-inspired, unprivileged-only).

mod access;
mod chmod;
mod chown;
mod fallocate_fs;
mod flock;
mod link;
mod mkdir;
mod mkfifo;
mod open;
mod pjdfstest_depth2;
mod pjdfstest_depth3;
mod rename;
mod rmdir;
mod stat;
mod statfs;
mod symlink;
mod sync;
mod timestamps;
mod truncate;
mod unlink;
mod utimensat;
