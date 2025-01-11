use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::Error as JWTError, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub email: String,
    pub is_admin: bool,
}
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64,
}

impl JwtConfig {
    pub fn new(secret: String, expiration: i64) -> Self {
        Self { secret, expiration }
    }

    pub fn generate_token(
        &self,
        user_id: Uuid,
        email: String,
        is_admin: bool,
    ) -> Result<String, JWTError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::seconds(self.expiration))
            .expect("Invalid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id,
            exp: expiration as usize,
            email,
            is_admin,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JWTError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )?;

        Ok(token_data.claims)
    }
}
