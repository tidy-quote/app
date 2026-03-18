use std::env;
use std::sync::Arc;

use lambda_http::{run, service_fn, Body, Request, Response};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use tidy_quote_backend::infrastructure::ai_client::{AiClientConfig, OpenAiCompatibleClient};
use tidy_quote_backend::infrastructure::mongo_store::MongoStore;
use tidy_quote_backend::infrastructure::ses_client::SesEmailClient;
use tidy_quote_backend::infrastructure::stripe_client::StripeClient;
use tidy_quote_backend::presentation::handlers;

struct AppState {
    store: MongoStore,
    ai_client: OpenAiCompatibleClient,
    email_sender: SesEmailClient,
    stripe_client: StripeClient,
    app_base_url: String,
    allowed_price_ids: Vec<String>,
}

async fn router(state: Arc<AppState>, req: Request) -> Result<Response<Body>, lambda_http::Error> {
    let path = req.uri().path().to_string();
    let method = req.method().as_str().to_string();

    info!(
        method = method.as_str(),
        path = path.as_str(),
        event = "request"
    );

    let response = match (method.as_str(), path.as_str()) {
        ("OPTIONS", _) => Response::builder()
            .status(204)
            .body(Body::Empty)
            .expect("failed to build response"),
        ("POST", "/api/auth/signup") => {
            handlers::handle_signup(
                req,
                &state.store,
                &state.email_sender,
                &state.store,
                &state.app_base_url,
            )
            .await
        }
        ("POST", "/api/auth/login") => handlers::handle_login(req, &state.store).await,
        ("POST", "/api/auth/verify-email") => {
            handlers::handle_verify_email(req, &state.store, &state.store).await
        }
        ("POST", "/api/auth/resend-verification") => {
            handlers::handle_resend_verification(
                req,
                &state.store,
                &state.email_sender,
                &state.store,
                &state.app_base_url,
            )
            .await
        }
        ("POST", "/api/auth/forgot-password") => {
            handlers::handle_forgot_password(
                req,
                &state.store,
                &state.email_sender,
                &state.store,
                &state.app_base_url,
            )
            .await
        }
        ("POST", "/api/auth/reset-password") => {
            handlers::handle_reset_password(req, &state.store, &state.store).await
        }
        ("POST", "/api/pricing") => {
            handlers::handle_save_pricing(req, &state.store, &state.store).await
        }
        ("GET", "/api/pricing") => {
            handlers::handle_get_pricing(req, &state.store, &state.store).await
        }
        ("POST", "/api/quote") => {
            handlers::handle_submit_lead(
                req,
                &state.store,
                &state.ai_client,
                &state.store,
                &state.store,
                &state.store,
                &state.allowed_price_ids,
            )
            .await
        }
        ("GET", "/api/usage") => {
            handlers::handle_get_usage(req, &state.store, &state.store, &state.allowed_price_ids)
                .await
        }
        ("GET", "/api/subscription") => handlers::handle_get_subscription(req, &state.store).await,
        ("GET", "/api/quotes") => {
            handlers::handle_list_quotes(req, &state.store, &state.store).await
        }
        ("POST", "/api/checkout") => {
            handlers::handle_checkout(
                req,
                &state.store,
                &state.stripe_client,
                &state.app_base_url,
                &state.allowed_price_ids,
            )
            .await
        }
        ("POST", "/api/webhook/stripe") => {
            handlers::handle_stripe_webhook(req, &state.stripe_client, &state.store).await
        }
        ("GET", path) if path.starts_with("/api/quotes/") => {
            let quote_id = &path["/api/quotes/".len()..];
            handlers::handle_get_quote(req, quote_id, &state.store, &state.store).await
        }
        _ => Response::builder()
            .status(404)
            .header("Content-Type", "application/json")
            .body(Body::Text(r#"{"error":"not found"}"#.to_string()))
            .expect("failed to build response"),
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    fmt::Subscriber::builder()
        .json()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_target(false)
        .without_time()
        .init();

    let mongo_uri =
        env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let ai_base_url =
        env::var("AI_BASE_URL").unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
    let ai_api_key = env::var("AI_API_KEY").expect("AI_API_KEY must be set");
    let ai_model = env::var("AI_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());
    let ses_sender = env::var("SES_SENDER").expect("SES_SENDER must be set");
    let app_base_url = env::var("APP_BASE_URL").expect("APP_BASE_URL must be set");
    let stripe_secret_key = env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
    let stripe_webhook_secret =
        env::var("STRIPE_WEBHOOK_SECRET").expect("STRIPE_WEBHOOK_SECRET must be set");
    let stripe_price_starter =
        env::var("STRIPE_PRICE_STARTER").expect("STRIPE_PRICE_STARTER must be set");
    let stripe_price_solo = env::var("STRIPE_PRICE_SOLO").expect("STRIPE_PRICE_SOLO must be set");
    let stripe_price_pro = env::var("STRIPE_PRICE_PRO").expect("STRIPE_PRICE_PRO must be set");

    let store = MongoStore::new(&mongo_uri)
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

    let state = Arc::new(AppState {
        store,
        ai_client,
        email_sender,
        stripe_client,
        app_base_url,
        allowed_price_ids,
    });

    run(service_fn(move |req: Request| {
        let state = Arc::clone(&state);
        async move { router(state, req).await }
    }))
    .await
}
