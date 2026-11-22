use anyhow::{Result, anyhow};
use arc_swap::ArcSwap;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

pub type ServiceId = String;
pub type ServiceKey = [u8; 32];

pub struct KeyStore {
    keys: ArcSwap<HashMap<ServiceId, ServiceKey>>,
    pg: deadpool_postgres::Pool,
}

impl KeyStore {
    pub fn new(pg: deadpool_postgres::Pool) -> Self {
        Self {
            keys: ArcSwap::from_pointee(HashMap::new()),
            pg,
        }
    }

    pub fn get(&self, service_id: &ServiceId) -> Option<ServiceKey> {
        self.keys.load().get(service_id).cloned()
    }

    fn apply_updates(&self, updates: Vec<ServiceWithKey>) {
        if updates.is_empty() {
            return;
        }

        let mut keys = (**self.keys.load()).clone();

        for update in updates {
            if update.is_active {
                keys.insert(update.service, update.key);
            } else {
                keys.remove(&update.service);
            }
        }

        info!("{} keys were updated", keys.len());

        self.keys.store(Arc::new(keys))
    }

    pub async fn fetch_and_apply(&self, last_update: DateTime<Utc>) -> Result<DateTime<Utc>> {
        let query_time = last_update - Duration::minutes(1);
        const QUERY: &str =
            "SELECT service, is_active, key, updated_at FROM keys WHERE updated_at >= $1";

        let client = self.pg.get().await?;

        let keys: Vec<ServiceWithKey> = client
            .query(QUERY, &[&query_time])
            .await?
            .iter()
            .filter_map(|row| match ServiceWithKey::try_from_row(row) {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    tracing::error!("Failed to parse key from DB: {}", e);
                    None
                }
            })
            .collect();

        let latest_update = keys
            .iter()
            .map(|key| key.updated_at)
            .max()
            .unwrap_or(last_update);

        self.apply_updates(keys);

        Ok(latest_update)
    }
}

struct ServiceWithKey {
    service: String,
    is_active: bool,
    key: ServiceKey,
    updated_at: DateTime<Utc>,
}

impl ServiceWithKey {
    pub fn try_from_row(row: &tokio_postgres::Row) -> anyhow::Result<Self> {
        let service = row.get("service");
        let is_active = row.get("is_active");
        let key_str: String = row.get("key");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        let key = key_str
            .as_bytes()
            .try_into()
            .map_err(|_| anyhow!("Key length must be 32 bytes"))?;

        Ok(Self {
            service,
            is_active,
            key,
            updated_at,
        })
    }
}
