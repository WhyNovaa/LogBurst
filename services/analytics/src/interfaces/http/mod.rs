mod middleware;
mod v1;

#[cfg(test)]
mod tests;

use crate::server::Server;
use axum::Router;
use config::http::HttpConfig;
use config::jwt::JwtConfig;
use std::sync::Arc;

pub fn router(server: Arc<Server>, jwt: JwtConfig) -> Router {
    Router::new()
        .nest("/v1", v1::routes(jwt))
        .with_state(server)
}

pub async fn run(server: Arc<Server>, config: HttpConfig, jwt: JwtConfig) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(config.url()).await?;
    let token = server.token.clone();
    axum::serve(listener, router(server, jwt))
        .with_graceful_shutdown(token.cancelled_owned())
        .await?;
    Ok(())
}
