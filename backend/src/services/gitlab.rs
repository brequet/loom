use crate::config::Config;
use crate::error::AppError;
use crate::models::GitLabMergeRequest;

#[derive(Clone)]
pub struct GitLabService {
    client: reqwest::Client,
}

impl GitLabService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn search_merge_requests(
        &self,
        config: &Config,
        query: &str,
    ) -> Result<Vec<GitLabMergeRequest>, AppError> {
        if !config.gitlab_configured() {
            return Ok(vec![]);
        }

        let base_url = config.gitlab_base_url.as_deref().unwrap();
        let token = config.gitlab_private_token.as_deref().unwrap();

        let url = format!(
            "{base_url}/api/v4/merge_requests?search={}&state=opened&scope=all&per_page=10&in=title",
            urlencoding(query)
        );

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .header("User-Agent", "Loom/0.1")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::HttpRequest {
                context: "GitLab MR search".into(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(status = status, body = %body, "GitLab search failed");
            return Err(AppError::GitLabApi { status, body });
        }

        let mrs: Vec<GitLabMergeRequest> = resp.json().await.map_err(|e| AppError::ResponseParse {
            service: "GitLab".into(),
            detail: format!("Failed to parse MR search response: {e}"),
        })?;
        Ok(mrs)
    }

    pub async fn get_merge_request(
        &self,
        config: &Config,
        project_id: &str,
        iid: i64,
    ) -> Result<Option<GitLabMergeRequest>, AppError> {
        if !config.gitlab_configured() {
            return Ok(None);
        }

        let base_url = config.gitlab_base_url.as_deref().unwrap();
        let token = config.gitlab_private_token.as_deref().unwrap();

        let url = format!(
            "{base_url}/api/v4/projects/{}/merge_requests/{iid}",
            urlencoding(project_id)
        );

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::HttpRequest {
                context: format!("GitLab get MR {iid}"),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!(status = status, body = %body, "GitLab get MR failed");
            return Ok(None);
        }

        let mr: GitLabMergeRequest = resp.json().await.map_err(|e| AppError::ResponseParse {
            service: "GitLab".into(),
            detail: format!("Failed to parse MR response: {e}"),
        })?;
        Ok(Some(mr))
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '"' => "%22".to_string(),
            _ if c.is_ascii_alphanumeric() || "-._~".contains(c) => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
