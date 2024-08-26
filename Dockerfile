FROM debian:stable-20240812-slim

ENV DEBIAN_FRONTEND=noninteractive

ENV NO_COLOR=true

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.80.1

RUN set -eux; \
    apt update; \
    apt install -y pkg-config curl libasound2-dev libudev-dev build-essential cmake libfontconfig-dev; \
    rm -rf /var/lib/apt/lists/*; \
    curl -LO https://static.rust-lang.org/rustup/archive/1.27.1/x86_64-unknown-linux-gnu/rustup-init; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain 1.80.1 --default-host x86_64-unknown-linux-gnu; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;