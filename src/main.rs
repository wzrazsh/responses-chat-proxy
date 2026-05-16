#![recursion_limit = "256"]

mod chat;
mod config;
mod error;
mod providers;
mod responses;
mod stream;
mod tray;
mod test_page;

use std::sync::Arc;
use std::time::Duration;

use axum::routing::{get, post};
use axum::{Json, Router};
use axum::body::Body;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::chat::{convert_request, convert_response, ChatResponse};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::providers::resolve_provider;
use crate::responses::ResponsesRequest;
use crate::stream::build_stream;
use crate::tray::setup_tray;
use crate::test_page::{test_page_handler, api_status_handler};

fn strip_opencode_prefix(model: &str) -> String {
    if model.starts_with("opencode-") {
        model.strip_prefix("opencode-").unwrap_or(model).to_string()
    } else {
        model.to_string()
    }
}

#[derive(Clone)]
struct AppState {
    config: Arc<AppConfig>,
    client: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Arc::new(AppConfig::from_env());
    let bind_addr = config.bind_addr.clone();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.upstream_timeout_secs))
        .build()?;

    let state = AppState {
        config,
        client,
    };

    let app = Router::new()
        .route("/", get(test_page_handler))
        .route("/health", get(health_handler))
        .route("/v1/models", get(models_handler))
        .route("/v1/responses", post(responses_handler))
        .route("/responses", post(responses_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/api/status", get(api_status_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    info!("starting responses-chat-proxy on {bind_addr}");

    if let Err(e) = setup_tray(&bind_addr) {
        eprintln!("Failed to setup tray: {}. Continuing without tray...", e);
    }

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> Json<Value> {
    Json(serde_json::json!({
        "ok": true,
        "service": "responses-chat-proxy"
    }))
}

async fn models_handler() -> Json<Value> {
    let reasoning_levels = serde_json::json!([
        {
            "slug": "none",
            "id": "none",
            "effort": "none",
            "display_name": "None",
            "description": "No extra reasoning"
        },
        {
            "slug": "minimal",
            "id": "minimal",
            "effort": "minimal",
            "display_name": "Minimal",
            "description": "Minimal reasoning"
        },
        {
            "slug": "low",
            "id": "low",
            "effort": "low",
            "display_name": "Low",
            "description": "Low reasoning"
        },
        {
            "slug": "medium",
            "id": "medium",
            "effort": "medium",
            "display_name": "Medium",
            "description": "Medium reasoning"
        },
        {
            "slug": "high",
            "id": "high",
            "effort": "high",
            "display_name": "High",
            "description": "High reasoning"
        }
    ]);

    let models = serde_json::json!([
        {
            "id": "deepseek-v4-flash",
            "slug": "deepseek-v4-flash",
            "display_name": "DeepSeek V4 Flash",
            "description": "DeepSeek model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 64000,
            "max_context_window": 64000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 0,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "deepseek"
        },
        {
            "id": "deepseek-chat",
            "slug": "deepseek-chat",
            "display_name": "DeepSeek Chat",
            "description": "DeepSeek chat model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 64000,
            "max_context_window": 64000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 1,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "deepseek"
        },
        {
            "id": "deepseek-reasoner",
            "slug": "deepseek-reasoner",
            "display_name": "DeepSeek Reasoner",
            "description": "DeepSeek reasoning model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 64000,
            "max_context_window": 64000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 2,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "deepseek"
        },
        {
            "id": "codex-MiniMax-M2.7",
            "slug": "codex-MiniMax-M2.7",
            "display_name": "MiniMax M2.7",
            "description": "MiniMax model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 64000,
            "max_context_window": 64000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 3,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "minimax"
        },
        {
            "id": "opencode-gpt-4o",
            "slug": "opencode-gpt-4o",
            "display_name": "OpenCode GPT-4o",
            "description": "OpenCode GPT-4o model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text", "image"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 128000,
            "max_context_window": 128000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 4,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "opencode"
        },
        {
            "id": "opencode-claude-sonnet-4",
            "slug": "opencode-claude-sonnet-4",
            "display_name": "OpenCode Claude Sonnet 4",
            "description": "OpenCode Claude Sonnet 4 model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text", "image"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 200000,
            "max_context_window": 200000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 5,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "opencode"
        },
        {
            "id": "opencode-deepseek-v4-flash",
            "slug": "opencode-deepseek-v4-flash",
            "display_name": "OpenCode DeepSeek V4 Flash",
            "description": "OpenCode DeepSeek V4 Flash model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 64000,
            "max_context_window": 64000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 6,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "opencode"
        },
        {
            "id": "opencode-qwen3.6-plus",
            "slug": "opencode-qwen3.6-plus",
            "display_name": "OpenCode Qwen3.6 Plus",
            "description": "OpenCode Qwen3.6 Plus multimodal model served through the local Responses proxy.",
            "prefer_websockets": false,
            "support_verbosity": true,
            "default_verbosity": "low",
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text_and_image",
            "input_modalities": ["text", "image"],
            "supports_image_detail_original": false,
            "truncation_policy": {
                "mode": "tokens",
                "limit": 10000
            },
            "supports_parallel_tool_calls": true,
            "context_window": 128000,
            "max_context_window": 128000,
            "auto_compact_token_limit": null,
            "reasoning_summary_format": "experimental",
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": reasoning_levels,
            "shell_type": "default",
            "base_instructions": "",
            "model_messages": null,
            "supports_reasoning_summaries": false,
            "default_reasoning_summary": "none",
            "visibility": "list",
            "minimal_client_version": "0.124.0",
            "supported_in_api": true,
            "availability_nux": null,
            "upgrade": null,
            "priority": 7,
            "supports_personality": false,
            "additional_speed_tiers": [],
            "is_default": false,
            "show_in_picker": true,
            "experimental_supported_tools": [],
            "supports_search_tool": false,
            "object": "model",
            "created": 0,
            "owned_by": "opencode"
        }
    ]);

    Json(serde_json::json!({
        "object": "list",
        "data": models,
        "models": models
    }))
}

async fn responses_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ResponsesRequest>,
) -> Result<axum::response::Response, AppError> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let resp_id = format!("resp_{request_id}");

    let model = req.model.clone();

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("request {resp_id}: model={model}, stream={}, input_type={}",
        req.stream,
        match &req.input {
            crate::responses::Input::String(_) => "string",
            crate::responses::Input::Array(_) => "array",
        }
    );

    if !state.config.log_prompts {
        info!("request {resp_id}: auth={}", redact_auth(auth_header));
    }

    let (api_key, provider_cfg) = resolve_provider(&model, &state.config)?;

    let upstream_api_key = if provider_cfg.name == "OpenCode" {
        // For OpenCode, always use the API key from auth.json or env
        api_key
    } else if auth_header.starts_with("Bearer ") {
        let bearer = auth_header.strip_prefix("Bearer ").unwrap_or("");
        if !bearer.is_empty() {
            bearer.to_string()
        } else {
            api_key
        }
    } else {
        api_key
    };

    let stream_requested = req.stream;
    let chat_req = convert_request(req);

    if state.config.log_prompts {
        info!("request {resp_id}: chat request body = {}", serde_json::to_string(&chat_req).unwrap_or_default());
    }

    let upstream_url = format!("{}{}", provider_cfg.base_url.trim_end_matches('/'), provider_cfg.chat_path);
    info!("request {resp_id}: upstream={upstream_url}, provider={}", provider_cfg.name);

    let mut upstream_req = state
        .client
        .post(&upstream_url)
        .header("Authorization", format!("Bearer {upstream_api_key}"))
        .header("Content-Type", "application/json");

    if provider_cfg.name == "OpenCode" {
        upstream_req = upstream_req.header("X-Provider", "opencode");
    }

    let upstream_resp = upstream_req
        .json(&chat_req)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::upstream_timeout()
            } else {
                AppError::upstream_error(format!("upstream request failed: {e}"))
            }
        })?;

    let status = upstream_resp.status();

    if !status.is_success() {
        let body = upstream_resp.text().await.unwrap_or_default();
        let capped = if body.len() > 2000 {
            format!("{}... (truncated)", &body[..2000])
        } else {
            body
        };
        info!("request {resp_id}: upstream error status={status}, body={capped}");
        return Err(AppError::upstream_error(format!(
            "upstream returned {status}: {capped}"
        )));
    }

    if stream_requested {
        return Ok(build_stream(upstream_resp, resp_id).into_response());
    }

    let chat_resp: ChatResponse = upstream_resp.json().await.map_err(|e| {
        AppError::upstream_error(format!("failed to parse upstream response: {e}"))
    })?;

    let resp = convert_response(chat_resp, &model, &resp_id);

    info!("request {resp_id}: completed model={}, output_text_len={}",
        resp.model,
        resp.output_text.len()
    );

    Ok(Json(resp).into_response())
}

