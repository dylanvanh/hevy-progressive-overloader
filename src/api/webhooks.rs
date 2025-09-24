use axum::Json;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};
use serde::Deserialize;
use std::result::Result;

use crate::clients::hevy::HevyClient;
use crate::config::Config;
use crate::services::progressive_overload::{
    ProgressiveOverloadRequest, ProgressiveOverloadResponse, ProgressiveOverloadService,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hevy_client: HevyClient,
    pub progressive_overload_service: ProgressiveOverloadService,
}

#[derive(Deserialize)]
pub struct WebhookPayload {
    pub payload: WorkoutIdPayload,
}

#[derive(Deserialize)]
pub struct WorkoutIdPayload {
    #[serde(rename = "workoutId")]
    pub workout_id: String,
}

fn authenticate_request(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    let auth_header = match headers.get(AUTHORIZATION) {
        Some(header) => header,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    if !auth_str.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_str[7..];

    if token != state.config.webhook_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
}

pub async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    if let Err(response) = authenticate_request(&headers, &state) {
        return response.into_response();
    }

    let workout = match state
        .hevy_client
        .get_workout(&payload.payload.workout_id)
        .await
    {
        Ok(workout) => workout,
        Err(e) => {
            tracing::error!(error = %e, "failed to fetch workout");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    tracing::info!(workout_title = %workout.title, "workout.retrieved");

    if workout.routine_id.is_empty() || workout.routine_id == "null" {
        tracing::info!("workout.no_routine_associated");
        return StatusCode::OK.into_response();
    }

    let routine = match state.hevy_client.get_routine(&workout.routine_id).await {
        Ok(routine) => routine,
        Err(e) => {
            tracing::error!(error = %e, "failed to fetch routine");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let request = ProgressiveOverloadRequest {
        current_workout: workout.clone(),
        routine,
    };

    let response = match state
        .progressive_overload_service
        .process_workout_completion(request)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(error = %e, "failed to process progressive overload");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    tracing::info!(next_week = %response.week_number, routine_title = %response.routine_title, "progressive_overload.processed");

    // Build a human-readable suggestion string instead of mutating sets/reps
    let routine_notes = build_routine_notes(&response);

    let update_result = state
        .hevy_client
        .update_routine(
            &workout.routine_id,
            crate::clients::models::requests::RoutineUpdate {
                title: Some(response.routine_title.clone()),
                notes: Some(routine_notes),
                exercises: None,
                folder_id: None,
            },
        )
        .await;

    match update_result {
        Ok(_) => {
            tracing::info!("routine.update_success");
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to update routine");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn build_routine_notes(response: &ProgressiveOverloadResponse) -> String {
    fn format_weight(weight: f32) -> String {
        if (weight.fract()).abs() < f32::EPSILON {
            format!("{:.0}", weight)
        } else {
            format!("{:.1}", weight)
        }
    }

    let mut notes = String::new();
    notes.push_str("AI suggestions for next session (no auto changes)\n");
    notes.push_str(&format!(
        "Target Week {} • {}\n\n",
        response.week_number, response.routine_title
    ));

    for exercise in &response.updated_exercises {
        let num_sets = exercise.sets.len();

        let all_reps: Vec<Option<u32>> = exercise.sets.iter().map(|s| s.reps).collect();
        let all_weights: Vec<Option<f32>> = exercise.sets.iter().map(|s| s.weight_kg).collect();

        let reps_uniform = all_reps.iter().all(|r| r.is_some() && *r == all_reps[0]);
        let weight_uniform = all_weights
            .iter()
            .all(|w| w.is_some() && *w == all_weights[0]);

        let mut line = String::new();
        line.push_str("- ");
        line.push_str(&exercise.title);
        line.push_str(": ");

        if reps_uniform {
            let reps_val = all_reps[0].unwrap();
            line.push_str(&format!("{} x {} reps", num_sets, reps_val));
        } else {
            let reps_list: Vec<String> = all_reps
                .into_iter()
                .map(|r| r.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string()))
                .collect();
            line.push_str("Reps per set: ");
            line.push_str(&reps_list.join(", "));
        }

        if weight_uniform {
            if let Some(w) = all_weights[0] {
                line.push_str(&format!(" @ {}kg", format_weight(w)));
            }
        }

        notes.push_str(&line);
        notes.push('\n');

        if let Some(extra) = &exercise.notes {
            let trimmed = extra.trim();
            if !trimmed.is_empty() {
                notes.push_str("  Note: ");
                notes.push_str(trimmed);
                notes.push('\n');
            }
        }
    }

    notes
}
