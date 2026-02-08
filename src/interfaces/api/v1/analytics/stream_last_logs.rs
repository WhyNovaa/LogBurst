use crate::db::clickhouse::structs::Log;
use crate::interfaces::api::error::ApiResult;
use crate::interfaces::api::v1::analytics::dto;
use crate::server::Server;
use axum::extract::{Query, State};
use axum::response::Sse;
use futures::Stream;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn stream_last_logs(
    State(server): State<Arc<Server>>,
    Query(query): Query<dto::stream_last_logs::Query>,
) -> ApiResult<Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>>> {
    let (tx, rx) = kanal::bounded_async::<Log>(10);

    tokio::spawn(async move { server.logs_db.stream_last_logs(tx, query.limit).await });

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
