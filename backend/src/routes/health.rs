use axum::{Json, extract::State};

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::HealthResponse;
use crate::state::AppState;

pub async fn health(State(_state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

pub async fn config(State(state): State<AppState>) -> Result<Json<AppConfig>, AppError> {
    Ok(Json(state.config.app_config()))
}
