use crate::LOG_GROUP_ID;
use anyhow::bail;
use config::clickhouse::ClickhouseConfig;
use config::kafka::KafkaConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::BorrowedMessage;
use rdkafka::{Message, Offset, TopicPartitionList};
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

    pub async fn start_receiving(self) -> anyhow::Result<()> {
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
                            let db_log = match process_borrowed_message(&msg) {
                                Ok(log) => log,
                                Err(e) => {
                                    error!("Error while receiving kafka msg: {e}");
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

pub fn process_borrowed_message(msg: &BorrowedMessage) -> anyhow::Result<ClickhouseLog> {
    let Some(bytes) = msg.payload() else {
        bail!("No payload in kafka message");
    };

    let log = match postcard::from_bytes::<domain::Log>(bytes) {
        Ok(log) => log,
        Err(e) => {
            bail!("Failed to deserialize kafka message: {e}");
        }
    };

    let db_log = ClickhouseLog::from(log);

    Ok(db_log)
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
