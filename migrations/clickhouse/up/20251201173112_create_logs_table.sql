CREATE TABLE IF NOT EXISTS logs (
    timestamp DateTime64(3 , 'UTC'),
    level LowCardinality(String),
    service LowCardinality(String),
    message String CODEC(ZSTD(1)),
    raw_data String CODEC(ZSTD(1)),

    kafka_partition Int32,
    kafka_offset Int64
)
ENGINE = ReplacingMergeTree()
PARTITION BY toYearWeek(timestamp)
ORDER BY (timestamp, kafka_partition, kafka_offset)
TTL timestamp + INTERVAL 1 MONTH DELETE;