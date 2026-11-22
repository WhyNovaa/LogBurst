mod dto;
mod routes;

use crate::interfaces::http::middleware::auth::{JwtState, jwt_guard};
use crate::server::Server;
use axum::{Extension, Router, middleware};
use config::jwt::JwtConfig;
use std::sync::Arc;

pub fn routes(config: JwtConfig) -> Router<Arc<Server>> {
    let jwt_state = JwtState {
        secret_key: config.secret_key.clone(),
    };

    Router::new()
        .nest("/analytics", routes::analytics_routes())
        .route_layer(middleware::from_fn_with_state(jwt_state, jwt_guard))
        .merge(routes::auth_routes().layer(Extension(Arc::new(config))))
}
