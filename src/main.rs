use axum::{
    Router,
    routing::{get, post},
};

use crate::api::webhooks::{AppState, webhook_handler};
use crate::clients::hevy::HevyClient;
use crate::config::Config;
use crate::services::progressive_overload::ProgressiveOverloadService;

mod api;
mod clients;
mod config;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_env()?;

    let hevy_client = HevyClient::new(&config)?;
    let gemini_client = crate::clients::gemini::GeminiClient::new(
        config.gemini_api_key.clone(),
        config.gemini_model.clone(),
    );
    let progressive_overload_service =
        ProgressiveOverloadService::new(gemini_client.clone(), hevy_client.clone());

    let state = AppState {
        config: config.clone(),
        hevy_client,
        progressive_overload_service,
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/webhook", post(webhook_handler))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    tracing::info!(port = %config.port, "server.listening");
    axum::serve(listener, app).await?;
    Ok(())
}
