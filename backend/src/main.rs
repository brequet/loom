mod config;
mod db;
mod error;
mod models;
mod routes;
mod services;
mod state;

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use config::Config;
use services::{
    gitlab::GitLabService, jira::JiraService, opencode::OpenCodeService, session::SessionService,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    // Ensure directories exist
    tokio::fs::create_dir_all(&config.sessions_dir)
        .await
        .expect("Failed to create sessions directory");
    tokio::fs::create_dir_all(&config.repos_dir)
        .await
        .expect("Failed to create repos directory");

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create services
    let session_service = SessionService::new();
    let opencode_service = OpenCodeService::new();
    let jira_service = JiraService::new();
    let gitlab_service = GitLabService::new();

    // Startup recovery: mark any running/provisioning sessions as stopped
    session_service
        .mark_running_as_stopped(&pool)
        .await
        .expect("Failed startup recovery");

    let app_state = state::AppState::new(
        pool,
        config.clone(),
        session_service,
        opencode_service,
        jira_service,
        gitlab_service,
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes::api_routes())
        .layer(cors)
        .with_state(app_state);

    let app = if cfg!(debug_assertions) {
        app.fallback(routes::dev_proxy)
    } else {
        app.fallback(routes::static_handler)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server error");
}
