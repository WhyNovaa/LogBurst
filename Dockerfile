FROM rust:1.91 as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

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

COPY --from=builder /usr/src/app/target/release/LogBurst .

CMD ["dockerize", "-wait", "tcp://pg:5432", "-timeout", "30s", "LogBurst"]

