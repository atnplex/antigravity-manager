use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSession {
    pub id: String,
    pub title: String,
    pub repo_name: String,
    pub branch_name: Option<String>,
    pub status: String,
    pub created_at: i64,
}

pub fn get_db_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()?;
    Ok(data_dir.join("chat.db"))
}

fn connect_db() -> Result<Connection, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL").map_err(|e| e.to_string())?;
    conn.pragma_update(None, "busy_timeout", 5000).map_err(|e| e.to_string())?;
    conn.pragma_update(None, "synchronous", "NORMAL").map_err(|e| e.to_string())?;

    Ok(conn)
}

pub fn init_db() -> Result<(), String> {
    let conn = connect_db()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            branch_name TEXT,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions (created_at DESC)",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn list_sessions() -> Result<Vec<TaskSession>, String> {
    let conn = connect_db()?;

    let mut stmt = conn.prepare(
        "SELECT id, title, repo_name, branch_name, status, created_at
         FROM sessions
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let session_iter = stmt.query_map([], |row| {
        Ok(TaskSession {
            id: row.get(0)?,
            title: row.get(1)?,
            repo_name: row.get(2)?,
            branch_name: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for session in session_iter {
        sessions.push(session.map_err(|e| e.to_string())?);
    }

    Ok(sessions)
}

/// Helper for testing: Insert a dummy session
#[allow(dead_code)]
pub fn insert_dummy_session(id: &str, title: &str) -> Result<(), String> {
    let conn = connect_db()?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO sessions (id, title, repo_name, branch_name, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, title, "test-repo", "main", "pending", now],
    ).map_err(|e| e.to_string())?;

    Ok(())
}
