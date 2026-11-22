use crate::LOG_GROUP_ID;
use config::clickhouse::ClickhouseConfig;
use config::kafka::KafkaConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::{Message, Offset, TopicPartitionList};
use service_utils::kafka_log::{self, LOG_TOPIC};
use std::collections::HashMap;
use storage::clickhouse::ClickhouseLog;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

const INSERTER_MAX_ROWS: u64 = 50_000;
const FLUSH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);

pub struct Writer {
    pub consumer: StreamConsumer,
    pub client: clickhouse::Client,
    pub token: CancellationToken,
}

impl Writer {
    pub async fn new(
        kafka_cfg: KafkaConfig,
        clickhouse_cfg: ClickhouseConfig,
        token: CancellationToken,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            consumer: kafka_cfg.create_consumer(LOG_GROUP_ID)?,
            client: clickhouse_cfg.client(),
            token,
        })
    }

    pub async fn start_writing(self) -> anyhow::Result<()> {
        self.consumer.subscribe(&[LOG_TOPIC])?;

        let mut inserter = self
            .client
            .inserter::<storage::clickhouse::ClickhouseLog>("logs")?;

        let mut tick_interval = interval(FLUSH_INTERVAL);

        let mut offsets_to_commit: HashMap<(String, i32), i64> = HashMap::new();

        loop {
            tokio::select! {
                msg = self.consumer.recv() => {
                    match msg {
                        Ok(msg) => {
                            let db_log = match process_message(&msg) {
                                Ok(log) => log,
                                Err(e) => {
                                    error!("Error while receiving kafka msg: {e:#}");
                                    continue;
                                }
                            };

                            if let Err(e) = inserter.write(&db_log) {
                                error!("Error while writing log into inserter: {}", e);
                                continue;
                            };

                                    /*if rand::random_range(1..=100) == 100 {
                                        if let Err(e) = live_tx.send(log) {
                                            error!("Error sending log into live channel: {}", e);
                                        }
                                    }*/

                            // todo: can be optimized
                            offsets_to_commit.insert((msg.topic().to_string(), msg.partition()), msg.offset());

                            let pending_amount = inserter.pending().rows;

                            if pending_amount >= INSERTER_MAX_ROWS {
                                info!("Batch limit reached, committing {} logs...", pending_amount);
                                match inserter.force_commit().await {
                                    Ok(res) => {
                                        if let Err(e) = commit_kafka_offsets(&self.consumer, &offsets_to_commit) {
                                            warn!("Recieved error while commiting some messages: {e}");
                                        }

                                        offsets_to_commit.clear();

                                        info!("{} rows were written", res.rows);
                                        tick_interval.reset();
                                    }
                                    Err(e) => error!("Failed to commit batch: {}", e),
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive log: {}", e);
                        }
                    }

                },
                _ = tick_interval.tick() => {
                    let pending_amount = inserter.pending().rows;

                    if pending_amount > 0 {
                        info!("Time interval, committing {} logs...", pending_amount);
                        match inserter.force_commit().await {
                            Ok(res) => {
                                if let Err(e) = commit_kafka_offsets(&self.consumer, &offsets_to_commit) {
                                    warn!("Received error while commiting some messages: {e}");
                                }
                                offsets_to_commit.clear();
                                info!("{} rows were written", res.rows);
                                tick_interval.reset();
                            }
                            Err(e) => error!("Failed to commit by timer: {}", e),
                        }
                    }
                }
                _ = self.token.cancelled() => {
                    warn!("Shutdown signal received. Draining remaining logs...");

                    let pending_amount = inserter.pending().rows;
                    if pending_amount > 0 {
                        match inserter.force_commit().await {
                            Ok(res) => {
                                if let Err(e) = commit_kafka_offsets(&self.consumer, &offsets_to_commit) {
                                    warn!("Failed to commit offsets on shutdown: {}", e);
                                }
                                info!("Successfully committed {} remaining logs on shutdown", res.rows);
                            }
                            Err(e) => error!("Failed to commit logs on shutdown: {}", e)
                        }
                    }

                    break;
                }
            }
        }
        Ok(())
    }
}

pub fn process_message(msg: &impl Message) -> anyhow::Result<ClickhouseLog> {
    let log = kafka_log::decode(msg)?;
    Ok(ClickhouseLog::from_kafka(
        log,
        msg.partition(),
        msg.offset(),
    ))
}

pub fn commit_kafka_offsets(
    consumer: &StreamConsumer,
    offsets: &HashMap<(String, i32), i64>,
) -> anyhow::Result<()> {
    if offsets.is_empty() {
        return Ok(());
    }

    let mut list = TopicPartitionList::new();

    for ((topic, partition), offset) in offsets {
        list.add_partition_offset(topic, *partition, Offset::Offset(*offset + 1))?;
    }

    consumer.commit(&list, CommitMode::Async)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::process_message;
    use rdkafka::message::{OwnedHeaders, OwnedMessage, Timestamp};
    use service_utils::kafka_log::{self, LOG_TOPIC};
    use std::time::SystemTime;
    use storage::clickhouse::ClickhouseLog;

    const OFFSET: i64 = 1_i64 << 40;

    fn payload() -> (Vec<u8>, OwnedHeaders) {
        kafka_log::encode(domain::Log {
            timestamp: SystemTime::now().into(),
            level: "info".into(),
            service: "writer-test".into(),
            message: "same event payload".into(),
            raw_data: "{}".into(),
        })
    }

    fn message(payload: &(Vec<u8>, OwnedHeaders), partition: i32, offset: i64) -> OwnedMessage {
        let (bytes, headers) = payload;
        OwnedMessage::new(
            Some(bytes.clone()),
            None,
            LOG_TOPIC.into(),
            Timestamp::NotAvailable,
            partition,
            offset,
            Some(headers.clone()),
        )
    }

    #[test]
    fn kafka_replays_keep_the_key_and_distinct_records_do_not_collide() {
        let payload = payload();
        let original = process_message(&message(&payload, 3, OFFSET)).unwrap();
        let replay = process_message(&message(&payload, 3, OFFSET)).unwrap();
        let next_offset = process_message(&message(&payload, 3, OFFSET + 1)).unwrap();
        let other_partition = process_message(&message(&payload, 4, OFFSET)).unwrap();
        let key = |row: &ClickhouseLog| (row.timestamp, row.kafka_partition, row.kafka_offset);

        assert_eq!(original.kafka_partition, 3);
        assert_eq!(original.kafka_offset, OFFSET);
        assert_eq!(key(&original), key(&replay));
        assert_ne!(key(&original), key(&next_offset));
        assert_ne!(key(&original), key(&other_partition));
    }

    #[tokio::test]
    #[ignore = "requires an isolated ClickHouse server at CLICKHOUSE_TEST_URL"]
    async fn clickhouse_deduplicates_replays_and_preserves_distinct_kafka_records()
    -> anyhow::Result<()> {
        let url = std::env::var("CLICKHOUSE_TEST_URL")?;
        let database = format!(
            "writer_coordinates_test_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos()
        );
        let admin = clickhouse::Client::default().with_url(url);
        admin
            .query(&format!("CREATE DATABASE {database}"))
            .execute()
            .await?;
        let client = admin.clone().with_database(&database);

        let result = async {
            client
                .query(include_str!(
                    "../../../migrations/clickhouse/up/20251201173112_create_logs_table.sql"
                ))
                .execute()
                .await?;

            let payload = payload();
            let mut inserter = client.inserter::<ClickhouseLog>("logs")?;
            for (partition, offset) in [(3, OFFSET), (3, OFFSET), (3, OFFSET + 1), (4, OFFSET)] {
                inserter.write(&process_message(&message(&payload, partition, offset))?)?;
            }
            inserter.force_commit().await?;

            let rows = client
                .query("SELECT ?fields FROM logs FINAL ORDER BY kafka_partition, kafka_offset")
                .fetch_all::<ClickhouseLog>()
                .await?;
            let coordinates = rows
                .iter()
                .map(|row| (row.kafka_partition, row.kafka_offset))
                .collect::<Vec<_>>();
            anyhow::ensure!(
                coordinates == [(3, OFFSET), (3, OFFSET + 1), (4, OFFSET)],
                "unexpected stored coordinates: {coordinates:?}"
            );

            client.query("OPTIMIZE TABLE logs FINAL").execute().await?;
            let count = client
                .query("SELECT count() FROM logs")
                .fetch_one::<u64>()
                .await?;
            anyhow::ensure!(count == 3, "expected three distinct records, got {count}");
            Ok::<_, anyhow::Error>(())
        }
        .await;

        let cleanup = admin
            .query(&format!("DROP DATABASE {database} SYNC"))
            .execute()
            .await;
        result?;
        cleanup?;
        Ok(())
    }
}
