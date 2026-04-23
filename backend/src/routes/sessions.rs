use axum::{
    Json,
    extract::{Path, State},
};

use crate::error::AppError;
use crate::models::{CreateSessionRequest, Session, SessionListResponse, SessionState};
use crate::services::orchestration;
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

pub async fn create_session(
    State(state): State<AppState>,
    Json(mut req): Json<CreateSessionRequest>,
) -> Result<Json<Session>, AppError> {
    // Resolve real MR title for GitLab sessions when none provided
    if req.source_type == crate::models::SourceType::Gitlab && req.title.is_none() {
        if let Some(ref url) = req.source_ref {
            if let Ok(Some(mr)) = state
                .gitlab_service
                .get_merge_request_by_url(&state.config, url)
                .await
            {
                req.title = Some(format!("MR !{}: {}", mr.iid, mr.title));
            }
        }
    }

    tracing::info!(
        source_type = %req.source_type,
        source_ref = ?req.source_ref,
        title = ?req.title,
        "Creating new session"
    );

    // Create DB record in provisioning state
    let session = state
        .session_service
        .create_session(&state.pool, &state.config, req)
        .await?;

    // Orchestrate: spawn process, create OC session, send prompt, mark running
    let session = orchestration::provision_session(&state, session).await?;

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

    let current = state
        .session_service
        .get_session(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Idempotent: if already stopped, just return
    if current.state == SessionState::Stopped {
        return Ok(Json(current));
    }

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
) -> Result<Json<Session>, AppError> {
    tracing::info!(session_id = %id, "Terminating session");

    state.opencode_service.stop_process(&id).await?;

    let session = state
        .session_service
        .terminate_session(&state.pool, &state.config, &id)
        .await?;

    tracing::info!(session_id = %id, "Session terminated");
    Ok(Json(session))
}
