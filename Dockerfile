# Stage 1: Build
FROM rust:1.86-slim AS builder

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git

RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
RUN cargo build --release

# Stage 2: Runtime (libssl3 included)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
  libssl3 \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/teleflow /usr/local/bin/teleflow

ENTRYPOINT ["teleflow"]
