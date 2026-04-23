use crate::error::AppError;
use crate::models::{Session, SessionState, SourceType};
use crate::services::prompt;
use crate::state::AppState;

/// Orchestrate session provisioning: spawn OpenCode, create session, send prompt, mark running.
/// Returns the fully provisioned session.
pub async fn provision_session(
    state: &AppState,
    mut session: Session,
) -> Result<Session, AppError> {
    let port = session.opencode_port.ok_or(AppError::MissingPort)?;
    let session_id = session.id.clone();

    // 1. Spawn opencode process
    if let Err(e) = state
        .opencode_service
        .spawn_session(&state.config, &session)
        .await
    {
        tracing::error!(session_id = %session_id, error = %e, "Provision step 1 failed: spawn OpenCode");
        let _ = state
            .session_service
            .update_state(&state.pool, &session_id, SessionState::Stopped)
            .await;
        return Err(e);
    }

    // 2. Create opencode session via REST API
    match state
        .opencode_service
        .create_opencode_session(port, &session.title)
        .await
    {
        Ok(oc_session_id) => {
            tracing::info!(
                session_id = %session_id,
                opencode_session_id = %oc_session_id,
                "OpenCode session created"
            );

            // Discover the web UI path prefix
            let workspace = session.workspace_path.as_deref().unwrap_or("");
            let path_prefix = state
                .opencode_service
                .get_web_path_prefix(port, workspace)
                .await
                .map_err(|e| {
                    tracing::warn!(session_id = %session_id, error = %e, "Failed to discover web UI prefix (non-fatal)");
                    e
                })
                .ok();

            // Persist the OpenCode session ID
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
                tracing::warn!(session_id = %session_id, error = %e, "Failed to persist OpenCode session ID (non-fatal)");
            }

            // 3. Build and send initial prompt
            let jira_issue = fetch_jira_context(state, &session).await;
            let gitlab_mr = fetch_gitlab_context(state, &session).await;

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
                tracing::warn!(session_id = %session_id, error = %e, "Failed to send initial prompt (non-fatal)");
            }
        }
        Err(e) => {
            tracing::warn!(
                session_id = %session_id, error = %e,
                "Failed to create OpenCode session (process is running, can be retried)"
            );
        }
    }

    // 4. Mark running
    session = state
        .session_service
        .update_state(&state.pool, &session_id, SessionState::Running)
        .await?;

    tracing::info!(session_id = %session_id, port = port, "Session provisioning complete");
    Ok(session)
}

async fn fetch_jira_context(
    state: &AppState,
    session: &Session,
) -> Option<crate::models::JiraIssue> {
    if !matches!(session.source_type, SourceType::Jira) {
        return None;
    }
    let key = session.source_ref.as_ref()?;
    match state.jira_service.get_issue(&state.config, key).await {
        Ok(issue) => issue,
        Err(e) => {
            tracing::warn!(session_id = %session.id, error = %e, "Failed to fetch Jira issue (non-fatal)");
            None
        }
    }
}

async fn fetch_gitlab_context(
    state: &AppState,
    session: &Session,
) -> Option<crate::models::GitLabMergeRequest> {
    if !matches!(session.source_type, SourceType::Gitlab) {
        return None;
    }
    let url = session.source_ref.as_ref()?;
    match state
        .gitlab_service
        .get_merge_request_by_url(&state.config, url)
        .await
    {
        Ok(mr) => mr,
        Err(e) => {
            tracing::warn!(session_id = %session.id, error = %e, "Failed to fetch GitLab MR (non-fatal)");
            None
        }
    }
}
