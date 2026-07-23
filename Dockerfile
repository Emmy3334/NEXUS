# syntax=docker/dockerfile:1
#
# Multi-stage image for NEXUS. Final stage is non-root distroless (glibc).
# Fully static (`distroless/static`) is not used: wasmtime/cranelift and several
# native deps link against glibc / libgcc — see doc/security.md.

FROM rust:1.89-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
RUN cargo build --release --locked --bin nexus \
    && strip -s target/release/nexus

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /src/target/release/nexus /nexus
USER nonroot:nonroot
ENTRYPOINT ["/nexus"]
