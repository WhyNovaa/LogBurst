use crate::security::keystore::KeyStore;
use chrono::TimeZone;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::log::warn;
use tracing::{error, info};

pub mod keystore;

const KEYS_UPDATE_TIME: std::time::Duration = tokio::time::Duration::from_secs(60);

pub async fn update_key_store(
    token: CancellationToken,
    key_store: Arc<KeyStore>,
) -> anyhow::Result<()> {
    let mut last_update = chrono::Utc.timestamp_opt(0, 0).unwrap();

    loop {
        match key_store.fetch_and_apply(last_update).await {
            Ok(new_last_update) => {
                last_update = new_last_update;
                info!("New keys were added successfully");
            }
            Err(e) => error!(%e, "Failed to load keys"),
        }

        tokio::select! {
            _ = tokio::time::sleep(KEYS_UPDATE_TIME) => {}
            _ = token.cancelled() => {
                warn!("Update key_store task finished via ctrl_c");
                break;
            }
        }
    }

    Ok(())
}
