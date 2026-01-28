use crate::interfaces::api::AppState;
use crate::interfaces::api::middlewares::hmac::{HmacState, hmac_guard};
use crate::interfaces::api::v1::add_log::add_log;
use crate::interfaces::api::v1::auth::login;
use crate::security::keystore::KeyStore;
use axum::routing::post;
use axum::{Router, middleware};
use std::sync::Arc;

mod add_log;
mod auth;
pub mod dto;

pub fn routes(key_store: Arc<KeyStore>) -> Router<AppState> {
    let hmac_state = HmacState { key_store };

    Router::new()
        .route("/logs", post(add_log))
        .layer(middleware::from_fn_with_state(hmac_state, hmac_guard))
        .route("/auth/login", post(login))
}
