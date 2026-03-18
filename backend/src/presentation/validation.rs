const MAX_EMAIL_LEN: usize = 254;
const MAX_PASSWORD_LEN: usize = 72;
const MIN_PASSWORD_LEN: usize = 8;
const MAX_LEAD_TEXT_LEN: usize = 10_000;
const MAX_IMAGES: usize = 5;
const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024; // 5 MB base64
const MAX_CATEGORIES: usize = 50;
const MAX_ADD_ONS: usize = 50;
const MAX_NAME_LEN: usize = 100;
const MAX_DESCRIPTION_LEN: usize = 500;
const MAX_NOTES_LEN: usize = 2_000;
const MAX_CURRENCY_LEN: usize = 3;
const MAX_COUNTRY_LEN: usize = 2;
const MAX_PRICE: f64 = 99_999.0;

use super::handlers::{AuthRequest, SavePricingRequest, SubmitLeadRequest};

pub fn validate_auth(req: &AuthRequest) -> Result<(), String> {
    if req.email.len() > MAX_EMAIL_LEN {
        return Err(format!(
            "email must be at most {} characters",
            MAX_EMAIL_LEN
        ));
    }
    if req.password.len() < MIN_PASSWORD_LEN || req.password.len() > MAX_PASSWORD_LEN {
        return Err(format!(
            "password must be between {} and {} characters",
            MIN_PASSWORD_LEN, MAX_PASSWORD_LEN
        ));
    }
    Ok(())
}

pub fn validate_submit_lead(req: &SubmitLeadRequest) -> Result<(), String> {
    let has_text = req.raw_text.as_ref().is_some_and(|t| !t.trim().is_empty());
    let has_images = !req.image_data.is_empty();

    if !has_text && !has_images {
        return Err("provide either text or at least one image".to_string());
    }

    if let Some(text) = &req.raw_text {
        if text.len() > MAX_LEAD_TEXT_LEN {
            return Err(format!(
                "lead text must be at most {} characters",
                MAX_LEAD_TEXT_LEN
            ));
        }
    }

    if req.image_data.len() > MAX_IMAGES {
        return Err(format!("at most {} images allowed", MAX_IMAGES));
    }

    for (i, img) in req.image_data.iter().enumerate() {
        if img.len() > MAX_IMAGE_BYTES {
            return Err(format!("image {} exceeds 5 MB limit", i + 1));
        }
    }

    Ok(())
}

