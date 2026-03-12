use std::env;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::application::ports::{StoreError, UserStore};
use crate::domain::entities::User;
use crate::domain::value_objects::UserId;

const BCRYPT_COST: u32 = 12;
const TOKEN_EXPIRY_HOURS: i64 = 24;
const JWT_SECRET_ENV: &str = "JWT_SECRET";

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("email already registered")]
    EmailTaken,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("password hashing failed: {0}")]
    HashError(String),
    #[error("token generation failed: {0}")]
    TokenError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
}

pub struct AuthResult {
    pub token: String,
    pub user_id: String,
    pub email: String,
}

pub struct AuthUseCase<'a> {
    user_store: &'a dyn UserStore,
}

impl<'a> AuthUseCase<'a> {
    pub fn new(user_store: &'a dyn UserStore) -> Self {
        Self { user_store }
    }

    pub async fn signup(&self, email: &str, password: &str) -> Result<AuthResult, AuthError> {
        let password_hash = bcrypt::hash(password, BCRYPT_COST)
            .map_err(|e| AuthError::HashError(e.to_string()))?;

        let user = User {
            id: UserId::generate(),
            email: email.to_string(),
            password_hash,
            created_at: Utc::now(),
        };

        self.user_store.create_user(&user).await.map_err(|e| {
            if matches!(e, StoreError::DuplicateEmail(_)) {
                AuthError::EmailTaken
            } else {
                AuthError::Store(e)
            }
        })?;

        let token = generate_token(user.id.as_str(), &user.email)?;

        Ok(AuthResult {
            token,
            user_id: user.id.to_string(),
            email: user.email,
        })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResult, AuthError> {
        let user = self
            .user_store
            .find_by_email(email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let valid =
            bcrypt::verify(password, &user.password_hash).unwrap_or(false);

        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        let token = generate_token(user.id.as_str(), &user.email)?;

        Ok(AuthResult {
            token,
            user_id: user.id.to_string(),
            email: user.email,
        })
    }
}

fn jwt_secret() -> String {
    env::var(JWT_SECRET_ENV).unwrap_or_else(|_| "dev-secret-do-not-use-in-production".to_string())
}

fn generate_token(user_id: &str, email: &str) -> Result<String, AuthError> {
    let expiration = Utc::now() + Duration::hours(TOKEN_EXPIRY_HOURS);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AuthError::TokenError(e.to_string()))
}

pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AuthError::TokenError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_and_validates_token() {
        env::set_var(JWT_SECRET_ENV, "test-secret");
        let token = generate_token("user-123", "test@example.com").unwrap();
        let claims = validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn rejects_invalid_token() {
        env::set_var(JWT_SECRET_ENV, "test-secret");
        let result = validate_token("invalid-token");
        assert!(result.is_err());
    }
}
