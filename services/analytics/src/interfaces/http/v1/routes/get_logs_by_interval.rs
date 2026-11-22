use crate::interfaces::http::v1::dto::get_logs_by_interval::Filters;
use crate::server::Server;
use axum::extract::{Query, State};
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use error::{ApiResult, IntoApiError};
use futures::Stream;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn get_logs_by_interval(
    State(server): State<Arc<Server>>,
    Query(query): Query<Filters>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    let mut cursor = server
        .analytics_queries
        .get_logs_by_interval(
            query.interval.from,
            query.interval.to,
            query.service,
            query.level,
            query.limit,
        )
        .internal()?;

    let stream = async_stream::stream! {
        loop {
            let next = tokio::select! {
                _ = server.token.cancelled() => break,
                next = cursor.next() => next,
            };
            match next {
                Ok(Some(log)) => match Event::default().json_data(log) {
                    Ok(event) => yield Ok(event),
                    Err(error) => tracing::error!("Failed to serialize log: {error}"),
                },
                Ok(None) => break,
                Err(error) => {
                    tracing::error!("Failed to query logs: {error}");
                    break;
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
