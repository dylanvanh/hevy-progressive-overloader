use axum::{Router, routing::post};
use std::collections::HashSet;
use std::sync::Arc;

use crate::api::webhooks::{AppState, handle_workout_completion};
use crate::clients::hevy::HevyClient;
use crate::config::Config;
use crate::scheduler::start_scheduler;
use crate::services::progressive_overload::ProgressiveOverloadService;

mod api;
mod clients;
mod config;
mod scheduler;
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
        processed_workout_ids: Arc::new(std::sync::Mutex::new(HashSet::new())),
    };

    let app = Router::new()
        .route("/webhook", post(handle_workout_completion))
        .with_state(state.clone());

    // cron scheduler
    let state_arc = Arc::new(state);
    let _scheduler = start_scheduler(Arc::clone(&state_arc)).await?;
    tracing::info!("scheduler.started");

    // Run initial sync on startup
    let state_for_sync = Arc::clone(&state_arc);
    tokio::spawn(async move {
        if let Err(e) = crate::scheduler::run_sync(state_for_sync).await {
            tracing::error!(error = %e, "initial.sync_failed");
        }
    });
    tracing::info!("initial.sync_started");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    tracing::info!(port = %config.port, "server.listening");

    axum::serve(listener, app).await?;
    Ok(())
}
