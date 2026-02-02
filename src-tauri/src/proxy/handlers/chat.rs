// Chat WebSocket handler for Control Plane
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
use tracing::{debug, error, info};

use crate::proxy::server::AppState;

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
                Ok(client_msg) => handle_client_message(client_msg, &state).await,
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

/// Process client messages and return server responses
async fn handle_client_message(msg: ClientMessage, _state: &AppState) -> ServerMessage {
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
            // TODO: Save message to database and trigger agent processing
            info!("User message in session {}: {}", session_id, content);

            // Mock echo response
            ServerMessage::MessageAppended {
                session_id: session_id.clone(),
                message: TaskMessageResponse {
                    id: chrono::Utc::now().timestamp(),
                    role: "assistant".to_string(),
                    content: format!("Echo: {} (Backend WebSocket is working!)", content),
                    created_at: chrono::Utc::now().timestamp(),
                },
            }
        }
    }
}
