FROM ubuntu:18.04

LABEL name="rust-musl"
LABEL version="0.1.0"
LABEL repository="https://github.com/kiron1/proxydetox"
LABEL homepage="https://github.com/kiron1/proxydetox"
LABEL maintainer="Kiron"

LABEL com.github.actions.name="Rust MUSL Builder"
LABEL com.github.actions.description="Provides a Rust MUSL environment"
LABEL com.github.actions.icon="settings"
LABEL com.github.actions.color="orange"

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates llvm-10 clang-10 musl musl-tools musl-dev \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    BUILD_TARGET=x86_64-unknown-linux-musl \
    LLVM_CONFIG_PATH=/usr/bin/llvm-config-10 \
    PATH=/usr/local/cargo/bin:$PATH

ADD https://sh.rustup.rs rustup.sh
RUN sh rustup.sh -y \
        --default-toolchain stable \
        --no-modify-path \
    && rustup toolchain add nightly \
    && rustup target add $BUILD_TARGET \
    && rustup --version \
    && cargo --version \
    && rustc --version

COPY entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
