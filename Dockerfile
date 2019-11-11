FROM rust:1.39-slim AS deps

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

RUN set -ex \
  && mkdir src \
  && echo 'fn main() {}' >> src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

COPY src src
RUN cargo build --offline --release --target x86_64-unknown-linux-musl

# Use scratch for delivery (makes it easier for people to download)
FROM scratch

COPY --from=deps /app/target/x86_64-unknown-linux-musl/release/f1-ext-install ./
