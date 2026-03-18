use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::web::{self, Bytes, Data};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer};

use tidy_quote_backend::application::ports::{EmailSender, PaymentProvider, TokenStore, UserStore};
use tidy_quote_backend::application::use_cases::auth::{validate_token, AuthError, AuthUseCase};
use tidy_quote_backend::application::use_cases::checkout::{self, CheckoutError};
use tidy_quote_backend::application::use_cases::email_verification;
use tidy_quote_backend::application::use_cases::manage_pricing::ManagePricingUseCase;
use tidy_quote_backend::application::use_cases::password_reset;
use tidy_quote_backend::application::use_cases::process_lead::ProcessLeadUseCase;
use tidy_quote_backend::application::use_cases::webhook;
use tidy_quote_backend::domain::entities::*;
use tidy_quote_backend::domain::value_objects::*;
use tidy_quote_backend::infrastructure::ai_client::{AiClientConfig, OpenAiCompatibleClient};
use tidy_quote_backend::infrastructure::mongo_store::MongoStore;
use tidy_quote_backend::infrastructure::ses_client::SesEmailClient;
use tidy_quote_backend::infrastructure::stripe_client::StripeClient;
use tidy_quote_backend::presentation::handlers::{
    AuthRequest, AuthResponse, AuthUserResponse, CheckoutRequest, ForgotPasswordRequest,
    ResetPasswordRequest, SavePricingRequest, SubmitLeadRequest, VerifyEmailRequest,
};

const DEV_SERVER_PORT: u16 = 3001;

struct AppState {
    store: MongoStore,
    ai_client: OpenAiCompatibleClient,
    email_sender: SesEmailClient,
    stripe_client: StripeClient,
    app_base_url: String,
    allowed_price_ids: Vec<String>,
}

fn extract_user_id(req: &HttpRequest) -> Result<UserId, HttpResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "missing Authorization header"}))
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "invalid Authorization header format"}))
    })?;

    let claims = validate_token(token).map_err(|_| {
        HttpResponse::Unauthorized().json(serde_json::json!({"error": "invalid or expired token"}))
    })?;

    Ok(UserId::new(claims.sub))
}

fn check_email_verified_dev(user: &User) -> Result<(), HttpResponse> {
    if !user.email_verified {
        return Err(
            HttpResponse::Forbidden().json(serde_json::json!({"error": "email_not_verified"}))
        );
    }
    Ok(())
}

fn check_subscription_dev(user: &User) -> Result<(), HttpResponse> {
    if user.subscription_status != SubscriptionStatus::Active {
        return Err(
            HttpResponse::Forbidden().json(serde_json::json!({"error": "subscription_required"}))
        );
    }
    Ok(())
}

