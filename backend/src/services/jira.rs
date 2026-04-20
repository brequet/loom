use crate::config::Config;
use crate::error::AppError;
use crate::models::JiraIssue;

#[derive(Clone)]
pub struct JiraService {
    client: reqwest::Client,
}

impl JiraService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_issue(
        &self,
        config: &Config,
        key: &str,
    ) -> Result<Option<JiraIssue>, AppError> {
        if !config.jira_configured() {
            return Ok(None);
        }

        let base_url = config.jira_base_url.as_deref().unwrap();
        let email = config.jira_email.as_deref().unwrap();
        let token = config.jira_api_token.as_deref().unwrap();

        let url = format!("{base_url}/rest/api/3/issue/{key}");

        let resp = self
            .client
            .get(&url)
            .basic_auth(email, Some(token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::HttpRequest {
                context: format!("Jira get issue {key}"),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!(status = status, key = key, body = %body, "Jira get issue failed");
            return Ok(None);
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| AppError::ResponseParse {
            service: "Jira".into(),
            detail: format!("Failed to parse issue response: {e}"),
        })?;
        let key = body["key"].as_str().unwrap_or("").to_string();
        let fields = &body["fields"];
        let summary = fields["summary"].as_str().unwrap_or("").to_string();
        let description = extract_description(&fields["description"]);
        let status = fields["status"]["name"].as_str().unwrap_or("").to_string();
        let issue_type = fields["issuetype"]["name"].as_str().map(String::from);
        let components = fields["components"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(JiraIssue {
            key,
            summary,
            description,
            status,
            issue_type,
            components,
        }))
    }
}

/// Extract plain text from a Jira description field.
/// API v3 returns ADF (JSON object), v2 returns plain string.
fn extract_description(value: &serde_json::Value) -> Option<String> {
    // Plain string (v2 style)
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }
    // ADF document (v3 style) — extract text content recursively
    if value.is_object() {
        let mut texts = Vec::new();
        extract_adf_text(value, &mut texts);
        let result = texts.join("\n");
        if result.is_empty() {
            return None;
        }
        return Some(result);
    }
    None
}

fn extract_adf_text(node: &serde_json::Value, texts: &mut Vec<String>) {
    if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
        texts.push(text.to_string());
    }
    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        for child in content {
            extract_adf_text(child, texts);
        }
    }
}
