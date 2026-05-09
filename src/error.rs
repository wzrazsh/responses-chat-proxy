use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
    pub code: String,
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
            code: "bad_request".to_string(),
        }
    }

    pub fn upstream_error(msg: impl Into<String>) -> Self {
        AppError {
            status: StatusCode::BAD_GATEWAY,
            message: msg.into(),
            code: "upstream_error".to_string(),
        }
    }

    pub fn upstream_timeout() -> Self {
        AppError {
            status: StatusCode::GATEWAY_TIMEOUT,
            message: "upstream request timed out".to_string(),
            code: "upstream_timeout".to_string(),
        }
    }

    pub fn unsupported_feature(msg: impl Into<String>) -> Self {
        AppError {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
            code: "unsupported_feature".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({
            "error": {
                "message": self.message,
                "type": self.code,
                "code": self.code,
            }
        });
        (self.status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
            code: "internal_error".to_string(),
        }
    }
}
