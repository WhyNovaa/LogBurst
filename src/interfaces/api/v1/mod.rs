use crate::interfaces::api::middlewares::auth::jwt_guard;
use crate::interfaces::api::middlewares::hmac::{hmac_guard, HmacState};
use crate::interfaces::api::v1::add_log::add_log;
use crate::interfaces::api::v1::auth::login;
use crate::interfaces::api::v1::get_interval_errors_count::get_interval_errors_count;
use crate::interfaces::api::AppState;
use crate::security::keystore::KeyStore;
use axum::routing::{get, post};
use axum::{middleware, Router};
use std::sync::Arc;

mod add_log;
pub mod auth;
pub mod dto;
mod get_interval_errors_count;

pub fn routes(key_store: Arc<KeyStore>) -> Router<AppState> {
    let hmac_state = HmacState { key_store };

    let hmac_protection = Router::new()
        .route("/logs", post(add_log))
        .layer(middleware::from_fn_with_state(hmac_state, hmac_guard));

    let auth = Router::new().route("/auth", post(login));

    let auth_protection = Router::new()
        .route("/errors/interval", get(get_interval_errors_count))
        .layer(middleware::from_fn(jwt_guard));

    hmac_protection.merge(auth_protection).merge(auth)
}
