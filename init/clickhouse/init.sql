CREATE TABLE logs (
    timestamp DateTime('UTC'),
    level     LowCardinality(String),
    service   LowCardinality(String),
    message   String
)
ENGINE = MergeTree
ORDER BY (timestamp, service, level)
TTL timestamp + INTERVAL 30 DAY
SETTINGS index_granularity = 8192;