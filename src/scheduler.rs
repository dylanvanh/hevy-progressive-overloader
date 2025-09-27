use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::api::webhooks::{AppState, process_single_workout};

pub async fn start_scheduler(state: Arc<AppState>) -> anyhow::Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;

    let state_clone = Arc::clone(&state);

    scheduler
        .add(Job::new_async("0 */15 * * * *", move |_uuid, _l| {
            // .add(Job::new_async("* * * * * *", move |_uuid, _l| {
            let state = Arc::clone(&state_clone);
            Box::pin(async move {
                if let Err(e) = run_sync(state).await {
                    tracing::error!(error = %e, "cron.sync_failed");
                }
            })
        })?)
        .await?;

    scheduler.start().await?;
    Ok(scheduler)
}

pub async fn run_sync(state: Arc<AppState>) -> anyhow::Result<()> {
    tracing::info!("cron.sync_started");

    // Fetch recent workouts, say last 100
    let workouts_response = state.hevy_client.get_workouts(1, 10).await?;
    let mut workouts = workouts_response.workouts;

    // Filter to workouts created in the last 24 hours
    let cutoff = Utc::now() - Duration::hours(24);
    workouts.retain(|w| {
        if let Ok(created) = DateTime::parse_from_rfc3339(&w.created_at) {
            created > cutoff
        } else {
            false
        }
    });

    tracing::info!(workout_count = workouts.len(), "workouts.fetched_recent");

    for workout in workouts {
        let workout_id = workout.id.clone();

        // Check if already processed
        {
            let processed = state.processed_workout_ids.lock().unwrap();
            if processed.contains(&workout_id) {
                tracing::debug!(%workout_id, "workout.already_processed");
                continue;
            }
        }

        // Process the workout using the shared function
        process_single_workout(&state, workout_id).await;
    }

    tracing::info!("cron.sync_completed");
    Ok(())
}
