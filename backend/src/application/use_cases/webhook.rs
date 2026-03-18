use thiserror::Error;
use tracing::{error, info};

use crate::application::ports::{PaymentProvider, SubscriptionStore, UserStore};
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
    subscription_store: &dyn SubscriptionStore,
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
                .provider_customer_id
                .as_deref()
                .ok_or_else(|| WebhookError::MissingField("provider_customer_id".into()))?;

            let user = user_store
                .find_by_email(email)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?
                .ok_or_else(|| {
                    error!(event = "webhook_user_not_found", email = %email);
                    WebhookError::Internal(format!("no user found for email {email}"))
                })?;

            // Use the provider's reported subscription status if available,
            // rather than blindly assuming active (covers deferred payments like ACH/SEPA)
            let status = match event.subscription_status.as_deref() {
                Some(s) => map_stripe_status(Some(s)),
                None => SubscriptionStatus::Active,
            };

            subscription_store
                .update_subscription(&user.id, customer_id, status, event.plan_id)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?;

            info!(user_id = %user.id, status = ?event.subscription_status, "subscription set via checkout");
        }
        "customer.subscription.updated" => {
            let customer_id = event
                .provider_customer_id
                .as_deref()
                .ok_or_else(|| WebhookError::MissingField("provider_customer_id".into()))?;

            let user = subscription_store
                .find_by_provider_customer_id(customer_id)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?
                .ok_or_else(|| {
                    error!(event = "webhook_customer_not_found", customer_id = %customer_id);
                    WebhookError::Internal(format!("no user for customer {customer_id}"))
                })?;

            let status = map_stripe_status(event.subscription_status.as_deref());

            subscription_store
                .update_subscription(&user.id, customer_id, status, event.plan_id)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?;

            info!(user_id = %user.id, "subscription updated");
        }
        "customer.subscription.deleted" => {
            let customer_id = event
                .provider_customer_id
                .as_deref()
                .ok_or_else(|| WebhookError::MissingField("provider_customer_id".into()))?;

            let user = subscription_store
                .find_by_provider_customer_id(customer_id)
                .await
                .map_err(|e| WebhookError::Internal(e.to_string()))?
                .ok_or_else(|| {
                    error!(event = "webhook_customer_not_found", customer_id = %customer_id);
                    WebhookError::Internal(format!("no user for customer {customer_id}"))
                })?;

            subscription_store
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{
        BillingEvent, PaymentError, PaymentProvider, StoreError, SubscriptionStore, UserStore,
    };
    use crate::domain::entities::{SubscriptionStatus, User};
    use crate::domain::value_objects::UserId;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;

    // --- Mock PaymentProvider ---

    struct MockPaymentProvider {
        event: Result<BillingEvent, PaymentError>,
    }

    #[async_trait]
    impl PaymentProvider for MockPaymentProvider {
        async fn create_checkout_session(
            &self,
            _email: &str,
            _price_id: &str,
            _success_url: &str,
            _cancel_url: &str,
        ) -> Result<String, PaymentError> {
            unimplemented!()
        }

        fn verify_webhook_signature(
            &self,
            _payload: &str,
            _signature: &str,
        ) -> Result<BillingEvent, PaymentError> {
            match &self.event {
                Ok(e) => Ok(BillingEvent {
                    event_type: e.event_type.clone(),
                    provider_customer_id: e.provider_customer_id.clone(),
                    customer_email: e.customer_email.clone(),
                    subscription_status: e.subscription_status.clone(),
                    plan_id: e.plan_id.clone(),
                }),
                Err(_) => Err(PaymentError::InvalidSignature),
            }
        }
    }

    // --- Mock UserStore ---

    struct MockUserStore {
        user: Option<User>,
    }

    #[async_trait]
    impl UserStore for MockUserStore {
        async fn create_user(&self, _user: &User) -> Result<(), StoreError> {
            unimplemented!()
        }
        async fn find_by_email(&self, _email: &str) -> Result<Option<User>, StoreError> {
            Ok(self.user.clone())
        }
        async fn set_email_verified(&self, _user_id: &UserId) -> Result<(), StoreError> {
            unimplemented!()
        }
        async fn update_password(&self, _user_id: &UserId, _hash: &str) -> Result<(), StoreError> {
            unimplemented!()
        }
        async fn find_by_id(&self, _user_id: &UserId) -> Result<Option<User>, StoreError> {
            Ok(self.user.clone())
        }
    }

    // --- Mock SubscriptionStore ---

    #[derive(Debug, Clone)]
    struct SubscriptionUpdate {
        user_id: String,
        customer_id: String,
        status: SubscriptionStatus,
        plan: Option<String>,
    }

    struct MockSubscriptionStore {
        user: Option<User>,
        updates: Mutex<Vec<SubscriptionUpdate>>,
    }

    #[async_trait]
    impl SubscriptionStore for MockSubscriptionStore {
        async fn update_subscription(
            &self,
            user_id: &UserId,
            provider_customer_id: &str,
            status: SubscriptionStatus,
            plan: Option<String>,
        ) -> Result<(), StoreError> {
            self.updates.lock().unwrap().push(SubscriptionUpdate {
                user_id: user_id.to_string(),
                customer_id: provider_customer_id.to_string(),
                status,
                plan,
            });
            Ok(())
        }

        async fn find_by_provider_customer_id(
            &self,
            _customer_id: &str,
        ) -> Result<Option<User>, StoreError> {
            Ok(self.user.clone())
        }
    }

    fn make_user(id: &str, email: &str) -> User {
        User {
            id: UserId::new(id),
            email: email.to_string(),
            password_hash: "hash".to_string(),
            email_verified: true,
            stripe_customer_id: Some("cus_123".to_string()),
            subscription_status: SubscriptionStatus::None,
            subscription_plan: None,
            password_changed_at: None,
            created_at: Utc::now(),
        }
    }

    // --- map_stripe_status tests ---

    #[test]
    fn maps_active_status() {
        assert_eq!(
            map_stripe_status(Some("active")),
            SubscriptionStatus::Active
        );
    }

    #[test]
    fn maps_past_due_status() {
        assert_eq!(
            map_stripe_status(Some("past_due")),
            SubscriptionStatus::PastDue
        );
    }

    #[test]
    fn maps_canceled_status() {
        assert_eq!(
            map_stripe_status(Some("canceled")),
            SubscriptionStatus::Cancelled
        );
    }

    #[test]
    fn maps_cancelled_british_spelling() {
        assert_eq!(
            map_stripe_status(Some("cancelled")),
            SubscriptionStatus::Cancelled
        );
    }

    #[test]
    fn maps_unknown_status_to_none() {
        assert_eq!(
            map_stripe_status(Some("trialing")),
            SubscriptionStatus::None
        );
    }

    #[test]
    fn maps_none_input_to_none_status() {
        assert_eq!(map_stripe_status(None), SubscriptionStatus::None);
    }

    // --- Webhook handler tests ---

    #[tokio::test]
    async fn returns_invalid_signature_when_verification_fails() {
        let provider = MockPaymentProvider {
            event: Err(PaymentError::InvalidSignature),
        };
        let user_store = MockUserStore { user: None };
        let sub_store = MockSubscriptionStore {
            user: None,
            updates: Mutex::new(Vec::new()),
        };

        let result =
            handle_stripe_webhook("payload", "bad-sig", &provider, &user_store, &sub_store).await;

        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[tokio::test]
    async fn checkout_completed_activates_subscription() {
        let user = make_user("user-1", "test@example.com");
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "checkout.session.completed".to_string(),
                provider_customer_id: Some("cus_123".to_string()),
                customer_email: Some("test@example.com".to_string()),
                subscription_status: Some("active".to_string()),
                plan_id: Some("price_solo".to_string()),
            }),
        };
        let user_store = MockUserStore {
            user: Some(user.clone()),
        };
        let sub_store = MockSubscriptionStore {
            user: Some(user),
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(result.is_ok());

        let updates = sub_store.updates.lock().unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].status, SubscriptionStatus::Active);
        assert_eq!(updates[0].plan, Some("price_solo".to_string()));
        assert_eq!(updates[0].customer_id, "cus_123");
    }

    #[tokio::test]
    async fn checkout_completed_defaults_to_active_when_no_status() {
        let user = make_user("user-1", "test@example.com");
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "checkout.session.completed".to_string(),
                provider_customer_id: Some("cus_123".to_string()),
                customer_email: Some("test@example.com".to_string()),
                subscription_status: None,
                plan_id: Some("price_starter".to_string()),
            }),
        };
        let user_store = MockUserStore {
            user: Some(user.clone()),
        };
        let sub_store = MockSubscriptionStore {
            user: Some(user),
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(result.is_ok());

        let updates = sub_store.updates.lock().unwrap();
        assert_eq!(updates[0].status, SubscriptionStatus::Active);
    }

    #[tokio::test]
    async fn checkout_completed_fails_when_email_missing() {
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "checkout.session.completed".to_string(),
                provider_customer_id: Some("cus_123".to_string()),
                customer_email: None,
                subscription_status: None,
                plan_id: None,
            }),
        };
        let user_store = MockUserStore { user: None };
        let sub_store = MockSubscriptionStore {
            user: None,
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(matches!(result, Err(WebhookError::MissingField(_))));
    }

    #[tokio::test]
    async fn checkout_completed_fails_when_user_not_found() {
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "checkout.session.completed".to_string(),
                provider_customer_id: Some("cus_123".to_string()),
                customer_email: Some("unknown@example.com".to_string()),
                subscription_status: None,
                plan_id: None,
            }),
        };
        let user_store = MockUserStore { user: None };
        let sub_store = MockSubscriptionStore {
            user: None,
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(matches!(result, Err(WebhookError::Internal(_))));
    }

    #[tokio::test]
    async fn subscription_updated_changes_status() {
        let user = make_user("user-1", "test@example.com");
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "customer.subscription.updated".to_string(),
                provider_customer_id: Some("cus_123".to_string()),
                customer_email: None,
                subscription_status: Some("past_due".to_string()),
                plan_id: Some("price_solo".to_string()),
            }),
        };
        let user_store = MockUserStore { user: None };
        let sub_store = MockSubscriptionStore {
            user: Some(user),
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(result.is_ok());

        let updates = sub_store.updates.lock().unwrap();
        assert_eq!(updates[0].status, SubscriptionStatus::PastDue);
    }

    #[tokio::test]
    async fn subscription_deleted_sets_cancelled() {
        let user = make_user("user-1", "test@example.com");
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "customer.subscription.deleted".to_string(),
                provider_customer_id: Some("cus_123".to_string()),
                customer_email: None,
                subscription_status: None,
                plan_id: None,
            }),
        };
        let user_store = MockUserStore { user: None };
        let sub_store = MockSubscriptionStore {
            user: Some(user),
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(result.is_ok());

        let updates = sub_store.updates.lock().unwrap();
        assert_eq!(updates[0].status, SubscriptionStatus::Cancelled);
        assert_eq!(updates[0].plan, None);
    }

    #[tokio::test]
    async fn subscription_deleted_fails_when_customer_not_found() {
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "customer.subscription.deleted".to_string(),
                provider_customer_id: Some("cus_unknown".to_string()),
                customer_email: None,
                subscription_status: None,
                plan_id: None,
            }),
        };
        let user_store = MockUserStore { user: None };
        let sub_store = MockSubscriptionStore {
            user: None,
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(matches!(result, Err(WebhookError::Internal(_))));
    }

    #[tokio::test]
    async fn unhandled_event_type_succeeds_silently() {
        let provider = MockPaymentProvider {
            event: Ok(BillingEvent {
                event_type: "invoice.payment_succeeded".to_string(),
                provider_customer_id: None,
                customer_email: None,
                subscription_status: None,
                plan_id: None,
            }),
        };
        let user_store = MockUserStore { user: None };
        let sub_store = MockSubscriptionStore {
            user: None,
            updates: Mutex::new(Vec::new()),
        };

        let result = handle_stripe_webhook("p", "s", &provider, &user_store, &sub_store).await;
        assert!(result.is_ok());
        assert!(sub_store.updates.lock().unwrap().is_empty());
    }
}
