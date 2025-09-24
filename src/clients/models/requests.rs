use serde::Serialize;

use crate::clients::models::common::ExerciseForUpdate;

#[derive(Debug, Serialize)]
pub struct RoutineUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercises: Option<Vec<ExerciseForUpdate>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateRoutineRequest {
    pub routine: RoutineUpdate,
}
