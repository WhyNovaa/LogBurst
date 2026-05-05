use crate::config::rest::ServerConfig;
use crate::interfaces::api::AppState;
use crate::interfaces::api::middlewares::auth::{JwtState, jwt_guard};
use crate::interfaces::api::middlewares::hmac::{HmacState, hmac_guard};
use crate::interfaces::api::v1::add_log::add_log;
use crate::interfaces::api::v1::auth::login;
use crate::security::keystore::KeyStore;
use axum::routing::post;
use axum::{Extension, Router, middleware};
use std::sync::Arc;

mod add_log;
mod analytics;
pub mod auth;
pub mod dto;

pub fn routes(cfg: ServerConfig, key_store: Arc<KeyStore>) -> Router<AppState> {
    let hmac_state = HmacState { key_store };
    let jwt_state = JwtState {
        secret_key: cfg.secret_key.clone(),
    };

    let hmac_protection = Router::new()
        .route("/logs", post(add_log))
        .layer(middleware::from_fn_with_state(hmac_state, hmac_guard));

    let auth = Router::new()
        .route("/auth", post(login))
        .layer(Extension(Arc::new(cfg)));

    let auth_protection = Router::new()
        .merge(analytics::routes())
        .layer(middleware::from_fn_with_state(jwt_state, jwt_guard));

    hmac_protection.merge(auth_protection).merge(auth)
}
