use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::*;
use crate::domain::value_objects::*;

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
    ) -> Result<String, AiError>;
}
