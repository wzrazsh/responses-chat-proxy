use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ResponsesRequest {
    pub model: String,
    #[serde(default)]
    pub input: Input,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Input {
    String(String),
    Array(Vec<InputMessage>),
}

impl Default for Input {
    fn default() -> Self {
        Input::String(String::new())
    }
}

#[derive(Debug, Deserialize)]
pub struct InputMessage {
    pub role: String,
    #[serde(default)]
    pub content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Content {
    String(String),
    Parts(Vec<ContentPart>),
}

impl Default for Content {
    fn default() -> Self {
        Content::String(String::new())
    }
}

#[derive(Debug, Deserialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResponsesResponse {
    pub id: String,
    #[serde(rename = "object")]
    pub object: String,
    pub created_at: u64,
    pub status: String,
    pub model: String,
    pub output: Vec<OutputMessage>,
    pub output_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize)]
pub struct OutputMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub status: String,
    pub role: String,
    pub content: Vec<OutputContent>,
}

#[derive(Debug, Serialize)]
pub struct OutputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
    pub annotations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct StreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Value>,
}
