use axum::{
    Router,
    routing::{get, post},
};

use crate::api::webhooks::{AppState, webhook_handler};
use crate::clients::gemini::GeminiClient;
use crate::clients::hevy::HevyClient;
use crate::config::Config;

mod api;
mod clients;
mod config;

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
