use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::clients::gemini::GeminiClient;
use crate::clients::hevy::HevyClient;
use crate::clients::models::{
    common::Exercise,
    responses::{RoutineResponse, WorkoutResponse},
};
use crate::services::deload::DeloadCalculator;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressiveOverloadRequest {
    pub current_workout: WorkoutResponse,
    pub routine: RoutineResponse,
}

#[derive(Debug, Deserialize)]
pub struct ProgressiveOverloadResponse {
    pub updated_exercises: Vec<Exercise>,
    pub week_number: u32,
    pub routine_title: String,
}

#[derive(Clone)]
pub struct ProgressiveOverloadService {
    gemini_client: GeminiClient,
    hevy_client: HevyClient,
    deload_calculator: DeloadCalculator,
}

impl ProgressiveOverloadService {
    pub fn new(gemini_client: GeminiClient, hevy_client: HevyClient) -> Self {
        let deload_calculator = DeloadCalculator::default();

        Self {
            gemini_client,
            hevy_client,
            deload_calculator,
        }
    }

    pub async fn process_workout_completion(
        &self,
        request: ProgressiveOverloadRequest,
    ) -> Result<ProgressiveOverloadResponse> {
        let prompt = self
            .build_progressive_overload_prompt(&request.current_workout, &request.routine)
            .await?;

        tracing::debug!(prompt = %prompt, "gemini.prompt");

        let gemini_response = if std::env::var("USE_MOCK_GEMINI").is_ok() {
            self.get_mock_gemini_response()
        } else {
            self.gemini_client.generate_text(&prompt).await?
        };

        tracing::debug!(response = %gemini_response, "gemini.response");

        let parsed_response = self.parse_gemini_response(&gemini_response)?;
        Ok(parsed_response)
    }

