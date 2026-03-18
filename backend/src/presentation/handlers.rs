use lambda_http::{Body, Request, Response};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::application::ports::{
    AiClient, EmailSender, PaymentProvider, PricingStore, QuoteStore, SubscriptionStore,
    TokenStore, UsageStore, UserStore,
};
use crate::application::use_cases::auth::{validate_token, AuthError, AuthUseCase, Claims};
use crate::application::use_cases::checkout::{self, CheckoutError};
use crate::application::use_cases::email_verification;
use crate::application::use_cases::manage_pricing::ManagePricingUseCase;
use crate::application::use_cases::password_reset;
use crate::application::use_cases::process_lead::{ProcessLeadError, ProcessLeadUseCase};
use crate::application::use_cases::webhook;
use crate::domain::entities::*;
use crate::domain::quota::{current_billing_period, quota_for_price, PlanConfig, QuotaLimit};
use crate::domain::value_objects::*;
use crate::presentation::validation;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutRequest {
    pub price_id: String,
}

const APPLICATION_JSON: &str = "application/json";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
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
pub fn extract_user_id_and_claims(
    req: &Request,
    jwt_secret: &str,
) -> Result<(UserId, Claims), Response<Body>> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| error_response(401, "missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| error_response(401, "invalid Authorization header format"))?;

    let claims = validate_token(token, jwt_secret)
        .map_err(|_| error_response(401, "invalid or expired token"))?;

    let user_id = UserId::new(claims.sub.clone());
    Ok((user_id, claims))
}

#[allow(clippy::result_large_err)]
pub fn extract_user_id(req: &Request, jwt_secret: &str) -> Result<UserId, Response<Body>> {
    extract_user_id_and_claims(req, jwt_secret).map(|(id, _)| id)
}

