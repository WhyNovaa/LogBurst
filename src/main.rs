use axum::response::Response;
use tokio::sync::oneshot;
use crate::models::http_client::HTTPClient;
use crate::models::log::Log;
use crate::traits::new::New;
use crate::traits::start::Start;
use dotenv::dotenv;
mod traits;
mod models;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let (s, r) = tokio::sync::mpsc::channel::<(Log, oneshot::Sender<Response>)>(100);
    let cl = HTTPClient::new(s).start().await;

    Ok(())
}
