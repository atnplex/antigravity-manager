use crate::commands::skills::SkillSelection;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskResult {
    /// Action requires user review (e.g. plan drafted)
    RequiresReview {
        artifact: String,
        next_step: String,
    },
    /// Debugging diagnosis complete
    DebugDiagnosis {
        root_cause: String,
        proposed_fix: String,
        confidence: f64,
    },
    /// Standard completion
    Completed {
        summary: String,
    },
}

pub mod plan;
pub mod debug;
