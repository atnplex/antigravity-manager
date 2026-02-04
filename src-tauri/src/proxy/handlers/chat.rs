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
use crate::commands::workflows::{
    parse_workflow_command, validate_widget_workflow, filter_skills_for_widget, WorkflowCommand
};
use crate::workflows::{plan, debug as debug_flow, TaskResult};

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

            // Phase 5.1: Workflow Parsing & Widget Security

            // 1. Parse workflow command (server-side only)
            let workflow = parse_workflow_command(&content);
            if let Some(cmd) = &workflow {
                info!("Detected workflow command: {:?}", cmd);
            }

            // 2. Security Check: Widget Mode Constraints
            if let Err(msg) = validate_widget_workflow(&session_id, &workflow) {
                return ServerMessage::Error { message: msg };
            }

            // 3. Send status update
            send_status_update(
                sender,
                session_id.clone(),
                "selecting_skills".to_string(),
                "Analyzing request and selecting relevant skills...".to_string(),
            ).await;

            // 4. Select skills using BM25 router
            let mut selection_result = match select_skills(content.clone(), Some(8), Some(80000)).await {
                Ok(selection) => selection,
                Err(e) => {
                    error!("Failed to select skills: {}", e);
                    return ServerMessage::Error {
                        message: format!("Skill selection failed: {}", e),
                    };
                }
            };

            // 5. Apply Workflow Overrides & Widget Limits
            if let Some(cmd) = &workflow {
                // Force persona based on workflow
                selection_result.persona = cmd.get_persona().to_string();
            }

            // Apply Widget allowed skills + count limit
            let skill_ids_ref = &mut selection_result.skills.iter_mut().map(|s| s.id.clone()).collect::<Vec<_>>();
            // Note: filter_skills_for_widget modifies a Vec<String>, we have Vec<Skill>.
            // We need to filter the skills vector directly.

            // Security: Enforce widget allowlist and max count
            use crate::commands::workflows::is_widget_mode;
            if is_widget_mode(&session_id) {
                let allowed = crate::commands::workflows::get_widget_allowed_skills();
                selection_result.skills.retain(|s| allowed.contains(&s.id));
                selection_result.skills.truncate(crate::commands::workflows::WIDGET_MAX_SKILLS);
            }

            info!(
                "Selected persona: {}, {} skills, {} bytes",
                selection_result.persona,
                selection_result.skills.len(),
                selection_result.total_bytes
            );

            // 6. Notify client of selected skills (with forced persona)
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

            // 7. Load skill content
            let skill_ids: Vec<String> = selection_result.skills.iter()
                .map(|s| s.id.clone())
                .collect();

            send_status_update(
                sender,
                session_id.clone(),
                "loading_skills".to_string(),
                "Loading selected skill content...".to_string(),
            ).await;

            let _skill_contents = match load_skill_content(skill_ids).await {
                Ok(contents) => contents,
                Err(e) => {
                    warn!("Failed to load skill content: {}", e);
                    std::collections::HashMap::new()
                }
            };

            // 8. Execute Workflow Logic
            send_status_update(
                sender,
                session_id.clone(),
                "processing".to_string(),
                format!(
                    "Executing {} workflow as {}...",
                    workflow.as_ref().map(|w| w.get_description()).unwrap_or("standard"),
                    selection_result.persona
                ),
            ).await;

            let exec_result = match workflow {
                Some(WorkflowCommand::Plan) => plan::execute(content.clone(), &selection_result).await,
                Some(WorkflowCommand::Debug) => debug_flow::execute(content.clone(), &selection_result).await,
                _ => {
                    // Standard flow (echo/mock for now)
                    Ok(TaskResult::Completed {
                        summary: format!(
                            "Standard response (Persona: {}). Skills: {}",
                            selection_result.persona,
                            skill_summaries.len()
                        )
                    })
                }
            };

            match exec_result {
                Ok(task_result) => {
                    let response_content = match task_result {
                        TaskResult::RequiresReview { artifact, next_step } => {
                            format!(
                                "📝 **Plan Created:** `{}`\n\n👉 **Next Step:** {}\n\n_Review the artifact to proceed._",
                                artifact, next_step
                            )
                        },
                        TaskResult::DebugDiagnosis { root_cause, proposed_fix, confidence } => {
                            format!(
                                "🔍 **Diagnosis:** {}\n\n🛠️ **Proposed Fix:** {}\n\n✅ **Confidence:** {:.0}%",
                                root_cause, proposed_fix, confidence * 100.0
                            )
                        },
                        TaskResult::Completed { summary } => {
                            format!("✅ **Done:** {}\n\n_Your message: {}_", summary, content)
                        }
                    };

                    ServerMessage::MessageAppended {
                        session_id,
                        message: TaskMessageResponse {
                            id: chrono::Utc::now().timestamp(),
                            role: "assistant".to_string(),
                            content: response_content,
                            created_at: chrono::Utc::now().timestamp(),
                        },
                    }
                },
                Err(e) => ServerMessage::Error {
                    message: format!("Workflow execution failed: {}", e)
                }
            }
        }
    }
}
