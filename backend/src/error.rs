use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// Domain-level errors for clean error handling across the application.
/// Each service has its own error variants so failures are traceable
/// to the exact operation that failed.
#[derive(Debug, Error)]
pub enum AppError {
    // --- Generic ---
    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    // --- Database ---
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // --- Session lifecycle ---
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Session has no workspace path")]
    MissingWorkspace,

    #[error("Session has no port assigned")]
    MissingPort,

    #[error("No available ports in range {start}..{end}")]
    NoAvailablePorts { start: u16, end: u16 },

    // --- OpenCode process ---
    #[error("Failed to spawn OpenCode process: {0}")]
    OpenCodeSpawn(String),

    #[error("OpenCode not ready after {attempts} attempts (last error: {last_error})")]
    OpenCodeNotReady { attempts: u32, last_error: String },

    #[error("OpenCode response missing session ID. Response body: {body}")]
    OpenCodeNoSessionId { body: String },

    #[error("Failed to send prompt to OpenCode: HTTP {status} — {body}")]
    OpenCodePromptFailed { status: u16, body: String },

    // --- Jira ---
    #[error("Jira API error: HTTP {status} -- {body}")]
    JiraApi { status: u16, body: String },

    // --- GitLab ---
    #[error("GitLab API error: HTTP {status} — {body}")]
    GitLabApi { status: u16, body: String },

    // --- Filesystem ---
    #[error("Filesystem error: {context} — {source}")]
    Filesystem {
        context: String,
        source: std::io::Error,
    },

    // --- HTTP client ---
    #[error("HTTP request failed: {context} — {source}")]
    HttpRequest {
        context: String,
        source: reqwest::Error,
    },

    // --- Response parsing ---
    #[error("Failed to parse response from {service}: {detail}")]
    ResponseParse { service: String, detail: String },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            AppError::InvalidStateTransition { .. } => (StatusCode::CONFLICT, self.to_string()),
            AppError::MissingWorkspace | AppError::MissingPort => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::NoAvailablePorts { .. } => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            AppError::OpenCodeSpawn(_)
            | AppError::OpenCodeNotReady { .. }
            | AppError::OpenCodeNoSessionId { .. }
            | AppError::OpenCodePromptFailed { .. }
            | AppError::JiraApi { .. }
            | AppError::GitLabApi { .. }
            | AppError::HttpRequest { .. }
            | AppError::ResponseParse { .. } => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Filesystem { .. } => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        tracing::error!("{self}");

        (status, Json(json!({ "error": message }))).into_response()
    }
}
