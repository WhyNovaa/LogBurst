use crate::db::clickhouse::structs::Log;
use crate::interfaces::api::error::ApiResult;
use axum::extract::State;
use axum::response::Sse;
use futures::Stream;
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;
use tracing::log::info;

pub async fn stream_last_logs(
    State(live_sender): State<tokio::sync::broadcast::Sender<Log>>,
) -> ApiResult<Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>>> {
    let (tx, rx) = async_channel::bounded::<Log>(15);

    let mut live_rx = live_sender.subscribe();
    tokio::spawn(async move {
        loop {
            match live_rx.recv().await {
                Ok(log) => {
                    if tx.send(log).await.is_err() {
                        break;
                    }
                }
                Err(e) => match e {
                    RecvError::Closed => break,
                    RecvError::Lagged(lagged) => {
                        info!("Lagged log entries: {:?}", lagged);
                    }
                },
            }
        }
    });

    let stream = async_stream::stream! {
        while let Ok(log) = rx.recv().await {
            match axum::response::sse::Event::default().json_data(log) {
                Ok(log) => yield Ok(log),
                Err(error) => tracing::error!("{}", error),
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()))
}
