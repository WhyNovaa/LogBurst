use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use axum::extract::State;
use axum::response::Response;
use axum::Router;
use tokio::sync::{mpsc, oneshot};
use crate::models::app::{AuthCommandSender, LogSender};
use crate::models::http_client::routes::auth_routes;
use crate::models::log::Log;
use crate::traits::client::Client;
use crate::traits::start::Start;

pub mod api;
mod routes;

pub struct HTTPClient {
    router: Router,
    addr: SocketAddr,
}


impl Client for HTTPClient {
    fn new(
        auth_command_sender: AuthCommandSender,
    ) -> Self {
        log::info!("Creating HTTPClient");

        let router = Router::new()
            .merge(auth_routes(auth_command_sender));

        let url = env::var("URL").expect("URL not found in .env file");

        let addr = SocketAddr::from_str(&url).expect("Invalid URL");

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