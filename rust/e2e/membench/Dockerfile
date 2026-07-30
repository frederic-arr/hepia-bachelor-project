FROM docker.io/rustlang/rust:nightly-slim-2026-07-30@sha256:028a33d644a64210575e24b7b377a3fb0c0bbdf348950c2cd9ba8fa9eefea2fd AS builder
WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl
COPY . .
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/membench /membench
ENTRYPOINT ["/membench"]
