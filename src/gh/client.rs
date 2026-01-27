use anyhow::{bail, Context, Result};
use octocrab::Octocrab;

use super::auth::get_token;
use super::types::{CiStatus, PullRequest};

pub struct GithubClient {
    client: Octocrab,
}

impl GithubClient {
    /// Create a new authenticated GitHub client
    pub fn new() -> Result<Self> {
        let token = get_token().context("Not authenticated. Run `hu gh login` first.")?;

        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .context("Failed to create GitHub client")?;

        Ok(Self { client })
    }

    /// Create client from provided token (for testing)
    #[allow(dead_code)]
    pub fn with_token(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .context("Failed to create GitHub client")?;

        Ok(Self { client })
    }

    /// List open PRs authored by the current user
    pub async fn list_user_prs(&self) -> Result<Vec<PullRequest>> {
        // Use the search API to find PRs where author is current user
        let result = self
            .client
            .search()
            .issues_and_pull_requests("is:pr is:open author:@me")
            .send()
            .await
            .context("Failed to search for PRs")?;

        let prs: Vec<PullRequest> = result
            .items
            .into_iter()
            .filter_map(|issue| {
                // Extract repo from URL: https://api.github.com/repos/owner/repo/issues/123
                let repo_full_name = issue
                    .repository_url
                    .path_segments()?
                    .skip(1) // skip "repos"
                    .take(2) // take "owner" and "repo"
                    .collect::<Vec<_>>()
                    .join("/");

                let state = match issue.state {
                    octocrab::models::IssueState::Open => "open",
                    octocrab::models::IssueState::Closed => "closed",
                    _ => "unknown",
                };

                Some(PullRequest {
                    number: issue.number,
                    title: issue.title,
                    html_url: issue.html_url.to_string(),
                    state: state.to_string(),
                    repo_full_name,
                    created_at: issue.created_at.to_rfc3339(),
                    updated_at: issue.updated_at.to_rfc3339(),
                    ci_status: None,
                })
            })
            .collect();

        Ok(prs)
    }

    /// Get CI status for a PR
    pub async fn get_ci_status(&self, owner: &str, repo: &str, pr_number: u64) -> Result<CiStatus> {
        // Get the PR to find the head SHA
        let pr = self
            .client
            .pulls(owner, repo)
            .get(pr_number)
            .await
            .context("Failed to get PR")?;

        let sha = &pr.head.sha;

        // Get combined status
        let status: serde_json::Value = self
            .client
            .get(
                format!("/repos/{}/{}/commits/{}/status", owner, repo, sha),
                None::<&()>,
            )
            .await
            .context("Failed to get commit status")?;

        let state = status["state"].as_str().unwrap_or("unknown");

        // Also check for check runs (GitHub Actions uses this)
        let checks: serde_json::Value = self
            .client
            .get(
                format!("/repos/{}/{}/commits/{}/check-runs", owner, repo, sha),
                None::<&()>,
            )
            .await
            .unwrap_or_default();

        let check_runs = checks["check_runs"].as_array();

        // Determine overall status
        let ci_status = if let Some(runs) = check_runs {
            if runs.is_empty() && state == "pending" {
                CiStatus::Pending
            } else {
                let any_failed = runs
                    .iter()
                    .any(|r| r["conclusion"].as_str() == Some("failure"));
                let any_pending = runs.iter().any(|r| {
                    r["status"].as_str() != Some("completed") || r["conclusion"].as_str().is_none()
                });
                let all_success = runs
                    .iter()
                    .all(|r| r["conclusion"].as_str() == Some("success"));

                if any_failed {
                    CiStatus::Failed
                } else if any_pending {
                    CiStatus::Pending
                } else if all_success && !runs.is_empty() {
                    CiStatus::Success
                } else {
                    match state {
                        "success" => CiStatus::Success,
                        "pending" => CiStatus::Pending,
                        "failure" | "error" => CiStatus::Failed,
                        _ => CiStatus::Unknown,
                    }
                }
            }
        } else {
            match state {
                "success" => CiStatus::Success,
                "pending" => CiStatus::Pending,
                "failure" | "error" => CiStatus::Failed,
                _ => CiStatus::Unknown,
            }
        };

        Ok(ci_status)
    }

    /// Verify the client is authenticated by checking the current user
    #[allow(dead_code)]
    pub async fn verify_auth(&self) -> Result<String> {
        let user = self
            .client
            .current()
            .user()
            .await
            .context("Failed to verify authentication")?;

        if user.login.is_empty() {
            bail!("Authentication verification failed");
        }

        Ok(user.login)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_token_returns_option() {
        // Just verify get_token doesn't panic
        let token = get_token();
        assert!(token.is_some() || token.is_none());
    }
}
