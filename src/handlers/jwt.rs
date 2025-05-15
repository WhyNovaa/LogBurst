use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use crate::models::http_client::claims::Claims;
use crate::models::http_client::role::Role;

/// Key for encoding/decoding
static SECRET: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("SECRET").expect("SECRET not found in .env file")
});

pub fn create_jwt(login: String, role: Role) -> anyhow::Result<String> {
    create_jwt_with_key(login, role, &SECRET)
}

pub fn validate_jwt(token: &str) -> anyhow::Result<Claims> {
    validate_jwt_with_key(token, &SECRET)
}

fn create_jwt_with_key(login: String, role: Role, key: &str) -> anyhow::Result<String> {
    log::info!("Creating jwt fot user: {login}");

    let claims = Claims {
        login,
        role,
        exp: (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600) as usize,
    };
    let header = Header::new(Algorithm::HS256);

    Ok(encode(&header, &claims, &EncodingKey::from_secret(key.as_bytes()))?)
}

fn validate_jwt_with_key(token: &str, key: &str) -> anyhow::Result<Claims> {
    log::info!("Validating jwt: {token}");

    let decoding_key = DecodingKey::from_secret(key.as_bytes());
    let validation = Validation::default();

    Ok(decode::<Claims>(token, &decoding_key, &validation)?.claims)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn jwt_correct_validation() {
        let (login, role, key) = ("login".to_string(), Role::User, "key");

        let jwt = create_jwt_with_key(login, role, key).unwrap();

        let res = validate_jwt_with_key(&jwt, key);

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn jwt_incorrect_validation() {
        let (login, role) = ("login".to_string(), Role::User);
        let (key, invalid_key) = ("key", "invalid_key");

        let jwt = create_jwt_with_key(login, role,  key).unwrap();

        let res = validate_jwt_with_key(&jwt, invalid_key);

        assert!(res.is_err());
    }
}