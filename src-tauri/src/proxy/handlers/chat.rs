// Chat WebSocket handler for Control Plane with Skills Integration
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::proxy::server::AppState;
use crate::commands::skills::{select_skills, load_skill_content};

// Client -> Server messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    CreateSession {
        title: String,
        repo: String,
        branch: Option<String>,
    },
    ListSessions,
    LoadSession {
        session_id: String,
    },
    UserMessage {
        session_id: String,
        content: String,
    },
}

// Server -> Client messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    SessionList {
        sessions: Vec<TaskSessionResponse>,
    },
    SessionLoaded {
        session: TaskSessionResponse,
        messages: Vec<TaskMessageResponse>,
    },
    MessageAppended {
        session_id: String,
        message: TaskMessageResponse,
    },
    /// Skills selected for this request
    SkillsSelected {
        session_id: String,
        persona: String,
        category: String,
        skills: Vec<SkillSummary>,
        total_bytes: usize,
    },
    /// Status update during task processing
    TaskStatus {
        session_id: String,
        status: String,
        details: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Serialize, Clone)]
struct TaskSessionResponse {
    id: String,
    title: String,
    repo_name: String,
    branch_name: Option<String>,
    status: String,
    created_at: i64,
}

#[derive(Debug, Serialize, Clone)]
struct TaskMessageResponse {
    id: i64,
    role: String,
    content: String,
    created_at: i64,
}

#[derive(Debug, Serialize, Clone)]
struct SkillSummary {
    id: String,
    name: String,
    score: f64,
}

/// WebSocket handler endpoint
pub async fn handle_chat_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}


/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver): (
        futures::stream::SplitSink<WebSocket, Message>,
        futures::stream::SplitStream<WebSocket>
    ) = socket.split();

    info!("Chat WebSocket connected");

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        };

        if let Message::Text(text) = msg {
            debug!("Received WebSocket message: {}", text);

            let response = match serde_json::from_str::<ClientMessage>(&text) {
                Ok(client_msg) => handle_client_message(client_msg, &state, &mut sender).await,
                Err(e) => ServerMessage::Error {
                    message: format!("Invalid message format: {}", e),
                },
            };

            let response_text = match serde_json::to_string(&response) {
                Ok(text) => text,
                Err(e) => {
                    error!("Failed to serialize response: {}", e);
                    continue;
                }
            };

            if let Err(e) = sender.send(Message::Text(response_text)).await {
                error!("Failed to send WebSocket message: {}", e);
                break;
            }
        } else if let Message::Close(_) = msg {
            info!("Chat WebSocket closed by client");
            break;
        }
    }

    info!("Chat WebSocket disconnected");
}

/// Send a status update message to client
async fn send_status_update(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    session_id: String,
    status: String,
    details: String,
) {
    let msg = ServerMessage::TaskStatus {
        session_id,
        status,
        details,
    };

    if let Ok(text) = serde_json::to_string(&msg) {
        let _ = sender.send(Message::Text(text)).await;
    }
}

