# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: MIT

FROM rust:1 as rust

# We don't want warnings for no reason
ARG DEBIAN_FRONTEND=noninteractive

# Install dev-tooling
ENV TOOL_DEPS "curl wget telnet netcat net-tools dnsutils vim-tiny bash-completion sudo jq"
ENV COMPILER_DEPS "gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf gcc-arm-linux-gnueabi"
RUN apt-get update \
    && apt-get upgrade -yqq \
    && apt-get install -yqq \
    ${TOOL_DEPS} ${COMPILER_DEPS} \
    && rm -rf /var/lib/apt/*

# Add our rust utils and nightly toolchain.
RUN rustup component add rust-std \
    && rustup component add rustfmt \
    && rustup component add clippy \
    && rustup component add rust-src \
    && rustup target add x86_64-unknown-linux-gnu \
    && rustup target add aarch64-unknown-linux-gnu \
    && rustup target add armv7-unknown-linux-gnueabihf \
    && rustup target add armv7-unknown-linux-gnueabi \
    && rustup target add arm-unknown-linux-gnueabihf \
    && rustup target add arm-unknown-linux-gnueabi \
    # TODO(csaide): Disabled for now need to fix cross platform builds.
    #
    # && rustup target add x86_64-pc-windows-msvc \
    # && rustup target add aarch64-pc-windows-msvc \
    # && rustup target add x86_64-apple-darwin \
    # && rustup target add aarch64-apple-darwin \
    # && rustup target add x86_64-unknown-freebsd \
    # && rustup target add aarch64-unknown-freebsd \
    # && rustup target add x86_64-unknown-openbsd \
    # && rustup target add aarch64-unknown-openbsd \
    # && rustup target add x86_64-unknown-netbsd \
    # && rustup target add aarch64-unknown-netbsd \
    && cargo install cargo-tarpaulin \
    && rustup update

# Install rust-analyzer
RUN curl -L https://github.com/rust-analyzer/rust-analyzer/releases/latest/download/rust-analyzer-linux -o /usr/bin/rust-analyzer \
    && chmod +x /usr/bin/rust-analyzer

# Add appropriate user ID.
ARG uid=1000
RUN useradd -s /bin/bash -d /home/code -u ${uid} -U -G sudo code \
    && mkdir -p /home/code /opt/riptun \
    && chown -R code:code /home/code /usr/local/cargo /opt/riptun \
    && echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

# Copy in a custom bashrc with util functions.
COPY --chown=code:code dist/docker/development/.bashrc /home/code/

WORKDIR /opt/riptun
