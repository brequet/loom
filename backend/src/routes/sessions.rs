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

            // 4. Send initial prompt if base prompt file exists
            if state.config.base_prompt_path.exists() {
                match tokio::fs::read_to_string(&state.config.base_prompt_path).await {
                    Ok(prompt) => {
                        if let Err(e) = state
                            .opencode_service
                            .send_initial_prompt(port, &oc_session_id, &prompt, &state.config.opencode_model)
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
                            path = %state.config.base_prompt_path.display(),
                            error = %e,
                            "Step 4 warning: failed to read base prompt file (non-fatal)"
                        );
                    }
                }
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
