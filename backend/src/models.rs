use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SessionState {
    #[serde(rename = "provisioning")]
    Provisioning,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "terminated")]
    Terminated,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Provisioning => write!(f, "provisioning"),
            SessionState::Running => write!(f, "running"),
            SessionState::Stopped => write!(f, "stopped"),
            SessionState::Terminated => write!(f, "terminated"),
        }
    }
}

impl SessionState {
    pub fn from_str_val(s: &str) -> Self {
        match s {
            "provisioning" => SessionState::Provisioning,
            "running" => SessionState::Running,
            "terminated" => SessionState::Terminated,
            _ => SessionState::Stopped,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SourceType {
    #[serde(rename = "jira")]
    Jira,
    #[serde(rename = "gitlab")]
    Gitlab,
    #[serde(rename = "scratch")]
    Scratch,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Jira => write!(f, "jira"),
            SourceType::Gitlab => write!(f, "gitlab"),
            SourceType::Scratch => write!(f, "scratch"),
        }
    }
}

impl SourceType {
    pub fn from_str_val(s: &str) -> Self {
        match s {
            "jira" => SourceType::Jira,
            "gitlab" => SourceType::Gitlab,
            _ => SourceType::Scratch,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub source_type: SourceType,
    pub source_ref: Option<String>,
    pub state: SessionState,
    #[ts(type = "number | null")]
    pub opencode_port: Option<i64>,
    pub opencode_session_id: Option<String>,
    pub opencode_path_prefix: Option<String>,
    pub workspace_path: Option<String>,
    pub model: String,
    pub custom_instructions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// -- Raw DB row types for sqlx::FromRow --

#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub id: String,
    pub title: String,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub state: String,
    pub opencode_port: Option<i64>,
    pub opencode_session_id: Option<String>,
    pub opencode_path_prefix: Option<String>,
    pub workspace_path: Option<String>,
    pub model: String,
    pub custom_instructions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<SessionRow> for Session {
    fn from(r: SessionRow) -> Self {
        Session {
            id: r.id,
            title: r.title,
            source_type: SourceType::from_str_val(&r.source_type),
            source_ref: r.source_ref,
            state: SessionState::from_str_val(&r.state),
            opencode_port: r.opencode_port,
            opencode_session_id: r.opencode_session_id,
            opencode_path_prefix: r.opencode_path_prefix,
            workspace_path: r.workspace_path,
            model: r.model,
            custom_instructions: r.custom_instructions,
            created_at: ensure_utc_suffix(&r.created_at),
            updated_at: ensure_utc_suffix(&r.updated_at),
        }
    }
}

// -- Request/Response types --

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateSessionRequest {
    pub source_type: SourceType,
    pub source_ref: Option<String>,
    pub title: Option<String>,
    pub model: Option<String>,
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SessionListResponse {
    pub sessions: Vec<Session>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JiraIssue {
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: String,
    pub issue_type: Option<String>,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GitLabMergeRequest {
    #[ts(type = "number")]
    pub iid: i64,
    pub title: String,
    pub description: Option<String>,
    pub source_branch: String,
    pub web_url: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Ensure datetime strings from SQLite have a `Z` suffix so JS parses them as UTC.
fn ensure_utc_suffix(s: &str) -> String {
    if s.ends_with('Z') {
        s.to_string()
    } else {
        format!("{s}Z")
    }
}
