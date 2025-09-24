use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub index: u32,
    pub title: String,
    pub notes: Option<String>,
    pub exercise_template_id: String,
    pub superset_id: Option<u32>,
    pub rest_seconds: Option<u32>,
    pub sets: Vec<ExerciseSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ExerciseSetForUpdate {
    #[serde(rename = "type")]
    pub set_type: String, // "warmup", "normal", "failure", "dropset"
    pub weight_kg: Option<f32>,
    pub reps: Option<u32>,
    pub distance_meters: Option<u32>,
    pub duration_seconds: Option<u32>,
    pub custom_metric: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rep_range: Option<RepRange>,
    // Note: rpe field intentionally omitted as it's not allowed in API updates
}

#[derive(Debug, Clone, Serialize)]
pub struct RepRange {
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExerciseForUpdate {
    pub exercise_template_id: String,
    pub superset_id: Option<u32>,
    pub rest_seconds: Option<u32>,
    pub notes: Option<String>,
    pub sets: Vec<ExerciseSetForUpdate>,
}

impl Exercise {
    pub fn to_update_format(&self) -> ExerciseForUpdate {
        ExerciseForUpdate {
            exercise_template_id: self.exercise_template_id.clone(),
            superset_id: self.superset_id,
            rest_seconds: self.rest_seconds,
            notes: self.notes.clone(),
            sets: self.sets.iter().map(|set| set.to_update_format()).collect(),
        }
    }
}

impl ExerciseSet {
    pub fn to_update_format(&self) -> ExerciseSetForUpdate {
        ExerciseSetForUpdate {
            set_type: self.set_type.clone(),
            weight_kg: self.weight_kg,
            reps: self.reps,
            distance_meters: self.distance_meters,
            duration_seconds: self.duration_seconds,
            custom_metric: self.custom_metric,
            rep_range: None, // Will be skipped during serialization due to skip_serializing_if
                             // Note: rpe field is intentionally skipped as it's not allowed in API updates
        }
    }
}
