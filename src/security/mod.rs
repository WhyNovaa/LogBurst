use crate::security::keystore::KeyStore;
use std::sync::Arc;
use tracing::{error, info};

pub mod keystore;

const KEYS_UPDATE_TIME: std::time::Duration = tokio::time::Duration::from_secs(60);

pub async fn update_key_store(key_store: Arc<KeyStore>) {
    let iteration_span = tracing::trace_span!("keys_replace_task");

    loop {
        tokio::time::sleep(KEYS_UPDATE_TIME).await;

        let _enter = iteration_span.enter();

        match KeyStore::load() {
            Ok(keys) => {
                key_store.replace(keys);
                info!("Keys replaced successfully");
            }
            Err(e) => error!(%e, "Failed to load keys"),
        }
    }
}
