use std::env;
use std::sync::Arc;

use lambda_http::{run, service_fn, Body, Request, Response};

use tidy_quote_backend::infrastructure::ai_client::{AiClientConfig, OpenAiCompatibleClient};
use tidy_quote_backend::infrastructure::mongo_store::MongoStore;
use tidy_quote_backend::presentation::handlers;

struct AppState {
    store: MongoStore,
    ai_client: OpenAiCompatibleClient,
}

async fn router(state: Arc<AppState>, req: Request) -> Result<Response<Body>, lambda_http::Error> {
    let path = req.uri().path();
    let method = req.method().as_str();

    let response = match (method, path) {
        ("OPTIONS", _) => Response::builder()
            .status(204)
            .body(Body::Empty)
            .expect("failed to build response"),
        ("POST", "/api/auth/signup") => handlers::handle_signup(req, &state.store).await,
        ("POST", "/api/auth/login") => handlers::handle_login(req, &state.store).await,
        ("POST", "/api/pricing") => handlers::handle_save_pricing(req, &state.store).await,
        ("GET", "/api/pricing") => handlers::handle_get_pricing(req, &state.store).await,
        ("POST", "/api/quote") => {
            handlers::handle_submit_lead(req, &state.store, &state.ai_client).await
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
    let mongo_uri =
        env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let ai_base_url =
        env::var("AI_BASE_URL").unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
    let ai_api_key = env::var("AI_API_KEY").expect("AI_API_KEY must be set");
    let ai_model = env::var("AI_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());

    let store = MongoStore::new(&mongo_uri)
        .await
        .expect("failed to connect to MongoDB");

    let ai_client = OpenAiCompatibleClient::new(AiClientConfig {
        base_url: ai_base_url,
        api_key: ai_api_key,
        model: ai_model,
    });

    let state = Arc::new(AppState { store, ai_client });

    run(service_fn(move |req: Request| {
        let state = Arc::clone(&state);
        async move { router(state, req).await }
    }))
    .await
}
