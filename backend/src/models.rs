use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
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

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SourceType {
    #[serde(rename = "jira")]
    Jira,
    #[serde(rename = "gitlab")]
    Gitlab,
    #[serde(rename = "scratch")]
    Scratch,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub source_type: SourceType,
    pub source_ref: Option<String>,
    pub state: SessionState,
    pub opencode_port: Option<i64>,
    pub workspace_path: Option<String>,
    pub project_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_url: String,
    pub bare_clone_path: Option<String>,
    pub created_at: String,
}
