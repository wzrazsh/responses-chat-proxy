use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::responses::{
    Content, Input, OutputContent, OutputMessage, ResponsesRequest, ResponsesResponse, Usage,
};

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: Option<String>,
    pub model: Option<String>,
    pub created: Option<u64>,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponseMessage {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChatStreamChunk {
    pub choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatStreamChoice {
    pub delta: ChatStreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatStreamDelta {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
}

pub fn convert_request(req: ResponsesRequest) -> ChatRequest {
    let mut messages = Vec::new();

    if let Some(instructions) = req.instructions {
        if !instructions.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: instructions,
            });
        }
    }

    match req.input {
        Input::String(text) => {
            if !text.is_empty() {
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: text,
                });
            }
        }
        Input::Array(items) => {
            for msg in items {
                messages.push(ChatMessage {
                    role: normalize_role(&msg.role).to_string(),
                    content: flatten_content(msg.content),
                });
            }
        }
    }

    ChatRequest {
        model: req.model,
        messages,
        temperature: req.temperature,
        max_tokens: req.max_output_tokens,
        top_p: req.top_p,
        stream: Some(req.stream),
        tools: convert_tools(req.tools),
    }
}

fn convert_tools(tools: Option<Value>) -> Option<Value> {
    let items = match tools {
        Some(Value::Array(items)) => items,
        other => return other,
    };

    let converted = items
        .into_iter()
        .filter_map(|item| match item {
            Value::Object(object) if object.contains_key("function") => Some(Value::Object(object)),
            Value::Object(mut object) => {
                if object.get("type").and_then(Value::as_str) != Some("function") {
                    return None;
                }

                let name = object.remove("name");
                let description = object.remove("description");
                let parameters = object.remove("parameters");
                let strict = object.remove("strict");

                if name.is_none() {
                    return None;
                }

                let mut function = serde_json::Map::new();
                function.insert("name".to_string(), name.unwrap());
                if let Some(description) = description {
                    function.insert("description".to_string(), description);
                }
                if let Some(parameters) = parameters {
                    function.insert("parameters".to_string(), parameters);
                }
                if let Some(strict) = strict {
                    function.insert("strict".to_string(), strict);
                }

                Some(serde_json::json!({
                    "type": object.remove("type").unwrap_or_else(|| Value::String("function".to_string())),
                    "function": Value::Object(function)
                }))
            }
            _ => None,
        })
        .collect();

    Some(Value::Array(converted))
}

fn normalize_role(role: &str) -> &str {
    match role {
        "developer" => "system",
        other => other,
    }
}

fn flatten_content(content: Content) -> String {
    match content {
        Content::String(s) => s,
        Content::Parts(parts) => {
            let mut result = String::new();
            for part in parts {
                match part.part_type.as_str() {
                    "input_text" => {
                        if let Some(text) = part.text {
                            result.push_str(&text);
                        }
                    }
                    other => {
                        result.push_str(&format!("[unsupported content type: {other}]"));
                    }
                }
            }
            result
        }
    }
}

