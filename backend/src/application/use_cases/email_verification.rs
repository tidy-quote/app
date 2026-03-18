use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use tracing::info;

use crate::application::ports::{EmailError, EmailSender, StoreError, TokenStore, UserStore};
use crate::domain::entities::{TokenPurpose, VerificationToken};
use crate::domain::value_objects::UserId;

const TOKEN_EXPIRY_HOURS: i64 = 24;

#[derive(Debug, thiserror::Error)]
pub enum EmailVerificationError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("email send failed: {0}")]
    Email(#[from] EmailError),
    #[error("invalid or expired token")]
    InvalidToken,
}

fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn send_verification_email(
    user_id: &UserId,
    email: &str,
    email_sender: &dyn EmailSender,
    token_store: &dyn TokenStore,
    app_base_url: &str,
) -> Result<(), EmailVerificationError> {
    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = hash_token(&raw_token);

    let verification_token = VerificationToken {
        user_id: user_id.clone(),
        token_hash,
        purpose: TokenPurpose::EmailVerification,
        expires_at: Utc::now() + Duration::hours(TOKEN_EXPIRY_HOURS),
        used: false,
    };

    token_store.store_token(&verification_token).await?;

    let verify_url = format!("{}/verify?token={}", app_base_url, raw_token);
    let html_body = format!(
        "<p>Welcome to Tidy Quote!</p>\
         <p>Please verify your email by clicking the link below:</p>\
         <p><a href=\"{url}\">{url}</a></p>\
         <p>This link expires in {hours} hours.</p>",
        url = verify_url,
        hours = TOKEN_EXPIRY_HOURS,
    );

    email_sender
        .send_email(email, "Verify your Tidy Quote email", &html_body)
        .await?;

    info!(event = "verification_email_sent", user_id = %user_id);

    Ok(())
}

pub async fn verify_email(
    raw_token: &str,
    user_store: &dyn UserStore,
    token_store: &dyn TokenStore,
) -> Result<(), EmailVerificationError> {
    let token_hash = hash_token(raw_token);

    let token = token_store
        .find_valid_token(&token_hash, TokenPurpose::EmailVerification)
        .await?
        .ok_or(EmailVerificationError::InvalidToken)?;

    user_store.set_email_verified(&token.user_id).await?;
    token_store.mark_token_used(&token_hash).await?;

    info!(event = "email_verified", user_id = %token.user_id);

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
        assert_ne!(hash_token("other"), hash1);
    }
}
