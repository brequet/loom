use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::error::AppError;
use crate::models::{JiraIssue, SearchQuery};
use crate::state::AppState;

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<JiraIssue>>, AppError> {
    let issues = state
        .jira_service
        .search_issues(&state.config, &query.q)
        .await?;
    Ok(Json(issues))
}

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
