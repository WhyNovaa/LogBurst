#!/usr/bin/env bash
  set -euo pipefail

  kafka-topics \
    --bootstrap-server kafka:29092 \
    --create \
    --if-not-exists \
    --topic "${KAFKA_LOG_TOPIC}" \
    --partitions ${KAFKA_PARTITIONS_NUMBER} \
    --replication-factor 1 \
    --config cleanup.policy=delete \
    --config retention.ms=259200000 # 3 days

  kafka-topics \
    --bootstrap-server kafka:29092 \
    --describe \
    --topic "${KAFKA_LOG_TOPIC}"