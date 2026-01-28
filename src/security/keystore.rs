use anyhow::{Result, anyhow};
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use toml::Value;

pub type ServiceId = String;
pub type ServiceKey = Box<[u8; 32]>;

pub struct KeyStore {
    keys: ArcSwap<HashMap<ServiceId, ServiceKey>>,
}

impl KeyStore {
    pub const KEYS_PATH: &str = "keys.toml";

    pub fn new(keys: HashMap<ServiceId, ServiceKey>) -> Self {
        Self {
            keys: ArcSwap::from_pointee(keys),
        }
    }

    pub fn get(&self, service_id: &ServiceId) -> Option<ServiceKey> {
        self.keys.load().get(service_id).cloned()
    }

    pub fn replace(&self, keys: HashMap<ServiceId, ServiceKey>) {
        self.keys.store(Arc::new(keys));
    }

    pub fn load() -> Result<HashMap<ServiceId, ServiceKey>> {
        let content = fs::read_to_string(Self::KEYS_PATH)?;
        let value: Value = content.parse::<Value>()?;

        let services_table = value
            .get("services")
            .and_then(Value::as_table)
            .ok_or_else(|| anyhow!("No [services] table found"))?;

        let mut keys = HashMap::new();

        for (k, v) in services_table {
            let key_bytes = v
                .as_str()
                .unwrap_or_else(|| {
                    tracing::warn!("Key for {k} not found, using zeroed key");
                    ""
                })
                .as_bytes();

            if key_bytes.len() != 32 {
                return Err(anyhow!(
                    "Key for service '{}' has invalid length {}, expected 32",
                    k,
                    key_bytes.len()
                ));
            }

            let mut arr = [0u8; 32];
            arr.copy_from_slice(key_bytes);

            keys.insert(k.clone(), Box::new(arr));
        }

        Ok(keys)
    }
}
