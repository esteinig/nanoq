FROM rust:1.54 AS builder

COPY . /nanoq

WORKDIR /nanoq

ARG TARGET="x86_64-unknown-linux-musl"

RUN apt update \
    && apt install -y musl-tools \
    && rustup target add "$TARGET" \
    && cargo build --release --target "$TARGET" \
    && strip target/${TARGET}/release/nanoq


FROM bash:5.0

ARG TARGET="x86_64-unknown-linux-musl"
COPY --from=builder /nanoq/target/${TARGET}/release/nanoq /bin/

RUN nanoq --version

ENTRYPOINT [ "/bin/bash", "-l", "-c" ]