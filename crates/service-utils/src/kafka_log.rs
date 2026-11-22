use anyhow::{Context, bail};
use domain::Log;
use prost::Message as _;
use prost_types::Timestamp;
use proto::kafka::KafkaLog;
use rdkafka::Message;
use rdkafka::message::{Header, Headers, OwnedHeaders};
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use time::OffsetDateTime;

pub const LOG_TOPIC: &str = "logs";
const CONTENT_TYPE: &str = "content-type";
const PROTOBUF_CONTENT_TYPE: &str = "application/protobuf";

pub async fn send(producer: &FutureProducer, log: Log) -> OwnedDeliveryResult {
    let (payload, headers) = encode(log);
    let record: FutureRecord<(), _> = FutureRecord::to(LOG_TOPIC)
        .payload(payload.as_slice())
        .headers(headers);

    producer.send(record, Duration::from_secs(1)).await
}

pub fn encode(log: Log) -> (Vec<u8>, OwnedHeaders) {
    let payload = KafkaLog {
        timestamp: Some(Timestamp {
            seconds: log.timestamp.unix_timestamp(),
            nanos: log.timestamp.nanosecond() as i32,
        }),
        level: log.level,
        service: log.service,
        message: log.message,
        raw_data: log.raw_data,
    }
    .encode_to_vec();

    let headers = OwnedHeaders::new().insert(Header {
        key: CONTENT_TYPE,
        value: Some(PROTOBUF_CONTENT_TYPE),
    });

    (payload, headers)
}

pub fn decode(message: &impl Message) -> anyhow::Result<Log> {
    let payload = message.payload().context("Kafka log has no payload")?;
    let content_type = message
        .headers()
        .and_then(|headers| headers.iter().find(|header| header.key == CONTENT_TYPE));

    match content_type {
        Some(header) if header.value == Some(PROTOBUF_CONTENT_TYPE.as_bytes()) => {
            decode_protobuf(payload)
        }
        // Before the Protobuf switch, producers sent Postcard records without headers.
        None => postcard::from_bytes(payload).context("Invalid Postcard Kafka log"),
        Some(_) => bail!("Unsupported Kafka log content type"),
    }
}

fn decode_protobuf(payload: &[u8]) -> anyhow::Result<Log> {
    let log = KafkaLog::decode(payload).context("Invalid Protobuf Kafka log")?;
    let timestamp = log.timestamp.context("Kafka log has no timestamp")?;
    let nanos = u32::try_from(timestamp.nanos).context("Invalid Kafka log timestamp")?;
    let timestamp = OffsetDateTime::from_unix_timestamp(timestamp.seconds)
        .and_then(|time| time.replace_nanosecond(nanos))
        .context("Invalid Kafka log timestamp")?;

    Ok(Log {
        timestamp,
        level: log.level,
        service: log.service,
        message: log.message,
        raw_data: log.raw_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdkafka::message::{OwnedMessage, Timestamp as KafkaTimestamp};

    fn sample_log() -> Log {
        Log {
            timestamp: OffsetDateTime::from_unix_timestamp_nanos(-123_456_789).unwrap(),
            level: "debug".into(),
            service: "ingest".into(),
            message: "Проверка сериализации".into(),
            raw_data: r#"{"request_id":42}"#.into(),
        }
    }

    fn message(payload: Vec<u8>, headers: Option<OwnedHeaders>) -> OwnedMessage {
        OwnedMessage::new(
            Some(payload),
            None,
            LOG_TOPIC.into(),
            KafkaTimestamp::NotAvailable,
            0,
            0,
            headers,
        )
    }

    fn assert_log_fields(log: Log) {
        let expected = sample_log();
        assert_eq!(log.timestamp, expected.timestamp);
        assert_eq!(log.level, expected.level);
        assert_eq!(log.service, expected.service);
        assert_eq!(log.message, expected.message);
        assert_eq!(log.raw_data, expected.raw_data);
    }

    #[test]
    fn protobuf_round_trip_preserves_log_fields() {
        let (payload, headers) = encode(sample_log());
        assert_eq!(headers.get(0).key, "content-type");
        assert_eq!(headers.get(0).value, Some(&b"application/protobuf"[..]));
        assert_log_fields(decode(&message(payload, Some(headers))).unwrap());
    }

    #[test]
    fn reads_postcard_records_without_headers() {
        let payload = postcard::to_allocvec(&sample_log()).unwrap();
        assert_log_fields(decode(&message(payload, None)).unwrap());
    }

    #[test]
    fn rejects_protobuf_records_without_timestamp() {
        let (_, headers) = encode(sample_log());
        let payload = KafkaLog::default().encode_to_vec();
        let error = decode(&message(payload, Some(headers))).unwrap_err();
        assert_eq!(error.to_string(), "Kafka log has no timestamp");
    }
}
