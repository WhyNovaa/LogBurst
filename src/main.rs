use crate::models::http_client::HTTPClient;
use dotenv::dotenv;
use crate::db::click_house_client::ClickHouseClient;
use crate::db::pg::AuthPool;
use crate::models::app::App;
use crate::models::db_worker::DBWorker;

mod traits;
mod models;
mod db;
mod handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let _ = App::<HTTPClient, DBWorker::<AuthPool, ClickHouseClient>>::new().await.start().await;

    Ok(())
}
