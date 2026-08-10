# Linux Container Test — freestanding Rust (no_std) suite
#
# Build:
#   docker build -t linux-container-test:latest .
#
# Run:
#   docker run --rm linux-container-test:latest
#   docker run --rm linux-container-test:latest --full
#   docker run --rm linux-container-test:latest --help

ARG ALPINE_VERSION=3.20
ARG RUST_VERSION=1.85.0

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS build
ARG TARGETARCH

RUN apk add --no-cache musl-dev

WORKDIR /src
COPY Cargo.toml ./
COPY lctp-macros ./lctp-macros
COPY .cargo ./.cargo
COPY src ./src

RUN case "${TARGETARCH}" in \
      amd64) echo x86_64-unknown-linux-musl > /tmp/target ;; \
      arm64) echo aarch64-unknown-linux-musl > /tmp/target ;; \
      *) echo "unsupported arch: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
 && rustup target add "$(cat /tmp/target)" \
 && cargo build --release --target "$(cat /tmp/target)" \
 && cp "target/$(cat /tmp/target)/release/linux-container-test" /linux-container-test \
 && chmod 755 /linux-container-test

FROM alpine:${ALPINE_VERSION}
RUN adduser -D -g "unprivileged test user" tester \
 && mkdir -p /tmp && chmod 1777 /tmp

COPY --from=build /linux-container-test /usr/local/bin/linux-container-test

USER tester
WORKDIR /home/tester

ENTRYPOINT ["/usr/local/bin/linux-container-test"]
CMD ["--quick"]
