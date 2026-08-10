# LogBurst

<div align="center">

**High-throughput log ingestion and realtime analytics for modern services**

<p>
  <img src="https://img.shields.io/badge/Rust-1.91%2B-000000?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Axum-REST-1f6feb?style=flat-square" alt="Axum">
  <img src="https://img.shields.io/badge/gRPC-streaming-0f766e?style=flat-square" alt="gRPC">
  <img src="https://img.shields.io/badge/ClickHouse-analytics-ffcc01?style=flat-square&logo=clickhouse&logoColor=000" alt="ClickHouse">
  <img src="https://img.shields.io/badge/PostgreSQL-auth-336791?style=flat-square&logo=postgresql&logoColor=fff" alt="PostgreSQL">
  <img src="https://img.shields.io/badge/React-19-61dafb?style=flat-square&logo=react&logoColor=000" alt="React">
  <img src="https://img.shields.io/badge/Vite-7-646cff?style=flat-square&logo=vite&logoColor=fff" alt="Vite">
</p>

<p>
  LogBurst is a Rust-based logging platform that accepts events over REST and gRPC,
  batches them into ClickHouse, secures analytics with JWT, and exposes a polished
  dashboard for live and historical log exploration.
</p>

</div>

## Table of Contents

- [Overview](#overview)
- [Why LogBurst](#why-logburst)
- [Architecture](#architecture)
- [Core Capabilities](#core-capabilities)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Security Model](#security-model)
- [API Overview](#api-overview)
- [gRPC Contract](#grpc-contract)
- [Frontend](#frontend)
- [Repository Layout](#repository-layout)
- [Performance Characteristics](#performance-characteristics)
- [Development Workflow](#development-workflow)
- [Load Testing](#load-testing)
- [Operational Notes](#operational-notes)

## Overview

LogBurst is designed for teams that want a dedicated log pipeline with a clean split between:

- ingestion,
- storage,
- authentication,
- analytics,
- and operator-facing UI.

It combines a high-performance Rust backend with ClickHouse for log storage and aggregation, PostgreSQL for user authentication, and a React dashboard for realtime visibility.

The current implementation already includes:

- REST ingestion with request signing,
- gRPC client-streaming ingestion,
- batched writes to ClickHouse,
- JWT-protected analytics endpoints,
- server-sent events for live and historical log streaming,
- a dashboard with overview metrics, interval charts, live logs, and filtered search.
<img width="1604" height="815" alt="image" src="https://github.com/user-attachments/assets/b0f2eb90-dc34-45d2-96f8-9c3c9b3c344f" />
<img width="1608" height="817" alt="image" src="https://github.com/user-attachments/assets/efd9321a-35b6-43f9-9a08-690a2da8cdfa" />

## Why LogBurst

Most internal logging tools break down in one of two ways: they are easy to ingest into but hard to explore, or easy to explore but expensive to keep fast under load. LogBurst is built around a simpler model:

- keep ingestion fast and asynchronous,
- keep storage optimized for analytics,
- keep security explicit,
- keep the operator experience lightweight and responsive.

This repository is a strong foundation for:

- internal platform logging,
- service diagnostics dashboards,
- observability experiments with Rust and ClickHouse,
- custom logging products where you want full control of the pipeline.

## Architecture

LogBurst is organized into four clear layers:

- producers send logs over REST or gRPC;
- the Rust backend validates, queues, and batches events;
- ClickHouse stores logs and powers analytics, while PostgreSQL handles authentication;
- the React dashboard consumes analytics over REST and logs over SSE.

```text
+---------------------------+         +--------------------------------------+
|       Log Producers       |         |         React Dashboard              |
|---------------------------|         |--------------------------------------|
| Service A                 |         | Overview metrics                     |
| Service B                 |         | Interval chart                       |
| Service N                 |         | Live Logs                            |
+-------------+-------------+         | Log Explorer                         |
              |                       +------------------+-------------------+
              | REST / gRPC                              |
              v                                          | JWT + REST + SSE
+--------------------------------------------------------v-------------------+
|                           LogBurst Backend                                 |
|----------------------------------------------------------------------------|
| REST ingestion (/v1/logs)                                                  |
|   -> BLAKE3 signature guard                                                |
|   -> timestamp freshness check                                             |
|                                                                            |
| gRPC ingestion (SendLogs stream)                                           |
|                                                                            |
| Both paths feed:                                                           |
|   -> async ingestion queue                                                 |
|   -> batch inserter                                                        |
|   -> analytics API (REST + SSE)                                            |
|                                                                            |
| Auth path:                                                                 |
|   -> /v1/auth                                                              |
|   -> JWT issuance                                                          |
+-------------------------------+-----------------------------+--------------+
                                |                             |
                                | writes / reads              | auth lookup
                                v                             v
                  +--------------------------+      +----------------------+
                  |        ClickHouse        |      |      PostgreSQL      |
                  |--------------------------|      |----------------------|
                  | logs table               |      | users table          |
                  | aggregates and queries   |      | password hashes      |
                  +--------------------------+      +----------------------+

                  +---------------------------------------------------------+
                  | keys.toml                                                |
                  | hot-reloaded service keys for REST signature validation  |
                  +---------------------------------------------------------+
```

### Request flow

1. A producer sends a log over REST or gRPC.
2. REST requests pass through a keyed `BLAKE3` signature check and timestamp freshness check.
3. Logs are pushed into an async ingestion queue.
4. A background inserter writes batches into ClickHouse.
5. Analytics endpoints query ClickHouse directly.
6. The frontend consumes metrics over REST and logs over SSE.

## Core Capabilities

| Capability | Implementation |
| --- | --- |
| Secure REST ingestion | `POST /v1/logs` with `X-SERVICE-ID` and `X-SIGNATURE` |
| Streaming gRPC ingestion | `SendLogs(stream LogEntry)` |
| High-throughput writes | batched ClickHouse inserter with size- and time-based flush |
| Authenticated analytics | JWT issued by `POST /v1/auth` |
| Live log tail | SSE stream from `GET /v1/analytics/last` |
| Historical log search | SSE stream from `GET /v1/analytics/logs/interval` |
| Level analytics | total counts and interval buckets |
| Rotating service keys | `keys.toml` reloaded automatically every 60 seconds |

## Quick Start

### Prerequisites

Backend:

- Rust `1.91+`
- `protobuf-compiler`
- OpenSSL development headers
- PostgreSQL
- ClickHouse

Frontend:

- Node.js `20+`
- npm

On Debian or Ubuntu:

```bash
sudo apt-get install pkg-config libssl-dev protobuf-compiler
```

### Option 1: Run the backend stack with Docker Compose

This is the fastest way to get the backend, PostgreSQL, and ClickHouse running.

#### 1. Create `.env`

For running the Rust service on your host:

```dotenv
POSTGRES_HOST=localhost
POSTGRES_USER=user
POSTGRES_PASSWORD=password
POSTGRES_DB=pg_db

SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_SECRET_KEY=change-me-to-a-long-random-secret

CLICKHOUSE_HOST=http://localhost
CLICKHOUSE_PORT=8123
CLICKHOUSE_USER=admin
CLICKHOUSE_PASSWORD=admin

GRPC_HOST=0.0.0.0
GRPC_PORT=50051
```

If the Rust service runs inside `docker compose`, use container hostnames instead:

```dotenv
POSTGRES_HOST=pg
CLICKHOUSE_HOST=http://clickhouse-server
```

`CLICKHOUSE_HOST` should include the protocol, for example `http://localhost` or `http://clickhouse-server`.

#### 2. Create `keys.toml`

```toml
service-a = "0123456789abcdef0123456789abcdef"
service-b = "fedcba9876543210fedcba9876543210"
```

Important:

- each key must be exactly `32` ASCII characters;
- the code uses the raw string bytes directly;
- keys are reloaded automatically every 60 seconds without restarting the service.

#### 3. Start the stack

```bash
docker compose up --build
```

Available services:

- REST API: `http://localhost:8080`
- gRPC: `localhost:50051`
- PostgreSQL: `localhost:5432`
- ClickHouse HTTP: `http://localhost:8123`
- ClickHouse native: `localhost:9000`

### Option 2: Hybrid local development

This is the most practical setup when working on the frontend.

1. Start only the databases:

```bash
docker compose up pg clickhouse -d
```

2. Run the Rust backend locally:

```bash
cargo run
```

3. Run the frontend in a separate shell:

```bash
cd frontend
npm install
npm run dev
```

Typical local endpoints:

- backend: `http://localhost:8080`
- frontend: `http://localhost:5173`

The Vite dev server proxies `/v1/*` to the backend automatically.

## Configuration

### Environment variables

| Variable | Example | Purpose |
| --- | --- | --- |
| `POSTGRES_HOST` | `localhost` | PostgreSQL host |
| `POSTGRES_USER` | `user` | PostgreSQL user |
| `POSTGRES_PASSWORD` | `password` | PostgreSQL password |
| `POSTGRES_DB` | `pg_db` | database containing the `users` table |
| `SERVER_HOST` | `0.0.0.0` | REST bind host |
| `SERVER_PORT` | `8080` | REST port |
| `SERVER_SECRET_KEY` | `very-secret-key` | JWT signing secret |
| `CLICKHOUSE_HOST` | `http://localhost` | ClickHouse base URL |
| `CLICKHOUSE_PORT` | `8123` | ClickHouse HTTP port |
| `CLICKHOUSE_USER` | `admin` | ClickHouse user |
| `CLICKHOUSE_PASSWORD` | `admin` | ClickHouse password |
| `GRPC_HOST` | `0.0.0.0` | gRPC bind host |
| `GRPC_PORT` | `50051` | gRPC port |

### Data model created by migrations

PostgreSQL:

- creates a `users` table,
- inserts a seeded `user` row with an Argon2 hash.

ClickHouse:

- creates a `logs` table with:
  - `timestamp DateTime64(3, 'UTC')`
  - `level LowCardinality(String)`
  - `service LowCardinality(String)`
  - `message String`
  - `raw_data String`
- uses `MergeTree`,
- partitions by `toYearWeek(timestamp)`,
- orders by `timestamp`,
- applies TTL deletion after `1 month`.

## Security Model

### REST ingestion protection

`POST /v1/logs` is protected by two headers:

- `X-SERVICE-ID`
- `X-SIGNATURE`

The server:

- looks up the service key from `keys.toml`,
- computes a keyed `BLAKE3` hash of the raw request body,
- compares it to `X-SIGNATURE`,
- checks that the JSON body includes a recent Unix `timestamp`,
- rejects payloads older than `120` seconds.

Signature model:

```text
signature = BLAKE3(key = service_key_32_bytes, data = raw_request_body).to_hex()
```

### JWT-protected analytics

Analytics endpoints require:

```http
Authorization: Bearer <jwt>
```

Current token lifetime: `3600 seconds`.

## API Overview

### Authentication

#### `POST /v1/auth`

Request:

```http
POST /v1/auth
Content-Type: application/json

{
  "username": "user",
  "password": "your-password"
}
```

Response:

```json
"<jwt-token>"
```

### REST ingestion

#### `POST /v1/logs`

Expected payload fields:

- `level` with values typically sent as `info`, `warn`, or `error`
- `service`
- `message`
- `timestamp` as Unix seconds for anti-replay validation

Example:

```http
POST /v1/logs
Content-Type: application/json
X-SERVICE-ID: service-a
X-SIGNATURE: <blake3-keyed-hex>

{
  "timestamp": 1770151080,
  "level": "error",
  "service": "billing-api",
  "message": "payment gateway timeout",
  "request_id": "af5c3f2f-8d4d-4ef9-bfd8-c6f4d9a16051",
  "region": "eu-central-1"
}
```

Implementation detail that matters:

- for REST ingestion, the stored `timestamp` column uses the server receive time;
- the original client payload remains in `raw_data`;
- the entire raw body is signed, so whitespace and field ordering affect the signature.

### Analytics endpoints

#### `GET /v1/analytics/levels/count`

Returns total counts by log level.

Example response:

```json
{
  "error_count": 128,
  "warn_count": 764,
  "info_count": 14392
}
```

#### `GET /v1/analytics/levels/intervals`

Returns hourly buckets between `from` and `to`.

Example:

```http
GET /v1/analytics/levels/intervals?from=2026-04-03T00:00:00Z&to=2026-04-03T12:00:00Z
Authorization: Bearer <jwt>
```

Example response:

```json
[
  {
    "time_bucket": "2026-04-03T10:00:00Z",
    "error_count": 5,
    "warn_count": 12,
    "info_count": 87
  }
]
```

#### `GET /v1/analytics/services`

Returns distinct service names.

```json
["billing-api", "gateway", "worker"]
```

#### `GET /v1/analytics/logs/interval`

Streams historical logs over `text/event-stream`.

Query parameters:

- `from` as RFC3339 datetime
- `to` as RFC3339 datetime
- `service` optional
- `level` optional
- `limit` with a server-side maximum of `5000`

Example:

```http
GET /v1/analytics/logs/interval?from=2026-04-03T09:00:00Z&to=2026-04-03T10:00:00Z&service=billing-api&level=error&limit=500
Authorization: Bearer <jwt>
Accept: text/event-stream
```

Event format:

```text
data: {"timestamp":"2026-04-03T09:15:12.123Z","level":"error","service":"billing-api","message":"timeout","raw_data":"{\"request_id\":\"...\"}"}
```

#### `GET /v1/analytics/last`

Streams live logs over SSE for the dashboard tail view.

Current behavior:

- this is not a guaranteed full-firehose tail;
- live events are sampled before being sent to the broadcast channel;
- the current implementation emits roughly `1%` of incoming logs.

## gRPC Contract

The protobuf definition lives at `src/interfaces/grpc/proto/log.proto`.

Service:

```proto
service LogCollector {
    rpc SendLogs (stream LogEntry) returns (LogResponse);
}
```

Messages:

```proto
message LogEntry {
    google.protobuf.Timestamp timestamp = 1;
    LogLevel level = 2;
    string service = 3;
    string message = 4;
}
```

```proto
enum LogLevel {
    INFO = 0;
    WARN = 1;
    ERROR = 2;
}
```

Behavior:

- if the gRPC timestamp is valid, it is written into the main `timestamp` column;
- if the timestamp is missing or invalid, the server falls back to current UTC time.

## Frontend

The frontend lives in `frontend/` and is a separate React application built with Vite.

Current UI capabilities:

- login screen,
- protected dashboard route,
- overview cards for error, warning, and info totals,
- stacked interval chart,
- terminal-style live log panel,
- filterable historical log explorer,
- service selector loaded from the backend,
- automatic redirect to login on `401`.

Implementation notes:

- REST requests use Axios,
- JWT is stored in `localStorage`,
- SSE is consumed through `fetch` and `ReadableStream`,
- `/v1/*` is proxied to `http://localhost:8080` in local development.

## Repository Layout

```text
.
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── build.rs
├── script.js
├── migrations/
│   ├── clickhouse/
│   └── pg/
├── src/
│   ├── config/              # env-driven configuration
│   ├── db/
│   │   ├── clickhouse/      # ingestion and analytics queries
│   │   └── pg/              # authentication queries
│   ├── interfaces/
│   │   ├── api/             # REST routes, DTOs, middleware
│   │   └── grpc/            # protobuf contract and server
│   ├── security/            # key store and reload loop
│   ├── main.rs
│   └── server.rs
└── frontend/
    ├── src/
    └── package.json
```

## Performance Characteristics

Important backend constants in the current implementation:

| Setting | Value |
| --- | --- |
| Ingestion queue size | `200_000` |
| Live broadcast channel size | `15` |
| Batch commit threshold | `50_000` rows |
| Forced flush interval | `5 seconds` |
| Historical query limit | capped at `5000` |
| PostgreSQL pool size | `16` |

What this means in practice:

- ingestion is decoupled from writes,
- the storage path is optimized for batch throughput,
- analytics reads are direct and simple,
- the live widget is intentionally lightweight.

## Development Workflow

Run the backend:

```bash
cargo run
```

Build the backend:

```bash
cargo build --release
```

Run the frontend:

```bash
cd frontend
npm install
npm run dev
```

Build the frontend:

```bash
cd frontend
npm run build
```

Lint the frontend:

```bash
cd frontend
npm run lint
```

## Load Testing

The repository includes a `k6` script at `script.js`.

It currently:

- ramps up to `1000` virtual users over `15s`,
- ramps back down over `15s`,
- targets fewer than `1%` failed requests,
- targets `p95 < 500ms`.

Run it with:

```bash
k6 run script.js
```

Before running load tests, verify:

- the target URL,
- `X-SERVICE-ID`,
- `X-SIGNATURE`,
- and the key in `keys.toml`.

## Operational Notes

### Docker Compose does not currently include the frontend

`docker-compose.yml` starts the backend and databases. The dashboard is intended to run separately during development unless you add your own production frontend build pipeline.

### Historical logs are streamed, not returned as a JSON array

`GET /v1/analytics/logs/interval` is an SSE endpoint. Clients must consume it as `text/event-stream`.

### Live logs are sampled

The live dashboard view is a realtime indicator, not a complete mirror of all ingested traffic.

### The seeded PostgreSQL user is not a full onboarding flow

The repository creates a `user` row through migrations, but production setups should create explicit users and manage credentials intentionally.

---

LogBurst already has the shape of a real logging platform: secure ingestion, analytics-friendly storage, authenticated access, and a dedicated UI for operators. If you want a compact Rust codebase that still covers the full log pipeline end to end, this repository is a strong place to build from.
