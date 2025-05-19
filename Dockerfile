FROM rust:latest

WORKDIR /app

COPY . .

ADD https://github.com/jwilder/dockerize/releases/download/v0.6.1/dockerize-linux-amd64-v0.6.1.tar.gz .
RUN tar -C /usr/local/bin -xzvf dockerize-linux-amd64-v0.6.1.tar.gz

RUN cargo build --release

CMD ["dockerize", "-wait", "tcp://clickhouse-server:8123", "-wait", "tcp://pg:5432", "-timeout", "30s", "./target/release/LogBurst"]