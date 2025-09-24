use anyhow::Result;
use regex::Regex;
use tracing::warn;

use crate::clients::hevy::HevyClient;
use crate::clients::models::responses::WorkoutResponse;

#[derive(Debug, Clone)]
pub struct DeloadCalculator {
    pub deload_intensity_percentage: f64,
}

pub struct DeloadContextBuilder<'a> {
    pub deload_calculator: &'a DeloadCalculator,
    pub hevy_client: &'a HevyClient,
}

pub struct DeloadContext {
    pub next_week_index: u32,
    pub cycle_instruction: String,
    pub reference_data: String,
}

impl Default for DeloadCalculator {
    fn default() -> Self {
        Self {
            deload_intensity_percentage: 0.60, // 60% intensity for deload
        }
    }
}

impl DeloadCalculator {
    pub fn generate_deload_instruction(&self, has_reference: bool) -> String {
        if has_reference {
            format!(
                " CYCLE TRANSITION: You are transitioning from Week 8 (deload) to Week 1 of a NEW 8-week block. \
                This should be a DELOAD week with {}% intensity and reduced volume based on the reference workout provided \
                (either Week 1 from previous cycle or Week 7 max effort as baseline). \
                Apply the deload percentage to the reference weights. Focus on form, recovery, and conservative loading.",
                (self.deload_intensity_percentage * 100.0) as u32
            )
        } else {
            format!(
                " CYCLE TRANSITION: You are transitioning from Week 8 (deload) to Week 1 of a NEW 8-week block. \
                This should be a DELOAD week with {}% intensity reduction from current weights and reduced volume. \
                Focus on form, recovery, and conservative loading to prepare for the new training cycle.",
                (self.deload_intensity_percentage * 100.0) as u32
            )
        }
    }

    pub fn extract_day_from_title(&self, title: &str) -> Option<u32> {
        let day_regex = Regex::new(r"(?i)day\s*(\d+)").unwrap();
        day_regex
            .captures(title)
            .and_then(|captures| captures.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
}

impl<'a> DeloadContextBuilder<'a> {
    pub async fn create_deload_transition_context(
        &self,
        current_week_index: u32,
        workout: &WorkoutResponse,
    ) -> DeloadContext {
        let next_week_index = if current_week_index >= 8 {
            1
        } else {
            current_week_index + 1
        };

        // If we're not in the 8th week, we don't need to generate a deload transition context
        if current_week_index < 8 {
            return DeloadContext {
                next_week_index,
                cycle_instruction: String::new(),
                reference_data: String::new(),
            };
        }

        // If we're in the 8th week, we need to find the Week 1 reference workout
        match self.find_week1_reference_with_fallback(workout).await {
            Ok(Some(week1_reference)) => {
                let instruction = self.deload_calculator.generate_deload_instruction(true);
                let week_label =
                    if super::ai_parser::extract_week_from_title(&week1_reference.title) == Some(1)
                    {
                        "WEEK 1 REFERENCE WORKOUT"
                    } else {
                        "WEEK 7 REFERENCE WORKOUT (max effort baseline)"
                    };

                let reference_data = format!(
                    "\n\n{} (for deload calculation):\n{}",
                    week_label,
                    super::ai_prompt::format_workout_for_prompt(&week1_reference)
                );

                DeloadContext {
                    next_week_index,
                    cycle_instruction: format!("\n\n{}", instruction),
                    reference_data,
                }
            }
            Ok(None) => {
                let instruction = self.deload_calculator.generate_deload_instruction(false);

                DeloadContext {
                    next_week_index,
                    cycle_instruction: format!("\n\n{}", instruction),
                    reference_data: String::new(),
                }
            }
            Err(error) => {
                warn!("Failed to find Week 1 reference: {}", error);
                let instruction = self.deload_calculator.generate_deload_instruction(false);

                DeloadContext {
                    next_week_index,
                    cycle_instruction: format!("\n\n{}", instruction),
                    reference_data: String::new(),
                }
            }
        }
    }

    async fn find_week1_reference(
        &self,
        current_workout: &WorkoutResponse,
    ) -> Result<Option<WorkoutResponse>> {
        let current_day = self
            .deload_calculator
            .extract_day_from_title(&current_workout.title);

        if current_day.is_none() {
            return Ok(None);
        }

        let current_day = current_day.unwrap();

        let max_pages = 10;
        let page_size = 10;

        for page in 0..max_pages {
            match self.hevy_client.get_workouts(page, page_size).await {
                Ok(workouts_response) => {
                    for workout in &workouts_response.workouts {
                        if self.is_week1_same_day_workout(workout, current_day) {
                            return Ok(Some(workout.clone()));
                        }
                    }

                    if (page + 1) * page_size >= workouts_response.total_count {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch workouts page {}: {}", page, e);
                    continue;
                }
            }
        }

        Ok(None)
    }

    async fn find_week1_reference_with_fallback(
        &self,
        current_workout: &WorkoutResponse,
    ) -> Result<Option<WorkoutResponse>> {
        if let Some(reference) = self.find_week1_reference(current_workout).await? {
            return Ok(Some(reference));
        }

        let max_pages = 10;
        let page_size = 10;

        for page in 0..max_pages {
            match self.hevy_client.get_workouts(page, page_size).await {
                Ok(workouts_response) => {
                    for workout in &workouts_response.workouts {
                        if super::ai_parser::extract_week_from_title(&workout.title) == Some(7)
                            && workout.routine_id == current_workout.routine_id
                        {
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

        Ok(None)
    }

    fn is_week1_same_day_workout(&self, workout: &WorkoutResponse, target_day: u32) -> bool {
        let week = super::ai_parser::extract_week_from_title(&workout.title);
        let day = self
            .deload_calculator
            .extract_day_from_title(&workout.title);

        match (week, day) {
            (Some(1), Some(workout_day)) => workout_day == target_day,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_deload_instruction_with_reference() {
        let calculator = DeloadCalculator::default();
        let instruction = calculator.generate_deload_instruction(true);

        assert!(instruction.contains(" CYCLE TRANSITION"));
        assert!(instruction.contains("60% intensity"));
        assert!(instruction.contains("reference workout provided"));
    }

    #[test]
    fn test_generate_deload_instruction_without_reference() {
        let calculator = DeloadCalculator::default();
        let instruction = calculator.generate_deload_instruction(false);

        assert!(instruction.contains(" CYCLE TRANSITION"));
        assert!(instruction.contains("60% intensity"));
        assert!(instruction.contains("current weights"));
    }

    #[test]
    fn test_extract_day_from_title() {
        let calculator = DeloadCalculator::default();

        assert_eq!(calculator.extract_day_from_title("Day 1 - Week 3"), Some(1));
        assert_eq!(calculator.extract_day_from_title("Week 5 - Day 2"), Some(2));
        assert_eq!(
            calculator.extract_day_from_title("Upper Body Day 3"),
            Some(3)
        );
        assert_eq!(calculator.extract_day_from_title("Push Day"), None);
    }
}
