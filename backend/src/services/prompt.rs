use crate::config::Config;
use crate::models::{GitLabMergeRequest, JiraIssue, Session, SourceType};

/// Build the initial prompt for an OpenCode session.
///
/// The prompt includes:
/// 1. An optional static preamble from the base prompt file
/// 2. Dynamic context about the session source (Jira issue, GitLab MR, or scratch)
pub async fn build_initial_prompt(
    config: &Config,
    session: &Session,
    jira_issue: Option<&JiraIssue>,
    gitlab_mr: Option<&GitLabMergeRequest>,
) -> String {
    let mut parts = Vec::new();

    // 1. Static preamble from file (if exists)
    if config.base_prompt_path.exists() {
        if let Ok(preamble) = tokio::fs::read_to_string(&config.base_prompt_path).await {
            let trimmed = preamble.trim();
            if !trimmed.is_empty() {
                parts.push(trimmed.to_string());
            }
        }
    }

    // 2. Dynamic context based on source type
    match session.source_type {
        SourceType::Jira => {
            if let Some(issue) = jira_issue {
                let mut ctx = format!(
                    "## Task\n\nYou are working on Jira issue **{}**: {}\n",
                    issue.key, issue.summary
                );
                if let Some(ref desc) = issue.description {
                    if !desc.is_empty() {
                        ctx.push_str(&format!("\n### Description\n\n{}\n", desc));
                    }
                }
                parts.push(ctx);
            } else if let Some(ref source_ref) = session.source_ref {
                parts.push(format!(
                    "## Task\n\nYou are working on Jira issue **{}**.\n",
                    source_ref
                ));
            }
        }
        SourceType::Gitlab => {
            if let Some(mr) = gitlab_mr {
                let mut ctx = format!(
                    "## Task\n\nYou are working on GitLab merge request **!{}**: {}\n\n- **Source branch**: `{}`\n- **URL**: {}\n",
                    mr.iid, mr.title, mr.source_branch, mr.web_url
                );
                if let Some(ref desc) = mr.description {
                    if !desc.is_empty() {
                        ctx.push_str(&format!("\n### Description\n\n{}\n", desc));
                    }
                }
                parts.push(ctx);
            } else if let Some(ref source_ref) = session.source_ref {
                parts.push(format!(
                    "## Task\n\nYou are working on a GitLab merge request: {}\n",
                    source_ref
                ));
            }
        }
        SourceType::Scratch => {
            parts.push(format!(
                "## Task\n\nThis is a scratch session: **{}**.\n",
                session.title
            ));
        }
    }

    // 3. Common instructions
    parts.push(
        "Figure out what project(s) you need to work with. \
         Clone them if needed, or if you are not sure, ask the user using the question tool."
            .to_string(),
    );

    parts.join("\n\n")
}
