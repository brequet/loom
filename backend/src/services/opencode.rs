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
            .ok_or_else(|| AppError::Internal("Session has no workspace path".into()))?;

        let port = session
            .opencode_port
            .ok_or_else(|| AppError::Internal("Session has no port assigned".into()))?;

        let child = Command::new(&config.opencode_bin)
            .arg("web")
            .arg("--port")
            .arg(port.to_string())
            .arg("--hostname")
            .arg("0.0.0.0")
            .current_dir(workspace)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to spawn opencode: {e}")))?;

        let mut procs = self.processes.lock().await;
        procs.insert(session.id.clone(), child);

        Ok(())
    }

    pub async fn create_opencode_session(
        &self,
        port: i64,
        model: &str,
    ) -> Result<String, AppError> {
        let url = format!("http://localhost:{port}/api/session");

        // Retry a few times since the process takes time to start
        let mut last_err = String::new();
        for attempt in 0..15 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            match self
                .client
                .post(&url)
                .json(&serde_json::json!({ "model": model }))
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let body: serde_json::Value = resp.json().await.map_err(|e| {
                        AppError::Internal(format!("Failed to parse opencode response: {e}"))
                    })?;
                    let session_id = body["id"]
                        .as_str()
                        .or_else(|| body["sessionID"].as_str())
                        .ok_or_else(|| {
                            AppError::Internal(format!(
                                "No session ID in opencode response: {body}"
                            ))
                        })?
                        .to_string();
                    return Ok(session_id);
                }
                Ok(resp) => {
                    last_err = format!("HTTP {}", resp.status());
                }
                Err(e) => {
                    last_err = e.to_string();
                }
            }
        }

        Err(AppError::ServiceUnavailable(format!(
            "OpenCode not ready after retries: {last_err}"
        )))
    }

    pub async fn send_initial_prompt(
        &self,
        port: i64,
        session_id: &str,
        prompt: &str,
    ) -> Result<(), AppError> {
        let url = format!("http://localhost:{port}/api/session/{session_id}/prompt_async");

        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "content": prompt }))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send prompt: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Prompt send failed: HTTP {status} - {body}"
            )));
        }

        Ok(())
    }

    pub async fn stop_process(&self, session_id: &str) -> Result<(), AppError> {
        let mut procs = self.processes.lock().await;
        if let Some(mut child) = procs.remove(session_id) {
            let _ = child.kill().await;
        }
        Ok(())
    }
}
