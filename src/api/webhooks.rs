use axum::Json;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};
use serde::Deserialize;
use std::result::Result;

use crate::clients::hevy::HevyClient;
use crate::clients::models::common::ExerciseForUpdate;
use crate::config::Config;
use crate::services::progressive_overload::{
    ProgressiveOverloadRequest, ProgressiveOverloadService,
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

pub async fn handle_workout_completion(
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

    let routine_exercises_for_update: Vec<ExerciseForUpdate> = routine
        .exercises
        .iter()
        .map(|exercise| exercise.to_update_format())
        .collect();

    let existing_exercise_count = routine_exercises_for_update.len();

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

    let exercise_suggestions = state
        .progressive_overload_service
        .build_exercise_suggestions(&response);

    let suggestion_count = exercise_suggestions.len();

    for (template_id, note) in &exercise_suggestions {
        tracing::debug!(exercise_template_id = %template_id, note = %note, "progressive_overload.exercise_suggestion");
    }

    tracing::info!(
        workout_id = %workout.id,
        routine_id = %workout.routine_id,
        exercise_count = existing_exercise_count,
        suggestion_count,
        "progressive_overload.update_prepared"
    );

    if suggestion_count == 0 {
        tracing::warn!(
            workout_id = %workout.id,
            routine_id = %workout.routine_id,
            "progressive_overload.no_suggestions_generated"
        );
    }

    let routine_notes_value = None;

    let updated_exercises = routine_exercises_for_update
        .into_iter()
        .map(|mut exercise| {
            if let Some(new_notes) = exercise_suggestions.get(&exercise.exercise_template_id) {
                exercise.notes = Some(new_notes.clone());
            }
            exercise
        })
        .collect();

    let update_result = state
        .hevy_client
        .update_routine(
            &workout.routine_id,
            crate::clients::models::requests::RoutineUpdate {
                title: Some(response.routine_title.clone()),
                notes: routine_notes_value,
                exercises: Some(updated_exercises),
                folder_id: None,
            },
        )
        .await;

    match update_result {
        Ok(_) => {
            tracing::info!(
                workout_id = %workout.id,
                routine_id = %workout.routine_id,
                suggestion_count,
                "routine.update_success"
            );
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                workout_id = %workout.id,
                routine_id = %workout.routine_id,
                suggestion_count,
                "failed to update routine"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
