mod add_log;

use crate::interfaces::AppState;
use crate::interfaces::middlewares::hmac::{HmacState, hmac_guard};
use crate::interfaces::v1::add_log::add_log;
use crate::security::keystore::KeyStore;
use axum::middleware;
use axum::routing::{Router, post};
use std::sync::Arc;

pub fn routes(key_store: Arc<KeyStore>) -> Router<AppState> {
    let hmac_state = HmacState { key_store };

    let hmac_protected = Router::new()
        .route("/log", post(add_log))
        .layer(middleware::from_fn_with_state(hmac_state, hmac_guard));
    hmac_protected
}
