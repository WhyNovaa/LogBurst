use crate::security::keystore::{KeyStore, ServiceId};
use axum::extract::State;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
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

        Self::const_time_compare_hashes(expected.as_bytes(), signature.as_bytes())
    }

    fn const_time_compare_hashes(a: &[u8], b: &[u8]) -> bool {
        let is_valid = subtle::ConstantTimeEq::ct_eq(a, b);

        bool::from(is_valid)
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
    let timestamp_opt = parts
        .headers
        .get("X-TIMESTAMP")
        .and_then(|h| h.to_str().ok());

    let (Some(service_id), Some(signature), Some(timestamp_str)) =
        (service_id_opt, signature_opt, timestamp_opt)
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let timestamp: u64 = timestamp_str
        .parse()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .as_secs();
    if timestamp + LOG_SECS_TO_PROCEED < now {
        //return Err(StatusCode::UNAUTHORIZED);
    }

    const MAX_BODY_SIZE: usize = 1024 * 1024;

    let body_bytes = axum::body::to_bytes(body, MAX_BODY_SIZE)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;

    let method = parts.method.as_str();
    let mut payload_to_sign = Vec::new();

    payload_to_sign.extend_from_slice(timestamp_str.as_bytes());
    payload_to_sign.extend_from_slice(b"\n");
    payload_to_sign.extend_from_slice(method.as_bytes());
    payload_to_sign.extend_from_slice(b"\n");
    payload_to_sign.extend_from_slice(body_bytes.as_ref());

    dbg!(String::from_utf8(payload_to_sign).map_err(|_| StatusCode::UNAUTHORIZED)?);
    let service_id_parsed: ServiceId = service_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    if !hmac.validate_signature(&service_id_parsed, &body_bytes, signature) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let req = Request::from_parts(parts, Body::from(body_bytes));

    Ok(next.run(req).await)
}