fn redact_auth(auth: &str) -> &str {
    if auth.is_empty() {
        return "<empty>";
    }
    if auth.len() > 12 {
        &auth[..12]
    } else {
        auth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_authorization_header() {
        let full = "Bearer sk-abc123def456";
        let redacted = redact_auth(full);
        assert!(redacted.len() < full.len());
        assert!(!redacted.contains("sk-abc123def456"));
    }

    #[test]
    fn test_redact_empty() {
        assert_eq!(redact_auth(""), "<empty>");
    }
}

async fn chat_completions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Response, AppError> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let req_id = format!("chatcmpl_{request_id}");

    let payload: Value = serde_json::from_slice(&body).map_err(|e| {
        AppError::bad_request(format!("invalid JSON body: {e}"))
    })?;

    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if model.is_empty() {
        return Err(AppError::bad_request("missing 'model' field in request body"));
    }

    let stream_requested = payload
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("chat request {req_id}: model={model}, stream={stream_requested}");

    if !state.config.log_prompts {
        info!("chat request {req_id}: auth={}", redact_auth(auth_header));
    }

    let (api_key, provider_cfg) = resolve_provider(&model, &state.config)?;

    let upstream_api_key = if provider_cfg.name == "OpenCode" {
        api_key
    } else if auth_header.starts_with("Bearer ") {
        let bearer = auth_header.strip_prefix("Bearer ").unwrap_or("");
        if !bearer.is_empty() {
            bearer.to_string()
        } else {
            api_key
        }
    } else {
        api_key
    };

    let upstream_model = strip_opencode_prefix(&model);

    let mut upstream_payload = payload.clone();
    if upstream_model != model {
        upstream_payload["model"] = serde_json::json!(upstream_model);
    }

    if state.config.log_prompts {
        info!("chat request {req_id}: upstream body = {}", serde_json::to_string(&upstream_payload).unwrap_or_default());
    }

    let upstream_url = format!("{}{}", provider_cfg.base_url.trim_end_matches('/'), provider_cfg.chat_path);
    info!("chat request {req_id}: upstream={upstream_url}, provider={}", provider_cfg.name);

    let mut upstream_req = state
        .client
        .post(&upstream_url)
        .header("Authorization", format!("Bearer {upstream_api_key}"))
        .header("Content-Type", "application/json");

    if provider_cfg.name == "OpenCode" {
        upstream_req = upstream_req.header("X-Provider", "opencode");
    }

    let upstream_resp = upstream_req
        .json(&upstream_payload)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::upstream_timeout()
            } else {
                AppError::upstream_error(format!("upstream request failed: {e}"))
            }
        })?;

    let status = upstream_resp.status();

    if !status.is_success() {
        let body = upstream_resp.text().await.unwrap_or_default();
        let capped = if body.len() > 2000 {
            format!("{}... (truncated)", &body[..2000])
        } else {
            body
        };
        info!("chat request {req_id}: upstream error status={status}, body={capped}");
        return Err(AppError::upstream_error(format!(
            "upstream returned {status}: {capped}"
        )));
    }

    if stream_requested {
        let stream = upstream_resp.bytes_stream();
        let body = Body::from_stream(stream);
        let mut response = Response::new(body);
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("text/event-stream"),
        );
        return Ok(response);
    }

    let resp_bytes = upstream_resp.bytes().await.map_err(|e| {
        AppError::upstream_error(format!("failed to read upstream response: {e}"))
    })?;

    info!("chat request {req_id}: completed model={model}, bytes={}", resp_bytes.len());

    let mut response = Response::new(Body::from(resp_bytes));
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/json"),
    );
    Ok(response)
}
