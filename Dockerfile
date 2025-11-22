# ==========================
# 1. Stage: Build
# ==========================
FROM rust:1.91 as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

COPY . .

RUN cargo build --release


FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ADD https://github.com/jwilder/dockerize/releases/download/v0.6.1/dockerize-linux-amd64-v0.6.1.tar.gz .
RUN tar -C /usr/local/bin -xzvf dockerize-linux-amd64-v0.6.1.tar.gz

WORKDIR /usr/local/bin

# Копируем бинарник из стадии сборки
COPY --from=builder /usr/src/app/target/release/LogBurst .

# CMD ["dockerize", "-wait", "tcp://clickhouse-server:8123", "-wait", "tcp://pg:5432", "-timeout", "30s", "./target/release/LogBurst"]
CMD ["dockerize", "-wait", "tcp://pg:5432", "-timeout", "30s", "LogBurst"]

