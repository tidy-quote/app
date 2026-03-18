use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::error;

use crate::application::ports::{PaymentError, PaymentProvider, StripeEvent};

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";
const WEBHOOK_TOLERANCE_SECONDS: i64 = 300;

pub struct StripeClient {
    secret_key: String,
    webhook_secret: String,
    http_client: reqwest::Client,
}

impl StripeClient {
    pub fn new(secret_key: String, webhook_secret: String) -> Self {
        Self {
            secret_key,
            webhook_secret,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PaymentProvider for StripeClient {
    async fn create_checkout_session(
        &self,
        customer_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<String, PaymentError> {
        let params = [
            ("mode", "subscription"),
            ("customer_email", customer_email),
            ("line_items[0][price]", price_id),
            ("line_items[0][quantity]", "1"),
            ("success_url", success_url),
            ("cancel_url", cancel_url),
        ];

        let response = self
            .http_client
            .post(format!("{STRIPE_API_BASE}/checkout/sessions"))
            .basic_auth(&self.secret_key, None::<&str>)
            .form(&params)
            .send()
            .await
            .map_err(|e| PaymentError::ProviderError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| PaymentError::ProviderError(e.to_string()))?;

        if !status.is_success() {
            error!(event = "stripe_checkout_error", status = %status, body = %body);
            return Err(PaymentError::ProviderError(format!(
                "Stripe returned {status}"
            )));
        }

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| PaymentError::ProviderError(e.to_string()))?;

        json["url"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| PaymentError::ProviderError("missing url in Stripe response".into()))
    }

    fn verify_webhook_signature(
        &self,
        payload: &str,
        signature: &str,
    ) -> Result<StripeEvent, PaymentError> {
        let (timestamp, sig) = parse_stripe_signature(signature)?;

        let now = chrono::Utc::now().timestamp();
        if (now - timestamp).abs() > WEBHOOK_TOLERANCE_SECONDS {
            return Err(PaymentError::InvalidSignature);
        }

        let signed_payload = format!("{timestamp}.{payload}");
        let mut mac = Hmac::<Sha256>::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|e| PaymentError::ProviderError(e.to_string()))?;
        mac.update(signed_payload.as_bytes());

        let expected = hex::encode(mac.finalize().into_bytes());
        if !constant_time_eq(expected.as_bytes(), sig.as_bytes()) {
            return Err(PaymentError::InvalidSignature);
        }

        parse_event_json(payload)
    }
}

fn parse_stripe_signature(header: &str) -> Result<(i64, String), PaymentError> {
    let mut timestamp = None;
    let mut signature = None;

    for part in header.split(',') {
        let mut kv = part.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("t"), Some(v)) => {
                timestamp = v.parse::<i64>().ok();
            }
            (Some("v1"), Some(v)) => {
                signature = Some(v.to_string());
            }
            _ => {}
        }
    }

    match (timestamp, signature) {
        (Some(t), Some(s)) => Ok((t, s)),
        _ => Err(PaymentError::InvalidSignature),
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn parse_event_json(payload: &str) -> Result<StripeEvent, PaymentError> {
    let json: serde_json::Value =
        serde_json::from_str(payload).map_err(|e| PaymentError::ProviderError(e.to_string()))?;

    let event_type = json["type"].as_str().unwrap_or_default().to_string();

    let data_object = &json["data"]["object"];

    let customer_id = data_object["customer"].as_str().map(|s| s.to_string());

    let customer_email = data_object["customer_email"]
        .as_str()
        .or_else(|| data_object["customer_details"]["email"].as_str())
        .map(|s| s.to_string());

    let subscription_status = data_object["status"].as_str().map(|s| s.to_string());

    let price_id = data_object["items"]["data"][0]["price"]["id"]
        .as_str()
        .or_else(|| data_object["plan"]["id"].as_str())
        .map(|s| s.to_string());

    Ok(StripeEvent {
        event_type,
        customer_id,
        customer_email,
        subscription_status,
        price_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::Mac;

    fn make_signature(secret: &str, payload: &str, timestamp: i64) -> String {
        let signed = format!("{timestamp}.{payload}");
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        format!("t={timestamp},v1={sig}")
    }

    #[test]
    fn verifies_valid_webhook_signature() {
        let secret = "whsec_test_secret";
        let payload = r#"{"type":"checkout.session.completed","data":{"object":{"customer":"cus_123","customer_email":"test@example.com"}}}"#;
        let timestamp = chrono::Utc::now().timestamp();
        let sig_header = make_signature(secret, payload, timestamp);

        let client = StripeClient::new("sk_test".into(), secret.into());
        let event = client.verify_webhook_signature(payload, &sig_header);
        assert!(event.is_ok());
        let event = event.unwrap();
        assert_eq!(event.event_type, "checkout.session.completed");
        assert_eq!(event.customer_id.as_deref(), Some("cus_123"));
    }

    #[test]
    fn rejects_invalid_webhook_signature() {
        let payload = r#"{"type":"test","data":{"object":{}}}"#;
        let sig_header = format!("t={},v1=invalidsig", chrono::Utc::now().timestamp());

        let client = StripeClient::new("sk_test".into(), "whsec_secret".into());
        let result = client.verify_webhook_signature(payload, &sig_header);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_expired_timestamp() {
        let secret = "whsec_test_secret";
        let payload = r#"{"type":"test","data":{"object":{}}}"#;
        let old_timestamp = chrono::Utc::now().timestamp() - 600;
        let sig_header = make_signature(secret, payload, old_timestamp);

        let client = StripeClient::new("sk_test".into(), secret.into());
        let result = client.verify_webhook_signature(payload, &sig_header);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_signature_parts() {
        let client = StripeClient::new("sk_test".into(), "whsec_secret".into());
        let result = client.verify_webhook_signature("{}", "garbage");
        assert!(result.is_err());
    }
}
