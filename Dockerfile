FROM rust:1.38-slim

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

RUN set -ex \
  && mkdir src \
  && echo 'fn main() {}' >> src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

COPY src src

RUN set -ex \
  && cargo build --offline --release --target x86_64-unknown-linux-musl \
  && cp target/x86_64-unknown-linux-musl/release/f1-ext-install ./
