CREATE TABLE logs (
    timestamp DateTime64(3, 'UTC'),
    level     LowCardinality(String),
    service   LowCardinality(String),
    message   String
)
ENGINE = MergeTree
ORDER BY (timestamp, service, level);