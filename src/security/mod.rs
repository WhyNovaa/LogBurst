use crate::security::keystore::KeyStore;
use std::sync::Arc;
use tracing::{error, info};

pub mod keystore;

const KEYS_UPDATE_TIME: std::time::Duration = tokio::time::Duration::from_secs(60);

pub async fn update_key_store(key_store: Arc<KeyStore>) {
    loop {
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
}
