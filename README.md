# Linux Container Test

Freestanding Rust (`no_std` / `no_main`) suite that verifies Linux container
runtimes by calling the kernel syscall ABI directly. One musl-static binary
covers the behaviours previously exercised via LTP, Open POSIX, and pjdfstest —
without Python, Perl, or a separate glibc build.

## Suites

| Suite | Flag | Role |
|-------|------|------|
| **bootstrap** | `--bootstrap` | Prerequisites for everything else. Always runs first; remaining suites are refused if it fails. |
| **syscall** | `--syscall` | Linux syscall behaviour (LTP-inspired, unprivileged only): files, process, memory, time, IPC, net, signals, inotify, pidfd, … |
| **posix** | `--posix` | POSIX path/open/errno/IO/signal/process semantics, plus freestanding `clone` thread tests (no libpthread) |
| **fs** | `--fs` | Filesystem semantics (pjdfstest-inspired): chmod, link, mkdir, mkfifo, open, rename, rmdir, symlink, truncate, unlink, utimensat, flock, statfs, sync, chown-EPERM |
Only tests that work in a **non-privileged** Docker container are included (~4184 cases in `--full`).

## Build

```bash
docker build -t linux-container-test:latest .
```

Multi-arch (`linux/amd64` + `linux/arm64`) images are published by CI.

## Run

```bash
# Quick pass, all suites (default)
docker run --rm linux-container-test:latest

# Full pass
docker run --rm linux-container-test:latest --full

# Selected suites (bootstrap still runs as a gate)
docker run --rm linux-container-test:latest --quick --syscall --fs

# List tests
docker run --rm linux-container-test:latest --list

docker run --rm linux-container-test:latest --help
```

### Modes

| Flag | Description |
|------|-------------|
| `-q`, `--quick` | Smoke pass (default); skips `full_only` cases |
| `-f`, `--full` | Include longer / fuller cases |
| `--no-color` | Disable ANSI colors on `[PASS]` / `[FAIL]` / `[SKIP]` |
## Why musl-only / no glibc image

Tests use raw syscalls, not libc wrappers, so glibc vs musl userspace differences
are out of scope. A single static binary (built for `*-linux-musl`) is enough.

## Layout

```
src/
  main.rs              _start + no_std entry
  syscall/             arch syscall ABI + wrappers
  runtime/             panic, print, mem builtins, path helpers
  harness/             CLI, asserts, runner, temp dirs, ALL_TESTS registry
  suites/
    bootstrap/         gate tests
    common.rs          shared helpers
    syscall/ … posix/ … fs/
lctp-macros/           #[lctp_test] proc-macro
```

Tests are registered with attributes instead of static arrays:

```rust
#[lctp_test(suite = fs)]
fn chmod_file_644() -> TestResult { ... }

#[lctp_test(suite = fs, full)]
fn chmod_file_777() -> TestResult { ... }
```

## Coverage notes

Privileged-only areas from the old LTP skip list (mount, reboot, modules,
fanotify-as-root, chown-to-other-uid success paths, etc.) are intentionally
omitted. Unprivileged equivalents and expected `EPERM`/`EACCES` failures are
covered where useful.
