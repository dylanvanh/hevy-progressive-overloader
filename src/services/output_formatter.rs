use std::collections::HashMap;

use crate::services::progressive_overload::ProgressiveOverloadResponse;

pub fn build_exercise_suggestions(
    response: &ProgressiveOverloadResponse,
) -> HashMap<String, String> {
    let mut suggestions = HashMap::new();

    for exercise in &response.updated_exercises {
        let working_sets: Vec<_> = exercise
            .sets
            .iter()
            .filter(|set| !set.set_type.eq_ignore_ascii_case("warmup"))
            .collect();

        let mut lines = Vec::new();

        if !working_sets.is_empty() {
            lines.push(format!("{} sets", working_sets.len()));

            if let Some(notes) = &exercise.notes
                && let Some(rpe) = extract_rpe_from_notes(notes)
            {
                lines.push(format!("RPE {}", rpe))
            }

            for set in working_sets {
                let reps = set
                    .reps
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".to_string());

                let weight = set.weight_kg.map(|value| {
                    if (value.fract()).abs() < f32::EPSILON {
                        format!("{:.0}", value)
                    } else {
                        format!("{:.1}", value)
                    }
                });

                let entry = match weight {
                    Some(weight_str) => format!("{}x{}", weight_str, reps),
                    None => format!("{} reps", reps),
                };

                lines.push(entry);
            }
        }

        if !lines.is_empty() {
            let note = lines.join("\n");
            suggestions.insert(exercise.exercise_template_id.clone(), note);
        }
    }

    suggestions
}

fn extract_rpe_from_notes(notes: &str) -> Option<String> {
    if let Some(start) = notes.to_lowercase().find("rpe") {
        let after_rpe = &notes[start + 3..];
        for word in after_rpe.split_whitespace().take(2) {
            if word.chars().any(|c| c.is_ascii_digit())
                && word.chars().all(|c| c.is_ascii_digit() || c == '-')
            {
                return Some(word.to_string());
            }
        }
    }
    None
}
