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
            "{base_url}/api/v4/merge_requests?search={}&state=opened&scope=all&per_page=10",
            urlencoding(query)
        );

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !resp.status().is_success() {
            tracing::error!("GitLab search failed: HTTP {}", resp.status());
            return Ok(vec![]);
        }

        let mrs: Vec<GitLabMergeRequest> = resp.json().await?;
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
            .await?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let mr: GitLabMergeRequest = resp.json().await?;
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
