pub mod auth {
    use std::time::{SystemTime, UNIX_EPOCH};
    use async_trait::async_trait;
    use axum::extract::{FromRequest, Request};
    use axum::http::StatusCode;
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        sub: String,
        exp: usize,
    }

    fn create_jwt(user_id: &str, secret: &str) -> String {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600) as usize,
        };
        let header = Header::new(Algorithm::HS256);
        encode(&header, &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
    }

    fn validate_jwt(token: &str, secret: &str) -> bool {
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::default();

        decode::<Claims>(token, &decoding_key, &validation).is_ok()
    }


    pub async fn login() {
        println!("login");
    }
    pub async fn registration() {
        println!("reg");
    }

    #[cfg(test)]
    mod test {
        use crate::models::http_client::api::handlers::auth::{create_jwt, validate_jwt};

        #[test]
        fn jwt_test() {
            let secret = "Test";

            let jwt = create_jwt("1", secret);

            let res = validate_jwt(&jwt, secret);

            assert_eq!(res, true);
        }
    }
}