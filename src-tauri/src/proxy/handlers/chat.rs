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
use crate::modules::chat_db;

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

impl From<chat_db::ChatSession> for TaskSessionResponse {
    fn from(s: chat_db::ChatSession) -> Self {
        Self {
            id: s.id,
            title: s.title,
            repo_name: s.repo_name,
            branch_name: s.branch_name,
            status: s.status,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct TaskMessageResponse {
    id: i64,
    role: String,
    content: String,
    created_at: i64,
}

impl From<chat_db::ChatMessage> for TaskMessageResponse {
    fn from(m: chat_db::ChatMessage) -> Self {
        Self {
            id: m.id,
            role: m.role,
            content: m.content,
            created_at: m.created_at,
        }
    }
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
            let result = tokio::task::spawn_blocking(move || {
                chat_db::create_session(title, repo, branch)
            }).await;

            match result {
                Ok(Ok(_session)) => {
                    // Fetch updated list
                    let list_result = tokio::task::spawn_blocking(|| {
                        chat_db::list_sessions()
                    }).await;

                     match list_result {
                        Ok(Ok(sessions)) => ServerMessage::SessionList {
                            sessions: sessions.into_iter().map(TaskSessionResponse::from).collect(),
                        },
                        Ok(Err(e)) => ServerMessage::Error { message: e },
                         Err(e) => ServerMessage::Error { message: e.to_string() },
                    }
                },
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error { message: e.to_string() },
            }
        }
        ClientMessage::ListSessions => {
            debug!("Listing sessions");
            let result = tokio::task::spawn_blocking(|| {
                chat_db::list_sessions()
            }).await;

            match result {
                Ok(Ok(sessions)) => ServerMessage::SessionList {
                    sessions: sessions.into_iter().map(TaskSessionResponse::from).collect(),
                },
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error { message: e.to_string() },
            }
        }
        ClientMessage::LoadSession { session_id } => {
            debug!("Loading session: {}", session_id);
            let sid = session_id.clone();

            let result = tokio::task::spawn_blocking(move || {
                let session = chat_db::get_session(&sid)?;
                let messages = chat_db::get_messages(&sid)?;
                Ok((session, messages))
            }).await;

            match result {
                Ok(Ok((session, messages))) => ServerMessage::SessionLoaded {
                    session: TaskSessionResponse::from(session),
                    messages: messages.into_iter().map(TaskMessageResponse::from).collect(),
                },
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error { message: e.to_string() },
            }
        }
        ClientMessage::UserMessage { session_id, content } => {
            info!("User message in session {}: {}", session_id, content);
            let sid = session_id.clone();
            let c = content.clone();

            // Save user message
            let result = tokio::task::spawn_blocking(move || {
                chat_db::add_message(&sid, "user", &c)
            }).await;

            match result {
                Ok(Ok(_user_msg)) => {
                    // Mock agent response
                    let sid2 = session_id.clone();
                    let response_content = format!("Echo: {} (Backend WebSocket is working!)", content);

                    let assistant_msg_result = tokio::task::spawn_blocking(move || {
                        chat_db::add_message(&sid2, "assistant", &response_content)
                    }).await;

                    match assistant_msg_result {
                        Ok(Ok(assistant_msg)) => ServerMessage::MessageAppended {
                            session_id,
                            message: TaskMessageResponse::from(assistant_msg),
                        },
                         Ok(Err(e)) => ServerMessage::Error { message: e },
                         Err(e) => ServerMessage::Error { message: e.to_string() },
                    }
                },
                Ok(Err(e)) => ServerMessage::Error { message: e },
                Err(e) => ServerMessage::Error { message: e.to_string() },
            }
        }
    }
}
