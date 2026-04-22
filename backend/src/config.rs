use serde::Serialize;
use std::path::PathBuf;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ModelDefinition {
    /// Full model identifier (e.g. "github-copilot/claude-sonnet-4.6")
    pub id: String,
    /// Human-readable label (e.g. "Claude Sonnet 4.6")
    pub label: String,
    /// Provider prefix (e.g. "github-copilot")
    pub provider: String,
}

/// Response for GET /api/config
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct AppConfig {
    pub models: Vec<ModelDefinition>,
    pub default_model: String,
    pub jira_configured: bool,
    pub gitlab_configured: bool,
    pub sessions_dir: String,
    pub repos_dir: String,
    pub base_prompt_path: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub sessions_dir: PathBuf,
    pub repos_dir: PathBuf,
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub opencode_bin: String,
    pub jira_base_url: Option<String>,
    pub jira_email: Option<String>,
    pub jira_api_token: Option<String>,
    pub gitlab_base_url: Option<String>,
    pub gitlab_private_token: Option<String>,
    pub base_prompt_path: PathBuf,
    pub default_model: String,
    pub models: Vec<ModelDefinition>,
}

impl Config {
    pub fn from_env() -> Self {
        let base = loom_base_dir();

        let default_model = env_or("LOOM_DEFAULT_MODEL", "github-copilot/claude-sonnet-4.6");

        let models = vec![
            ModelDefinition {
                id: "github-copilot/claude-sonnet-4.6".into(),
                label: "Claude Sonnet 4.6".into(),
                provider: "github-copilot".into(),
            },
            ModelDefinition {
                id: "github-copilot/claude-opus-4.6".into(),
                label: "Claude Opus 4.6".into(),
                provider: "github-copilot".into(),
            },
            ModelDefinition {
                id: "github-copilot/claude-haiku-4.5".into(),
                label: "Claude Haiku 4.5".into(),
                provider: "github-copilot".into(),
            },
            ModelDefinition {
                id: "github-copilot/gpt-5-mini".into(),
                label: "GPT-5 Mini".into(),
                provider: "github-copilot".into(),
            },
        ];

        Self {
            port: env_or("LOOM_PORT", "3000").parse().unwrap_or(3000),
            sessions_dir: PathBuf::from(env_or(
                "LOOM_SESSIONS_DIR",
                &base.join("sessions").to_string_lossy(),
            )),
            repos_dir: PathBuf::from(env_or(
                "LOOM_REPOS_DIR",
                &base.join("repos").to_string_lossy(),
            )),
            port_range_start: env_or("LOOM_PORT_RANGE_START", "10000")
                .parse()
                .unwrap_or(10000),
            port_range_end: env_or("LOOM_PORT_RANGE_END", "10999")
                .parse()
                .unwrap_or(10999),
            opencode_bin: env_or("LOOM_OPENCODE_BIN", "opencode"),
            jira_base_url: std::env::var("JIRA_BASE_URL").ok(),
            jira_email: std::env::var("JIRA_EMAIL").ok(),
            jira_api_token: std::env::var("JIRA_API_TOKEN").ok(),
            gitlab_base_url: std::env::var("GITLAB_BASE_URL").ok(),
            gitlab_private_token: std::env::var("GITLAB_PRIVATE_TOKEN").ok(),
            base_prompt_path: PathBuf::from(env_or(
                "LOOM_BASE_PROMPT_PATH",
                &base.join("opencode-prompt.md").to_string_lossy(),
            )),
            default_model,
            models,
        }
    }

    pub fn jira_configured(&self) -> bool {
        self.jira_base_url.is_some() && self.jira_email.is_some() && self.jira_api_token.is_some()
    }

    pub fn gitlab_configured(&self) -> bool {
        self.gitlab_base_url.is_some() && self.gitlab_private_token.is_some()
    }

    /// Build the public-facing config response for the frontend
    pub fn app_config(&self) -> AppConfig {
        AppConfig {
            models: self.models.clone(),
            default_model: self.default_model.clone(),
            jira_configured: self.jira_configured(),
            gitlab_configured: self.gitlab_configured(),
            sessions_dir: self.sessions_dir.to_string_lossy().to_string(),
            repos_dir: self.repos_dir.to_string_lossy().to_string(),
            base_prompt_path: self.base_prompt_path.to_string_lossy().to_string(),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_or_else(|_| PathBuf::from("."), PathBuf::from)
}

/// Returns ~/.config/loom - the base data directory for loom.
pub fn loom_base_dir() -> PathBuf {
    dirs_home().join(".config").join("loom")
}
