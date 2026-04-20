use axum::{
    Json,
    extract::{Path, State},
};

use crate::error::AppError;
use crate::models::{CreateSessionRequest, Session, SessionListResponse, SessionState, SourceType};
use crate::services::prompt;
use crate::state::AppState;

pub async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<SessionListResponse>, AppError> {
    let sessions = state.session_service.list_sessions(&state.pool).await?;
    Ok(Json(SessionListResponse { sessions }))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, AppError> {
    let session = state
        .session_service
        .get_session(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(session))
}

#[allow(clippy::too_many_lines)]
pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>, AppError> {
    tracing::info!(
        source_type = %req.source_type,
        source_ref = ?req.source_ref,
        title = ?req.title,
        "Creating new session"
    );

    // 1. Create DB record in provisioning state
    let mut session = state
        .session_service
        .create_session(&state.pool, &state.config, req)
        .await?;

    let port = session.opencode_port.unwrap();
    let session_id = session.id.clone();

    // 2. Spawn opencode process
    if let Err(e) = state
        .opencode_service
        .spawn_session(&state.config, &session)
        .await
    {
        tracing::error!(session_id = %session_id, error = %e, "Step 2 failed: spawn OpenCode");
        let _ = state
            .session_service
            .update_state(&state.pool, &session_id, SessionState::Stopped)
            .await;
        return Err(e);
    }

    // 3. Create opencode session via REST API
    match state
        .opencode_service
        .create_opencode_session(port, &session.title)
        .await
    {
        Ok(oc_session_id) => {
            tracing::info!(
                session_id = %session_id,
                opencode_session_id = %oc_session_id,
                "Step 3 succeeded: OpenCode session created"
            );

            // Discover the web UI path prefix for constructing the correct URL
            let workspace = session.workspace_path.as_deref().unwrap_or("");
            let path_prefix = match state
                .opencode_service
                .get_web_path_prefix(port, workspace)
                .await
            {
                Ok(prefix) => Some(prefix),
                Err(e) => {
                    tracing::warn!(
                        session_id = %session_id,
                        error = %e,
                        "Failed to discover OpenCode web UI prefix (non-fatal)"
                    );
                    None
                }
            };

            // Persist the OpenCode session ID so the frontend can build the correct URL
            if let Err(e) = state
                .session_service
                .update_opencode_session_id(
                    &state.pool,
                    &session_id,
                    &oc_session_id,
                    path_prefix.as_deref(),
                )
                .await
            {
                tracing::warn!(
                    session_id = %session_id,
                    error = %e,
                    "Failed to persist OpenCode session ID (non-fatal)"
                );
            }

            // 4. Build and send initial prompt with session context
            let jira_issue = if matches!(session.source_type, SourceType::Jira) {
                if let Some(ref key) = session.source_ref {
                    match state.jira_service.get_issue(&state.config, key).await {
                        Ok(issue) => issue,
                        Err(e) => {
                            tracing::warn!(
                                session_id = %session_id,
                                error = %e,
                                "Failed to fetch Jira issue for prompt (non-fatal)"
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let gitlab_mr = if matches!(session.source_type, SourceType::Gitlab) {
                if let Some(ref url) = session.source_ref {
                    match state
                        .gitlab_service
                        .get_merge_request_by_url(&state.config, url)
                        .await
                    {
                        Ok(mr) => mr,
                        Err(e) => {
                            tracing::warn!(
                                session_id = %session_id,
                                error = %e,
                                "Failed to fetch GitLab MR for prompt (non-fatal)"
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let initial_prompt = prompt::build_initial_prompt(
                &state.config,
                &session,
                jira_issue.as_ref(),
                gitlab_mr.as_ref(),
            )
            .await;

            if let Err(e) = state
                .opencode_service
                .send_initial_prompt(port, &oc_session_id, &initial_prompt, &session.model)
                .await
            {
                tracing::warn!(
                    session_id = %session_id,
                    error = %e,
                    "Step 4 warning: failed to send initial prompt (non-fatal)"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                session_id = %session_id,
                error = %e,
                "Step 3 warning: failed to create OpenCode session (process is running, session creation can be retried)"
            );
            // Still mark as running - the process is up
        }
    }

    // 5. Update to running
    session = state
        .session_service
        .update_state(&state.pool, &session_id, SessionState::Running)
        .await?;

    tracing::info!(session_id = %session_id, port = port, "Session creation complete");
    Ok(Json(session))
}

pub async fn resume_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, AppError> {
    tracing::info!(session_id = %id, "Resuming session");

    let session = state
        .session_service
        .get_session(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    if session.state != SessionState::Stopped {
        return Err(AppError::InvalidStateTransition {
            from: session.state.to_string(),
            to: "running".into(),
        });
    }

    state
        .opencode_service
        .spawn_session(&state.config, &session)
        .await?;

    let session = state
        .session_service
        .update_state(&state.pool, &id, SessionState::Running)
        .await?;

    tracing::info!(session_id = %id, "Session resumed");
    Ok(Json(session))
}

pub async fn stop_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, AppError> {
    tracing::info!(session_id = %id, "Stopping session");

    state.opencode_service.stop_process(&id).await?;

    let session = state
        .session_service
        .update_state(&state.pool, &id, SessionState::Stopped)
        .await?;

    tracing::info!(session_id = %id, "Session stopped");
    Ok(Json(session))
}

pub async fn terminate_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!(session_id = %id, "Terminating session");

    state.opencode_service.stop_process(&id).await?;

    state
        .session_service
        .terminate_session(&state.pool, &state.config, &id)
        .await?;

    tracing::info!(session_id = %id, "Session terminated");
    Ok(Json(serde_json::json!({ "ok": true })))
}
