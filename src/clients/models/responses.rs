use serde::{Deserialize, Serialize};

use crate::clients::models::common::Exercise;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutineResponse {
    pub id: String,
    pub title: String,
    pub folder_id: String,
    pub updated_at: String,
    pub created_at: String,
    pub exercises: Vec<Exercise>,
}
