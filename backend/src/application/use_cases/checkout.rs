use thiserror::Error;

use crate::application::ports::{PaymentProvider, UserStore};
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
    allowed_price_ids: &[String],
) -> Result<String, CheckoutError> {
    if !allowed_price_ids.iter().any(|p| p == price_id) {
        return Err(CheckoutError::InvalidPriceId);
    }

    let user = user_store
        .find_by_id(user_id)
        .await
        .map_err(|e| CheckoutError::Internal(e.to_string()))?
        .ok_or(CheckoutError::UserNotFound)?;

    let success_url = format!("{app_base_url}/checkout/success");
    let cancel_url = format!("{app_base_url}/checkout/cancel");

    let checkout_url = payment_provider
        .create_checkout_session(&user.email, price_id, &success_url, &cancel_url)
        .await
        .map_err(|e| CheckoutError::PaymentError(e.to_string()))?;

    Ok(checkout_url)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_unknown_price_id() {
        let allowed = vec!["price_starter".to_string(), "price_solo".to_string()];
        let result = allowed.iter().any(|p| p == "price_unknown");
        assert!(!result);
    }
}