pub fn convert_response(
    chat_resp: ChatResponse,
    model: &str,
    resp_id: &str,
) -> ResponsesResponse {
    let msg_id = format!("msg_{}", &resp_id[5..]);

    let output_text = chat_resp
        .choices
        .first()
        .and_then(|c| c.message.content.as_deref())
        .unwrap_or("")
        .to_string();

    let status = if output_text.is_empty() {
        "incomplete"
    } else {
        "completed"
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    ResponsesResponse {
        id: resp_id.to_string(),
        object: "response".to_string(),
        created_at: chat_resp.created.unwrap_or(now),
        status: status.to_string(),
        model: chat_resp.model.unwrap_or_else(|| model.to_string()),
        output: vec![OutputMessage {
            id: msg_id,
            msg_type: "message".to_string(),
            status: status.to_string(),
            role: "assistant".to_string(),
            content: vec![OutputContent {
                content_type: "output_text".to_string(),
                text: output_text.clone(),
                annotations: Vec::new(),
            }],
        }],
        output_text,
        usage: chat_resp.usage.map(|u| Usage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::responses::{ContentPart, InputMessage};

    #[test]
    fn test_string_input_to_chat_messages() {
        let req = ResponsesRequest {
            model: "deepseek-chat".to_string(),
            input: Input::String("hello".to_string()),
            instructions: Some("be helpful".to_string()),
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            stream: false,
            tools: None,
            tool_choice: None,
        };
        let chat = convert_request(req);
        assert_eq!(chat.messages.len(), 2);
        assert_eq!(chat.messages[0].role, "system");
        assert_eq!(chat.messages[0].content, "be helpful");
        assert_eq!(chat.messages[1].role, "user");
        assert_eq!(chat.messages[1].content, "hello");
    }

    #[test]
    fn test_array_input_to_chat_messages() {
        let req = ResponsesRequest {
            model: "deepseek-chat".to_string(),
            input: Input::Array(vec![
                InputMessage {
                    role: "user".to_string(),
                    content: Content::Parts(vec![
                        ContentPart {
                            part_type: "input_text".to_string(),
                            text: Some("hello ".to_string()),
                        },
                        ContentPart {
                            part_type: "input_text".to_string(),
                            text: Some("world".to_string()),
                        },
                    ]),
                },
                InputMessage {
                    role: "assistant".to_string(),
                    content: Content::String("ok".to_string()),
                },
            ]),
            instructions: None,
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            stream: false,
            tools: None,
            tool_choice: None,
        };
        let chat = convert_request(req);
        assert_eq!(chat.messages.len(), 2);
        assert_eq!(chat.messages[0].content, "hello world");
        assert_eq!(chat.messages[1].content, "ok");
    }

    #[test]
    fn test_instructions_to_system_message() {
        let req = ResponsesRequest {
            model: "deepseek-chat".to_string(),
            input: Input::String("hi".to_string()),
            instructions: Some("system prompt".to_string()),
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            stream: false,
            tools: None,
            tool_choice: None,
        };
        let chat = convert_request(req);
        assert_eq!(chat.messages[0].role, "system");
        assert_eq!(chat.messages[0].content, "system prompt");
    }

    #[test]
    fn test_chat_response_to_responses() {
        let chat = ChatResponse {
            id: Some("chatcmpl-123".to_string()),
            model: Some("deepseek-v4-flash".to_string()),
            created: Some(1778287813),
            choices: vec![ChatChoice {
                message: ChatResponseMessage {
                    role: Some("assistant".to_string()),
                    content: Some("hello world".to_string()),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(ChatUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        };
        let resp = convert_response(chat, "deepseek-chat", "resp_abc123");
        assert_eq!(resp.object, "response");
        assert_eq!(resp.output_text, "hello world");
        assert_eq!(resp.output[0].role, "assistant");
        assert_eq!(resp.output[0].content[0].text, "hello world");
        assert_eq!(resp.usage.as_ref().unwrap().input_tokens, 10);
        assert_eq!(resp.usage.as_ref().unwrap().output_tokens, 5);
    }

    #[test]
    fn test_unsupported_content_part() {
        let content = Content::Parts(vec![
            ContentPart {
                part_type: "input_text".to_string(),
                text: Some("hello ".to_string()),
            },
            ContentPart {
                part_type: "input_image".to_string(),
                text: None,
            },
        ]);
        let flat = flatten_content(content);
        assert!(flat.contains("hello "));
        assert!(flat.contains("[unsupported content type: input_image]"));
    }

    #[test]
    fn test_developer_role_normalized_to_system() {
        let req = ResponsesRequest {
            model: "codex-MiniMax-M2.7".to_string(),
            input: Input::Array(vec![InputMessage {
                role: "developer".to_string(),
                content: Content::String("developer instructions".to_string()),
            }]),
            instructions: None,
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            stream: false,
            tools: None,
            tool_choice: None,
        };
        let chat = convert_request(req);
        assert_eq!(chat.messages[0].role, "system");
    }

    #[test]
    fn test_responses_tool_shape_converts_to_chat_tool_shape() {
        let req = ResponsesRequest {
            model: "deepseek-chat".to_string(),
            input: Input::String("hi".to_string()),
            instructions: None,
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            stream: false,
            tools: Some(serde_json::json!([{
                "type": "function",
                "name": "shell_command",
                "description": "run a command",
                "parameters": {
                    "type": "object",
                    "properties": {}
                },
                "strict": false
            }])),
            tool_choice: None,
        };
        let chat = convert_request(req);
        let tools = chat.tools.unwrap();
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "shell_command");
        assert_eq!(tools[0]["function"]["description"], "run a command");
        assert_eq!(tools[0]["function"]["strict"], false);
    }

    #[test]
    fn test_non_function_tools_are_dropped() {
        let req = ResponsesRequest {
            model: "deepseek-v4-flash".to_string(),
            input: Input::String("hi".to_string()),
            instructions: None,
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            stream: false,
            tools: Some(serde_json::json!([
                { "type": "web_search" },
                {
                    "type": "function",
                    "name": "noop_tool",
                    "parameters": { "type": "object", "properties": {} }
                }
            ])),
            tool_choice: None,
        };
        let chat = convert_request(req);
        let tools = chat.tools.unwrap();
        assert_eq!(tools.as_array().unwrap().len(), 1);
        assert_eq!(tools[0]["function"]["name"], "noop_tool");
    }
}
