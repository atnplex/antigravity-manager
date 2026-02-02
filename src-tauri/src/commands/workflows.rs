// Workflow command parsing and routing
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;

/// Workflow command types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowCommand {
    /// /plan - Create implementation plan
    Plan,
    /// /debug - Systematic debugging and troubleshooting
    Debug,
    /// /create - Generate new feature (future)
    Create,
    /// /test - Write and run tests (future)
    Test,
    /// /deploy - Deployment procedures (future)
    Deploy,
}

impl WorkflowCommand {
    /// Get the forced persona for this workflow
    pub fn get_persona(&self) -> &'static str {
        match self {
            WorkflowCommand::Plan => "architect",
            WorkflowCommand::Debug => "troubleshooter",
            WorkflowCommand::Create => "builder",
            WorkflowCommand::Test => "qa-engineer",
            WorkflowCommand::Deploy => "devops-engineer",
        }
    }

    /// Get the workflow description
    pub fn get_description(&self) -> &'static str {
        match self {
            WorkflowCommand::Plan => "Creating structured implementation plan",
            WorkflowCommand::Debug => "Performing systematic troubleshooting",
            WorkflowCommand::Create => "Generating new feature",
            WorkflowCommand::Test => "Writing and executing tests",
            WorkflowCommand::Deploy => "Executing deployment procedures",
        }
    }
}

/// Parse workflow command from user message
/// SECURITY: Server-side only - never trust client input
pub fn parse_workflow_command(message: &str) -> Option<WorkflowCommand> {
    let trimmed = message.trim_start().to_lowercase();

    if trimmed.starts_with("/plan") {
        Some(WorkflowCommand::Plan)
    } else if trimmed.starts_with("/debug") {
        Some(WorkflowCommand::Debug)
    } else if trimmed.starts_with("/create") {
        Some(WorkflowCommand::Create)
    } else if trimmed.starts_with("/test") {
        Some(WorkflowCommand::Test)
    } else if trimmed.starts_with("/deploy") {
        Some(WorkflowCommand::Deploy)
    } else {
        None
    }
}

/// Widget mode session tracking
/// SECURITY: Server-side state - client cannot bypass
static WIDGET_SESSIONS: Lazy<Arc<RwLock<HashSet<String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashSet::new())));

/// Check if session is in widget mode
pub fn is_widget_mode(session_id: &str) -> bool {
    WIDGET_SESSIONS
        .read()
        .unwrap()
        .contains(session_id)
}

/// Register a session as widget mode
pub fn register_widget_session(session_id: String) {
    WIDGET_SESSIONS
        .write()
        .unwrap()
        .insert(session_id);
}

/// Unregister widget session
pub fn unregister_widget_session(session_id: &str) {
    WIDGET_SESSIONS
        .write()
        .unwrap()
        .remove(session_id);
}

/// Get allowed workflows for widget mode
pub fn get_widget_allowed_workflows() -> Vec<WorkflowCommand> {
    vec![WorkflowCommand::Debug] // Only debugging allowed in widget mode
}

/// Get allowed skill IDs for widget mode (security allowlist)
pub fn get_widget_allowed_skills() -> Vec<String> {
    vec![
        "awesome-troubleshooting".to_string(),
        "awesome-error-analysis".to_string(),
        "awesome-debugging-mindset".to_string(),
        "awesome-root-cause-analysis".to_string(),
        // Curated list only - no unrestricted access
    ]
}

/// Widget mode constraints
pub const WIDGET_MAX_SKILLS: usize = 3;
pub const WIDGET_MAX_BYTES: usize = 30_000; // 30KB max

/// Validate workflow is allowed for widget mode
/// Returns Err if blocked
pub fn validate_widget_workflow(
    session_id: &str,
    workflow: &Option<WorkflowCommand>,
) -> Result<(), String> {
    if !is_widget_mode(session_id) {
        return Ok(()); // Not widget mode, no restrictions
    }

    match workflow {
        Some(cmd) => {
            let allowed = get_widget_allowed_workflows();
            if !allowed.contains(cmd) {
                return Err(format!(
                    "Widget mode: only {:?} workflows allowed",
                    allowed
                ));
            }
            Ok(())
        }
        None => Ok(()), // Standard message OK in widget mode
    }
}

/// Filter skills to widget allowlist
/// Modifies skill_ids in place
pub fn filter_skills_for_widget(
    session_id: &str,
    skill_ids: &mut Vec<String>,
) {
    if !is_widget_mode(session_id) {
        return; // Not widget mode
    }

    let allowed = get_widget_allowed_skills();
    skill_ids.retain(|id| allowed.contains(id));

    // Enforce max skill count
    skill_ids.truncate(WIDGET_MAX_SKILLS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workflow_commands() {
        assert_eq!(parse_workflow_command("/plan"), Some(WorkflowCommand::Plan));
        assert_eq!(parse_workflow_command("/debug issue"), Some(WorkflowCommand::Debug));
        assert_eq!(parse_workflow_command("  /PLAN  "), Some(WorkflowCommand::Plan));
        assert_eq!(parse_workflow_command("regular message"), None);
    }

    #[test]
    fn test_widget_mode_tracking() {
        let session = "test-session-123";
        assert!(!is_widget_mode(session));

        register_widget_session(session.to_string());
        assert!(is_widget_mode(session));

        unregister_widget_session(session);
        assert!(!is_widget_mode(session));
    }

    #[test]
    fn test_widget_workflow_validation() {
        let session = "widget-test";

        // Normal mode - all allowed
        assert!(validate_widget_workflow(session, &Some(WorkflowCommand::Plan)).is_ok());

        // Widget mode - only debug allowed
        register_widget_session(session.to_string());
        assert!(validate_widget_workflow(session, &Some(WorkflowCommand::Debug)).is_ok());
        assert!(validate_widget_workflow(session, &Some(WorkflowCommand::Plan)).is_err());

        unregister_widget_session(session);
    }
}
