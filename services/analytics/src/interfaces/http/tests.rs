use super::router;
use crate::application::queries::clickhouse::AnalyticsQueries;
use crate::application::queries::postgres::UserQueries;
use crate::infrastructure::clickhouse::repository::ClickhouseAnalyticsRepository;
use crate::infrastructure::postgres::repository::PostgresUserRepository;
use crate::security::jwt::Claims;
use crate::server::Server;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use clickhouse::test::{Mock, handlers};
use config::jwt::JwtConfig;
use futures::StreamExt;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use storage::clickhouse::ClickhouseLog;
use time::OffsetDateTime;
use tokio::sync::broadcast;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

const SECRET: &[u8] = b"analytics-test-secret";
const INTERVAL: &str = "from=2026-10-21T00:00:00Z&to=2026-10-21T01:00:00Z";

fn setup(clickhouse_url: &str) -> (Router, Arc<Server>) {
    let manager =
        deadpool_postgres::Manager::new(tokio_postgres::Config::new(), tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager).build().unwrap();
    let (live_log_sender, _) = broadcast::channel(15);
    let server = Arc::new(Server {
        analytics_queries: AnalyticsQueries::new(ClickhouseAnalyticsRepository::new(
            clickhouse::Client::default().with_url(clickhouse_url),
        )),
        user_queries: UserQueries::new(PostgresUserRepository::new(pool)),
        live_log_sender,
        token: CancellationToken::new(),
    });
    let app = router(
        server.clone(),
        JwtConfig {
            secret_key: SECRET.to_vec(),
            expiration_seconds: Duration::from_secs(60),
        },
    );
    (app, server)
}

fn authenticated_request(uri: &str) -> Request<Body> {
    let token = Claims::new("test-user".into(), Duration::from_secs(60))
        .into_jwt(SECRET)
        .unwrap();
    Request::builder()
        .uri(uri)
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

fn log() -> ClickhouseLog {
    ClickhouseLog {
        timestamp: OffsetDateTime::from_unix_timestamp(1_792_540_800).unwrap(),
        level: "info".into(),
        service: "test-service".into(),
        message: "test-message".into(),
        raw_data: "{}".into(),
        kafka_partition: 3,
        kafka_offset: 42,
    }
}

async fn body_json(response: axum::response::Response) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn all_analytics_routes_require_jwt_and_login_is_public() {
    let mock = Mock::new();
    let (app, _) = setup(mock.url());
    for path in [
        "levels/count",
        "levels/intervals",
        "services",
        "logs/interval",
        "last",
    ] {
        for token in [None, Some("Bearer invalid")] {
            let mut request = Request::builder().uri(format!("/v1/analytics/{path}"));
            if let Some(token) = token {
                request = request.header("Authorization", token);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path}");
        }
    }
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn analytic_json_routes_keep_their_response_shapes() {
    let mock = Mock::new();
    let (app, _) = setup(mock.url());
    mock.add(handlers::provide(["test-service"]));
    let response = app
        .clone()
        .oneshot(authenticated_request("/v1/analytics/services"))
        .await
        .unwrap();
    assert_eq!(body_json(response).await, json!(["test-service"]));

    mock.add(handlers::provide([(2_u64, 3_u64, 5_u64)]));
    let response = app
        .clone()
        .oneshot(authenticated_request("/v1/analytics/levels/count"))
        .await
        .unwrap();
    assert_eq!(
        body_json(response).await,
        json!({"error_count": 2, "warn_count": 3, "info_count": 5})
    );

    mock.add(handlers::provide([(
        1_792_540_800_000_i64,
        2_u64,
        3_u64,
        5_u64,
    )]));
    let response = app
        .oneshot(authenticated_request(&format!(
            "/v1/analytics/levels/intervals?{INTERVAL}"
        )))
        .await
        .unwrap();
    let body = body_json(response).await;
    assert_eq!(body[0]["error_count"], 2);
    assert_eq!(body[0]["warn_count"], 3);
    assert_eq!(body[0]["info_count"], 5);
    assert!(body[0]["time_bucket"].is_string());
}

#[tokio::test]
async fn historical_logs_stream_rows() {
    let mock = Mock::new();
    let (app, _) = setup(mock.url());
    mock.add(handlers::provide([log()]));
    let response = app
        .clone()
        .oneshot(authenticated_request(&format!(
            "/v1/analytics/logs/interval?{INTERVAL}&limit=10"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let bytes = timeout(Duration::from_secs(2), to_bytes(response.into_body(), 4096))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        String::from_utf8(bytes.to_vec()).unwrap(),
        format!("data: {}\n\n", serde_json::to_string(&log()).unwrap())
    );
}

#[tokio::test]
async fn historical_logs_preserve_filters_and_limit() {
    use axum::extract::Query;
    use axum::routing::get;
    use std::collections::HashMap;

    let (query_sender, mut query_receiver) = tokio::sync::mpsc::channel(1);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let mock = Router::new().route(
        "/",
        get(
            move |Query(params): Query<HashMap<String, String>>| async move {
                query_sender.send(params["query"].clone()).await.unwrap();
                ""
            },
        ),
    );
    let task = tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
    let (app, _) = setup(&url);
    let response = app
        .oneshot(authenticated_request(&format!(
            "/v1/analytics/logs/interval?{INTERVAL}&service=test-service&level=warn&limit=9999"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    timeout(Duration::from_secs(2), to_bytes(response.into_body(), 4096))
        .await
        .unwrap()
        .unwrap();
    let query = timeout(Duration::from_secs(2), query_receiver.recv())
        .await
        .unwrap()
        .unwrap();
    task.abort();
    assert!(query.contains("FROM logs FINAL"), "{query}");
    assert!(
        query.contains("timestamp >= toDateTime64(1792540800, 3, 'UTC')"),
        "{query}"
    );
    assert!(
        query.contains("timestamp <= toDateTime64(1792544400, 3, 'UTC')"),
        "{query}"
    );
    assert!(query.contains("service = 'test-service'"), "{query}");
    assert!(query.contains("level = 'warn'"), "{query}");
    assert!(query.contains("ORDER BY timestamp LIMIT 5000"), "{query}");
}

#[tokio::test]
async fn live_logs_stream_broadcast_events_and_stop_on_shutdown() {
    let mock = Mock::new();
    let (app, server) = setup(mock.url());
    let response = app
        .oneshot(authenticated_request("/v1/analytics/last"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut body = response.into_body().into_data_stream();
    server.live_log_sender.send(log()).unwrap();
    let event = timeout(Duration::from_secs(2), body.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        String::from_utf8(event.to_vec()).unwrap(),
        format!("data: {}\n\n", serde_json::to_string(&log()).unwrap())
    );
    server.token.cancel();
    assert!(
        timeout(Duration::from_secs(2), body.next())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(server.live_log_sender.receiver_count(), 0);
}
