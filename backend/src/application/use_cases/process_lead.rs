use chrono::Utc;
use tracing::info;

use crate::application::ports::{
    AiClient, AiError, PricingStore, QuoteStore, StoreError, UsageStore, UserStore,
};
use crate::domain::entities::*;
use crate::domain::quota::{current_billing_period, quota_for_price, QuotaLimit};
use crate::domain::value_objects::*;

#[derive(Debug, thiserror::Error)]
pub enum ProcessLeadError {
    #[error("pricing template not found for user")]
    TemplateNotFound,
    #[error("quota exceeded: {used}/{limit} quotes used this period")]
    QuotaExceeded { used: u32, limit: u32 },
    #[error("user not found")]
    UserNotFound,
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("AI error: {0}")]
    Ai(#[from] AiError),
}

pub struct ProcessLeadUseCase<'a> {
    pricing_store: &'a dyn PricingStore,
    ai_client: &'a dyn AiClient,
    quote_store: &'a dyn QuoteStore,
    usage_store: &'a dyn UsageStore,
    user_store: &'a dyn UserStore,
    allowed_price_ids: &'a [String],
}

impl<'a> ProcessLeadUseCase<'a> {
    pub fn new(
        pricing_store: &'a dyn PricingStore,
        ai_client: &'a dyn AiClient,
        quote_store: &'a dyn QuoteStore,
        usage_store: &'a dyn UsageStore,
        user_store: &'a dyn UserStore,
        allowed_price_ids: &'a [String],
    ) -> Self {
        Self {
            pricing_store,
            ai_client,
            quote_store,
            usage_store,
            user_store,
            allowed_price_ids,
        }
    }

    pub async fn execute(
        &self,
        lead: &Lead,
        tone: ToneOption,
    ) -> Result<QuoteDraft, ProcessLeadError> {
        self.check_quota(&lead.user_id).await?;

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
            user_id: lead.user_id.clone(),
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
            .generate_follow_up(&quote.job_summary, &quote, &tone, &template.currency)
            .await?;
        quote.follow_up_message = follow_up;

        if !quote.job_summary.missing_info.is_empty() {
            quote.clarification_message =
                Some(build_clarification_message(&quote.job_summary.missing_info));
        }

        self.quote_store.save_quote(&quote).await?;

        let (period_start, _) = current_billing_period(Utc::now());
        self.usage_store
            .increment_quote_count(&lead.user_id, period_start)
            .await?;

        info!(
            event = "quota_incremented",
            user_id = %lead.user_id,
        );

        Ok(quote)
    }

    async fn check_quota(&self, user_id: &UserId) -> Result<(), ProcessLeadError> {
        let user = self
            .user_store
            .find_by_id(user_id)
            .await?
            .ok_or(ProcessLeadError::UserNotFound)?;

        let price_id = user.subscription_plan.as_deref().unwrap_or("");
        let limit = quota_for_price(price_id, self.allowed_price_ids);

        let max = match limit {
            QuotaLimit::Unlimited => return Ok(()),
            QuotaLimit::Limited(n) => n,
        };

        let (period_start, period_end) = current_billing_period(Utc::now());
        let usage = self
            .usage_store
            .get_or_create_usage(user_id, period_start, period_end)
            .await?;

        if usage.quote_count >= max {
            return Err(ProcessLeadError::QuotaExceeded {
                used: usage.quote_count,
                limit: max,
            });
        }

        Ok(())
    }
}

pub(crate) fn calculate_price(
    summary: &JobSummary,
    template: &PricingTemplate,
) -> (f64, Vec<PriceLineItem>) {
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

pub(crate) fn build_clarification_message(missing_info: &[String]) -> String {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn make_template(categories: Vec<(&str, f64)>, minimum_callout: f64) -> PricingTemplate {
        PricingTemplate {
            id: TemplateId::generate(),
            user_id: UserId::new("test-user"),
            currency: "GBP".to_string(),
            country: "UK".to_string(),
            minimum_callout,
            categories: categories
                .into_iter()
                .enumerate()
                .map(|(i, (name, price))| ServiceCategory {
                    id: format!("cat-{}", i),
                    name: name.to_string(),
                    base_price: price,
                    description: String::new(),
                })
                .collect(),
            add_ons: Vec::new(),
            custom_notes: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_summary(service_type: &str) -> JobSummary {
        JobSummary {
            service_type: service_type.to_string(),
            property_size: None,
            requested_date: None,
            requested_time: None,
            missing_info: Vec::new(),
            extracted_details: HashMap::new(),
        }
    }

    #[test]
    fn calculates_price_from_matching_category() {
        let template = make_template(vec![("Deep Clean", 120.0), ("Regular Clean", 80.0)], 0.0);
        let summary = make_summary("Deep Clean");

        let (total, breakdown) = calculate_price(&summary, &template);

        assert_eq!(total, 120.0);
        assert_eq!(breakdown.len(), 1);
        assert_eq!(breakdown[0].description, "Deep Clean");
        assert_eq!(breakdown[0].amount, 120.0);
    }

    #[test]
    fn applies_minimum_callout_when_price_below() {
        let template = make_template(vec![("Quick Tidy", 20.0)], 50.0);
        let summary = make_summary("Quick Tidy");

        let (total, breakdown) = calculate_price(&summary, &template);

        assert_eq!(total, 50.0);
        assert_eq!(breakdown.len(), 2);
        assert_eq!(breakdown[0].amount, 20.0);
        assert_eq!(breakdown[1].description, "Minimum callout fee");
        assert_eq!(breakdown[1].amount, 30.0);
    }

    #[test]
    fn returns_zero_when_no_category_matches() {
        let template = make_template(vec![("Deep Clean", 120.0)], 0.0);
        let summary = make_summary("Window Washing");

        let (total, breakdown) = calculate_price(&summary, &template);

        assert_eq!(total, 0.0);
        assert!(breakdown.is_empty());
    }

    #[test]
    fn case_insensitive_category_matching() {
        let template = make_template(vec![("Deep Clean", 120.0)], 0.0);
        let summary = make_summary("deep clean");

        let (total, breakdown) = calculate_price(&summary, &template);

        assert_eq!(total, 120.0);
        assert_eq!(breakdown.len(), 1);
    }

    #[test]
    fn formats_single_missing_item() {
        let missing = vec!["property size".to_string()];

        let message = build_clarification_message(&missing);

        assert_eq!(
            message,
            "Before I can give you a final quote, I need a few more details:\n1. property size"
        );
    }

    #[test]
    fn formats_multiple_missing_items() {
        let missing = vec![
            "property size".to_string(),
            "preferred date".to_string(),
            "access instructions".to_string(),
        ];

        let message = build_clarification_message(&missing);

        assert_eq!(
            message,
            "Before I can give you a final quote, I need a few more details:\n\
             1. property size\n\
             2. preferred date\n\
             3. access instructions"
        );
    }
}
