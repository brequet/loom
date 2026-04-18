use axum::{
    extract::Request,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use reqwest::StatusCode;
use rust_embed::Embed;

use crate::state::AppState;

#[derive(Embed)]
#[folder = "../frontend/dist/"]
struct FrontendAssets;

pub fn api_routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Dev mode: proxy non-API requests to Vite dev server
pub async fn dev_proxy(req: Request) -> Response {
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let url = format!("http://localhost:5173{}{}", path, query);

    match reqwest::get(&url).await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let headers = resp.headers().clone();
            let body = resp.bytes().await.unwrap_or_default();

            let mut response = (status, body).into_response();
            for (key, value) in headers.iter() {
                if let Ok(name) = axum::http::HeaderName::from_bytes(key.as_ref()) {
                    if let Ok(val) = axum::http::HeaderValue::from_bytes(value.as_ref()) {
                        response.headers_mut().insert(name, val);
                    }
                }
            }
            response
        }
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            "Vite dev server not running. Start it with: cd frontend && pnpm dev",
        )
            .into_response(),
    }
}

/// Prod mode: serve embedded frontend assets
pub async fn static_handler(req: Request) -> Response {
    let path = req.uri().path().trim_start_matches('/');

    // Try exact path first, then fall back to index.html (SPA routing)
    let path = if path.is_empty() { "index.html" } else { path };

    match FrontendAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            (
                [(axum::http::header::CONTENT_TYPE, mime)],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => {
            // SPA fallback: serve index.html for non-file routes
            match FrontendAssets::get("index.html") {
                Some(content) => Html(
                    String::from_utf8_lossy(&content.data).to_string(),
                )
                .into_response(),
                None => (StatusCode::NOT_FOUND, "Not found").into_response(),
            }
        }
    }
}
