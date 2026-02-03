use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub repo_name: String,
    pub branch_name: Option<String>,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}

pub fn get_db_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()?;
    Ok(data_dir.join("chat.db"))
}

fn connect_db() -> Result<Connection, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

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
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages (session_id)",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_session(
    title: String,
    repo_name: String,
    branch_name: Option<String>,
) -> Result<ChatSession, String> {
    let conn = connect_db()?;
    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().timestamp();
    let status = "pending".to_string();

    let session = ChatSession {
        id: id.clone(),
        title: title.clone(),
        repo_name: repo_name.clone(),
        branch_name: branch_name.clone(),
        status: status.clone(),
        created_at,
    };

    conn.execute(
        "INSERT INTO sessions (id, title, repo_name, branch_name, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            session.id,
            session.title,
            session.repo_name,
            session.branch_name,
            session.status,
            session.created_at
        ],
    ).map_err(|e| e.to_string())?;

    Ok(session)
}

pub fn list_sessions() -> Result<Vec<ChatSession>, String> {
    let conn = connect_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, title, repo_name, branch_name, status, created_at FROM sessions ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let sessions_iter = stmt.query_map([], |row| {
        Ok(ChatSession {
            id: row.get(0)?,
            title: row.get(1)?,
            repo_name: row.get(2)?,
            branch_name: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for session in sessions_iter {
        sessions.push(session.map_err(|e| e.to_string())?);
    }

    Ok(sessions)
}

pub fn get_session(id: &str) -> Result<ChatSession, String> {
    let conn = connect_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, title, repo_name, branch_name, status, created_at FROM sessions WHERE id = ?1"
    ).map_err(|e| e.to_string())?;

    stmt.query_row([id], |row| {
        Ok(ChatSession {
            id: row.get(0)?,
            title: row.get(1)?,
            repo_name: row.get(2)?,
            branch_name: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())
}

pub fn add_message(
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<ChatMessage, String> {
    let conn = connect_db()?;
    let created_at = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO messages (session_id, role, content, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![session_id, role, content, created_at],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(ChatMessage {
        id,
        session_id: session_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        created_at,
    })
}

pub fn get_messages(session_id: &str) -> Result<Vec<ChatMessage>, String> {
    let conn = connect_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;

    let messages_iter = stmt.query_map([session_id], |row| {
        Ok(ChatMessage {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for msg in messages_iter {
        messages.push(msg.map_err(|e| e.to_string())?);
    }

    Ok(messages)
}
