use axum::{
    Json, Router,
    routing::{get, post},
};

use crate::clients::gemini::GeminiClient;
use crate::clients::hevy::HevyClient;
use crate::config::Config;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header::AUTHORIZATION};
use axum::response::IntoResponse;
use serde::Deserialize;

#[derive(Clone)]
struct AppState {
    config: Config,
    hevy_client: HevyClient,
    gemini_client: GeminiClient,
}

mod clients;
mod config;

#[derive(Deserialize)]
struct WebhookPayload {
    id: String,
    payload: WorkoutIdPayload,
}

#[derive(Deserialize)]
struct WorkoutIdPayload {
    #[serde(rename = "workoutId")]
    workout_id: String,
}

async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    let auth_header = match headers.get(AUTHORIZATION) {
        Some(header) => header,
        None => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    if !auth_str.starts_with("Bearer ") {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let token = &auth_str[7..];

    if token != state.config.webhook_token {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Use the HevyClient to fetch workout data
    match state
        .hevy_client
        .get_workout(&payload.payload.workout_id)
        .await
    {
        Ok(workout) => {
            println!("Retrieved workout: {}", workout.title);
            Json(workout).into_response()
        }
        Err(e) => {
            eprintln!("Failed to fetch workout: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("Failed to load config");

    let hevy_client = HevyClient::new(&config).expect("Failed to create HevyClient");
    let gemini_client =
        GeminiClient::new(config.gemini_api_key.clone(), config.gemini_model.clone());

    let state = AppState {
        config: config.clone(),
        hevy_client,
        gemini_client,
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/webhook", post(webhook_handler))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
