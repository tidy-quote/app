use thiserror::Error;

use crate::application::ports::{PaymentProvider, UserStore};
use crate::domain::quota::PlanConfig;
use crate::domain::value_objects::UserId;

#[derive(Debug, Error)]
pub enum CheckoutError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid price ID")]
    InvalidPriceId,
    #[error("payment provider error: {0}")]
    PaymentError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub async fn create_checkout(
    user_id: &UserId,
    price_id: &str,
    user_store: &dyn UserStore,
    payment_provider: &dyn PaymentProvider,
    app_base_url: &str,
    plan_config: &PlanConfig,
) -> Result<String, CheckoutError> {
    if !plan_config.contains(price_id) {
        return Err(CheckoutError::InvalidPriceId);
    }

    let user = user_store
        .find_by_id(user_id)
        .await
        .map_err(|e| CheckoutError::Internal(e.to_string()))?
        .ok_or(CheckoutError::UserNotFound)?;

    let success_url = format!("{app_base_url}/checkout-success");
    let cancel_url = format!("{app_base_url}/choose-plan");

    let checkout_url = payment_provider
        .create_checkout_session(&user.email, price_id, &success_url, &cancel_url)
        .await
        .map_err(|e| CheckoutError::PaymentError(e.to_string()))?;

    Ok(checkout_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{
        BillingEvent, PaymentError, PaymentProvider, StoreError, UserStore,
    };
    use crate::domain::entities::{SubscriptionStatus, User};
    use async_trait::async_trait;
    use chrono::Utc;

    fn plan_config() -> PlanConfig {
        PlanConfig {
            starter_price_id: "price_starter".to_string(),
            solo_price_id: "price_solo".to_string(),
            pro_price_id: "price_pro".to_string(),
        }
    }

    fn make_user() -> User {
        User {
            id: UserId::new("user-1"),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            email_verified: true,
            stripe_customer_id: None,
            subscription_status: SubscriptionStatus::Active,
            subscription_plan: Some("price_solo".to_string()),
            password_changed_at: None,
            created_at: Utc::now(),
        }
    }

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

    struct MockPaymentProvider {
        url: Result<String, PaymentError>,
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
            match &self.url {
                Ok(url) => Ok(url.clone()),
                Err(_) => Err(PaymentError::ProviderError("stripe error".into())),
            }
        }

        fn verify_webhook_signature(
            &self,
            _payload: &str,
            _signature: &str,
        ) -> Result<BillingEvent, PaymentError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn rejects_unknown_price_id() {
        let user_store = MockUserStore {
            user: Some(make_user()),
        };
        let payment = MockPaymentProvider {
            url: Ok("https://checkout.stripe.com/session".into()),
        };

        let result = create_checkout(
            &UserId::new("user-1"),
            "price_unknown",
            &user_store,
            &payment,
            "https://app.example.com",
            &plan_config(),
        )
        .await;

        assert!(matches!(result, Err(CheckoutError::InvalidPriceId)));
    }

    #[tokio::test]
    async fn returns_checkout_url_for_valid_price() {
        let user_store = MockUserStore {
            user: Some(make_user()),
        };
        let payment = MockPaymentProvider {
            url: Ok("https://checkout.stripe.com/session_123".into()),
        };

        let result = create_checkout(
            &UserId::new("user-1"),
            "price_solo",
            &user_store,
            &payment,
            "https://app.example.com",
            &plan_config(),
        )
        .await;

        assert_eq!(result.unwrap(), "https://checkout.stripe.com/session_123");
    }

    #[tokio::test]
    async fn returns_error_when_user_not_found() {
        let user_store = MockUserStore { user: None };
        let payment = MockPaymentProvider {
            url: Ok("https://checkout.stripe.com/session".into()),
        };

        let result = create_checkout(
            &UserId::new("unknown"),
            "price_starter",
            &user_store,
            &payment,
            "https://app.example.com",
            &plan_config(),
        )
        .await;

        assert!(matches!(result, Err(CheckoutError::UserNotFound)));
    }

    #[tokio::test]
    async fn returns_payment_error_when_stripe_fails() {
        let user_store = MockUserStore {
            user: Some(make_user()),
        };
        let payment = MockPaymentProvider {
            url: Err(PaymentError::ProviderError("stripe down".into())),
        };

        let result = create_checkout(
            &UserId::new("user-1"),
            "price_pro",
            &user_store,
            &payment,
            "https://app.example.com",
            &plan_config(),
        )
        .await;

        assert!(matches!(result, Err(CheckoutError::PaymentError(_))));
    }
}
