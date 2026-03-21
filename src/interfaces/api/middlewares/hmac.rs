use crate::security::keystore::{KeyStore, ServiceId};
use axum::extract::State;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use serde_json::Value;
use std::sync::Arc;

const LOG_SECS_TO_PROCEED: u64 = 120;

#[derive(Clone)]
pub struct HmacState {
    pub key_store: Arc<KeyStore>,
}
impl HmacState {
    pub fn validate_signature(&self, service_id: &ServiceId, data: &[u8], signature: &str) -> bool {
        let Some(key) = self.key_store.get(service_id) else {
            return false;
        };

        let mut hasher = blake3::Hasher::new_keyed(key.as_ref());

        let expected = hasher.update(data).finalize().to_hex();

        signature == expected.as_str()
    }
}

pub async fn hmac_guard(
    State(hmac): State<HmacState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let (parts, body) = req.into_parts();

    let service_id_opt = parts
        .headers
        .get("X-SERVICE-ID")
        .and_then(|h| h.to_str().ok());
    let signature_opt = parts
        .headers
        .get("X-SIGNATURE")
        .and_then(|h| h.to_str().ok());

    match (service_id_opt, signature_opt) {
        (Some(service_id), Some(signature)) if !service_id.is_empty() && !signature.is_empty() => {
            const MAX_BODY_SIZE: usize = 1024 * 1024;

            let body_bytes = axum::body::to_bytes(body, MAX_BODY_SIZE)
                .await
                .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;

            let service_id_parsed: ServiceId =
                service_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

            if !hmac.validate_signature(&service_id_parsed, &body_bytes, signature) {
                return Err(StatusCode::UNAUTHORIZED);
            }

            let jsn: Value =
                serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

            let timestamp = jsn
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .as_secs();

            if timestamp + LOG_SECS_TO_PROCEED < now {
                return Err(StatusCode::UNAUTHORIZED);
            }

            let req = Request::from_parts(parts, Body::from(body_bytes));

            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
