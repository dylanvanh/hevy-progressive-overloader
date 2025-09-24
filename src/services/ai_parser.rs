use anyhow::Result;
use regex::Regex;
use serde_json::Value;

use crate::clients::models::common::Exercise;
use crate::services::progressive_overload::ProgressiveOverloadResponse;

pub fn parse_gemini_response(response: &str) -> Result<ProgressiveOverloadResponse> {
    let json_content = extract_json_from_response(response);
    let parsed_json = parse_json_string(&json_content)?;
    let exercises = extract_exercises_from_json(&parsed_json)?;
    let week_number = extract_week_number_from_json(&parsed_json);
    let routine_title = extract_routine_title_from_json(&parsed_json);

    Ok(ProgressiveOverloadResponse {
        updated_exercises: exercises,
        week_number,
        routine_title,
    })
}

pub fn extract_week_and_day(title: &str) -> (u32, u32) {
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

pub fn determine_routine_title_format(title: &str) -> String {
    let week_regex = Regex::new(r"(?i)week\s*(\d+)").unwrap();
    let day_regex = Regex::new(r"(?i)day\s*(\d+)").unwrap();

    let has_week = week_regex.captures(title).is_some();
    let has_day = day_regex.captures(title).is_some();

    let (current_week, current_day) = extract_week_and_day(title);

    let next_week = if current_week >= 8 {
        1
    } else {
        current_week + 1
    };

    match (has_day, has_week) {
        (true, true) => format!("Day {} - Week {}", current_day, next_week),
        (true, false) => format!("Day {}", current_day + 1),
        (false, true) => format!("Week {}", next_week),
        (false, false) => "Week 2".to_string(),
    }
}

pub fn extract_week_from_title(title: &str) -> Option<u32> {
    let week_regex = Regex::new(r"(?i)week\s*(\d+)").unwrap();
    week_regex
        .captures(title)
        .and_then(|captures| captures.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn extract_json_from_response(response: &str) -> String {
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

fn parse_json_string(json_str: &str) -> Result<Value> {
    serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))
}

fn extract_exercises_from_json(json: &Value) -> Result<Vec<Exercise>> {
    let exercises_value = json
        .get("updated_exercises")
        .ok_or_else(|| anyhow::anyhow!("Missing 'updated_exercises' field in JSON response"))?;

    serde_json::from_value(exercises_value.clone())
        .map_err(|e| anyhow::anyhow!("Failed to parse exercises array: {}", e))
}

fn extract_week_number_from_json(json: &Value) -> u32 {
    json.get("week_number")
        .and_then(|w| w.as_u64())
        .map(|n| n as u32)
        .unwrap_or(1)
}

fn extract_routine_title_from_json(json: &Value) -> String {
    json.get("routine_title")
        .and_then(|t| t.as_str())
        .unwrap_or("Updated Routine")
        .to_string()
}
