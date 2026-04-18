use sqlx::sqlite::SqlitePool;

use crate::config::Config;
use crate::services::{
    gitlab::GitLabService, jira::JiraService, opencode::OpenCodeService, session::SessionService,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
    pub session_service: SessionService,
    pub opencode_service: OpenCodeService,
    pub jira_service: JiraService,
    pub gitlab_service: GitLabService,
}

impl AppState {
    pub fn new(
        pool: SqlitePool,
        config: Config,
        session_service: SessionService,
        opencode_service: OpenCodeService,
        jira_service: JiraService,
        gitlab_service: GitLabService,
    ) -> Self {
        Self {
            pool,
            config,
            session_service,
            opencode_service,
            jira_service,
            gitlab_service,
        }
    }
}