pub async fn handle_signup(
    req: Request,
    user_store: &dyn UserStore,
    email_sender: &dyn EmailSender,
    token_store: &dyn TokenStore,
    app_base_url: &str,
    jwt_secret: &str,
) -> Response<Body> {
    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: AuthRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    if let Err(msg) = validation::validate_auth(&payload) {
        return error_response(400, &msg);
    }

    let use_case = AuthUseCase::new(user_store, jwt_secret);

    match use_case.signup(&payload.email, &payload.password).await {
        Ok(result) => {
            info!(event = "signup", user_id = %result.user_id);

            let user_id = UserId::new(&result.user_id);
            if let Err(e) = email_verification::send_verification_email(
                &user_id,
                &result.email,
                email_sender,
                token_store,
                app_base_url,
            )
            .await
            {
                error!(event = "verification_email_error", error = %e);
            }

            json_response(
                201,
                AuthResponse {
                    token: result.token,
                    user: AuthUserResponse {
                        id: result.user_id,
                        email: result.email,
                    },
                },
            )
        }
        Err(AuthError::EmailTaken) => error_response(409, "email already registered"),
        Err(AuthError::InvalidEmail) => error_response(400, "invalid email format"),
        Err(AuthError::InvalidPassword) => {
            error_response(400, "password must be between 8 and 72 characters")
        }
        Err(e) => {
            error!(event = "signup_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_login(
    req: Request,
    user_store: &dyn UserStore,
    jwt_secret: &str,
) -> Response<Body> {
    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: AuthRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    if let Err(msg) = validation::validate_auth(&payload) {
        return error_response(400, &msg);
    }

    let use_case = AuthUseCase::new(user_store, jwt_secret);

    match use_case.login(&payload.email, &payload.password).await {
        Ok(result) => {
            info!(event = "login", user_id = %result.user_id);
            json_response(
                200,
                AuthResponse {
                    token: result.token,
                    user: AuthUserResponse {
                        id: result.user_id,
                        email: result.email,
                    },
                },
            )
        }
        Err(AuthError::InvalidCredentials) => error_response(401, "invalid credentials"),
        Err(e) => {
            error!(event = "login_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

#[allow(clippy::result_large_err)]
async fn authorize_user(
    user_id: &UserId,
    token_iat: usize,
    user_store: &dyn UserStore,
) -> Result<User, Response<Body>> {
    let user = user_store
        .find_by_id(user_id)
        .await
        .map_err(|e| {
            error!(event = "authorize_user_error", error = %e);
            error_response(500, "an internal error occurred")
        })?
        .ok_or_else(|| error_response(401, "user not found"))?;

    if !user.email_verified {
        return Err(error_response(403, "email_not_verified"));
    }

    // Revoke tokens issued before a password change
    if let Some(changed_at) = user.password_changed_at {
        if (token_iat as i64) < changed_at.timestamp() {
            return Err(error_response(401, "token revoked after password change"));
        }
    }

    if user.subscription_status != SubscriptionStatus::Active {
        return Err(error_response(403, "subscription_required"));
    }

    Ok(user)
}

pub async fn handle_save_pricing(
    req: Request,
    store: &dyn PricingStore,
    user_store: &dyn UserStore,
    jwt_secret: &str,
) -> Response<Body> {
    let (user_id, claims) = match extract_user_id_and_claims(&req, jwt_secret) {
        Ok(v) => v,
        Err(r) => return r,
    };

    let _user = match authorize_user(&user_id, claims.iat, user_store).await {
        Ok(u) => u,
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

    if let Err(msg) = validation::validate_save_pricing(&payload) {
        return error_response(400, &msg);
    }

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
            error!(event = "save_pricing_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_get_pricing(
    req: Request,
    store: &dyn PricingStore,
    user_store: &dyn UserStore,
    jwt_secret: &str,
) -> Response<Body> {
    let (user_id, claims) = match extract_user_id_and_claims(&req, jwt_secret) {
        Ok(v) => v,
        Err(r) => return r,
    };

    let _user = match authorize_user(&user_id, claims.iat, user_store).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    let use_case = ManagePricingUseCase::new(store);

    match use_case.get_template(&user_id).await {
        Ok(Some(template)) => json_response(200, template),
        Ok(None) => error_response(404, "pricing template not found"),
        Err(e) => {
            error!(event = "get_pricing_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_submit_lead(
    req: Request,
    store: &dyn PricingStore,
    ai_client: &dyn AiClient,
    user_store: &dyn UserStore,
    quote_store: &dyn QuoteStore,
    usage_store: &dyn UsageStore,
    plan_config: &PlanConfig,
    jwt_secret: &str,
) -> Response<Body> {
    let (user_id, claims) = match extract_user_id_and_claims(&req, jwt_secret) {
        Ok(v) => v,
        Err(r) => return r,
    };

    let _user = match authorize_user(&user_id, claims.iat, user_store).await {
        Ok(u) => u,
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

    if let Err(msg) = validation::validate_submit_lead(&payload) {
        return error_response(400, &msg);
    }

    let lead = Lead {
        id: LeadId::generate(),
        user_id,
        raw_text: payload.raw_text,
        image_data: payload.image_data,
        created_at: chrono::Utc::now(),
    };

    let use_case = ProcessLeadUseCase::new(
        store,
        ai_client,
        quote_store,
        usage_store,
        user_store,
        plan_config,
    );

    match use_case.execute(&lead, payload.tone).await {
        Ok(quote) => {
            info!(event = "quote_generated", user_id = %lead.user_id);
            json_response(200, quote)
        }
        Err(ProcessLeadError::TemplateNotFound) => error_response(
            404,
            "pricing template not found — please set up your pricing first",
        ),
        Err(ProcessLeadError::QuotaExceeded { used, limit }) => json_response(
            429,
            serde_json::json!({
                "error": "quota_exceeded",
                "used": used,
                "limit": limit,
            }),
        ),
        Err(e) => {
            error!(event = "submit_lead_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_verify_email(
    req: Request,
    user_store: &dyn UserStore,
    token_store: &dyn TokenStore,
) -> Response<Body> {
    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: VerifyEmailRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    match email_verification::verify_email(&payload.token, user_store, token_store).await {
        Ok(()) => json_response(200, serde_json::json!({"message": "email verified"})),
        Err(email_verification::EmailVerificationError::InvalidToken) => {
            error_response(400, "invalid or expired token")
        }
        Err(e) => {
            error!(event = "verify_email_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_resend_verification(
    req: Request,
    user_store: &dyn UserStore,
    email_sender: &dyn EmailSender,
    token_store: &dyn TokenStore,
    app_base_url: &str,
    jwt_secret: &str,
) -> Response<Body> {
    let user_id = match extract_user_id(&req, jwt_secret) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let user = match user_store.find_by_id(&user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return error_response(401, "user not found"),
        Err(e) => {
            error!(event = "resend_verification_error", error = %e);
            return error_response(500, "an internal error occurred");
        }
    };

    if user.email_verified {
        return error_response(400, "email already verified");
    }

    match email_verification::send_verification_email(
        &user_id,
        &user.email,
        email_sender,
        token_store,
        app_base_url,
    )
    .await
    {
        Ok(()) => json_response(
            200,
            serde_json::json!({"message": "verification email sent"}),
        ),
        Err(e) => {
            error!(event = "resend_verification_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_forgot_password(
    req: Request,
    user_store: &dyn UserStore,
    email_sender: &dyn EmailSender,
    token_store: &dyn TokenStore,
    app_base_url: &str,
) -> Response<Body> {
    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: ForgotPasswordRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    match password_reset::send_reset_email(
        &payload.email,
        user_store,
        email_sender,
        token_store,
        app_base_url,
    )
    .await
    {
        Ok(()) => json_response(
            200,
            serde_json::json!({"message": "if the email exists, a reset link has been sent"}),
        ),
        Err(e) => {
            error!(event = "forgot_password_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_reset_password(
    req: Request,
    user_store: &dyn UserStore,
    token_store: &dyn TokenStore,
) -> Response<Body> {
    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: ResetPasswordRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    match password_reset::reset_password(&payload.token, &payload.password, user_store, token_store)
        .await
    {
        Ok(()) => json_response(
            200,
            serde_json::json!({"message": "password reset successful"}),
        ),
        Err(password_reset::PasswordResetError::InvalidToken) => {
            error_response(400, "invalid or expired token")
        }
        Err(password_reset::PasswordResetError::InvalidPassword) => {
            error_response(400, "password must be between 8 and 72 characters")
        }
        Err(e) => {
            error!(event = "reset_password_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_checkout(
    req: Request,
    user_store: &dyn UserStore,
    payment_provider: &dyn PaymentProvider,
    app_base_url: &str,
    plan_config: &PlanConfig,
    jwt_secret: &str,
) -> Response<Body> {
    let user_id = match extract_user_id(&req, jwt_secret) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    let payload: CheckoutRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return error_response(400, &format!("invalid JSON: {}", e)),
    };

    match checkout::create_checkout(
        &user_id,
        &payload.price_id,
        user_store,
        payment_provider,
        app_base_url,
        plan_config,
    )
    .await
    {
        Ok(url) => json_response(200, serde_json::json!({"url": url})),
        Err(CheckoutError::InvalidPriceId) => error_response(400, "invalid price ID"),
        Err(CheckoutError::UserNotFound) => error_response(404, "user not found"),
        Err(e) => {
            error!(event = "checkout_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 100;

pub async fn handle_list_quotes(
    req: Request,
    quote_store: &dyn QuoteStore,
    user_store: &dyn UserStore,
    jwt_secret: &str,
) -> Response<Body> {
    let (user_id, claims) = match extract_user_id_and_claims(&req, jwt_secret) {
        Ok(v) => v,
        Err(r) => return r,
    };

    let _user = match authorize_user(&user_id, claims.iat, user_store).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    let query = req.uri().query().unwrap_or("");
    let params: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let page = params
        .iter()
        .find(|(k, _)| k == "page")
        .and_then(|(_, v)| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_PAGE)
        .max(1);

    let limit = params
        .iter()
        .find(|(k, _)| k == "limit")
        .and_then(|(_, v)| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);

    match quote_store.list_quotes(&user_id, page, limit).await {
        Ok(quotes) => {
            info!(event = "list_quotes", user_id = %user_id, page, count = quotes.len());
            json_response(200, quotes)
        }
        Err(e) => {
            error!(event = "list_quotes_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_get_quote(
    req: Request,
    quote_id: &str,
    quote_store: &dyn QuoteStore,
    user_store: &dyn UserStore,
    jwt_secret: &str,
) -> Response<Body> {
    let (user_id, claims) = match extract_user_id_and_claims(&req, jwt_secret) {
        Ok(v) => v,
        Err(r) => return r,
    };

    let _user = match authorize_user(&user_id, claims.iat, user_store).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    let quote_id = QuoteId::new(quote_id);

    match quote_store.get_quote(&quote_id, &user_id).await {
        Ok(Some(quote)) => {
            info!(event = "get_quote", user_id = %user_id, quote_id = %quote_id);
            json_response(200, quote)
        }
        Ok(None) => error_response(404, "quote not found"),
        Err(e) => {
            error!(event = "get_quote_error", error = %e);
            error_response(500, "an internal error occurred")
        }
    }
}

pub async fn handle_get_usage(
    req: Request,
    user_store: &dyn UserStore,
    usage_store: &dyn UsageStore,
    plan_config: &PlanConfig,
    jwt_secret: &str,
) -> Response<Body> {
    let (user_id, claims) = match extract_user_id_and_claims(&req, jwt_secret) {
        Ok(v) => v,
        Err(r) => return r,
    };

    let user = match authorize_user(&user_id, claims.iat, user_store).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    let price_id = user.subscription_plan.as_deref().unwrap_or("");
    let limit = quota_for_price(price_id, plan_config);

    let now = chrono::Utc::now();
    let (period_start, period_end) = current_billing_period(now);

    let usage = match usage_store
        .get_or_create_usage(&user_id, period_start, period_end)
        .await
    {
        Ok(u) => u,
        Err(e) => {
            error!(event = "get_usage_error", error = %e);
            return error_response(500, "an internal error occurred");
        }
    };

    let limit_value = match limit {
        QuotaLimit::Unlimited => serde_json::Value::Null,
        QuotaLimit::Limited(n) => serde_json::Value::Number(n.into()),
    };

    json_response(
        200,
        serde_json::json!({
            "used": usage.quote_count,
            "limit": limit_value,
            "periodEnd": period_end.to_rfc3339(),
        }),
    )
}

pub async fn handle_stripe_webhook(
    req: Request,
    payment_provider: &dyn PaymentProvider,
    user_store: &dyn UserStore,
    subscription_store: &dyn SubscriptionStore,
) -> Response<Body> {
    let signature = match req
        .headers()
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => return error_response(400, "missing Stripe-Signature header"),
    };

    let body = match parse_body(&req) {
        Ok(b) => b,
        Err(r) => return r,
    };

    match webhook::handle_stripe_webhook(
        &body,
        &signature,
        payment_provider,
        user_store,
        subscription_store,
    )
    .await
    {
        Ok(()) => json_response(200, serde_json::json!({"received": true})),
        Err(webhook::WebhookError::InvalidSignature) => {
            error_response(400, "invalid webhook signature")
        }
        Err(e) => {
            error!(event = "webhook_error", error = %e);
            error_response(500, "webhook processing failed")
        }
    }
}

pub async fn handle_get_subscription(
    req: Request,
    user_store: &dyn UserStore,
    jwt_secret: &str,
) -> Response<Body> {
    let user_id = match extract_user_id(&req, jwt_secret) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let user = match user_store.find_by_id(&user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return error_response(401, "user not found"),
        Err(e) => {
            error!(event = "get_subscription_error", error = %e);
            return error_response(500, "an internal error occurred");
        }
    };

    json_response(
        200,
        serde_json::json!({
            "status": user.subscription_status,
            "plan": user.subscription_plan,
        }),
    )
}

pub fn handle_get_plans(plan_config: &PlanConfig) -> Response<Body> {
    json_response(200, plan_config.plans())
}
