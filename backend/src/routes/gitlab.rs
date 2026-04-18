use axum::{
    extract::{Query, State},
    Json,
};

use crate::error::AppError;
use crate::models::{GitLabMergeRequest, SearchQuery};
use crate::state::AppState;

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<GitLabMergeRequest>>, AppError> {
    let mrs = state
        .gitlab_service
        .search_merge_requests(&state.config, &query.q)
        .await?;
    Ok(Json(mrs))
}
