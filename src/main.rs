use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use crate::models::http_client::HTTPClient;
use crate::models::log::Log;
use crate::traits::start::Start;
use dotenv::dotenv;
use crate::db::click_house::ClickHousePool;
use crate::db::pg::AuthPool;
use crate::models::app::App;
use crate::models::db_worker::DBWorker;
use crate::models::http_client::api::handlers::auth_command::AuthCommand;
use crate::traits::client::Client;

mod traits;
mod models;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let _ = App::<HTTPClient, DBWorker::<AuthPool, ClickHousePool>>::new().await.start().await;

    Ok(())
}
