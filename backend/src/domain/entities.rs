use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::value_objects::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    #[default]
    None,
    Active,
    Cancelled,
    PastDue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub stripe_customer_id: Option<String>,
    #[serde(default)]
    pub subscription_status: SubscriptionStatus,
    #[serde(default)]
    pub subscription_plan: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationToken {
    pub user_id: UserId,
    pub token_hash: String,
    pub purpose: TokenPurpose,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenPurpose {
    EmailVerification,
    PasswordReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCategory {
    pub id: String,
    pub name: String,
    pub base_price: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddOn {
    pub id: String,
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingTemplate {
    pub id: TemplateId,
    pub user_id: UserId,
    pub currency: String,
    pub country: String,
    pub minimum_callout: f64,
    pub categories: Vec<ServiceCategory>,
    pub add_ons: Vec<AddOn>,
    pub custom_notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToneOption {
    Friendly,
    Direct,
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lead {
    pub id: LeadId,
    pub user_id: UserId,
    pub raw_text: Option<String>,
    pub image_data: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    pub service_type: String,
    pub property_size: Option<String>,
    pub requested_date: Option<String>,
    pub requested_time: Option<String>,
    pub missing_info: Vec<String>,
    pub extracted_details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLineItem {
    pub description: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDraft {
    pub id: QuoteId,
    pub lead_id: LeadId,
    #[serde(default)]
    pub user_id: UserId,
    pub job_summary: JobSummary,
    pub estimated_price: f64,
    pub price_breakdown: Vec<PriceLineItem>,
    pub assumptions: Vec<String>,
    pub follow_up_message: String,
    pub clarification_message: Option<String>,
    pub tone: ToneOption,
    pub created_at: DateTime<Utc>,
}
