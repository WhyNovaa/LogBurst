use crate::security::keystore::KeyStore;
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
    loop {
        tokio::select! {
            _ = tokio::time::sleep(KEYS_UPDATE_TIME) => {}
            _ = token.cancelled() => {
                warn!("Update key_store task finished via ctrl_c");
                break;
            }
        }
        tokio::time::sleep(KEYS_UPDATE_TIME).await;

        match KeyStore::load() {
            Ok(keys) => {
                let len = keys.len();
                key_store.replace(keys);
                info!("{} keys were replaced successfully", len);
            }
            Err(e) => error!(%e, "Failed to load keys"),
        }
    }

    Ok(())
}
