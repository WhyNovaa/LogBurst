use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::TypedHeader;
use headers::Authorization;
use headers::authorization::Bearer;
use serde::{Deserialize, Serialize};
use crate::handlers::errors::AuthError;
use crate::handlers::jwt::validate_jwt;
use crate::models::http_client::role::Role;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub login: String,
    pub role: Role,
    pub exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Sync + Send + 'static,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let claims = validate_jwt(bearer.token())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(claims)
    }
}