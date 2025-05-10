use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use crate::models::http_client::HTTPClient;
use crate::models::log::Log;
use crate::traits::start::Start;
use dotenv::dotenv;
use crate::models::http_client::api::handlers::auth_command::AuthCommand;
use crate::traits::client::Client;

mod traits;
mod models;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let (l_s, l_r) = mpsc::channel::<(Log, oneshot::Sender<Response>)>(100);
    let (c_s, c_r) = mpsc::channel::<(AuthCommand, oneshot::Sender<Response>)>(100);
    let _ = HTTPClient::new(c_s, l_s).start().await;

    Ok(())
}
