use std::path::PathBuf;

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
}

impl Config {
    pub fn from_env() -> Self {
        let home = dirs_home();

        Self {
            port: env_or("LOOM_PORT", "3000").parse().unwrap_or(3000),
            sessions_dir: PathBuf::from(env_or(
                "LOOM_SESSIONS_DIR",
                &home.join(".loom").join("sessions").to_string_lossy(),
            )),
            repos_dir: PathBuf::from(env_or(
                "LOOM_REPOS_DIR",
                &home.join(".loom").join("repos").to_string_lossy(),
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
                &home
                    .join(".loom")
                    .join("opencode-prompt.md")
                    .to_string_lossy(),
            )),
        }
    }

    pub fn jira_configured(&self) -> bool {
        self.jira_base_url.is_some() && self.jira_email.is_some() && self.jira_api_token.is_some()
    }

    pub fn gitlab_configured(&self) -> bool {
        self.gitlab_base_url.is_some() && self.gitlab_private_token.is_some()
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
