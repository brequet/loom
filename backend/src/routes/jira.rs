use axum::{
    Json,
    extract::{Path, State},
};

use crate::error::AppError;
use crate::models::JiraIssue;
use crate::state::AppState;

pub async fn get_issue(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<JiraIssue>, AppError> {
    let issue = state
        .jira_service
        .get_issue(&state.config, &key)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(issue))
}
