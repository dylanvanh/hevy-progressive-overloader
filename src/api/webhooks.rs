use axum::Json;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::clients::gemini::GeminiClient;
use crate::clients::hevy::HevyClient;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hevy_client: HevyClient,
    pub gemini_client: GeminiClient,
}

#[derive(Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    pub payload: WorkoutIdPayload,
}

#[derive(Deserialize)]
pub struct WorkoutIdPayload {
    #[serde(rename = "workoutId")]
    pub workout_id: String,
}

pub async fn webhook_handler(
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
