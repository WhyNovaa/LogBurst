use std::env;
use std::future::ready;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use axum::extract::Request;
use axum::response::Response;
use axum::{middleware, Router};
use axum::routing::get;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use crate::handlers::middlewares::track_metrics;
use crate::handlers::routes::{auth_routes, logs_routes};
use crate::models::app::{AuthCommandSender, LogCommandSender};
use crate::traits::client::Client;
use crate::traits::logs_repository::LogsRepository;
use crate::traits::start::Start;

pub mod role;
pub mod auth_payload;
pub mod reg_payload;
pub mod claims;
pub mod creation_payload;
pub mod get_logs_params;

pub struct HTTPClient<L: LogsRepository> {
    router: Router,
    addr: SocketAddr,
    metrics_router: Router,
    metrics_addr: SocketAddr,
    db: Arc<L>,
}

impl<L: LogsRepository> Client<L> for HTTPClient<L> {
    fn new(
        auth_command_sender: AuthCommandSender,
    ) -> Self {
        log::info!("Creating HTTPClient");

        let db = Arc::new(L::new());

        let router = Router::new()
            .merge(auth_routes(auth_command_sender))
            .merge(logs_routes(Arc::clone(&db)))
            .layer(middleware::from_fn(track_metrics));

        let host = env::var("SERVICE_HOST").expect("SERVICE_HOST not found in .env file");
        let port = env::var("SERVICE_PORT").expect("SERVICE_PORT not found in .env file");

        let addr = format!("{}:{}", host, port);
        let addr = SocketAddr::from_str(&addr).expect("Invalid URL");

        let metrics_host = env::var("METRICS_HOST").expect("METRICS_HOST not found in .env file");
        let metrics_port = env::var("METRICS_PORT").expect("METRICS_PORT not found in .env file");

        let metrics_addr = format!("{}:{}", metrics_host, metrics_port);
        let metrics_addr = SocketAddr::from_str(&metrics_addr).expect("Invalid URL");

        let metrics_recorder = setup_metrics_recorder();

        let metrics_router = Router::new()
            .route("/metrics", get(move || ready(metrics_recorder.render())));

        Self {
            router,
            addr,
            metrics_router,
            metrics_addr,
            db
        }
    }
}

#[async_trait]
impl<L: LogsRepository> Start for HTTPClient<L> {
    async fn start(self) {
        log::info!("Starting HTTPClient");

        let (_main_server, _metrics_server) = tokio::join!(start_server(self.router, self.addr), start_server(self.metrics_router, self.metrics_addr));
    }
}

pub async fn start_server(router: Router, addr: SocketAddr) {
    let tcp_listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(tcp_listener, router.into_make_service()).await.unwrap();
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}