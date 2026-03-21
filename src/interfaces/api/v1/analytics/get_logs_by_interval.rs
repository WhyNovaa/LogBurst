use crate::db::clickhouse::structs::Log;
use crate::interfaces::api::error::ApiResult;
use crate::interfaces::api::v1::analytics::dto;
use crate::server::Server;
use axum::extract::{Query, State};
use axum::response::Sse;
use futures::Stream;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn get_logs_by_interval(
    State(server): State<Arc<Server>>,
    Query(query): Query<dto::get_logs_by_interval::Filters>,
) -> ApiResult<Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>>> {
    let (tx, rx) = kanal::bounded_async::<Log>(100);

    // todo remove tracing
    tokio::spawn(async move {
        if let Err(e) = server
            .logs_db
            .get_logs_by_interval(
                query.interval.from,
                query.interval.to,
                query.service,
                query.level,
                query.limit,
                tx,
            )
            .await
        {
            tracing::error!("Error in db: {}", e)
        } else {
            tracing::info!("DB connection closed");
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