async fn signup(state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let payload: AuthRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    let use_case = AuthUseCase::new(&state.store);

    match use_case.signup(&payload.email, &payload.password).await {
        Ok(result) => {
            let user_id = UserId::new(&result.user_id);
            if let Err(e) = email_verification::send_verification_email(
                &user_id,
                &result.email,
                &state.email_sender as &dyn EmailSender,
                &state.store as &dyn TokenStore,
                &state.app_base_url,
            )
            .await
            {
                eprintln!("Failed to send verification email: {e}");
            }

            HttpResponse::Created().json(AuthResponse {
                token: result.token,
                user: AuthUserResponse {
                    id: result.user_id,
                    email: result.email,
                },
            })
        }
        Err(AuthError::EmailTaken) => {
            HttpResponse::Conflict().json(serde_json::json!({"error": "email already registered"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn login(state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let payload: AuthRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    let use_case = AuthUseCase::new(&state.store);

    match use_case.login(&payload.email, &payload.password).await {
        Ok(result) => HttpResponse::Ok().json(AuthResponse {
            token: result.token,
            user: AuthUserResponse {
                id: result.user_id,
                email: result.email,
            },
        }),
        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(serde_json::json!({"error": "invalid credentials"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn verify_email(state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let payload: VerifyEmailRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    match email_verification::verify_email(
        &payload.token,
        &state.store as &dyn UserStore,
        &state.store as &dyn TokenStore,
    )
    .await
    {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"message": "email verified"})),
        Err(email_verification::EmailVerificationError::InvalidToken) => {
            HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "invalid or expired token"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn resend_verification(
    req: HttpRequest,
    state: Data<Arc<AppState>>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let user = match UserStore::find_by_id(&state.store, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "user not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    if user.email_verified {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "email already verified"}));
    }

    match email_verification::send_verification_email(
        &user_id,
        &user.email,
        &state.email_sender as &dyn EmailSender,
        &state.store as &dyn TokenStore,
        &state.app_base_url,
    )
    .await
    {
        Ok(()) => {
            HttpResponse::Ok().json(serde_json::json!({"message": "verification email sent"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn forgot_password(state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let payload: ForgotPasswordRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    match password_reset::send_reset_email(
        &payload.email,
        &state.store as &dyn UserStore,
        &state.email_sender as &dyn EmailSender,
        &state.store as &dyn TokenStore,
        &state.app_base_url,
    )
    .await
    {
        Ok(()) => HttpResponse::Ok().json(
            serde_json::json!({"message": "if the email exists, a reset link has been sent"}),
        ),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn reset_password(state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let payload: ResetPasswordRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    match password_reset::reset_password(
        &payload.token,
        &payload.password,
        &state.store as &dyn UserStore,
        &state.store as &dyn TokenStore,
    )
    .await
    {
        Ok(()) => {
            HttpResponse::Ok().json(serde_json::json!({"message": "password reset successful"}))
        }
        Err(password_reset::PasswordResetError::InvalidToken) => {
            HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "invalid or expired token"}))
        }
        Err(password_reset::PasswordResetError::InvalidPassword) => {
            HttpResponse::BadRequest().json(
                serde_json::json!({"error": "password must be between 8 and 72 characters"}),
            )
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn save_pricing(req: HttpRequest, state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let user = match UserStore::find_by_id(&state.store, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "user not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    if let Err(r) = check_email_verified_dev(&user) {
        return r;
    }
    if let Err(r) = check_subscription_dev(&user) {
        return r;
    }

    let payload: SavePricingRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    let use_case = ManagePricingUseCase::new(&state.store);

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
        Ok(template) => HttpResponse::Ok().json(template),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn get_pricing(req: HttpRequest, state: Data<Arc<AppState>>) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let user = match UserStore::find_by_id(&state.store, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "user not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    if let Err(r) = check_email_verified_dev(&user) {
        return r;
    }
    if let Err(r) = check_subscription_dev(&user) {
        return r;
    }

    let use_case = ManagePricingUseCase::new(&state.store);

    match use_case.get_template(&user_id).await {
        Ok(Some(template)) => HttpResponse::Ok().json(template),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "pricing template not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn submit_lead(req: HttpRequest, state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let user = match UserStore::find_by_id(&state.store, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "user not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    if let Err(r) = check_email_verified_dev(&user) {
        return r;
    }
    if let Err(r) = check_subscription_dev(&user) {
        return r;
    }

    let payload: SubmitLeadRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    let lead = Lead {
        id: LeadId::generate(),
        user_id,
        raw_text: payload.raw_text,
        image_data: payload.image_data,
        created_at: chrono::Utc::now(),
    };

    let use_case = ProcessLeadUseCase::new(&state.store, &state.ai_client);

    match use_case.execute(&lead, payload.tone).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn create_checkout(req: HttpRequest, state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(r) => return r,
    };

    let payload: CheckoutRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    match checkout::create_checkout(
        &user_id,
        &payload.price_id,
        &state.store as &dyn UserStore,
        &state.stripe_client as &dyn PaymentProvider,
        &state.app_base_url,
        &state.allowed_price_ids,
    )
    .await
    {
        Ok(url) => HttpResponse::Ok().json(serde_json::json!({"url": url})),
        Err(CheckoutError::InvalidPriceId) => {
            HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid price ID"}))
        }
        Err(CheckoutError::UserNotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "user not found"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn stripe_webhook(req: HttpRequest, state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let signature = match req
        .headers()
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "missing Stripe-Signature header"}))
        }
    };

    let payload = String::from_utf8_lossy(&body).to_string();

    match webhook::handle_stripe_webhook(
        &payload,
        &signature,
        &state.stripe_client as &dyn PaymentProvider,
        &state.store as &dyn UserStore,
    )
    .await
    {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"received": true})),
        Err(webhook::WebhookError::InvalidSignature) => {
            HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "invalid webhook signature"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv::dotenv();

    let mongo_uri =
        env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongo_db = env::var("MONGODB_DATABASE").unwrap_or_else(|_| "tidy-quote".to_string());
    let ai_base_url =
        env::var("AI_BASE_URL").unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
    let ai_api_key = env::var("AI_API_KEY").expect("AI_API_KEY must be set");
    let ai_model =
        env::var("AI_MODEL").unwrap_or_else(|_| "google/gemini-3-flash-preview".to_string());
    let ses_sender = env::var("SES_SENDER").expect("SES_SENDER must be set");
    let app_base_url = env::var("APP_BASE_URL").expect("APP_BASE_URL must be set");
    let stripe_secret_key =
        env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
    let stripe_webhook_secret =
        env::var("STRIPE_WEBHOOK_SECRET").expect("STRIPE_WEBHOOK_SECRET must be set");
    let stripe_price_starter =
        env::var("STRIPE_PRICE_STARTER").expect("STRIPE_PRICE_STARTER must be set");
    let stripe_price_solo =
        env::var("STRIPE_PRICE_SOLO").expect("STRIPE_PRICE_SOLO must be set");
    let stripe_price_pro =
        env::var("STRIPE_PRICE_PRO").expect("STRIPE_PRICE_PRO must be set");

    let store = MongoStore::with_database(&mongo_uri, &mongo_db)
        .await
        .expect("failed to connect to MongoDB");

    let ai_client = OpenAiCompatibleClient::new(AiClientConfig {
        base_url: ai_base_url,
        api_key: ai_api_key,
        model: ai_model,
    });

    let email_sender = SesEmailClient::new(ses_sender).await;
    let stripe_client = StripeClient::new(stripe_secret_key, stripe_webhook_secret);
    let allowed_price_ids = vec![stripe_price_starter, stripe_price_solo, stripe_price_pro];

    let state = Data::new(Arc::new(AppState {
        store,
        ai_client,
        email_sender,
        stripe_client,
        app_base_url,
        allowed_price_ids,
    }));

    println!("Tidy-Quote dev server running on http://localhost:{DEV_SERVER_PORT}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3001")
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .route("/api/auth/signup", web::post().to(signup))
            .route("/api/auth/login", web::post().to(login))
            .route("/api/auth/verify-email", web::post().to(verify_email))
            .route(
                "/api/auth/resend-verification",
                web::post().to(resend_verification),
            )
            .route(
                "/api/auth/forgot-password",
                web::post().to(forgot_password),
            )
            .route(
                "/api/auth/reset-password",
                web::post().to(reset_password),
            )
            .route("/api/pricing", web::post().to(save_pricing))
            .route("/api/pricing", web::get().to(get_pricing))
            .route("/api/quote", web::post().to(submit_lead))
            .route("/api/checkout", web::post().to(create_checkout))
            .route("/api/webhook/stripe", web::post().to(stripe_webhook))
    })
    .bind(("127.0.0.1", DEV_SERVER_PORT))?
    .run()
    .await
}
