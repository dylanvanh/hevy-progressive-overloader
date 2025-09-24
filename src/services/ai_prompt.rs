use crate::clients::models::{
    common::{Exercise, ExerciseSet},
    responses::{RoutineResponse, WorkoutResponse},
};

use crate::services::deload::DeloadContext;

pub fn format_workout_for_prompt(workout: &WorkoutResponse) -> String {
    let mut output = format!("Workout Title: {}\n", workout.title);
    output.push_str(&format!("Start Time: {}\n", workout.start_time));
    output.push_str(&format!("End Time: {}\n", workout.end_time));
    output.push_str("\nExercises:\n");
    output.push_str(&format_exercise_list(&workout.exercises));
    output
}

pub fn format_routine_for_prompt(routine: &RoutineResponse) -> String {
    let mut output = format!(
        "ROUTINE TEMPLATE:\nRoutine: {}\n\nExercises:\n",
        routine.title
    );

    output.push_str(&format_exercise_list(&routine.exercises));
    output
}

fn format_exercise_list(exercises: &[Exercise]) -> String {
    exercises
        .iter()
        .map(|exercise| {
            let mut block = format!("- {} ({})\n", exercise.title, exercise.exercise_template_id);
            block.push_str(&format_set_list(&exercise.sets));
            block.push('\n');
            block
        })
        .collect::<Vec<_>>()
        .join("")
}

fn format_set_list(sets: &[ExerciseSet]) -> String {
    sets.iter()
        .map(|set| {
            format!(
                "  * Set {}: {} x {} ({})\n",
                set.index + 1,
                format_weight(set.weight_kg),
                format_reps(set.reps),
                set.set_type
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn format_weight(weight: Option<f32>) -> String {
    match weight {
        Some(value) if (value.fract()).abs() > f32::EPSILON => format!("{:.1}kg", value),
        Some(value) => format!("{:.0}kg", value),
        None => "BW".to_string(),
    }
}

fn format_reps(reps: Option<u32>) -> String {
    reps.map(|value| value.to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

pub fn build_progressive_overload_prompt(
    workout: &WorkoutResponse,
    routine: &RoutineResponse,
    deload_context: &DeloadContext,
    current_week_index: u32,
    routine_title: &str,
) -> String {
    format!(
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
7. Keep exercise notes CONCISE - only include RPE targets, no explanatory text
8. For any field that has no meaningful value, ALWAYS use null, never "N/A" or empty strings

OUTPUT FORMAT:
Return ONLY a JSON object with this exact structure:
{{
    "updated_exercises": [
        {{
            "index": 0,
            "title": "Exercise Name",
            "notes": "RPE 8",
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
        format_workout_for_prompt(workout),
        format_routine_for_prompt(routine),
        deload_context.reference_data,
        current_week_index,
        deload_context.cycle_instruction,
        deload_context.next_week_index,
        routine_title,
        current_week_index,
        deload_context.next_week_index
    )
}
