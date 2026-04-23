use crate::config::Config;
use crate::models::{GitLabMergeRequest, JiraIssue, Session, SourceType};
use std::fmt::Write;

/// Build the initial prompt for an `OpenCode` session.
///
/// The prompt includes:
/// 1. An optional static preamble from the base prompt file
/// 2. Dynamic context about the session source (Jira issue, GitLab MR, or scratch)
/// 3. Repository setup instructions
/// 4. Behavioral guidelines
pub async fn build_initial_prompt(
    config: &Config,
    session: &Session,
    jira_issue: Option<&JiraIssue>,
    gitlab_mr: Option<&GitLabMergeRequest>,
) -> String {
    let mut parts = Vec::new();

    // 1. Static preamble from file (if exists)
    if config.base_prompt_path.exists()
        && let Ok(preamble) = tokio::fs::read_to_string(&config.base_prompt_path).await
    {
        let trimmed = preamble.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }

    // 2. Dynamic context
    parts.push(build_context_section(session, jira_issue, gitlab_mr));

    // 3. Repository setup instructions
    parts.push(build_repo_section(config, session, jira_issue));

    // 4. Custom user instructions (if provided)
    if let Some(ref instructions) = session.custom_instructions {
        let trimmed = instructions.trim();
        if !trimmed.is_empty() {
            parts.push(format!("## User Instructions\n\n{trimmed}"));
        }
    }

    // 5. Behavioral guidelines
    parts.push(
        "## Guidelines\n\n\
         - Be autonomous when you are confident about what to do\n\
         - When uncertain, use the question tool to ask the user\n\
         - At the end of every response, ask: \
         \"Do you want to continue the current sub-session?\" \
         with no choice, only the question. This is mandatory."
            .to_string(),
    );

    parts.join("\n\n")
}

fn build_context_section(
    session: &Session,
    jira_issue: Option<&JiraIssue>,
    gitlab_mr: Option<&GitLabMergeRequest>,
) -> String {
    match session.source_type {
        SourceType::Jira => build_jira_context(session, jira_issue),
        SourceType::Gitlab => build_gitlab_context(session, gitlab_mr),
        SourceType::Scratch => {
            format!(
                "## Context\n\nThis is a scratch session: **{}**.",
                session.title
            )
        }
    }
}

fn build_jira_context(session: &Session, jira_issue: Option<&JiraIssue>) -> String {
    let Some(issue) = jira_issue else {
        let key = session.source_ref.as_deref().unwrap_or("unknown");
        return format!("## Context\n\n- **Issue**: {key}");
    };

    let mut ctx = String::from("## Context\n\n");
    let _ = writeln!(ctx, "- **Issue**: {} -- {}", issue.key, issue.summary);
    if let Some(ref t) = issue.issue_type {
        let _ = writeln!(ctx, "- **Type**: {t}");
    }
    let _ = writeln!(ctx, "- **Status**: {}", issue.status);
    if !issue.components.is_empty() {
        let _ = writeln!(ctx, "- **Components**: {}", issue.components.join(", "));
    }
    if let Some(ref desc) = issue.description
        && !desc.is_empty()
    {
        let _ = write!(ctx, "\n### Description\n\n{desc}");
    }
    ctx
}

fn build_gitlab_context(session: &Session, gitlab_mr: Option<&GitLabMergeRequest>) -> String {
    let Some(mr) = gitlab_mr else {
        let src = session.source_ref.as_deref().unwrap_or("unknown");
        return format!("## Context\n\n- **Merge Request**: {src}");
    };

    let mut ctx = String::from("## Context\n\n");
    let _ = writeln!(
        ctx,
        "- **Merge Request**: !{} -- {}\n- **Branch**: `{}`\n- **URL**: {}",
        mr.iid, mr.title, mr.source_branch, mr.web_url
    );
    if let Some(ref desc) = mr.description
        && !desc.is_empty()
    {
        let _ = write!(ctx, "\n### Description\n\n{desc}");
    }
    ctx
}

fn build_repo_section(
    config: &Config,
    session: &Session,
    jira_issue: Option<&JiraIssue>,
) -> String {
    let repos_dir = config.repos_dir.to_string_lossy();
    let workspace = session.workspace_path.as_deref().unwrap_or("(unknown)");
    let branch_hint = match session.source_type {
        SourceType::Jira => session.source_ref.as_deref().map(str::to_lowercase),
        _ => None,
    };

    let mut section = format!(
        "## Repository Setup\n\n\
         1. Check `{repos_dir}` for existing clones of the relevant project\n\
         2. If found: `git pull` to update the default branch\n\
         3. If not found: clone there\n\
         4. MANDATORY: Create a git worktree in the session workspace:\n\
            `git worktree add \"{workspace}\" -b {branch}`\n\
         5. If you cannot determine which repository to use, ask the user\n\
         REMEMBER: you have to use a git worktree of the used repository to\
         make sur no collision happens with other opencode sessions!",
        branch = branch_hint.as_deref().unwrap_or("<branch-name>"),
    );

    if matches!(session.source_type, SourceType::Jira)
        && jira_issue.as_ref().is_some_and(|i| !i.components.is_empty())
    {
        let _ = write!(
            section,
            "\n\nThe Jira components ({}) may help identify which repository to work with.",
            jira_issue.unwrap().components.join(", ")
        );
    }

    section
}
