use sqlx::sqlite::SqlitePool;

use crate::config::Config;
use crate::error::AppError;
use crate::models::{CreateSessionRequest, Session, SessionRow, SessionState};

#[derive(Clone)]
pub struct SessionService;

impl SessionService {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_sessions(&self, pool: &SqlitePool) -> Result<Vec<Session>, AppError> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, title, source_type, source_ref, state, opencode_port, workspace_path, project_id, created_at, updated_at FROM sessions WHERE state != 'terminated' ORDER BY updated_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Session::from).collect())
    }

    pub async fn get_session(
        &self,
        pool: &SqlitePool,
        id: &str,
    ) -> Result<Option<Session>, AppError> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, title, source_type, source_ref, state, opencode_port, workspace_path, project_id, created_at, updated_at FROM sessions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Session::from))
    }

    pub async fn create_session(
        &self,
        pool: &SqlitePool,
        config: &Config,
        req: CreateSessionRequest,
    ) -> Result<Session, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let title = req.title.unwrap_or_else(|| {
            match (&req.source_type, &req.source_ref) {
                (crate::models::SourceType::Jira, Some(r)) => format!("Jira: {r}"),
                (crate::models::SourceType::Gitlab, Some(r)) => format!("MR: {r}"),
                _ => format!("Scratch session"),
            }
        });

        let source_type_str = req.source_type.to_string();
        let state_str = SessionState::Provisioning.to_string();

        let port = self.allocate_port(pool, config).await?;
        let workspace_path = config.sessions_dir.join(&id);

        // Create workspace directory
        tokio::fs::create_dir_all(&workspace_path).await?;

        let workspace_str = workspace_path.to_string_lossy().to_string();

        sqlx::query(
            "INSERT INTO sessions (id, title, source_type, source_ref, state, opencode_port, workspace_path, project_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&id)
        .bind(&title)
        .bind(&source_type_str)
        .bind(&req.source_ref)
        .bind(&state_str)
        .bind(port as i64)
        .bind(&workspace_str)
        .bind(&req.project_id)
        .execute(pool)
        .await?;

        self.get_session(pool, &id)
            .await?
            .ok_or(AppError::Internal("Failed to fetch created session".into()))
    }

    pub async fn update_state(
        &self,
        pool: &SqlitePool,
        id: &str,
        new_state: SessionState,
    ) -> Result<Session, AppError> {
        let session = self
            .get_session(pool, id)
            .await?
            .ok_or(AppError::NotFound)?;

        // Validate transitions
        let valid = matches!(
            (&session.state, &new_state),
            (SessionState::Provisioning, SessionState::Running)
                | (SessionState::Provisioning, SessionState::Stopped)
                | (SessionState::Running, SessionState::Stopped)
                | (SessionState::Running, SessionState::Terminated)
                | (SessionState::Stopped, SessionState::Running)
                | (SessionState::Stopped, SessionState::Terminated)
        );

        if !valid {
            return Err(AppError::BadRequest(format!(
                "Invalid state transition from {} to {}",
                session.state, new_state
            )));
        }

        let state_str = new_state.to_string();
        sqlx::query("UPDATE sessions SET state = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&state_str)
            .bind(id)
            .execute(pool)
            .await?;

        self.get_session(pool, id)
            .await?
            .ok_or(AppError::Internal("Failed to fetch updated session".into()))
    }

    pub async fn terminate_session(
        &self,
        pool: &SqlitePool,
        config: &Config,
        id: &str,
    ) -> Result<(), AppError> {
        let session = self
            .get_session(pool, id)
            .await?
            .ok_or(AppError::NotFound)?;

        let state_str = SessionState::Terminated.to_string();
        sqlx::query("UPDATE sessions SET state = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&state_str)
            .bind(id)
            .execute(pool)
            .await?;

        // Cleanup workspace
        if let Some(ref ws) = session.workspace_path {
            let path = std::path::PathBuf::from(ws);
            if path.exists() {
                let _ = tokio::fs::remove_dir_all(&path).await;
            }
        }

        let _ = config; // config available for future cleanup logic
        Ok(())
    }

    pub async fn mark_running_as_stopped(&self, pool: &SqlitePool) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE sessions SET state = 'stopped', updated_at = datetime('now') WHERE state = 'running' OR state = 'provisioning'",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn allocate_port(&self, pool: &SqlitePool, config: &Config) -> Result<u16, AppError> {
        let used_ports: Vec<(i64,)> = sqlx::query_as(
            "SELECT opencode_port FROM sessions WHERE state != 'terminated' AND opencode_port IS NOT NULL",
        )
        .fetch_all(pool)
        .await?;

        let used: std::collections::HashSet<u16> =
            used_ports.iter().map(|(p,)| *p as u16).collect();

        for port in config.port_range_start..=config.port_range_end {
            if !used.contains(&port) {
                return Ok(port);
            }
        }

        Err(AppError::ServiceUnavailable(
            "No available ports in range".into(),
        ))
    }
}
