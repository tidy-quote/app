use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use tracing::info;

use crate::application::ports::{EmailError, EmailSender, StoreError, TokenStore, UserStore};
use crate::domain::entities::{TokenPurpose, VerificationToken};

const TOKEN_EXPIRY_HOURS: i64 = 1;
const BCRYPT_COST: u32 = 12;
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 72;

#[derive(Debug, thiserror::Error)]
pub enum PasswordResetError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("email send failed: {0}")]
    Email(#[from] EmailError),
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("password must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH} characters")]
    InvalidPassword,
    #[error("password hashing failed: {0}")]
    HashError(String),
}

fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn send_reset_email(
    email: &str,
    user_store: &dyn UserStore,
    email_sender: &dyn EmailSender,
    token_store: &dyn TokenStore,
    app_base_url: &str,
) -> Result<(), PasswordResetError> {
    let user = user_store.find_by_email(email).await?;

    let Some(user) = user else {
        // Don't leak whether the email exists — sleep to match timing of the happy path
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return Ok(());
    };

    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = hash_token(&raw_token);

    let token = VerificationToken {
        user_id: user.id.clone(),
        token_hash,
        purpose: TokenPurpose::PasswordReset,
        expires_at: Utc::now() + Duration::hours(TOKEN_EXPIRY_HOURS),
        used: false,
    };

    token_store.store_token(&token).await?;

    let reset_url = format!("{}/reset-password?token={}", app_base_url, raw_token);
    let html_body = format!(
        "<p>You requested a password reset for your Tidy Quote account.</p>\
         <p>Click the link below to reset your password:</p>\
         <p><a href=\"{url}\">{url}</a></p>\
         <p>This link expires in {hours} hour(s). If you didn't request this, ignore this email.</p>",
        url = reset_url,
        hours = TOKEN_EXPIRY_HOURS,
    );

    email_sender
        .send_email(email, "Reset your Tidy Quote password", &html_body)
        .await?;

    info!(event = "reset_email_sent", user_id = %user.id);

    Ok(())
}

pub async fn reset_password(
    raw_token: &str,
    new_password: &str,
    user_store: &dyn UserStore,
    token_store: &dyn TokenStore,
) -> Result<(), PasswordResetError> {
    if new_password.len() < MIN_PASSWORD_LENGTH || new_password.len() > MAX_PASSWORD_LENGTH {
        return Err(PasswordResetError::InvalidPassword);
    }

    let token_hash = hash_token(raw_token);

    let token = token_store
        .find_valid_token(&token_hash, TokenPurpose::PasswordReset)
        .await?
        .ok_or(PasswordResetError::InvalidToken)?;

    let password_hash = bcrypt::hash(new_password, BCRYPT_COST)
        .map_err(|e| PasswordResetError::HashError(e.to_string()))?;

    user_store
        .update_password(&token.user_id, &password_hash)
        .await?;

    token_store.mark_token_used(&token_hash).await?;

    info!(event = "password_reset", user_id = %token.user_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_token_produces_consistent_hex() {
        let hash1 = hash_token("test-token");
        let hash2 = hash_token("test-token");
        assert_eq!(hash1, hash2);
    }
}
