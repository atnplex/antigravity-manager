// OpenAI Stream Collector
// Used for auto-converting streaming responses to JSON for non-streaming requests

use super::models::*;
use bytes::Bytes;
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io;

#[derive(Default)]
struct ToolCallBuilder {
    id: Option<String>,
    r#type: Option<String>,
    name: String,
    arguments: String,
}

/// Collects an OpenAI SSE stream into a complete OpenAIResponse
pub async fn collect_stream_to_json<S, E>(
    mut stream: S,
) -> Result<OpenAIResponse, String>
where
    S: futures::Stream<Item = Result<Bytes, E>> + Unpin,
    E: std::fmt::Display,
{
    let mut response = OpenAIResponse {
        id: "chatcmpl-unknown".to_string(),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        model: "unknown".to_string(),
        choices: Vec::new(),
        usage: None,
    };

    let mut role: Option<String> = None;
    let mut content_parts: Vec<String> = Vec::new();
    let mut reasoning_parts: Vec<String> = Vec::new();
    let mut finish_reason: Option<String> = None;
    let mut tool_call_builders: BTreeMap<u32, ToolCallBuilder> = BTreeMap::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("data: ") {
                let data_str = line.trim_start_matches("data: ").trim();
                if data_str == "[DONE]" {
                    continue;
                }

                if let Ok(json) = serde_json::from_str::<Value>(data_str) {
                    // Update meta fields
                    if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                        response.id = id.to_string();
                    }
                    if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
                        response.model = model.to_string();
                    }
                    if let Some(created) = json.get("created").and_then(|v| v.as_u64()) {
                        response.created = created;
                    }

                    // Collect Usage
                    if let Some(usage) = json.get("usage") {
                        if let Ok(u) = serde_json::from_value::<OpenAIUsage>(usage.clone()) {
                            response.usage = Some(u);
                        }
                    }

                    // Collect Choices Delta
                    if let Some(choices) = json.get("choices").and_then(|v| v.as_array()) {
                        if let Some(choice) = choices.first() {
                            if let Some(delta) = choice.get("delta") {
                                // Role
                                if let Some(r) = delta.get("role").and_then(|v| v.as_str()) {
                                    role = Some(r.to_string());
                                }
                                
                                // Content
                                if let Some(c) = delta.get("content").and_then(|v| v.as_str()) {
                                    content_parts.push(c.to_string());
                                }

                                // Reasoning Content
                                if let Some(rc) = delta.get("reasoning_content").and_then(|v| v.as_str()) {
                                    reasoning_parts.push(rc.to_string());
                                }

                                // Tool Calls
                                if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|v| v.as_array()) {
                                    for tc in tool_calls_arr {
                                        if let Some(index) = tc.get("index").and_then(|v| v.as_u64()).map(|v| v as u32) {
                                            let builder = tool_call_builders.entry(index).or_default();

                                            if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                                                builder.id = Some(id.to_string());
                                            }
                                            if let Some(t) = tc.get("type").and_then(|v| v.as_str()) {
                                                builder.r#type = Some(t.to_string());
                                            }

                                            if let Some(function) = tc.get("function") {
                                                if let Some(name) = function.get("name").and_then(|v| v.as_str()) {
                                                    builder.name.push_str(name);
                                                }
                                                if let Some(args) = function.get("arguments").and_then(|v| v.as_str()) {
                                                    builder.arguments.push_str(args);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(fr) = choice.get("finish_reason").and_then(|v| v.as_str()) {
                                finish_reason = Some(fr.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Construct final message
    let full_content = content_parts.join("");
    let full_reasoning = if reasoning_parts.is_empty() {
        None
    } else {
        Some(reasoning_parts.join(""))
    };

    let tool_calls_vec = if !tool_call_builders.is_empty() {
        let mut calls = Vec::new();
        // BTreeMap iterates in sorted order of keys (indices), which is what we want
        for (_, builder) in tool_call_builders {
            calls.push(ToolCall {
                id: builder.id.unwrap_or_default(),
                r#type: builder.r#type.unwrap_or_else(|| "function".to_string()),
                function: ToolFunction {
                    name: builder.name,
                    arguments: builder.arguments,
                },
            });
        }
        Some(calls)
    } else {
        None
    };

    let message = OpenAIMessage {
        role: role.unwrap_or("assistant".to_string()),
        content: Some(OpenAIContent::String(full_content)),
        reasoning_content: full_reasoning,
        tool_calls: tool_calls_vec,
        tool_call_id: None,
        name: None,
    };

    response.choices.push(Choice {
        index: 0,
        message,
        finish_reason: finish_reason.or(Some("stop".to_string())),
    });

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use serde_json::json;

    #[tokio::test]
    async fn test_collect_stream_with_tool_calls() {
        let chunk1 = json!({
            "id": "chatcmpl-123",
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": ""
                        }
                    }]
                },
                "finish_reason": null
            }]
        });

        let chunk2 = json!({
            "id": "chatcmpl-123",
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "function": {
                            "arguments": "{\"loc"
                        }
                    }]
                },
                "finish_reason": null
            }]
        });

        let chunk3 = json!({
            "id": "chatcmpl-123",
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "function": {
                            "arguments": "ation\": \"NY\"}"
                        }
                    }]
                },
                "finish_reason": null
            }]
        });

        // Simulating second tool call
         let chunk4 = json!({
            "id": "chatcmpl-123",
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 1,
                        "id": "call_456",
                        "type": "function",
                        "function": {
                            "name": "get_time",
                            "arguments": "{}"
                        }
                    }]
                },
                "finish_reason": null
            }]
        });

        let chunk_done = "[DONE]";

        let chunks = vec![
            Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", chunk1))),
            Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", chunk2))),
            Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", chunk3))),
            Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", chunk4))),
            Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", chunk_done))),
        ];

        let stream = stream::iter(chunks);

        let result = collect_stream_to_json(stream).await.expect("Failed to collect");

        let msg = &result.choices[0].message;
        assert!(msg.tool_calls.is_some(), "Tool calls should be present");
        let tools = msg.tool_calls.as_ref().unwrap();
        assert_eq!(tools.len(), 2);

        assert_eq!(tools[0].id, "call_123");
        assert_eq!(tools[0].function.name, "get_weather");
        assert_eq!(tools[0].function.arguments, "{\"location\": \"NY\"}");

        assert_eq!(tools[1].id, "call_456");
        assert_eq!(tools[1].function.name, "get_time");
        assert_eq!(tools[1].function.arguments, "{}");
    }
}
