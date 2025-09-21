use serde::Serialize;

use crate::clients::models::common::Exercise;

#[derive(Debug, Serialize)]
pub struct UpdateRoutineRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercises: Option<Vec<Exercise>>,
}
