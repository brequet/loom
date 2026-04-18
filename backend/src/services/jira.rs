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

    pub async fn search_issues(
        &self,
        config: &Config,
        query: &str,
    ) -> Result<Vec<JiraIssue>, AppError> {
        if !config.jira_configured() {
            return Ok(vec![]);
        }

        let base_url = config.jira_base_url.as_deref().unwrap();
        let email = config.jira_email.as_deref().unwrap();
        let token = config.jira_api_token.as_deref().unwrap();

        let jql = format!("text ~ \"{}\" ORDER BY updated DESC", query);
        let url = format!(
            "{base_url}/rest/api/2/search?jql={}&maxResults=10&fields=summary,description,status",
            urlencoding(&jql)
        );

        let resp = self
            .client
            .get(&url)
            .basic_auth(email, Some(token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !resp.status().is_success() {
            tracing::error!("Jira search failed: HTTP {}", resp.status());
            return Ok(vec![]);
        }

        let body: serde_json::Value = resp.json().await?;
        let issues = body["issues"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|issue| {
                        let key = issue["key"].as_str()?.to_string();
                        let fields = &issue["fields"];
                        let summary = fields["summary"].as_str().unwrap_or("").to_string();
                        let description = fields["description"].as_str().map(|s| s.to_string());
                        let status = fields["status"]["name"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        Some(JiraIssue {
                            key,
                            summary,
                            description,
                            status,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(issues)
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

        let url = format!("{base_url}/rest/api/2/issue/{key}");

        let resp = self
            .client
            .get(&url)
            .basic_auth(email, Some(token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let body: serde_json::Value = resp.json().await?;
        let key = body["key"].as_str().unwrap_or("").to_string();
        let fields = &body["fields"];
        let summary = fields["summary"].as_str().unwrap_or("").to_string();
        let description = fields["description"].as_str().map(|s| s.to_string());
        let status = fields["status"]["name"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(Some(JiraIssue {
            key,
            summary,
            description,
            status,
        }))
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '"' => "%22".to_string(),
            '~' => "%7E".to_string(),
            _ if c.is_ascii_alphanumeric() || "-._".contains(c) => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
