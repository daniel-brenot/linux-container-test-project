# Linux Container Test Project

This repository contains the code for a project meant to verify a container runtimes ability
to run linux programs. It combines multiple different testing projects into one single test container,
and allows the user to invoke some or all of the tests for either quick verification or full comprehensive testing.

This project uses three testing projects:

**Linux Test Project:** Testing most of the linux syscalls
**POSIX Test Suite:** Testing userspace/posix functionality
**pjdfstest:** Tests filesystem semantics

## Usage

Two images are provided so you can exercise either musl or glibc userspace.
Published tags are multi-arch (`linux/amd64` and `linux/arm64`), so the same tag works on both.

| Dockerfile | Libc | Local tag | Published tag |
|------------|------|-----------|---------------|
| `Dockerfile` | musl (Alpine) | `linux-container-test:latest-musl` | `<dockerhub-user>/linux-container-test:latest-musl` |
| `Dockerfile.glibc` | glibc (Ubuntu) | `linux-container-test:latest-glibc` | `<dockerhub-user>/linux-container-test:latest-glibc` |

### Build

```bash
# musl / Alpine (current machine arch)
docker build -f Dockerfile -t linux-container-test:latest-musl .

# glibc / Ubuntu (current machine arch)
docker build -f Dockerfile.glibc -t linux-container-test:latest-glibc .
```

Published images from CI are multi-arch (`linux/amd64` + `linux/arm64`). To build and push a multi-arch image yourself:

```bash
docker buildx build --platform linux/amd64,linux/arm64 \
  -f Dockerfile -t <dockerhub-user>/linux-container-test:latest-musl --push .
```

### Run

The image entrypoint is `container-test`. With no arguments it runs a quick pass of all suites:

```bash
docker run --rm linux-container-test:latest-musl
docker run --rm linux-container-test:latest-glibc
```

Show help:

```bash
docker run --rm linux-container-test:latest-musl -h
```

### Modes

| Flag | Description |
|------|-------------|
| `-q`, `--quick` | Quick smoke pass (default) |
| `-f`, `--full` | Full comprehensive suites |

### Suites

If no suite flags are given, all suites run.

| Flag | Suite |
|------|-------|
| `--ltp` | Linux Test Project |
| `--posix` | Open POSIX Test Suite |
| `--pjdfstest` | pjdfstest filesystem suite |

### Examples

```bash
# Quick pass, all suites (default) — musl
docker run --rm linux-container-test:latest-musl

# Quick pass, LTP only — glibc
docker run --rm linux-container-test:latest-glibc --quick --ltp

# Quick pass, POSIX and pjdfstest
docker run --rm linux-container-test:latest-musl --posix --pjdfstest

# Full pass, all suites
docker run --rm linux-container-test:latest-glibc --full

# Full LTP syscalls (may need privileged for fewer skips)
docker run --rm --privileged linux-container-test:latest-glibc --full --ltp
```

### Publishing

On pushes to `main`/`master` (and via `workflow_dispatch`), GitHub Actions builds each image on native `amd64` and `arm64` runners, then publishes multi-arch manifests to Docker Hub:

- `<dockerhub-user>/linux-container-test:latest-musl` (`linux/amd64`, `linux/arm64`)
- `<dockerhub-user>/linux-container-test:latest-glibc` (`linux/amd64`, `linux/arm64`)

On an ARM host, `docker pull` / `docker run` of those tags uses the arm64 variant automatically.

Add these repository secrets before the workflow can publish:

| Secret | Value |
|--------|-------|
| `DOCKERHUB_USERNAME` | Docker Hub username |
| `DOCKERHUB_TOKEN` | Docker Hub access token |

Optional repository variable `DOCKERHUB_REPOSITORY` overrides the image name (default: `linux-container-test`).

See [`.github/workflows/publish-images.yml`](.github/workflows/publish-images.yml).

### Quick vs full

| Suite | Quick | Full |
|-------|-------|------|
| LTP | `smoketest` | `syscalls` |
| Open POSIX | `SIG` option group | all option groups |
| pjdfstest | `tests/chmod` | entire `tests/` |