    fn get_mock_gemini_response(&self) -> String {
        r#"{
    "updated_exercises": [{
        "index": 0,
        "title": "Bench Press (Barbell)",
        "notes": "Mock progressive overload response for testing",
        "exercise_template_id": "79D0BB3A",
        "superset_id": null,
        "sets": [{
            "index": 0,
            "type": "normal",
            "weight_kg": 75.0,
            "reps": 5,
            "distance_meters": null,
            "duration_seconds": null,
            "rpe": 8,
            "custom_metric": null
        }]
    }],
    "week_number": 2,
    "routine_title": "Week 2 - Day 1"
}"#
        .to_string()
    }

    async fn build_progressive_overload_prompt(
        &self,
        workout: &WorkoutResponse,
        routine: &RoutineResponse,
    ) -> Result<String> {
        let (current_week_index, _) = self.extract_week_and_day(&workout.title);

        // Day 1 - Week 1
        let routine_title = self.determine_routine_title_format(&workout.title);

        // TODO: move the mesocycle week period to the env file so it can be 6,8,12 etc
        // Reset to Week 1 after Week 8 (end of 8-week cycle)
        let next_week_index = if current_week_index >= 8 {
            1
        } else {
            current_week_index + 1
        };

        // Handle deload logic when transitioning from Week 8 to Week 1
        let (cycle_instruction, reference_data) = if current_week_index >= 8 {
            // Try to find Week 1 reference for deload
            match self.find_week1_reference_with_fallback(workout).await {
                Ok(Some(week1_reference)) => {
                    let instruction = self.deload_calculator.generate_deload_instruction(true);
                    let reference_data = format!(
                        "\n\nWEEK 1 REFERENCE WORKOUT (for deload calculation):\n{}",
                        self.format_workout_for_prompt(&week1_reference)
                    );
                    (format!("\n\n{}", instruction), reference_data)
                }
                Ok(None) => {
                    let instruction = self.deload_calculator.generate_deload_instruction(false);
                    (format!("\n\n{}", instruction), String::new())
                }
                Err(e) => {
                    warn!("Failed to find Week 1 reference: {}", e);
                    let instruction = self.deload_calculator.generate_deload_instruction(false);
                    (format!("\n\n{}", instruction), String::new())
                }
            }
        } else {
            (String::new(), String::new())
        };

        let prompt = format!(
            r#"You are a professional strength and conditioning coach specializing in block periodization for an 8-week strength-focused training cycle.

CURRENT WORKOUT DATA:
{}

{}{}

TRAINING CONTEXT:
- Client is a hybrid athlete (strength + cardio)
- Focuses on main compound movements: Bench Press, Squat, Overhead Press, Romanian Deadlift, Pendlay Row
- Prefers low-moderate volume (2-4 sets per exercise)
- Uses 3-day split: Day 1 (Upper), Day 2 (Lower), Day 3 (Full Body)
- Prioritizes strength gains over hypertrophy
- Currently in week {} of 8-week block
- If there is a set with 1 rep with weight of 1, then it was a to failure set on an arbitrary weight. Keep the weight at 1 when.
- The smallest weight plate for barbell exercises available is 2.5kg (5kg if both sides)
- Don't add a warmup, if there was a warmup from the workout leave it as is{}

PERIODIZATION STRATEGY:
Week 1-2: Foundation (7 reps @ 75%, 2-3 sets)
Week 3-4: Intensity increase (6 reps @ 80%, 3-4 sets)
Week 5-6: Heavy work (5 reps @ 85%, 3-4 sets)
Week 7: Testing (3-5RM attempts @ 90%+)
Week 8: Deload (5 reps @ 60%, 2-3 sets)

PROGRESSION RULES:
1. Start conservatively with 2 sets, build to 3-4 sets max
2. Prioritize intensity over volume
3. Use same exercises throughout block
4. Progress: reps → weight → sets → testing
5. Accessories stay minimal (2 sets, RPE 6-7)
6. You MUST use the SAME exercises from the current workout

OUTPUT FORMAT:
Return ONLY a JSON object with this exact structure:
{{
    "updated_exercises": [
        {{
            "index": 0,
            "title": "Exercise Name",
            "notes": "Updated progression notes",
            "exercise_template_id": "original_id",
            "superset_id": null,
            "sets": [
                {{
                    "index": 0,
                    "type": "normal",
                    "weight_kg": 85.0,
                    "reps": 7,
                    "distance_meters": null,
                    "duration_seconds": null,
                    "rpe": 7,
                    "custom_metric": null
                }}
            ]
        }}
    ],
    "week_number": {},
    "routine_title": "{}"
}}

CURRENT WEEK: {}
NEXT WEEK TARGET: {}"#,
            self.format_workout_for_prompt(workout),
            self.format_routine_for_prompt(routine),
            reference_data,
            current_week_index,
            cycle_instruction,
            next_week_index,
            routine_title,
            current_week_index,
            next_week_index
        );

        Ok(prompt)
    }

    fn format_workout_for_prompt(&self, workout: &WorkoutResponse) -> String {
        let mut output = format!("Workout Title: {}\n", workout.title);
        output.push_str(&format!("Start Time: {}\n", workout.start_time));
        output.push_str(&format!("End Time: {}\n", workout.end_time));
        output.push_str("\nExercises:\n");

        for exercise in &workout.exercises {
            output.push_str(&format!(
                "- {} ({})\n",
                exercise.title, exercise.exercise_template_id
            ));
            for set in &exercise.sets {
                let weight = set.weight_kg.map_or("BW".to_string(), |w| w.to_string());
                let reps = set.reps.map_or("N/A".to_string(), |r| r.to_string());
                let set_type = &set.set_type;
                output.push_str(&format!(
                    "  * Set {}: {}kg x {} reps ({})\n",
                    set.index + 1,
                    weight,
                    reps,
                    set_type
                ));
            }
            output.push('\n');
        }

        output
    }

    fn format_routine_for_prompt(&self, routine: &RoutineResponse) -> String {
        let mut output = format!(
            "ROUTINE TEMPLATE:\nRoutine: {}\n\nExercises:\n",
            routine.title
        );

        for exercise in &routine.exercises {
            output.push_str(&format!(
                "- {} ({})\n",
                exercise.title, exercise.exercise_template_id
            ));
            for set in &exercise.sets {
                let weight = set.weight_kg.map_or("BW".to_string(), |w| w.to_string());
                let reps = set.reps.map_or("N/A".to_string(), |r| r.to_string());
                let set_type = &set.set_type;
                output.push_str(&format!(
                    "  * Set {}: {}kg x {} reps ({})\n",
                    set.index + 1,
                    weight,
                    reps,
                    set_type
                ));
            }
            output.push('\n');
        }

        output
    }

    fn extract_week_and_day(&self, title: &str) -> (u32, u32) {
        let week_regex = Regex::new(r"(?i)week\s*(\d+)").unwrap();
        let day_regex = Regex::new(r"(?i)day\s*(\d+)").unwrap();

        let week = week_regex
            .captures(title)
            .and_then(|captures| captures.get(1))
            .and_then(|m| m.as_str().parse().ok());

        let day = day_regex
            .captures(title)
            .and_then(|captures| captures.get(1))
            .and_then(|m| m.as_str().parse().ok());

        (week.unwrap_or(1), day.unwrap_or(1))
    }

    fn determine_routine_title_format(&self, title: &str) -> String {
        let week_regex = Regex::new(r"(?i)week\s*(\d+)").unwrap();
        let day_regex = Regex::new(r"(?i)day\s*(\d+)").unwrap();

        let has_week = week_regex.captures(title).is_some();
        let has_day = day_regex.captures(title).is_some();

        let (current_week, current_day) = self.extract_week_and_day(title);

        // Reset to Week 1 after Week 8 (end of 8-week cycle)
        let next_week = if current_week >= 8 {
            1
        } else {
            current_week + 1
        };

        match (has_day, has_week) {
            (true, true) => format!("Day {} - Week {}", current_day, next_week),
            (true, false) => format!("Day {}", current_day + 1),
            (false, true) => format!("Week {}", next_week),
            // if no week assume it was week 1
            (false, false) => "Week 2".to_string(),
        }
    }

    #[allow(dead_code)]
    fn extract_week_number(&self, title: &str) -> u32 {
        self.extract_week_and_day(title).0
    }

    /// Parse Gemini response into ProgressiveOverloadResponse
    fn parse_gemini_response(&self, response: &str) -> Result<ProgressiveOverloadResponse> {
        let json_content = self.extract_json_from_response(response);
        let parsed_json = self.parse_json_string(&json_content)?;
        let exercises = self.extract_exercises_from_json(&parsed_json)?;
        let week_number = self.extract_week_number_from_json(&parsed_json);
        let routine_title = self.extract_routine_title_from_json(&parsed_json);

        Ok(ProgressiveOverloadResponse {
            updated_exercises: exercises,
            week_number,
            routine_title,
        })
    }

    fn extract_json_from_response(&self, response: &str) -> String {
        if let Some(json_block_start) = response.find("```json") {
            let content_start = json_block_start + "```json".len();

            if let Some(remaining_content) = response.get(content_start..) {
                if let Some(code_block_end) = remaining_content.find("```") {
                    return remaining_content[..code_block_end].trim().to_string();
                }
                return remaining_content.trim().to_string();
            }
        }

        response.trim().to_string()
    }

    fn parse_json_string(&self, json_str: &str) -> Result<serde_json::Value> {
        serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))
    }

    fn extract_exercises_from_json(&self, json: &serde_json::Value) -> Result<Vec<Exercise>> {
        let exercises_value = json
            .get("updated_exercises")
            .ok_or_else(|| anyhow::anyhow!("Missing 'updated_exercises' field in JSON response"))?;

        serde_json::from_value(exercises_value.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse exercises array: {}", e))
    }

    fn extract_week_number_from_json(&self, json: &serde_json::Value) -> u32 {
        json.get("week_number")
            .and_then(|w| w.as_u64())
            .map(|n| n as u32)
            .unwrap_or(1)
    }

    fn extract_routine_title_from_json(&self, json: &serde_json::Value) -> String {
        json.get("routine_title")
            .and_then(|t| t.as_str())
            .unwrap_or("Updated Routine")
            .to_string()
    }

    /// Find Week 1 reference workout for the same day as the current workout
    async fn find_week1_reference(
        &self,
        current_workout: &WorkoutResponse,
    ) -> Result<Option<WorkoutResponse>> {
        let current_day = self
            .deload_calculator
            .extract_day_from_title(&current_workout.title);

        if current_day.is_none() {
            info!(
                "Current workout '{}' doesn't have a day number, cannot find Week 1 reference",
                current_workout.title
            );
            return Ok(None);
        }

        let current_day = current_day.unwrap();
        info!("Looking for Week 1 reference for Day {}", current_day);

        // Search through multiple pages to find Week 1 reference
        // 100 workout search
        // (if 6 day split for 12 weeks , that is 72 (still catered))
        let max_pages = 10;
        let page_size = 10;

        for page in 0..max_pages {
            debug!("Searching page {} for Week 1 reference", page);

            match self.hevy_client.get_workouts(page, page_size).await {
                Ok(workouts_response) => {
                    // Look through workouts in this page
                    for workout in &workouts_response.workouts {
                        if self.is_week1_same_day_workout(workout, current_day) {
                            info!(
                                "Found Week 1 reference: '{}' (ID: {}) for Day {}",
                                workout.title, workout.id, current_day
                            );
                            return Ok(Some(workout.clone()));
                        }
                    }

                    // If we've searched all available workouts, stop
                    if (page + 1) * page_size >= workouts_response.total_count {
                        debug!("Reached end of available workouts at page {}", page);
                        break;
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch workouts page {}: {}", page, e);
                    // Continue to next page on error
                    continue;
                }
            }
        }

        info!(
            "No Week 1 reference found for Day {} after searching {} pages",
            current_day, max_pages
        );
        Ok(None)
    }

    /// Find Week 1 reference with fallback strategies
    async fn find_week1_reference_with_fallback(
        &self,
        current_workout: &WorkoutResponse,
    ) -> Result<Option<WorkoutResponse>> {
        // First try exact match
        if let Some(reference) = self.find_week1_reference(current_workout).await? {
            return Ok(Some(reference));
        }

        // Fallback: try to find any Week 1 workout from the same routine
        info!("No exact Day match found, looking for any Week 1 workout from same routine");

        let max_pages = 10;
        let page_size = 10;

        for page in 0..max_pages {
            match self.hevy_client.get_workouts(page, page_size).await {
                Ok(workouts_response) => {
                    for workout in &workouts_response.workouts {
                        if self.extract_week_from_title(&workout.title) == Some(1)
                            && workout.routine_id == current_workout.routine_id
                        {
                            info!(
                                "Found Week 1 fallback reference: '{}' (same routine)",
                                workout.title
                            );
                            return Ok(Some(workout.clone()));
                        }
                    }

                    if (page + 1) * page_size >= workouts_response.total_count {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch workouts page {} for fallback: {}", page, e);
                    continue;
                }
            }
        }

        info!("No Week 1 reference found even with fallback strategy");
        Ok(None)
    }

    /// Check if a workout is Week 1 and matches the target day
    fn is_week1_same_day_workout(&self, workout: &WorkoutResponse, target_day: u32) -> bool {
        let week = self.extract_week_from_title(&workout.title);
        let day = self
            .deload_calculator
            .extract_day_from_title(&workout.title);

        match (week, day) {
            (Some(1), Some(workout_day)) => {
                debug!(
                    "Checking workout '{}': Week {}, Day {} (target Day {})",
                    workout.title,
                    week.unwrap(),
                    workout_day,
                    target_day
                );
                workout_day == target_day
            }
            _ => {
                debug!(
                    "Workout '{}' doesn't match criteria: Week {:?}, Day {:?}",
                    workout.title, week, day
                );
                false
            }
        }
    }

    /// Extract week number from workout title
    fn extract_week_from_title(&self, title: &str) -> Option<u32> {
        let week_regex = Regex::new(r"(?i)week\s*(\d+)").unwrap();
        week_regex
            .captures(title)
            .and_then(|captures| captures.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_week_number() {
        let service = ProgressiveOverloadService::new(
            GeminiClient::new("test".to_string(), "test".to_string()),
            HevyClient::new(&crate::config::Config {
                hevy_api_url: "https://api.hevyapp.com".to_string(),
                hevy_api_key: "test".to_string(),
                webhook_token: "test".to_string(),
                gemini_api_key: "test".to_string(),
                gemini_model: "test".to_string(),
                port: "3000".to_string(),
            })
            .unwrap(),
        );

        assert_eq!(service.extract_week_number("Week 1 - Day 1: Push"), 1);
        assert_eq!(service.extract_week_number("week 5 - chest day"), 5);
        assert_eq!(service.extract_week_number("Push Day"), 1);
    }

    #[test]
    fn test_extract_week_and_day() {
        let service = ProgressiveOverloadService::new(
            GeminiClient::new("test".to_string(), "test".to_string()),
            HevyClient::new(&crate::config::Config {
                hevy_api_url: "https://api.hevyapp.com".to_string(),
                hevy_api_key: "test".to_string(),
                webhook_token: "test".to_string(),
                gemini_api_key: "test".to_string(),
                gemini_model: "test".to_string(),
                port: "3000".to_string(),
            })
            .unwrap(),
        );

        assert_eq!(service.extract_week_and_day("Day 1 - Week 2"), (2, 1));
        assert_eq!(service.extract_week_and_day("Day 3 - Week 5"), (5, 3));
        assert_eq!(service.extract_week_and_day("Week 4 - Day 2"), (4, 2));
        assert_eq!(service.extract_week_and_day("Push Day"), (1, 1));
        assert_eq!(service.extract_week_and_day("Day 1"), (1, 1));
        assert_eq!(service.extract_week_and_day("Day4 -week 2"), (2, 4));
    }

    #[test]
    fn test_determine_routine_title_format() {
        let service = ProgressiveOverloadService::new(
            GeminiClient::new("test".to_string(), "test".to_string()),
            HevyClient::new(&crate::config::Config {
                hevy_api_url: "https://api.hevyapp.com".to_string(),
                hevy_api_key: "test".to_string(),
                webhook_token: "test".to_string(),
                gemini_api_key: "test".to_string(),
                gemini_model: "test".to_string(),
                port: "3000".to_string(),
            })
            .unwrap(),
        );

        assert_eq!(
            service.determine_routine_title_format("Day 1 - Week 2"),
            "Day 1 - Week 3"
        );
        assert_eq!(
            service.determine_routine_title_format("Day4 -week 2"),
            "Day 4 - Week 3"
        );
        assert_eq!(service.determine_routine_title_format("Day 1"), "Day 2");
        assert_eq!(service.determine_routine_title_format("Week 2"), "Week 3");
        // "Push Day" contains "day" but no number, so should be treated as no day
        assert_eq!(service.determine_routine_title_format("Push Day"), "Week 2");
        assert_eq!(
            service.determine_routine_title_format("Chest Press"),
            "Week 2"
        );
    }

    #[test]
    fn test_week_8_boundary_condition() {
        let service = ProgressiveOverloadService::new(
            GeminiClient::new("test".to_string(), "test".to_string()),
            HevyClient::new(&crate::config::Config {
                hevy_api_url: "https://api.hevyapp.com".to_string(),
                hevy_api_key: "test".to_string(),
                webhook_token: "test".to_string(),
                gemini_api_key: "test".to_string(),
                gemini_model: "test".to_string(),
                port: "3000".to_string(),
            })
            .unwrap(),
        );

        // Test Week 8 resets to Week 1
        assert_eq!(
            service.determine_routine_title_format("Day 1 - Week 8"),
            "Day 1 - Week 1"
        );
        assert_eq!(service.determine_routine_title_format("Week 8"), "Week 1");

        // Test Week 9+ also resets to Week 1
        assert_eq!(
            service.determine_routine_title_format("Day 2 - Week 9"),
            "Day 2 - Week 1"
        );
        assert_eq!(service.determine_routine_title_format("Week 10"), "Week 1");

        // Test normal weeks still increment normally
        assert_eq!(
            service.determine_routine_title_format("Day 1 - Week 7"),
            "Day 1 - Week 8"
        );
        assert_eq!(service.determine_routine_title_format("Week 7"), "Week 8");
    }

    #[test]
    fn test_extract_week_from_title() {
        let service = ProgressiveOverloadService::new(
            GeminiClient::new("test".to_string(), "test".to_string()),
            HevyClient::new(&crate::config::Config {
                hevy_api_url: "https://api.hevyapp.com".to_string(),
                hevy_api_key: "test".to_string(),
                webhook_token: "test".to_string(),
                gemini_api_key: "test".to_string(),
                gemini_model: "test".to_string(),
                port: "3000".to_string(),
            })
            .unwrap(),
        );

        assert_eq!(service.extract_week_from_title("Week 1 - Day 1"), Some(1));
        assert_eq!(service.extract_week_from_title("Day 2 - Week 3"), Some(3));
        assert_eq!(service.extract_week_from_title("Push Day"), None);
        assert_eq!(service.extract_week_from_title("Week 8 - Upper"), Some(8));
    }
}
