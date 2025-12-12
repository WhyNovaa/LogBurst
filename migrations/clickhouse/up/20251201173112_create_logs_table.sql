CREATE TABLE IF NOT EXISTS logs (
    timestamp DateTime64(3),
    level LowCardinality(String),
    service LowCardinality(String),
    message String CODEC(ZSTD(1)),
    raw_data String CODEC(ZSTD(1))
)
ENGINE = MergeTree()
ORDER BY timestamp;