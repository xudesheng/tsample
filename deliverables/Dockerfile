FROM rust:latest AS build
LABEL maintainer="Desheng Xu"

#
# -- Install Rust build toolchain
#
ENV RUSTFLAGS="-C target-feature=+crt-static"
WORKDIR /build/tsample
CMD ["cargo","build","--release"]
