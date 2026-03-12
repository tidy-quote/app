use chrono::Utc;

use crate::application::ports::{AiClient, AiError, PricingStore, StoreError};
use crate::domain::entities::*;
use crate::domain::value_objects::*;

#[derive(Debug, thiserror::Error)]
pub enum ProcessLeadError {
    #[error("pricing template not found for user")]
    TemplateNotFound,
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("AI error: {0}")]
    Ai(#[from] AiError),
}

pub struct ProcessLeadUseCase<'a> {
    pricing_store: &'a dyn PricingStore,
    ai_client: &'a dyn AiClient,
}

impl<'a> ProcessLeadUseCase<'a> {
    pub fn new(pricing_store: &'a dyn PricingStore, ai_client: &'a dyn AiClient) -> Self {
        Self {
            pricing_store,
            ai_client,
        }
    }

    pub async fn execute(
        &self,
        lead: &Lead,
        tone: ToneOption,
    ) -> Result<QuoteDraft, ProcessLeadError> {
        let template = self
            .pricing_store
            .get_template(&lead.user_id)
            .await?
            .ok_or(ProcessLeadError::TemplateNotFound)?;

        let job_summary = self.ai_client.extract_job_details(lead, &template).await?;

        let (estimated_price, price_breakdown) = calculate_price(&job_summary, &template);

        let quote_id = QuoteId::generate();

        let mut quote = QuoteDraft {
            id: quote_id,
            lead_id: lead.id.clone(),
            job_summary,
            estimated_price,
            price_breakdown,
            assumptions: Vec::new(),
            follow_up_message: String::new(),
            clarification_message: None,
            tone: tone.clone(),
            created_at: Utc::now(),
        };

        let follow_up = self
            .ai_client
            .generate_follow_up(&quote.job_summary, &quote, &tone)
            .await?;
        quote.follow_up_message = follow_up;

        if !quote.job_summary.missing_info.is_empty() {
            quote.clarification_message =
                Some(build_clarification_message(&quote.job_summary.missing_info));
        }

        Ok(quote)
    }
}

fn calculate_price(summary: &JobSummary, template: &PricingTemplate) -> (f64, Vec<PriceLineItem>) {
    let mut breakdown = Vec::new();
    let mut total = 0.0;

    let matching_category = template
        .categories
        .iter()
        .find(|c| c.name.to_lowercase() == summary.service_type.to_lowercase());

    if let Some(category) = matching_category {
        let item = PriceLineItem {
            description: category.name.clone(),
            amount: category.base_price,
        };
        total += item.amount;
        breakdown.push(item);
    }

    if total < template.minimum_callout {
        let callout_item = PriceLineItem {
            description: "Minimum callout fee".to_string(),
            amount: template.minimum_callout - total,
        };
        total = template.minimum_callout;
        breakdown.push(callout_item);
    }

    (total, breakdown)
}

fn build_clarification_message(missing_info: &[String]) -> String {
    let items: Vec<String> = missing_info
        .iter()
        .enumerate()
        .map(|(i, info)| format!("{}. {}", i + 1, info))
        .collect();

    format!(
        "Before I can give you a final quote, I need a few more details:\n{}",
        items.join("\n")
    )
}
