use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::web::{self, Bytes, Data, Query};
use actix_web::{App, HttpResponse, HttpServer};
use serde::Deserialize;

use quotesnap_backend::application::use_cases::manage_pricing::ManagePricingUseCase;
use quotesnap_backend::application::use_cases::process_lead::ProcessLeadUseCase;
use quotesnap_backend::domain::entities::*;
use quotesnap_backend::domain::value_objects::*;
use quotesnap_backend::infrastructure::ai_client::{AiClientConfig, OpenAiCompatibleClient};
use quotesnap_backend::infrastructure::mongo_store::MongoStore;
use quotesnap_backend::presentation::handlers::{SavePricingRequest, SubmitLeadRequest};

const DEV_SERVER_PORT: u16 = 3001;

struct AppState {
    store: MongoStore,
    ai_client: OpenAiCompatibleClient,
}

async fn save_pricing(state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
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
        Ok(template) => HttpResponse::Ok().json(template),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

#[derive(Deserialize)]
struct UserIdQuery {
    user_id: Option<String>,
}

async fn get_pricing(state: Data<Arc<AppState>>, query: Query<UserIdQuery>) -> HttpResponse {
    let user_id = match &query.user_id {
        Some(id) if !id.is_empty() => id.clone(),
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "missing user_id query parameter"}))
        }
    };

    let use_case = ManagePricingUseCase::new(&state.store);

    match use_case.get_template(&UserId::new(user_id)).await {
        Ok(Some(template)) => HttpResponse::Ok().json(template),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "pricing template not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

async fn submit_lead(state: Data<Arc<AppState>>, body: Bytes) -> HttpResponse {
    let payload: SubmitLeadRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("invalid JSON: {}", e)}))
        }
    };

    let lead = Lead {
        id: LeadId::generate(),
        user_id: UserId::new(payload.user_id),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri =
        env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongo_db = env::var("MONGODB_DATABASE").unwrap_or_else(|_| "quotesnap".to_string());
    let ai_base_url =
        env::var("AI_BASE_URL").unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
    let ai_api_key = env::var("AI_API_KEY").expect("AI_API_KEY must be set");
    let ai_model =
        env::var("AI_MODEL").unwrap_or_else(|_| "google/gemini-3-flash-preview".to_string());

    let store = MongoStore::with_database(&mongo_uri, &mongo_db)
        .await
        .expect("failed to connect to MongoDB");

    let ai_client = OpenAiCompatibleClient::new(AiClientConfig {
        base_url: ai_base_url,
        api_key: ai_api_key,
        model: ai_model,
    });

    let state = Data::new(Arc::new(AppState { store, ai_client }));

    println!("QuoteSnap dev server running on http://localhost:{DEV_SERVER_PORT}");

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
            .route("/api/pricing", web::post().to(save_pricing))
            .route("/api/pricing", web::get().to(get_pricing))
            .route("/api/quote", web::post().to(submit_lead))
    })
    .bind(("0.0.0.0", DEV_SERVER_PORT))?
    .run()
    .await
}
