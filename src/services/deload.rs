use regex::Regex;

#[derive(Debug, Clone)]
pub struct DeloadCalculator {
    pub deload_intensity_percentage: f64, // Default 60% for Week 8 deload
}

impl Default for DeloadCalculator {
    fn default() -> Self {
        Self {
            deload_intensity_percentage: 0.60, // 60% intensity for deload
        }
    }
}

impl DeloadCalculator {
    /// Generate deload instruction for AI prompt
    pub fn generate_deload_instruction(&self, has_reference: bool) -> String {
        if has_reference {
            format!(
                "🔄 CYCLE TRANSITION: You are transitioning from Week 8 (deload) to Week 1 of a NEW 8-week block. \
                This should be a DELOAD week with {}% intensity and reduced volume based on the Week 1 reference workout provided. \
                Use the Week 1 weights as baseline and apply the deload percentage. Focus on form, recovery, and conservative loading.",
                (self.deload_intensity_percentage * 100.0) as u32
            )
        } else {
            format!(
                "🔄 CYCLE TRANSITION: You are transitioning from Week 8 (deload) to Week 1 of a NEW 8-week block. \
                This should be a DELOAD week with {}% intensity reduction from current weights and reduced volume. \
                Focus on form, recovery, and conservative loading to prepare for the new training cycle.",
                (self.deload_intensity_percentage * 100.0) as u32
            )
        }
    }

    /// Extract day information from workout title
    pub fn extract_day_from_title(&self, title: &str) -> Option<u32> {
        let day_regex = Regex::new(r"(?i)day\s*(\d+)").unwrap();
        day_regex
            .captures(title)
            .and_then(|captures| captures.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_deload_instruction_with_reference() {
        let calculator = DeloadCalculator::default();
        let instruction = calculator.generate_deload_instruction(true);

        assert!(instruction.contains("🔄 CYCLE TRANSITION"));
        assert!(instruction.contains("60% intensity"));
        assert!(instruction.contains("Week 1 reference workout provided"));
    }

    #[test]
    fn test_generate_deload_instruction_without_reference() {
        let calculator = DeloadCalculator::default();
        let instruction = calculator.generate_deload_instruction(false);

        assert!(instruction.contains("🔄 CYCLE TRANSITION"));
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
