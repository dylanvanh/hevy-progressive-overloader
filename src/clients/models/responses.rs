use serde::{Deserialize, Serialize};

use crate::clients::models::common::Exercise;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutResponse {
    pub id: String,
    pub title: String,
    pub routine_id: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub updated_at: String,
    pub created_at: String,
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineResponse {
    pub id: String,
    pub title: String,
    pub folder_id: Option<String>,
    pub updated_at: String,
    pub created_at: String,
    pub exercises: Vec<Exercise>,
}

// Single routine envelope used by GET /v1/routines/{id}
#[derive(Debug, Deserialize)]
pub struct RoutineApiResponse {
    pub routine: RoutineResponse,
}

// Update routine envelope returned by PUT /v1/routines/{id} (array)
#[derive(Debug, Deserialize)]
pub struct RoutineUpdateApiResponse {
    pub routine: Vec<RoutineResponse>,
}

// Workouts list response from GET /v1/workouts
#[derive(Debug, Deserialize)]
pub struct WorkoutsListResponse {
    pub workouts: Vec<WorkoutResponse>,
    pub page: i32,
    pub page_size: i32,
    pub total_count: i32,
}
