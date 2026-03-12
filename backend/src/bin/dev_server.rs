use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::web::{self, Bytes, Data};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer};

use quotesnap_backend::application::use_cases::auth::{validate_token, AuthUseCase};
use quotesnap_backend::application::use_cases::manage_pricing::ManagePricingUseCase;
use quotesnap_backend::application::use_cases::process_lead::ProcessLeadUseCase;
use quotesnap_backend::domain::entities::*;
use quotesnap_backend::domain::value_objects::*;
use quotesnap_backend::infrastructure::ai_client::{AiClientConfig, OpenAiCompatibleClient};
use quotesnap_backend::infrastructure::mongo_store::MongoStore;
use quotesnap_backend::presentation::handlers::{
    AuthRequest, AuthResponse, AuthUserResponse, SavePricingRequest, SubmitLeadRequest,
};

const DEV_SERVER_PORT: u16 = 3001;

struct AppState {
    store: MongoStore,
    ai_client: OpenAiCompatibleClient,
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
        Ok(result) => HttpResponse::Created().json(AuthResponse {
            token: result.token,
            user: AuthUserResponse {
                id: result.user_id,
                email: result.email,
            },
        }),
        Err(quotesnap_backend::application::use_cases::auth::AuthError::EmailTaken) => {
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
        Err(quotesnap_backend::application::use_cases::auth::AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(serde_json::json!({"error": "invalid credentials"}))
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv::dotenv();

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
            .route("/api/auth/signup", web::post().to(signup))
            .route("/api/auth/login", web::post().to(login))
            .route("/api/pricing", web::post().to(save_pricing))
            .route("/api/pricing", web::get().to(get_pricing))
            .route("/api/quote", web::post().to(submit_lead))
    })
    .bind(("127.0.0.1", DEV_SERVER_PORT))?
    .run()
    .await
}
