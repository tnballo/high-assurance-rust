# aarch64 (Apple silicon, etc)
FROM arm64v8/rust:1.59-slim

# Non-Rust tooling
ENV TZ=US/New_York
RUN apt-get update -y
RUN DEBIAN_FRONTEND="noninteractive" apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    tree \
    xxd \
    git \
    vim

# Rust tooling
RUN rustup toolchain install nightly
RUN rustup component add llvm-tools-preview
RUN cargo install mdbook
RUN cargo install cargo-fuzz
RUN cargo install cargo-binutils
RUN cargo install cargo-modules
RUN cargo install cargo-audit

# Src import
RUN mkdir /code_snippets
WORKDIR /code_snippets
COPY ./code_snippets /code_snippets/