/// Process client messages and return server responses
async fn handle_client_message(
    msg: ClientMessage,
    _state: &AppState,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> ServerMessage {
    match msg {
        ClientMessage::CreateSession { title, repo, branch } => {
            // TODO: Create session in database
            debug!("Creating session: {} for repo {}", title, repo);

            // Mock response for now
            ServerMessage::SessionList {
                sessions: vec![TaskSessionResponse {
                    id: "mock-session-1".to_string(),
                    title,
                    repo_name: repo,
                    branch_name: branch,
                    status: "pending".to_string(),
                    created_at: chrono::Utc::now().timestamp(),
                }],
            }
        }
        ClientMessage::ListSessions => {
            // TODO: Query database for sessions
            debug!("Listing sessions");

            // Mock response
            ServerMessage::SessionList {
                sessions: vec![
                    TaskSessionResponse {
                        id: "mock-session-1".to_string(),
                        title: "Fix Docker Networking".to_string(),
                        repo_name: "atnplex/homelab".to_string(),
                        branch_name: None,
                        status: "running".to_string(),
                        created_at: chrono::Utc::now().timestamp() - 3600,
                    },
                    TaskSessionResponse {
                        id: "mock-session-2".to_string(),
                        title: "Audit Secrets".to_string(),
                        repo_name: "atnplex/antigravity-manager".to_string(),
                        branch_name: Some("main".to_string()),
                        status: "completed".to_string(),
                        created_at: chrono::Utc::now().timestamp() - 7200,
                    },
                ],
            }
        }
        ClientMessage::LoadSession { session_id } => {
            // TODO: Load session and messages from database
            debug!("Loading session: {}", session_id);

            // Mock response
            ServerMessage::SessionLoaded {
                session: TaskSessionResponse {
                    id: session_id.clone(),
                    title: "Mock Session".to_string(),
                    repo_name: "atnplex/mock-repo".to_string(),
                    branch_name: None,
                    status: "running".to_string(),
                    created_at: chrono::Utc::now().timestamp(),
                },
                messages: vec![
                    TaskMessageResponse {
                        id: 1,
                        role: "user".to_string(),
                        content: "Hello, start working on this task".to_string(),
                        created_at: chrono::Utc::now().timestamp() - 120,
                    },
                    TaskMessageResponse {
                        id: 2,
                        role: "assistant".to_string(),
                        content: "I understand. I'll begin working on this task right away.".to_string(),
                        created_at: chrono::Utc::now().timestamp() - 60,
                    },
                ],
            }
        }
        ClientMessage::UserMessage { session_id, content } => {
            info!("User message in session {}: {}", session_id, content);

            // Phase 4: Skills Integration Pipeline
            // Step 1: Send status update
            send_status_update(
                sender,
                session_id.clone(),
                "selecting_skills".to_string(),
                "Analyzing request and selecting relevant skills...".to_string(),
            ).await;

            // Step 2: Select skills using BM25 router
            let selection_result = match select_skills(content.clone(), Some(8), Some(80000)).await {
                Ok(selection) => selection,
                Err(e) => {
                    error!("Failed to select skills: {}", e);
                    return ServerMessage::Error {
                        message: format!("Skill selection failed: {}", e),
                    };
                }
            };

            info!(
                "Selected persona: {}, {} skills, {} bytes",
                selection_result.persona,
                selection_result.skills.len(),
                selection_result.total_bytes
            );

            // Step 3: Notify client of selected skills
            let skill_summaries: Vec<SkillSummary> = selection_result.skills.iter()
                .map(|s| SkillSummary {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    score: s.score,
                })
                .collect();

            let skills_msg = ServerMessage::SkillsSelected {
                session_id: session_id.clone(),
                persona: selection_result.persona.clone(),
                category: selection_result.category.clone(),
                skills: skill_summaries.clone(),
                total_bytes: selection_result.total_bytes,
            };

            if let Ok(text) = serde_json::to_string(&skills_msg) {
                let _ = sender.send(Message::Text(text)).await;
            }

            // Step 4: Load skill content (optional - for future LLM integration)
            let skill_ids: Vec<String> = selection_result.skills.iter()
                .map(|s| s.id.clone())
                .collect();

            send_status_update(
                sender,
                session_id.clone(),
                "loading_skills".to_string(),
                "Loading selected skill content...".to_string(),
            ).await;

            let skill_contents = match load_skill_content(skill_ids).await {
                Ok(contents) => contents,
                Err(e) => {
                    warn!("Failed to load skill content: {}", e);
                    // Continue anyway with mock response
                    std::collections::HashMap::new()
                }
            };

            debug!("Loaded {} skill files", skill_contents.len());

            // Step 5: TODO: Integrate with LLM
            // - Build prompt with persona system message
            // - Inject skill SKILL.md content
            // - Send to Gemini/Claude
            // - Stream response back to client

            send_status_update(
                sender,
                session_id.clone(),
                "processing".to_string(),
                format!(
                    "Processing as {} with {} skills loaded ({} KB)...",
                    selection_result.persona,
                    skill_summaries.len(),
                    selection_result.total_bytes / 1024
                ),
            ).await;

            // For now, return mock response with skill info
            let response_content = format!(
                "🤖 **Persona:** {}\n📂 **Category:** {}\n📚 **Skills:** {}\n💾 **Content:** {} KB loaded\n\n🔨 **Next Steps:**\n- TODO: Integrate with LLM API\n- TODO: Create git worktree\n- TODO: Execute task pipeline\n- TODO: Open PR\n\n_Your message: {}_",
                selection_result.persona,
                selection_result.category,
                skill_summaries.iter()
                    .map(|s| format!("{} ({:.1})", s.name, s.score))
                    .collect::<Vec<_>>()
                    .join(", "),
                selection_result.total_bytes / 1024,
                content
            );

            ServerMessage::MessageAppended {
                session_id,
                message: TaskMessageResponse {
                    id: chrono::Utc::now().timestamp(),
                    role: "assistant".to_string(),
                    content: response_content,
                    created_at: chrono::Utc::now().timestamp(),
                },
            }
        }
    }
}
