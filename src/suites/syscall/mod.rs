//! Linux syscall behaviour tests (LTP-inspired, unprivileged-only).

mod file;
mod flock_statfs;
mod futex_basic;
mod ipc;
mod memfd;
mod memory;
mod misc;
mod mremap_msync;
mod net;
mod prctl_name;
mod process;
mod process_ids;
mod sched;
mod sendfile_io;
mod signal;
mod sigmask;
mod sockopt;
mod sync_ops;
mod sysinfo_rusage;
mod time;
mod timerfd;
mod waitid_test;
