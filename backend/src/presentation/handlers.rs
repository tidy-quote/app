use lambda_http::{Body, Request, Response};
use serde::{Deserialize, Serialize};

use crate::application::ports::{AiClient, PricingStore, UserStore};
use crate::application::use_cases::auth::{validate_token, AuthError, AuthUseCase};
use crate::application::use_cases::manage_pricing::ManagePricingUseCase;
use crate::application::use_cases::process_lead::ProcessLeadUseCase;
use crate::domain::entities::*;
use crate::domain::value_objects::*;

const APPLICATION_JSON: &str = "application/json";

#[derive(Deserialize)]
pub struct SavePricingRequest {
    pub currency: String,
    pub country: String,
    pub minimum_callout: f64,
    pub categories: Vec<ServiceCategory>,
    pub add_ons: Vec<AddOn>,
    pub custom_notes: String,
}

#[derive(Deserialize)]
pub struct SubmitLeadRequest {
    pub raw_text: Option<String>,
    pub image_data: Vec<String>,
    pub tone: ToneOption,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUserResponse,
}

#[derive(Serialize)]
pub struct AuthUserResponse {
    pub id: String,
    pub email: String,
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

#[allow(clippy::result_large_err)]
fn parse_body(req: &Request) -> Result<String, Response<Body>> {
    match req.body() {
        Body::Text(text) => Ok(text.clone()),
        Body::Binary(bytes) => Ok(String::from_utf8_lossy(bytes).to_string()),
        Body::Empty => Err(error_response(400, "empty request body")),
    }
}

#[allow(clippy::result_large_err)]
pub fn extract_user_id(req: &Request) -> Result<UserId, Response<Body>> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| error_response(401, "missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| error_response(401, "invalid Authorization header format"))?;

    let claims =
        validate_token(token).map_err(|_| error_response(401, "invalid or expired token"))?;

    Ok(UserId::new(claims.sub))
}

pub async fn handle_signup(req: Request, user_store: &dyn UserStore) -> Response<Body> {
    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: AuthRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    let use_case = AuthUseCase::new(user_store);

    match use_case.signup(&payload.email, &payload.password).await {
        Ok(result) => json_response(
            201,
            AuthResponse {
                token: result.token,
                user: AuthUserResponse {
                    id: result.user_id,
                    email: result.email,
                },
            },
        ),
        Err(AuthError::EmailTaken) => error_response(409, "email already registered"),
        Err(AuthError::InvalidEmail) => error_response(400, "invalid email format"),
        Err(AuthError::InvalidPassword) => {
            error_response(400, "password must be between 8 and 72 characters")
        }
        Err(e) => {
            eprintln!("signup error: {e}");
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_login(req: Request, user_store: &dyn UserStore) -> Response<Body> {
    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: AuthRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    let use_case = AuthUseCase::new(user_store);

    match use_case.login(&payload.email, &payload.password).await {
        Ok(result) => json_response(
            200,
            AuthResponse {
                token: result.token,
                user: AuthUserResponse {
                    id: result.user_id,
                    email: result.email,
                },
            },
        ),
        Err(AuthError::InvalidCredentials) => error_response(401, "invalid credentials"),
        Err(e) => {
            eprintln!("login error: {e}");
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_save_pricing(req: Request, store: &dyn PricingStore) -> Response<Body> {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: SavePricingRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    let use_case = ManagePricingUseCase::new(store);

    match use_case
        .save_template(
            user_id,
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
        Err(e) => {
            eprintln!("save pricing error: {e}");
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_get_pricing(req: Request, store: &dyn PricingStore) -> Response<Body> {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let use_case = ManagePricingUseCase::new(store);

    match use_case.get_template(&user_id).await {
        Ok(Some(template)) => json_response(200, template),
        Ok(None) => error_response(404, "pricing template not found"),
        Err(e) => {
            eprintln!("get pricing error: {e}");
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_submit_lead(
    req: Request,
    store: &dyn PricingStore,
    ai_client: &dyn AiClient,
) -> Response<Body> {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: SubmitLeadRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    let lead = Lead {
        id: LeadId::generate(),
        user_id,
        raw_text: payload.raw_text,
        image_data: payload.image_data,
        created_at: chrono::Utc::now(),
    };

    let use_case = ProcessLeadUseCase::new(store, ai_client);

    match use_case.execute(&lead, payload.tone).await {
        Ok(quote) => json_response(200, quote),
        Err(e) => {
            eprintln!("submit lead error: {e}");
            error_response(500, "an internal error occurred")
        }
    }
}
