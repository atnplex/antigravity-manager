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
use crate::modules::proxy_db;

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
            debug!("Creating session: {} for repo {}", title, repo);

            let sessions = tokio::task::spawn_blocking(move || {
                // 1. Create session
                let _ = proxy_db::create_session(title, repo, branch)?;
                // 2. Return all sessions
                proxy_db::list_sessions()
            })
            .await;

            match sessions {
                Ok(Ok(sessions)) => {
                    let response_sessions = sessions
                        .into_iter()
                        .map(|s| TaskSessionResponse {
                            id: s.id,
                            title: s.title,
                            repo_name: s.repo_name,
                            branch_name: s.branch_name,
                            status: s.status,
                            created_at: s.created_at,
                        })
                        .collect();

                    ServerMessage::SessionList {
                        sessions: response_sessions,
                    }
                }
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error {
                    message: e.to_string(),
                },
            }
        }
        ClientMessage::ListSessions => {
            debug!("Listing sessions");

            let sessions = tokio::task::spawn_blocking(move || proxy_db::list_sessions()).await;

            match sessions {
                Ok(Ok(sessions)) => {
                    let response_sessions = sessions
                        .into_iter()
                        .map(|s| TaskSessionResponse {
                            id: s.id,
                            title: s.title,
                            repo_name: s.repo_name,
                            branch_name: s.branch_name,
                            status: s.status,
                            created_at: s.created_at,
                        })
                        .collect();

                    ServerMessage::SessionList {
                        sessions: response_sessions,
                    }
                }
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error {
                    message: e.to_string(),
                },
            }
        }
        ClientMessage::LoadSession { session_id } => {
            debug!("Loading session: {}", session_id);
            let session_id_clone = session_id.clone();

            let result = tokio::task::spawn_blocking(move || {
                let session = proxy_db::get_session(&session_id_clone)?;
                let messages = proxy_db::get_messages(&session_id_clone)?;
                Ok((session, messages))
            })
            .await;

            match result {
                Ok(Ok((session, messages))) => ServerMessage::SessionLoaded {
                    session: TaskSessionResponse {
                        id: session.id,
                        title: session.title,
                        repo_name: session.repo_name,
                        branch_name: session.branch_name,
                        status: session.status,
                        created_at: session.created_at,
                    },
                    messages: messages
                        .into_iter()
                        .map(|m| TaskMessageResponse {
                            id: m.id,
                            role: m.role,
                            content: m.content,
                            created_at: m.created_at,
                        })
                        .collect(),
                },
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error {
                    message: e.to_string(),
                },
            }
        }
        ClientMessage::UserMessage {
            session_id,
            content,
        } => {
            info!("User message in session {}: {}", session_id, content);
            let session_id_clone = session_id.clone();
            let content_clone = content.clone();

            let result = tokio::task::spawn_blocking(move || {
                // Save user message
                let _ = proxy_db::add_message(&session_id_clone, "user", &content_clone)?;

                // TODO: Trigger agent processing
                // For now, mock echo
                let echo_content =
                    format!("Echo: {} (Backend WebSocket is working!)", content_clone);
                let assistant_msg =
                    proxy_db::add_message(&session_id_clone, "assistant", &echo_content)?;

                Ok(assistant_msg)
            })
            .await;

            match result {
                Ok(Ok(msg)) => ServerMessage::MessageAppended {
                    session_id,
                    message: TaskMessageResponse {
                        id: msg.id,
                        role: msg.role,
                        content: msg.content,
                        created_at: msg.created_at,
                    },
                },
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error {
                    message: e.to_string(),
                },
            }
        }
    }
}
