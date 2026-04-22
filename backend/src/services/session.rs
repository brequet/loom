use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::AppError;
use crate::models::{CreateSessionRequest, Session, SessionRow, SessionState};

#[derive(Clone)]
pub struct SessionService {
    port_lock: Arc<Mutex<()>>,
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            port_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn list_sessions(&self, pool: &SqlitePool) -> Result<Vec<Session>, AppError> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, title, source_type, source_ref, state, opencode_port, opencode_session_id, opencode_path_prefix, workspace_path, model, custom_instructions, created_at, updated_at FROM sessions WHERE state != 'terminated' ORDER BY updated_at DESC",
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
            "SELECT id, title, source_type, source_ref, state, opencode_port, opencode_session_id, opencode_path_prefix, workspace_path, model, custom_instructions, created_at, updated_at FROM sessions WHERE id = ?",
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
        let title = req
            .title
            .unwrap_or_else(|| match (&req.source_type, &req.source_ref) {
                (crate::models::SourceType::Jira, Some(r)) => format!("Jira: {r}"),
                (crate::models::SourceType::Gitlab, Some(r)) => format!("MR: {r}"),
                _ => "Scratch session".to_string(),
            });

        let source_type_str = req.source_type.to_string();
        let state_str = SessionState::Provisioning.to_string();
        let model = req.model.unwrap_or_else(|| config.default_model.clone());

        let port = self.allocate_port(pool, config).await?;
        let workspace_path = config.sessions_dir.join(&id);

        tracing::info!(
            session_id = %id,
            title = %title,
            source_type = %source_type_str,
            port = port,
            workspace = %workspace_path.display(),
            "Creating session"
        );

        tokio::fs::create_dir_all(&workspace_path)
            .await
            .map_err(|e| AppError::Filesystem {
                context: format!("creating workspace at {}", workspace_path.display()),
                source: e,
            })?;

        let workspace_str = workspace_path.to_string_lossy().to_string();

        sqlx::query(
            "INSERT INTO sessions (id, title, source_type, source_ref, state, opencode_port, workspace_path, model, custom_instructions, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&id)
        .bind(&title)
        .bind(&source_type_str)
        .bind(&req.source_ref)
        .bind(&state_str)
        .bind(i64::from(port))
        .bind(&workspace_str)
        .bind(&model)
        .bind(&req.custom_instructions)
        .execute(pool)
        .await?;

        tracing::info!(session_id = %id, "Session record created");

        self.get_session(pool, &id)
            .await?
            .ok_or_else(|| AppError::BadRequest("Failed to fetch created session".into()))
    }

    pub async fn update_opencode_session_id(
        &self,
        pool: &SqlitePool,
        id: &str,
        opencode_session_id: &str,
        opencode_path_prefix: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE sessions SET opencode_session_id = ?, opencode_path_prefix = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(opencode_session_id)
            .bind(opencode_path_prefix)
            .bind(id)
            .execute(pool)
            .await?;

        tracing::info!(session_id = %id, opencode_session_id = %opencode_session_id, "OpenCode session ID saved");
        Ok(())
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

        let valid = matches!(
            (&session.state, &new_state),
            (
                SessionState::Provisioning | SessionState::Stopped,
                SessionState::Running
            ) | (
                SessionState::Provisioning | SessionState::Running,
                SessionState::Stopped
            ) | (
                SessionState::Running | SessionState::Stopped,
                SessionState::Terminated
            )
        );

        if !valid {
            return Err(AppError::InvalidStateTransition {
                from: session.state.to_string(),
                to: new_state.to_string(),
            });
        }

        let state_str = new_state.to_string();
        sqlx::query("UPDATE sessions SET state = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&state_str)
            .bind(id)
            .execute(pool)
            .await?;

        tracing::info!(session_id = %id, from = %session.state, to = %new_state, "Session state updated");

        self.get_session(pool, id)
            .await?
            .ok_or_else(|| AppError::BadRequest("Failed to fetch updated session".into()))
    }

    pub async fn terminate_session(
        &self,
        pool: &SqlitePool,
        _config: &Config,
        id: &str,
    ) -> Result<Session, AppError> {
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

        if let Some(ref ws) = session.workspace_path {
            let path = std::path::PathBuf::from(ws);
            if path.exists() {
                // Retry workspace cleanup -- files may still be locked briefly after process kill
                let mut removed = false;
                for attempt in 1..=5 {
                    match tokio::fs::remove_dir_all(&path).await {
                        Ok(()) => {
                            removed = true;
                            break;
                        }
                        Err(e) => {
                            tracing::warn!(
                                session_id = %id, path = %path.display(),
                                attempt = attempt, error = %e,
                                "Failed to cleanup workspace, retrying..."
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    }
                }
                if !removed {
                    tracing::error!(
                        session_id = %id, path = %path.display(),
                        "Failed to cleanup workspace after 5 attempts"
                    );
                }
            }
        }

        tracing::info!(session_id = %id, "Session terminated");
        self.get_session(pool, id).await?.ok_or(AppError::NotFound)
    }

    pub async fn mark_running_as_stopped(&self, pool: &SqlitePool) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE sessions SET state = 'stopped', updated_at = datetime('now') WHERE state = 'running' OR state = 'provisioning'",
        )
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            tracing::info!(
                count = result.rows_affected(),
                "Startup recovery: marked sessions as stopped"
            );
        }
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    async fn allocate_port(&self, pool: &SqlitePool, config: &Config) -> Result<u16, AppError> {
        let _guard = self.port_lock.lock().await;

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

        Err(AppError::NoAvailablePorts {
            start: config.port_range_start,
            end: config.port_range_end,
        })
    }
}