pub fn validate_save_pricing(req: &SavePricingRequest) -> Result<(), String> {
    if req.currency.len() != MAX_CURRENCY_LEN {
        return Err("currency must be a 3-letter code".to_string());
    }
    if req.country.len() != MAX_COUNTRY_LEN {
        return Err("country must be a 2-letter code".to_string());
    }
    if req.minimum_callout < 0.0 || req.minimum_callout > MAX_PRICE {
        return Err(format!(
            "minimum callout must be between 0 and {}",
            MAX_PRICE
        ));
    }
    if req.categories.is_empty() {
        return Err("at least one service category is required".to_string());
    }
    if req.categories.len() > MAX_CATEGORIES {
        return Err(format!("at most {} categories allowed", MAX_CATEGORIES));
    }
    if req.add_ons.len() > MAX_ADD_ONS {
        return Err(format!("at most {} add-ons allowed", MAX_ADD_ONS));
    }
    for cat in &req.categories {
        if cat.name.is_empty() || cat.name.len() > MAX_NAME_LEN {
            return Err(format!(
                "category name must be between 1 and {} characters",
                MAX_NAME_LEN
            ));
        }
        if cat.description.len() > MAX_DESCRIPTION_LEN {
            return Err(format!(
                "category description must be at most {} characters",
                MAX_DESCRIPTION_LEN
            ));
        }
        if cat.base_price < 0.0 || cat.base_price > MAX_PRICE {
            return Err(format!(
                "category base price must be between 0 and {}",
                MAX_PRICE
            ));
        }
    }
    for addon in &req.add_ons {
        if addon.name.is_empty() || addon.name.len() > MAX_NAME_LEN {
            return Err(format!(
                "add-on name must be between 1 and {} characters",
                MAX_NAME_LEN
            ));
        }
        if addon.price < 0.0 || addon.price > MAX_PRICE {
            return Err(format!("add-on price must be between 0 and {}", MAX_PRICE));
        }
    }
    if req.custom_notes.len() > MAX_NOTES_LEN {
        return Err(format!(
            "custom notes must be at most {} characters",
            MAX_NOTES_LEN
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{AddOn, ServiceCategory, ToneOption};

    fn valid_auth() -> AuthRequest {
        AuthRequest {
            email: "test@example.com".to_string(),
            password: "securepass".to_string(),
        }
    }

    fn valid_lead() -> SubmitLeadRequest {
        SubmitLeadRequest {
            raw_text: Some("Please clean my 3-bed house".to_string()),
            image_data: vec![],
            tone: ToneOption::Friendly,
        }
    }

    fn valid_pricing() -> SavePricingRequest {
        SavePricingRequest {
            currency: "USD".to_string(),
            country: "US".to_string(),
            minimum_callout: 50.0,
            categories: vec![ServiceCategory {
                id: "1".to_string(),
                name: "Standard Clean".to_string(),
                base_price: 80.0,
                description: "Regular cleaning".to_string(),
            }],
            add_ons: vec![],
            custom_notes: String::new(),
        }
    }

    #[test]
    fn accepts_valid_auth() {
        assert!(validate_auth(&valid_auth()).is_ok());
    }

    #[test]
    fn rejects_short_password() {
        let mut req = valid_auth();
        req.password = "short".to_string();
        assert!(validate_auth(&req).is_err());
    }

    #[test]
    fn rejects_long_email() {
        let mut req = valid_auth();
        req.email = "a".repeat(255);
        assert!(validate_auth(&req).is_err());
    }

    #[test]
    fn accepts_valid_lead() {
        assert!(validate_submit_lead(&valid_lead()).is_ok());
    }

    #[test]
    fn rejects_empty_lead() {
        let req = SubmitLeadRequest {
            raw_text: None,
            image_data: vec![],
            tone: ToneOption::Friendly,
        };
        assert!(validate_submit_lead(&req).is_err());
    }

    #[test]
    fn rejects_too_many_images() {
        let req = SubmitLeadRequest {
            raw_text: None,
            image_data: vec!["img".to_string(); 6],
            tone: ToneOption::Friendly,
        };
        assert!(validate_submit_lead(&req).is_err());
    }

    #[test]
    fn rejects_oversized_lead_text() {
        let mut req = valid_lead();
        req.raw_text = Some("x".repeat(10_001));
        assert!(validate_submit_lead(&req).is_err());
    }

    #[test]
    fn accepts_valid_pricing() {
        assert!(validate_save_pricing(&valid_pricing()).is_ok());
    }

    #[test]
    fn rejects_empty_categories() {
        let mut req = valid_pricing();
        req.categories = vec![];
        assert!(validate_save_pricing(&req).is_err());
    }

    #[test]
    fn rejects_invalid_currency_length() {
        let mut req = valid_pricing();
        req.currency = "USDX".to_string();
        assert!(validate_save_pricing(&req).is_err());
    }

    #[test]
    fn rejects_negative_price() {
        let mut req = valid_pricing();
        req.minimum_callout = -1.0;
        assert!(validate_save_pricing(&req).is_err());
    }

    #[test]
    fn rejects_excessive_price() {
        let mut req = valid_pricing();
        req.categories[0].base_price = 100_000.0;
        assert!(validate_save_pricing(&req).is_err());
    }

    #[test]
    fn rejects_empty_category_name() {
        let mut req = valid_pricing();
        req.categories[0].name = String::new();
        assert!(validate_save_pricing(&req).is_err());
    }

    #[test]
    fn rejects_addon_with_negative_price() {
        let mut req = valid_pricing();
        req.add_ons = vec![AddOn {
            id: "1".to_string(),
            name: "Extra".to_string(),
            price: -5.0,
        }];
        assert!(validate_save_pricing(&req).is_err());
    }
}
