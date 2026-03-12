use lambda_http::{Body, Request, Response};
use serde::{Deserialize, Serialize};

use crate::application::ports::{AiClient, PricingStore};
use crate::application::use_cases::manage_pricing::ManagePricingUseCase;
use crate::application::use_cases::process_lead::ProcessLeadUseCase;
use crate::domain::entities::*;
use crate::domain::value_objects::*;

const APPLICATION_JSON: &str = "application/json";

#[derive(Deserialize)]
pub struct SavePricingRequest {
    pub user_id: String,
    pub currency: String,
    pub country: String,
    pub minimum_callout: f64,
    pub categories: Vec<ServiceCategory>,
    pub add_ons: Vec<AddOn>,
    pub custom_notes: String,
}

#[derive(Deserialize)]
pub struct GetPricingRequest {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct SubmitLeadRequest {
    pub user_id: String,
    pub raw_text: Option<String>,
    pub image_data: Vec<String>,
    pub tone: ToneOption,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn json_response(status: u16, body: impl Serialize) -> Response<Body> {
    let json = serde_json::to_string(&body)
        .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string());

    Response::builder()
        .status(status)
        .header("Content-Type", APPLICATION_JSON)
        .body(Body::Text(json))
        .expect("failed to build response")
}

fn error_response(status: u16, message: &str) -> Response<Body> {
    json_response(
        status,
        ErrorResponse {
            error: message.to_string(),
        },
    )
}

pub async fn handle_save_pricing(req: Request, store: &dyn PricingStore) -> Response<Body> {
    let body = match req.body() {
        Body::Text(text) => text.clone(),
        Body::Binary(bytes) => String::from_utf8_lossy(bytes).to_string(),
        Body::Empty => return error_response(400, "empty request body"),
    };

    let payload: SavePricingRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    let use_case = ManagePricingUseCase::new(store);

    match use_case
        .save_template(
            UserId::new(payload.user_id),
            payload.currency,
            payload.country,
            payload.minimum_callout,
            payload.categories,
            payload.add_ons,
            payload.custom_notes,
        )
        .await
    {
        Ok(template) => json_response(200, template),
        Err(e) => error_response(500, &e.to_string()),
    }
}

pub async fn handle_get_pricing(req: Request, store: &dyn PricingStore) -> Response<Body> {
    let user_id: String = req
        .uri()
        .query()
        .and_then(|q| {
            q.split('&')
                .filter_map(|pair| pair.split_once('='))
                .find(|(k, _)| *k == "user_id")
                .map(|(_, v)| v.to_string())
        })
        .unwrap_or_default();

    if user_id.is_empty() {
        return error_response(400, "missing user_id query parameter");
    }

    let use_case = ManagePricingUseCase::new(store);

    match use_case.get_template(&UserId::new(user_id)).await {
        Ok(Some(template)) => json_response(200, template),
        Ok(None) => error_response(404, "pricing template not found"),
        Err(e) => error_response(500, &e.to_string()),
    }
}

pub async fn handle_submit_lead(
    req: Request,
    store: &dyn PricingStore,
    ai_client: &dyn AiClient,
) -> Response<Body> {
    let body = match req.body() {
        Body::Text(text) => text.clone(),
        Body::Binary(bytes) => String::from_utf8_lossy(bytes).to_string(),
        Body::Empty => return error_response(400, "empty request body"),
    };

    let payload: SubmitLeadRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    let lead = Lead {
        id: LeadId::generate(),
        user_id: UserId::new(payload.user_id),
        raw_text: payload.raw_text,
        image_data: payload.image_data,
        created_at: chrono::Utc::now(),
    };

    let use_case = ProcessLeadUseCase::new(store, ai_client);

    match use_case.execute(&lead, payload.tone).await {
        Ok(quote) => json_response(200, quote),
        Err(e) => error_response(500, &e.to_string()),
    }
}
