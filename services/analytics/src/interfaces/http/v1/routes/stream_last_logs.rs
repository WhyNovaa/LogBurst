use crate::server::Server;
use axum::extract::State;
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use error::ApiResult;
use futures::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;

pub async fn stream_last_logs(
    State(server): State<Arc<Server>>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    let mut receiver = server.live_log_sender.subscribe();
    let stream = async_stream::stream! {
        loop {
            let next = tokio::select! {
                _ = server.token.cancelled() => break,
                next = receiver.recv() => next,
            };
            match next {
                Ok(log) => match Event::default().json_data(log) {
                    Ok(event) => yield Ok(event),
                    Err(error) => tracing::error!("Failed to serialize live log: {error}"),
                },
                Err(RecvError::Closed) => break,
                Err(RecvError::Lagged(count)) => tracing::info!("Lagged log entries: {count}"),
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
