use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::AppError;
use crate::models::Session;

#[derive(Clone)]
pub struct OpenCodeService {
    processes: Arc<Mutex<HashMap<String, Child>>>,
    client: reqwest::Client,
}

impl OpenCodeService {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    pub async fn spawn_session(
        &self,
        config: &Config,
        session: &Session,
    ) -> Result<(), AppError> {
        let workspace = session
            .workspace_path
            .as_deref()
            .ok_or(AppError::MissingWorkspace)?;

        let port = session.opencode_port.ok_or(AppError::MissingPort)?;

        tracing::info!(
            session_id = %session.id,
            port = port,
            workspace = workspace,
            bin = %config.opencode_bin,
            "Spawning OpenCode process"
        );

        let child = Command::new(&config.opencode_bin)
            .arg("web")
            .arg("--port")
            .arg(port.to_string())
            .arg("--hostname")
            .arg("0.0.0.0")
            .current_dir(workspace)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::OpenCodeSpawn(format!("{e} (bin: {})", config.opencode_bin)))?;

        let mut procs = self.processes.lock().await;
        procs.insert(session.id.clone(), child);

        tracing::info!(session_id = %session.id, "OpenCode process spawned");
        Ok(())
    }

    /// Create an OpenCode session via the REST API.
    /// API: POST /session  body: { title?: string }  returns: Session { id, ... }
    pub async fn create_opencode_session(
        &self,
        port: i64,
        title: &str,
    ) -> Result<String, AppError> {
        let url = format!("http://localhost:{port}/session");

        tracing::info!(port = port, title = title, "Creating OpenCode session");

        let mut last_err = String::new();
        for attempt in 1..=20 {
            if attempt > 1 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            tracing::debug!(attempt = attempt, url = %url, "Attempting to reach OpenCode");

            let resp = match self
                .client
                .post(&url)
                .json(&serde_json::json!({ "title": title }))
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    last_err = e.to_string();
                    tracing::debug!(attempt = attempt, error = %e, "OpenCode not reachable yet");
                    continue;
                }
            };

            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();

            if !status.is_success() {
                tracing::warn!(
                    attempt = attempt,
                    status = status.as_u16(),
                    body = %body_text,
                    "OpenCode returned non-success"
                );
                last_err = format!("HTTP {status}: {body_text}");
                continue;
            }

            tracing::debug!(body = %body_text, "OpenCode session creation response");

            let body: serde_json::Value =
                serde_json::from_str(&body_text).map_err(|e| AppError::ResponseParse {
                    service: "OpenCode".into(),
                    detail: format!("Invalid JSON: {e}. Body: {body_text}"),
                })?;

            // The response is a Session object with an "id" field
            let session_id = body["id"]
                .as_str()
                .ok_or_else(|| AppError::OpenCodeNoSessionId {
                    body: body_text.clone(),
                })?
                .to_string();

            tracing::info!(opencode_session_id = %session_id, "OpenCode session created");
            return Ok(session_id);
        }

        Err(AppError::OpenCodeNotReady {
            attempts: 20,
            last_error: last_err,
        })
    }

    /// Send initial prompt to OpenCode session asynchronously.
    /// API: POST /session/:id/prompt_async
    /// body: { parts: [{ type: "text", text: "..." }], model?: { providerID, modelID } }
    /// Returns 204 No Content on success.
    pub async fn send_initial_prompt(
        &self,
        port: i64,
        session_id: &str,
        prompt: &str,
        model: &str,
    ) -> Result<(), AppError> {
        let url = format!("http://localhost:{port}/session/{session_id}/prompt_async");

        tracing::info!(
            port = port,
            session_id = session_id,
            model = model,
            prompt_len = prompt.len(),
            "Sending initial prompt to OpenCode"
        );

        // Parse model string "provider/model" into providerID and modelID
        let (provider_id, model_id) = model.split_once('/').unwrap_or(("", model));

        let body = serde_json::json!({
            "parts": [{ "type": "text", "text": prompt }],
            "model": {
                "providerID": provider_id,
                "modelID": model_id,
            }
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::HttpRequest {
                context: "send initial prompt to OpenCode".into(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::OpenCodePromptFailed { status, body });
        }

        tracing::info!("Initial prompt sent successfully");
        Ok(())
    }

    /// Discover the web UI path prefix used by OpenCode.
    /// OpenCode's web frontend routes are `/{base64_project_path}/session/{id}`.
    /// We query `GET /project/current` to learn the project path, then base64-encode it.
    /// Falls back to base64-encoding the workspace path's root drive.
    pub async fn get_web_path_prefix(&self, port: i64, workspace: &str) -> Result<String, AppError> {
        use base64::Engine;

        let url = format!("http://localhost:{port}/project/current");

        let resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        if let Ok(resp) = resp {
            if resp.status().is_success() {
                let body_text = resp.text().await.unwrap_or_default();
                tracing::info!(body = %body_text, "OpenCode /project/current response");

                if let Ok(body) = serde_json::from_str::<serde_json::Value>(&body_text) {
                    let path = body["path"]
                        .as_str()
                        .or_else(|| body["root"].as_str())
                        .or_else(|| body["dir"].as_str())
                        .unwrap_or("");

                    if !path.is_empty() {
                        let prefix = base64::engine::general_purpose::STANDARD_NO_PAD.encode(path);
                        tracing::info!(path = %path, prefix = %prefix, "Discovered OpenCode web UI prefix from API");
                        return Ok(prefix);
                    }
                }
            }
        }

        // Fallback: base64-encode the workspace path itself
        // OpenCode typically resolves to the workspace dir or its git root
        let prefix = base64::engine::general_purpose::STANDARD_NO_PAD.encode(workspace);
        tracing::info!(workspace = %workspace, prefix = %prefix, "Using workspace path as OpenCode web UI prefix (fallback)");
        Ok(prefix)
    }

    pub async fn stop_process(&self, session_id: &str) -> Result<(), AppError> {
        let mut procs = self.processes.lock().await;
        if let Some(mut child) = procs.remove(session_id) {
            tracing::info!(session_id = session_id, "Stopping OpenCode process");
            let _ = child.kill().await;
        } else {
            tracing::debug!(session_id = session_id, "No running OpenCode process found");
        }
        Ok(())
    }
}
