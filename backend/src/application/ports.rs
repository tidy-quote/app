use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::entities::*;
use crate::domain::value_objects::*;

#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("payment provider error: {0}")]
    ProviderError(String),
    #[error("invalid webhook signature")]
    InvalidSignature,
}

pub struct BillingEvent {
    pub event_type: String,
    pub provider_customer_id: Option<String>,
    pub customer_email: Option<String>,
    pub subscription_status: Option<String>,
    pub plan_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("failed to send email: {0}")]
    SendFailed(String),
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("document not found: {0}")]
    NotFound(String),
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("store operation failed: {0}")]
    Internal(String),
    #[error("duplicate email: {0}")]
    DuplicateEmail(String),
    #[error("quota exceeded: {used}/{limit}")]
    QuotaExceeded { used: u32, limit: u32 },
}

#[derive(Debug, Error)]
pub enum AiError {
    #[error("AI API request failed: {0}")]
    RequestFailed(String),
    #[error("failed to parse AI response: {0}")]
    ParseError(String),
    #[error("AI rate limit exceeded")]
    RateLimited,
    #[error("AI configuration error: {0}")]
    Configuration(String),
}

#[async_trait]
pub trait PricingStore: Send + Sync {
    async fn get_template(&self, user_id: &UserId) -> Result<Option<PricingTemplate>, StoreError>;
    async fn save_template(&self, template: &PricingTemplate) -> Result<(), StoreError>;
}

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), StoreError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, StoreError>;
    async fn set_email_verified(&self, user_id: &UserId) -> Result<(), StoreError>;
    async fn update_password(
        &self,
        user_id: &UserId,
        password_hash: &str,
    ) -> Result<(), StoreError>;
    async fn find_by_id(&self, user_id: &UserId) -> Result<Option<User>, StoreError>;
}

#[async_trait]
pub trait SubscriptionStore: Send + Sync {
    async fn update_subscription(
        &self,
        user_id: &UserId,
        provider_customer_id: &str,
        status: SubscriptionStatus,
        plan: Option<String>,
    ) -> Result<(), StoreError>;
    async fn find_by_provider_customer_id(
        &self,
        customer_id: &str,
    ) -> Result<Option<User>, StoreError>;
}

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn store_token(&self, token: &VerificationToken) -> Result<(), StoreError>;
    async fn find_valid_token(
        &self,
        token_hash: &str,
        purpose: TokenPurpose,
    ) -> Result<Option<VerificationToken>, StoreError>;
    async fn mark_token_used(&self, token_hash: &str) -> Result<(), StoreError>;
}

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, html_body: &str) -> Result<(), EmailError>;
}

#[async_trait]
pub trait QuoteStore: Send + Sync {
    async fn save_quote(&self, quote: &QuoteDraft) -> Result<(), StoreError>;
    async fn list_quotes(
        &self,
        user_id: &UserId,
        page: u32,
        limit: u32,
    ) -> Result<Vec<QuoteDraft>, StoreError>;
    async fn get_quote(
        &self,
        quote_id: &QuoteId,
        user_id: &UserId,
    ) -> Result<Option<QuoteDraft>, StoreError>;
}

#[async_trait]
pub trait UsageStore: Send + Sync {
    async fn get_or_create_usage(
        &self,
        user_id: &UserId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<UsageRecord, StoreError>;

    /// Atomically increment quote_count and return the new value.
    /// If the new count exceeds `limit`, the increment is rolled back and `QuotaExceeded` is returned.
    async fn increment_and_check_quota(
        &self,
        user_id: &UserId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: Option<u32>,
    ) -> Result<u32, StoreError>;
}

#[async_trait]
pub trait AiClient: Send + Sync {
    async fn extract_job_details(
        &self,
        lead: &Lead,
        template: &PricingTemplate,
    ) -> Result<JobSummary, AiError>;

    async fn generate_follow_up(
        &self,
        summary: &JobSummary,
        quote: &QuoteDraft,
        tone: &ToneOption,
        currency: &str,
    ) -> Result<String, AiError>;
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_checkout_session(
        &self,
        customer_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<String, PaymentError>;

    fn verify_webhook_signature(
        &self,
        payload: &str,
        signature: &str,
    ) -> Result<BillingEvent, PaymentError>;
}
