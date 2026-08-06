# Linux Container Test Project — musl (Alpine) image
# Combines LTP, Open POSIX Test Suite, and pjdfstest behind a single CLI.
#
# Build:
#   docker build -f Dockerfile -t linux-container-test:latest-musl .
#
# Run:
#   docker run --rm linux-container-test:latest-musl
#   docker run --rm linux-container-test:latest-musl -h
#
# See Dockerfile.glibc for the Ubuntu/glibc variant.

ARG ALPINE_VERSION=3.20
ARG LTP_REF=master
ARG PJDFSTEST_REF=master
ARG LTPROOT=/opt/ltp
ARG PJDFSTEST_ROOT=/opt/pjdfstest

FROM alpine:${ALPINE_VERSION} AS build
ARG LTP_REF
ARG PJDFSTEST_REF
ARG LTPROOT
ARG PJDFSTEST_ROOT

RUN apk add --no-cache \
    acl-dev \
    autoconf \
    automake \
    clang \
    curl \
    e2fsprogs-extra \
    gcc \
    git \
    jq \
    keyutils-dev \
    libaio-dev \
    libcap-dev \
    libselinux-dev \
    libsepol-dev \
    libtirpc-dev \
    linux-headers \
    make \
    musl-dev \
    numactl-dev \
    openssl-dev \
    pkgconfig \
    python3

# --- Linux Test Project (includes Open POSIX Test Suite for in-tree builds) ---
WORKDIR /build/ltp
# kirk is a submodule (tools/kirk/kirk-src); without it, make install skips kirk.
RUN git clone --depth 1 --recurse-submodules --shallow-submodules \
      --branch "${LTP_REF}" \
      https://github.com/linux-test-project/ltp.git .

# Alpine/musl cannot build a few cases yet (upstream ci/alpine.sh + off64_t gaps).
RUN rm -rfv \
      testcases/kernel/syscalls/fmtmsg/fmtmsg01.c \
      testcases/kernel/syscalls/timer_create/timer_create01.c \
      testcases/kernel/syscalls/timer_create/timer_create03.c \
      testcases/kernel/mem/hugetlb/hugemmap/hugemmap36.c

# In-tree build enables --with-open-posix-testsuite by default via build.sh.
# _LARGEFILE64_SOURCE helps remaining LFS64 typedefs on older musl headers.
RUN CPPFLAGS="${CPPFLAGS:-} -D_LARGEFILE64_SOURCE" \
    ./build.sh -p "${LTPROOT}" -i \
    && test -x "${LTPROOT}/kirk" \
    && test -d "${LTPROOT}/testcases/open_posix_testsuite"

# --- pjdfstest ---
WORKDIR /build/pjdfstest
RUN git clone --depth 1 --branch "${PJDFSTEST_REF}" \
      https://github.com/pjd/pjdfstest.git . \
    && autoreconf -ifs \
    && ./configure \
    && make pjdfstest \
    && mkdir -p "${PJDFSTEST_ROOT}" \
    && cp -a pjdfstest tests "${PJDFSTEST_ROOT}/" \
    && test -x "${PJDFSTEST_ROOT}/pjdfstest"

FROM alpine:${ALPINE_VERSION}
ARG LTPROOT
ARG PJDFSTEST_ROOT

RUN apk add --no-cache \
    acl \
    curl \
    jq \
    keyutils \
    libaio \
    libacl \
    libcap \
    libselinux \
    libsepol \
    libtirpc \
    make \
    numactl \
    openssl \
    perl \
    perl-test-harness \
    perl-test-harness-utils \
    py3-msgpack \
    python3 \
    && adduser -D -g "Unprivileged LTP user" ltp

COPY --from=build ${LTPROOT} ${LTPROOT}
COPY --from=build ${PJDFSTEST_ROOT} ${PJDFSTEST_ROOT}
COPY skipfiles/musl/ltp-unprivileged.skip ${LTPROOT}/docker-unprivileged.skip
COPY skipfiles/musl/open-posix.skip ${LTPROOT}/docker-posix.skip
COPY container-test.sh /usr/local/bin/container-test

RUN chmod 755 /usr/local/bin/container-test \
    && ln -sf /usr/local/bin/container-test /usr/local/bin/container-test.sh

ENV LTPROOT=${LTPROOT}
ENV PJDFSTEST_ROOT=${PJDFSTEST_ROOT}
ENV OPEN_POSIX_ROOT=${LTPROOT}/testcases/open_posix_testsuite
ENV CONTAINER_TEST_LIBC=musl
ENV PATH=${PJDFSTEST_ROOT}:${LTPROOT}/testcases/bin:${LTPROOT}/bin:${LTPROOT}:/usr/local/bin:${PATH}

WORKDIR ${LTPROOT}

# Default: quick pass across LTP, Open POSIX, and pjdfstest.
ENTRYPOINT ["/usr/local/bin/container-test"]
CMD ["--quick"]
