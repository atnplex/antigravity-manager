// Skills router integration commands
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use tauri::State;
use tracing::{debug, error, info};

/// Skill selection result from BM25 router
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillSelection {
    pub persona: String,
    pub category: String,
    pub skills: Vec<SkillScore>,
    pub total_bytes: usize,
    pub limits: SelectionLimits,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillScore {
    pub id: String,
    pub name: String,
    pub score: f64,
    pub matched_terms: Vec<String>,
    pub size_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectionLimits {
    pub max_skills: usize,
    pub max_bytes: usize,
    pub actual_skills: usize,
    pub actual_bytes: usize,
}

/// Skill metadata from index
#[derive(Debug, Deserialize)]
struct SkillMetadata {
    id: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct SkillsIndex {
    skills: Vec<SkillMetadata>,
}

/// Select top-K skills using BM25 router
#[tauri::command]
pub async fn select_skills(
    query: String,
    k: Option<usize>,
    max_bytes: Option<usize>,
) -> Result<SkillSelection, String> {
    let k = k.unwrap_or(8);
    let max_bytes = max_bytes.unwrap_or(80000);

    debug!("Selecting skills for query: {}", query);
    debug!("  K: {}, Max bytes: {}", k, max_bytes);

    // Get project root (where tools/ lives)
    let project_root = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    let router_script = project_root
        .join("tools")
        .join("skills-indexer")
        .join("src")
        .join("02-router.ts");

    if !router_script.exists() {
        return Err(format!(
            "Skills router not found at: {}",
            router_script.display()
        ));
    }

    // Run TypeScript router via npx tsx
    let output = Command::new("npx")
        .args(&[
            "tsx",
            router_script.to_str().unwrap(),
            &query,
            "--k",
            &k.to_string(),
            "--max-bytes",
            &max_bytes.to_string(),
            "--json",
        ])
        .current_dir(&project_root)
        .output()
        .map_err(|e| format!("Failed to execute router: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Router failed: {}", stderr);
        return Err(format!("Router execution failed: {}", stderr));
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: SkillSelection = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse router output: {}", e))?;

    info!(
        "Selected persona: {}, {} skills, {} bytes",
        result.persona, result.skills.len(), result.total_bytes
    );

    Ok(result)
}

/// Load skill content from disk
#[tauri::command]
pub async fn load_skill_content(skill_ids: Vec<String>) -> Result<HashMap<String, String>, String> {
    debug!("Loading content for {} skills", skill_ids.len());

    // Read skills index
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "HOME/USERPROFILE not set".to_string())?;

    let index_path = PathBuf::from(home)
        .join(".agent")
        .join("skills-index.json");

    if !index_path.exists() {
        return Err(format!(
            "Skills index not found at: {}. Run: npm run index",
            index_path.display()
        ));
    }

    let index_content = std::fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read index: {}", e))?;

    let index: SkillsIndex = serde_json::from_str(&index_content)
        .map_err(|e| format!("Failed to parse index: {}", e))?;

    // Load each skill
    let mut contents = HashMap::new();
    let mut total_bytes = 0;

    for skill_id in skill_ids {
        // Find skill in index
        let skill = index
            .skills
            .iter()
            .find(|s| s.id == skill_id)
            .ok_or_else(|| format!("Skill not found: {}", skill_id))?;

        // Read SKILL.md
        let content = std::fs::read_to_string(&skill.path)
            .map_err(|e| format!("Failed to read skill {}: {}", skill_id, e))?;

        total_bytes += content.len();
        contents.insert(skill_id.clone(), content);

        debug!("  Loaded {} ({} bytes)", skill_id, content.len());
    }

    info!("Loaded {} skills, {} bytes total", contents.len(), total_bytes);

    Ok(contents)
}

/// Get skill router statistics
#[tauri::command]
pub async fn get_skill_stats() -> Result<serde_json::Value, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "HOME/USERPROFILE not set".to_string())?;

    let stats_path = PathBuf::from(home)
        .join(".agent")
        .join("skills-stats.json");

    if !stats_path.exists() {
        return Err("Skills stats not found. Run: npm run index".to_string());
    }

    let stats_content = std::fs::read_to_string(&stats_path)
        .map_err(|e| format!("Failed to read stats: {}", e))?;

    let stats: serde_json::Value = serde_json::from_str(&stats_content)
        .map_err(|e| format!("Failed to parse stats: {}", e))?;

    Ok(stats)
}
