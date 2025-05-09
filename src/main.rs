use axum::response::Response;
use tokio::sync::oneshot;
use crate::models::http_client::HTTPClient;
use crate::models::log::Log;
use crate::traits::start::Start;
use dotenv::dotenv;
use crate::traits::client::Client;

mod traits;
mod models;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let (s, r) = tokio::sync::mpsc::channel::<(Log, oneshot::Sender<Response>)>(100);
    let _ = HTTPClient::new(s).start().await;

    Ok(())
}
