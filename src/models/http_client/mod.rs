use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use async_trait::async_trait;
use axum::Router;
use crate::handlers::routes::{auth_routes, logs_routes};
use crate::models::app::{AuthCommandSender, LogCommandSender};
use crate::traits::client::Client;
use crate::traits::start::Start;

pub mod role;
pub mod auth_payload;
pub mod reg_payload;
pub mod claims;
pub mod creation_payload;

pub struct HTTPClient {
    router: Router,
    addr: SocketAddr,
}

impl Client for HTTPClient {
    fn new(
        auth_command_sender: AuthCommandSender,
        log_command_sender: LogCommandSender,
    ) -> Self {
        log::info!("Creating HTTPClient");

        let router = Router::new()
            .merge(auth_routes(auth_command_sender))
            .merge(logs_routes(log_command_sender));

        let host = env::var("SERVICE_HOST").expect("SERVICE_HOST not found in .env file");
        let port = env::var("SERVICE_PORT").expect("SERVICE_PORT not found in .env file");

        let addr = format!("{}:{}", host, port);

        let addr = SocketAddr::from_str(&addr).expect("Invalid URL");

        Self {
            router,
            addr,
        }
    }
}

#[async_trait]
impl Start for HTTPClient {
    async fn start(self) {
        log::info!("Starting HTTPClient");

        let tcp_listener = tokio::net::TcpListener::bind(self.addr).await.unwrap();

        axum::serve(tcp_listener, self.router.into_make_service()).await.unwrap();
    }
}