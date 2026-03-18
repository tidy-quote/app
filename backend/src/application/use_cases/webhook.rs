use thiserror::Error;
use tracing::{error, info};

use crate::application::ports::{PaymentProvider, UserStore};
use crate::domain::entities::SubscriptionStatus;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("invalid signature")]
    InvalidSignature,
    #[error("unhandled event type: {0}")]
    UnhandledEvent(String),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub async fn handle_stripe_webhook(
    payload: &str,
    signature: &str,
    payment_provider: &dyn PaymentProvider,
    user_store: &dyn UserStore,
) -> Result<(), WebhookError> {
    let event = payment_provider
        .verify_webhook_signature(payload, signature)
        .map_err(|_| WebhookError::InvalidSignature)?;

    info!(event_type = %event.event_type, "processing stripe webhook");

    match event.event_type.as_str() {
        "checkout.session.completed" => {
            let email = event
                .customer_email
                .as_deref()
                .ok_or_else(|| WebhookError::MissingField("customer_email".into()))?;

            let customer_id = event
                .customer_id
                .as_deref()
                .ok_or_else(|| WebhookError::MissingField("customer_id".into()))?;

            let user = user_store
                .find_by_email(email)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?
                .ok_or_else(|| {
                    error!(event = "webhook_user_not_found", email = %email);
                    WebhookError::Internal(format!("no user found for email {email}"))
                })?;

            user_store
                .update_subscription(
                    &user.id,
                    customer_id,
                    SubscriptionStatus::Active,
                    event.price_id,
                )
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?;

            info!(user_id = %user.id, "subscription activated via checkout");
        }
        "customer.subscription.updated" => {
            let customer_id = event
                .customer_id
                .as_deref()
                .ok_or_else(|| WebhookError::MissingField("customer_id".into()))?;

            let user = user_store
                .find_by_stripe_customer_id(customer_id)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?
                .ok_or_else(|| {
                    error!(event = "webhook_customer_not_found", customer_id = %customer_id);
                    WebhookError::Internal(format!("no user for customer {customer_id}"))
                })?;

            let status = map_stripe_status(event.subscription_status.as_deref());

            user_store
                .update_subscription(&user.id, customer_id, status, event.price_id)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?;

            info!(user_id = %user.id, "subscription updated");
        }
        "customer.subscription.deleted" => {
            let customer_id = event
                .customer_id
                .as_deref()
                .ok_or_else(|| WebhookError::MissingField("customer_id".into()))?;

            let user = user_store
                .find_by_stripe_customer_id(customer_id)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?
                .ok_or_else(|| {
                    error!(event = "webhook_customer_not_found", customer_id = %customer_id);
                    WebhookError::Internal(format!("no user for customer {customer_id}"))
                })?;

            user_store
                .update_subscription(&user.id, customer_id, SubscriptionStatus::Cancelled, None)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?;

            info!(user_id = %user.id, "subscription cancelled");
        }
        other => {
            info!(event_type = %other, "ignoring unhandled stripe event");
        }
    }

    Ok(())
}

fn map_stripe_status(status: Option<&str>) -> SubscriptionStatus {
    match status {
        Some("active") => SubscriptionStatus::Active,
        Some("past_due") => SubscriptionStatus::PastDue,
        Some("canceled") | Some("cancelled") => SubscriptionStatus::Cancelled,
        _ => SubscriptionStatus::None,
    }
}
