//! GitHub service layer - business logic that returns data
//!
//! Functions in this module accept trait objects and return typed data.
//! They never print - that's the CLI layer's job.

use anyhow::Result;

use super::client::{GithubApi, GithubClient};
use super::types::{CiStatus, PullRequest, RunsQuery, WorkflowRun};

/// List open PRs authored by the current user
pub async fn list_user_prs(api: &impl GithubApi) -> Result<Vec<PullRequest>> {
    api.list_user_prs().await
}

/// Get CI status for a PR
pub async fn get_ci_status(
    api: &impl GithubApi,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<CiStatus> {
    api.get_ci_status(owner, repo, pr_number).await
}

/// Get the branch name for a PR
#[allow(dead_code)]
pub async fn get_pr_branch(
    api: &impl GithubApi,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<String> {
    api.get_pr_branch(owner, repo, pr_number).await
}

/// Get the latest failed workflow run for a branch
#[allow(dead_code)]
pub async fn get_latest_failed_run(
    api: &impl GithubApi,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Option<u64>> {
    api.get_latest_failed_run_for_branch(owner, repo, branch)
        .await
}

/// Get failed jobs for a workflow run
pub async fn get_failed_jobs(
    api: &impl GithubApi,
    owner: &str,
    repo: &str,
    run_id: u64,
) -> Result<Vec<(u64, String)>> {
    api.get_failed_jobs(owner, repo, run_id).await
}

/// Download logs for a job
#[allow(dead_code)]
pub async fn get_job_logs(
    api: &impl GithubApi,
    owner: &str,
    repo: &str,
    job_id: u64,
) -> Result<String> {
    api.get_job_logs(owner, repo, job_id).await
}

/// Find PR number for a branch
pub async fn find_pr_for_branch(
    api: &impl GithubApi,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Option<u64>> {
    api.find_pr_for_branch(owner, repo, branch).await
}

/// List workflow runs for a repository
pub async fn list_workflow_runs(
    api: &impl GithubApi,
    query: &RunsQuery<'_>,
) -> Result<Vec<WorkflowRun>> {
    api.list_workflow_runs(query).await
}

/// Search PRs by title/branch containing a query string
pub async fn search_prs(
    api: &impl GithubApi,
    owner: &str,
    repo: &str,
    query: &str,
) -> Result<Vec<PullRequest>> {
    api.search_prs_by_title(owner, repo, query).await
}

/// Create a new authenticated client
pub fn create_client() -> Result<GithubClient> {
    GithubClient::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockApi {
        prs: Vec<PullRequest>,
        runs: Vec<WorkflowRun>,
    }

    impl MockApi {
        fn new() -> Self {
            Self {
                prs: vec![],
                runs: vec![],
            }
        }

        fn with_prs(mut self, prs: Vec<PullRequest>) -> Self {
            self.prs = prs;
            self
        }

        fn with_runs(mut self, runs: Vec<WorkflowRun>) -> Self {
            self.runs = runs;
            self
        }
    }

    impl GithubApi for MockApi {
        async fn list_user_prs(&self) -> Result<Vec<PullRequest>> {
            Ok(self.prs.clone())
        }

        async fn get_ci_status(&self, _owner: &str, _repo: &str, _pr: u64) -> Result<CiStatus> {
            Ok(CiStatus::Success)
        }

        async fn get_pr_branch(&self, _owner: &str, _repo: &str, _pr: u64) -> Result<String> {
            Ok("main".to_string())
        }

        async fn get_latest_failed_run_for_branch(
            &self,
            _owner: &str,
            _repo: &str,
            _branch: &str,
        ) -> Result<Option<u64>> {
            Ok(self.runs.first().map(|r| r.id))
        }

        async fn get_latest_failed_run(&self, _owner: &str, _repo: &str) -> Result<Option<u64>> {
            Ok(self.runs.first().map(|r| r.id))
        }

        async fn get_failed_jobs(
            &self,
            _owner: &str,
            _repo: &str,
            _run_id: u64,
        ) -> Result<Vec<(u64, String)>> {
            Ok(vec![(123, "test".to_string())])
        }

        async fn get_job_logs(&self, _owner: &str, _repo: &str, _job: u64) -> Result<String> {
            Ok("Test logs".to_string())
        }

        async fn find_pr_for_branch(
            &self,
            _owner: &str,
            _repo: &str,
            _branch: &str,
        ) -> Result<Option<u64>> {
            Ok(self.prs.first().map(|p| p.number))
        }

        async fn list_workflow_runs(&self, _query: &RunsQuery<'_>) -> Result<Vec<WorkflowRun>> {
            Ok(self.runs.clone())
        }

        async fn search_prs_by_title(
            &self,
            _owner: &str,
            _repo: &str,
            query: &str,
        ) -> Result<Vec<PullRequest>> {
            let query_lower = query.to_lowercase();
            Ok(self
                .prs
                .iter()
                .filter(|p| p.title.to_lowercase().contains(&query_lower))
                .cloned()
                .collect())
        }
    }

    fn make_pr(number: u64, title: &str) -> PullRequest {
        PullRequest {
            number,
            title: title.to_string(),
            html_url: format!("https://github.com/owner/repo/pull/{}", number),
            state: "open".to_string(),
            repo_full_name: "owner/repo".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            ci_status: None,
        }
    }

    fn make_run(id: u64, name: &str, status: &str) -> WorkflowRun {
        WorkflowRun {
            id,
            name: name.to_string(),
            status: status.to_string(),
            conclusion: Some("success".to_string()),
            branch: "main".to_string(),
            html_url: format!("https://github.com/owner/repo/actions/runs/{}", id),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            run_number: id,
        }
    }

    #[tokio::test]
    async fn list_user_prs_returns_all() {
        let api = MockApi::new().with_prs(vec![make_pr(1, "Fix bug"), make_pr(2, "Add feature")]);

        let result = list_user_prs(&api).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn get_ci_status_returns_status() {
        let api = MockApi::new();
        let result = get_ci_status(&api, "owner", "repo", 1).await.unwrap();
        assert_eq!(result, CiStatus::Success);
    }

    #[tokio::test]
    async fn list_workflow_runs_returns_all() {
        let api = MockApi::new().with_runs(vec![
            make_run(1, "CI", "completed"),
            make_run(2, "Deploy", "in_progress"),
        ]);

        let query = RunsQuery {
            owner: "owner",
            repo: "repo",
            branch: None,
            status: None,
            limit: 10,
        };
        let result = list_workflow_runs(&api, &query).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn search_prs_filters_by_title() {
        let api = MockApi::new().with_prs(vec![
            make_pr(1, "Fix authentication bug"),
            make_pr(2, "Add new feature"),
        ]);

        let result = search_prs(&api, "owner", "repo", "bug").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Fix authentication bug");
    }

    #[tokio::test]
    async fn find_pr_for_branch_returns_first() {
        let api = MockApi::new().with_prs(vec![make_pr(42, "My PR")]);
        let result = find_pr_for_branch(&api, "owner", "repo", "feature")
            .await
            .unwrap();
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn get_failed_jobs_returns_list() {
        let api = MockApi::new();
        let result = get_failed_jobs(&api, "owner", "repo", 123).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, "test");
    }
}
