FROM rust:1.91-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    cmake \
    libssl-dev \
    libcurl4-openssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release \
    -p ingest \
    -p writer \
    -p api

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

FROM runtime AS ingest
COPY --from=builder /app/target/release/ingest /usr/local/bin/ingest
ENTRYPOINT ["/usr/local/bin/ingest"]

FROM runtime AS writer
COPY --from=builder /app/target/release/writer /usr/local/bin/writer
ENTRYPOINT ["/usr/local/bin/writer"]

FROM runtime AS api
COPY --from=builder /app/target/release/api /usr/local/bin/api
ENTRYPOINT ["/usr/local/bin/api"]