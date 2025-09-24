use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use crate::clients::gemini::GeminiClient;
use crate::clients::hevy::HevyClient;
use crate::clients::models::{
    common::Exercise,
    responses::{RoutineResponse, WorkoutResponse},
};
use crate::services::deload::{DeloadCalculator, DeloadContextBuilder};
use crate::services::{ai_parser, ai_prompt, output_formatter};

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
        Self {
            gemini_client,
            hevy_client,
            deload_calculator: DeloadCalculator::default(),
        }
    }

    pub async fn process_workout_completion(
        &self,
        request: ProgressiveOverloadRequest,
    ) -> Result<ProgressiveOverloadResponse> {
        let prompt = self
            .build_progressive_overload_prompt(&request.current_workout, &request.routine)
            .await?;

        debug!(prompt = %prompt, "gemini.prompt");

        let gemini_response = self.gemini_client.generate_text(&prompt).await?;

        debug!(response = %gemini_response, "gemini.response");

        let parsed_response = self.parse_gemini_response(&gemini_response)?;
        Ok(parsed_response)
    }

    async fn build_progressive_overload_prompt(
        &self,
        workout: &WorkoutResponse,
        routine: &RoutineResponse,
    ) -> Result<String> {
        let (current_week_index, _) = ai_parser::extract_week_and_day(&workout.title);
        let routine_title = ai_parser::determine_routine_title_format(&workout.title);

        let deload_context = DeloadContextBuilder {
            deload_calculator: &self.deload_calculator,
            hevy_client: &self.hevy_client,
        }
        .create_deload_transition_context(current_week_index, workout)
        .await;

        Ok(ai_prompt::build_progressive_overload_prompt(
            workout,
            routine,
            &deload_context,
            current_week_index,
            &routine_title,
        ))
    }

    pub fn build_exercise_suggestions(
        &self,
        response: &ProgressiveOverloadResponse,
    ) -> HashMap<String, String> {
        output_formatter::build_exercise_suggestions(response)
    }

    fn parse_gemini_response(&self, response: &str) -> Result<ProgressiveOverloadResponse> {
        ai_parser::parse_gemini_response(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_week_and_day() {
        assert_eq!(ai_parser::extract_week_and_day("Day 1 - Week 2"), (2, 1));
        assert_eq!(ai_parser::extract_week_and_day("Day 3 - Week 5"), (5, 3));
        assert_eq!(ai_parser::extract_week_and_day("Week 4 - Day 2"), (4, 2));
        assert_eq!(ai_parser::extract_week_and_day("Push Day"), (1, 1));
        assert_eq!(ai_parser::extract_week_and_day("Day 1"), (1, 1));
        assert_eq!(ai_parser::extract_week_and_day("Day4 -week 2"), (2, 4));
    }

    #[test]
    fn test_determine_routine_title_format() {
        assert_eq!(
            ai_parser::determine_routine_title_format("Day 1 - Week 2"),
            "Day 1 - Week 3"
        );
        assert_eq!(
            ai_parser::determine_routine_title_format("Day4 -week 2"),
            "Day 4 - Week 3"
        );
        assert_eq!(ai_parser::determine_routine_title_format("Day 1"), "Day 2");
        assert_eq!(
            ai_parser::determine_routine_title_format("Week 2"),
            "Week 3"
        );
        assert_eq!(
            ai_parser::determine_routine_title_format("Push Day"),
            "Week 2"
        );
        assert_eq!(
            ai_parser::determine_routine_title_format("Chest Press"),
            "Week 2"
        );
    }

    #[test]
    fn test_week_8_boundary_condition() {
        assert_eq!(
            ai_parser::determine_routine_title_format("Day 1 - Week 8"),
            "Day 1 - Week 1"
        );
        assert_eq!(
            ai_parser::determine_routine_title_format("Week 8"),
            "Week 1"
        );

        assert_eq!(
            ai_parser::determine_routine_title_format("Day 2 - Week 9"),
            "Day 2 - Week 1"
        );
        assert_eq!(
            ai_parser::determine_routine_title_format("Week 10"),
            "Week 1"
        );

        assert_eq!(
            ai_parser::determine_routine_title_format("Day 1 - Week 7"),
            "Day 1 - Week 8"
        );
        assert_eq!(
            ai_parser::determine_routine_title_format("Week 7"),
            "Week 8"
        );
    }

    #[test]
    fn test_extract_week_from_title() {
        assert_eq!(
            ai_parser::extract_week_from_title("Week 1 - Day 1"),
            Some(1)
        );
        assert_eq!(
            ai_parser::extract_week_from_title("Day 2 - Week 3"),
            Some(3)
        );
        assert_eq!(ai_parser::extract_week_from_title("Push Day"), None);
        assert_eq!(
            ai_parser::extract_week_from_title("Week 8 - Upper"),
            Some(8)
        );
    }
}
