use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Exercise {
    pub index: u32,
    pub title: String,
    pub notes: String,
    pub exercise_template_id: String,
    pub superset_id: Option<u32>,
    pub sets: Vec<ExerciseSet>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExerciseSet {
    pub index: u32,
    #[serde(rename = "type")]
    pub set_type: String, // "warmup", "normal", "failure", "dropset"
    pub weight_kg: Option<f32>,
    pub reps: Option<u32>,
    pub distance_meters: Option<u32>,
    pub duration_seconds: Option<u32>,
    pub rpe: Option<f32>,
    pub custom_metric: Option<f32>,
}
