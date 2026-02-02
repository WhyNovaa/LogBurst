CREATE TABLE IF NOT EXISTS logs (
    timestamp DateTime64(3),
    level LowCardinality(String),
    service LowCardinality(String),
    message String CODEC(ZSTD(1)),
    raw_data String CODEC(ZSTD(1))
)
ENGINE = MergeTree()
PARTITION BY toYearWeek(timestamp)
ORDER BY timestamp
TTL timestamp + INTERVAL 1 MONTH DELETE;