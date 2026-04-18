use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::AppError;
use crate::models::{
    CreateSessionRequest, Session, SessionListResponse, SessionState,
};
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
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>, AppError> {
    // 1. Create DB record in provisioning state
    let mut session = state
        .session_service
        .create_session(&state.pool, &state.config, req)
        .await?;

    let port = session.opencode_port.unwrap();

    // 2. Spawn opencode process
    if let Err(e) = state
        .opencode_service
        .spawn_session(&state.config, &session)
        .await
    {
        tracing::error!("Failed to spawn opencode: {e}");
        let _ = state
            .session_service
            .update_state(&state.pool, &session.id, SessionState::Stopped)
            .await;
        return Err(e);
    }

    // 3. Create opencode session
    match state
        .opencode_service
        .create_opencode_session(port, &state.config.opencode_model)
        .await
    {
        Ok(oc_session_id) => {
            // 4. Send initial prompt if base prompt file exists
            if state.config.base_prompt_path.exists() {
                if let Ok(prompt) =
                    tokio::fs::read_to_string(&state.config.base_prompt_path).await
                {
                    if let Err(e) = state
                        .opencode_service
                        .send_initial_prompt(port, &oc_session_id, &prompt)
                        .await
                    {
                        tracing::error!("Failed to send initial prompt: {e}");
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to create opencode session: {e}");
            // Still mark as running - the process is up, session creation can be retried
        }
    }

    // 5. Update to running
    session = state
        .session_service
        .update_state(&state.pool, &session.id, SessionState::Running)
        .await?;

    Ok(Json(session))
}

pub async fn resume_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, AppError> {
    let session = state
        .session_service
        .get_session(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Spawn opencode again
    state
        .opencode_service
        .spawn_session(&state.config, &session)
        .await?;

    let session = state
        .session_service
        .update_state(&state.pool, &id, SessionState::Running)
        .await?;

    Ok(Json(session))
}

pub async fn stop_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, AppError> {
    state.opencode_service.stop_process(&id).await?;

    let session = state
        .session_service
        .update_state(&state.pool, &id, SessionState::Stopped)
        .await?;

    Ok(Json(session))
}

pub async fn terminate_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.opencode_service.stop_process(&id).await?;

    state
        .session_service
        .terminate_session(&state.pool, &state.config, &id)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
