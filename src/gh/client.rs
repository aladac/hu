use anyhow::{Context, Result};
use octocrab::Octocrab;

use super::auth::get_token;
use super::types::{CiStatus, PullRequest, TestFailure};

/// Trait for GitHub API operations (enables mocking in tests)
pub trait GithubApi: Send + Sync {
    /// List open PRs authored by the current user
    fn list_user_prs(&self) -> impl std::future::Future<Output = Result<Vec<PullRequest>>> + Send;

    /// Get CI status for a PR
    fn get_ci_status(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> impl std::future::Future<Output = Result<CiStatus>> + Send;

    /// Get the branch name for a PR
    fn get_pr_branch(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Get the latest failed workflow run for a branch
    fn get_latest_failed_run_for_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> impl std::future::Future<Output = Result<Option<u64>>> + Send;

    /// Get failed jobs for a workflow run
    fn get_failed_jobs(
        &self,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> impl std::future::Future<Output = Result<Vec<(u64, String)>>> + Send;

    /// Download logs for a job
    fn get_job_logs(
        &self,
        owner: &str,
        repo: &str,
        job_id: u64,
    ) -> impl std::future::Future<Output = Result<String>> + Send;
}

/// Parse CI status from GitHub API responses (pure function, testable)
pub fn parse_ci_status(state: &str, check_runs: Option<&Vec<serde_json::Value>>) -> CiStatus {
    if let Some(runs) = check_runs {
        if runs.is_empty() && state == "pending" {
            return CiStatus::Pending;
        }

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
            parse_state_string(state)
        }
    } else {
        parse_state_string(state)
    }
}

/// Parse state string to CiStatus
fn parse_state_string(state: &str) -> CiStatus {
    match state {
        "success" => CiStatus::Success,
        "pending" => CiStatus::Pending,
        "failure" | "error" => CiStatus::Failed,
        _ => CiStatus::Unknown,
    }
}

/// Extract failed jobs from GitHub jobs API response (pure function, testable)
pub fn extract_failed_jobs(jobs: &serde_json::Value) -> Vec<(u64, String)> {
    jobs["jobs"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|j| j["conclusion"].as_str() == Some("failure"))
        .filter_map(|j| {
            let id = j["id"].as_u64()?;
            let name = j["name"].as_str()?.to_string();
            Some((id, name))
        })
        .collect()
}

/// Extract run ID from workflow runs response (pure function, testable)
pub fn extract_run_id(runs: &serde_json::Value) -> Option<u64> {
    runs["workflow_runs"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|r| r["id"].as_u64())
}

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
}

impl GithubApi for GithubClient {
    async fn list_user_prs(&self) -> Result<Vec<PullRequest>> {
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

    async fn get_ci_status(&self, owner: &str, repo: &str, pr_number: u64) -> Result<CiStatus> {
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

        Ok(parse_ci_status(state, check_runs))
    }

    async fn get_pr_branch(&self, owner: &str, repo: &str, pr_number: u64) -> Result<String> {
        let pr = self
            .client
            .pulls(owner, repo)
            .get(pr_number)
            .await
            .context("Failed to get PR")?;

        Ok(pr.head.ref_field)
    }

    async fn get_latest_failed_run_for_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Option<u64>> {
        let runs: serde_json::Value = self
            .client
            .get(
                format!(
                    "/repos/{}/{}/actions/runs?branch={}&status=failure&per_page=1",
                    owner, repo, branch
                ),
                None::<&()>,
            )
            .await
            .context("Failed to get workflow runs")?;

        Ok(extract_run_id(&runs))
    }

    async fn get_failed_jobs(
        &self,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> Result<Vec<(u64, String)>> {
        let jobs: serde_json::Value = self
            .client
            .get(
                format!("/repos/{}/{}/actions/runs/{}/jobs", owner, repo, run_id),
                None::<&()>,
            )
            .await
            .context("Failed to get jobs")?;

        Ok(extract_failed_jobs(&jobs))
    }

    async fn get_job_logs(&self, owner: &str, repo: &str, job_id: u64) -> Result<String> {
        // The logs endpoint returns a redirect to a download URL
        // We need to use reqwest directly for this
        let token = get_token().context("Not authenticated")?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/jobs/{}/logs",
            owner, repo, job_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hu-cli")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .context("Failed to request job logs")?;

        let logs = response.text().await.context("Failed to read job logs")?;

        Ok(logs)
    }
}

/// Extract test failures from logs (RSpec format)
pub fn parse_test_failures(logs: &str) -> Vec<TestFailure> {
    let mut failures = Vec::new();

    // Collect failure error messages in order
    let mut error_messages: Vec<String> = Vec::new();

    // Find the Failures section and parse each failure block
    if let Some(failures_start) = logs.find("Failures:") {
        let failures_end = logs.find("Failed examples:").unwrap_or(logs.len());
        let failures_section = &logs[failures_start..failures_end];

        // Split by numbered failure pattern "N) description"
        let block_starts: Vec<usize> = regex::Regex::new(r"\d+\)\s+\S")
            .ok()
            .map(|re| re.find_iter(failures_section).map(|m| m.start()).collect())
            .unwrap_or_default();

        let mut positions = block_starts.clone();
        positions.push(failures_section.len());

        for i in 0..block_starts.len() {
            let block = &failures_section[positions[i]..positions[i + 1]];

            // Extract error: code line after Failure/Error: and the error message on next line
            if let Some(fe_idx) = block.find("Failure/Error:") {
                let after_fe = &block[fe_idx..];
                let lines: Vec<String> = after_fe
                    .lines()
                    .map(clean_ci_line)
                    .filter(|l| !l.is_empty())
                    .take(4)
                    .collect();

                // lines[0] = "Failure/Error: <code>"
                // lines[1] = "<error message>" or "# <stack trace>"
                let code_line = lines
                    .first()
                    .map(|l| l.strip_prefix("Failure/Error:").unwrap_or(l).trim())
                    .unwrap_or("");
                let error_msg = lines.get(1).map(|s| s.as_str()).unwrap_or("");

                let error_text = if error_msg.is_empty() || error_msg.starts_with("# ") {
                    code_line.to_string()
                } else {
                    format!("{}\n{}", code_line, error_msg)
                };

                error_messages.push(error_text);
            }
        }
    }

    // Extract failed examples from the "Failed examples:" section
    // Format: rspec ./spec/helpers/prices_api_helper_spec.rb:289 # description
    let failed_examples_re = regex::Regex::new(r"rspec\s+(\./spec/[^\s]+:\d+)").ok();

    if let Some(re) = &failed_examples_re {
        for (i, cap) in re.captures_iter(logs).enumerate() {
            let spec_file = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            // Get error message by index (failures appear in same order)
            let failure_text = error_messages
                .get(i)
                .cloned()
                .unwrap_or_else(|| "Test failed".to_string());

            // Avoid duplicates
            if !failures
                .iter()
                .any(|f: &TestFailure| f.spec_file == spec_file)
            {
                failures.push(TestFailure {
                    spec_file: spec_file.to_string(),
                    failure_text,
                });
            }
        }
    }

    failures
}

/// Clean up CI log line by removing timestamp prefix
fn clean_ci_line(line: &str) -> String {
    // Remove timestamp prefix like "2026-01-27T18:51:46.1029380Z"
    let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T[\d:.]+Z\s*").ok();
    if let Some(re) = re {
        re.replace(line, "").trim().to_string()
    } else {
        line.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_token_returns_option() {
        // Just verify get_token doesn't panic
        let token = get_token();
        assert!(token.is_some() || token.is_none());
    }

    // parse_ci_status tests
    #[test]
    fn parse_ci_status_success_from_runs() {
        let runs = vec![json!({"status": "completed", "conclusion": "success"})];
        assert_eq!(parse_ci_status("pending", Some(&runs)), CiStatus::Success);
    }

    #[test]
    fn parse_ci_status_failed_from_runs() {
        let runs = vec![
            json!({"status": "completed", "conclusion": "success"}),
            json!({"status": "completed", "conclusion": "failure"}),
        ];
        assert_eq!(parse_ci_status("pending", Some(&runs)), CiStatus::Failed);
    }

    #[test]
    fn parse_ci_status_pending_from_runs() {
        let runs = vec![
            json!({"status": "completed", "conclusion": "success"}),
            json!({"status": "in_progress", "conclusion": null}),
        ];
        assert_eq!(parse_ci_status("pending", Some(&runs)), CiStatus::Pending);
    }

    #[test]
    fn parse_ci_status_empty_runs_pending() {
        let runs: Vec<serde_json::Value> = vec![];
        assert_eq!(parse_ci_status("pending", Some(&runs)), CiStatus::Pending);
    }

    #[test]
    fn parse_ci_status_no_runs_uses_state() {
        assert_eq!(parse_ci_status("success", None), CiStatus::Success);
        assert_eq!(parse_ci_status("failure", None), CiStatus::Failed);
        assert_eq!(parse_ci_status("error", None), CiStatus::Failed);
        assert_eq!(parse_ci_status("pending", None), CiStatus::Pending);
        assert_eq!(parse_ci_status("unknown", None), CiStatus::Unknown);
    }

    #[test]
    fn parse_state_string_all_cases() {
        assert_eq!(parse_state_string("success"), CiStatus::Success);
        assert_eq!(parse_state_string("pending"), CiStatus::Pending);
        assert_eq!(parse_state_string("failure"), CiStatus::Failed);
        assert_eq!(parse_state_string("error"), CiStatus::Failed);
        assert_eq!(parse_state_string("other"), CiStatus::Unknown);
    }

    // extract_failed_jobs tests
    #[test]
    fn extract_failed_jobs_filters_failures() {
        let jobs = json!({
            "jobs": [
                {"id": 1, "name": "build", "conclusion": "success"},
                {"id": 2, "name": "test", "conclusion": "failure"},
                {"id": 3, "name": "lint", "conclusion": "failure"},
            ]
        });
        let failed = extract_failed_jobs(&jobs);
        assert_eq!(failed.len(), 2);
        assert_eq!(failed[0], (2, "test".to_string()));
        assert_eq!(failed[1], (3, "lint".to_string()));
    }

    #[test]
    fn extract_failed_jobs_empty_when_all_success() {
        let jobs = json!({
            "jobs": [
                {"id": 1, "name": "build", "conclusion": "success"},
            ]
        });
        assert!(extract_failed_jobs(&jobs).is_empty());
    }

    #[test]
    fn extract_failed_jobs_handles_missing_jobs() {
        let jobs = json!({});
        assert!(extract_failed_jobs(&jobs).is_empty());
    }

    #[test]
    fn extract_failed_jobs_handles_null_jobs() {
        let jobs = json!({"jobs": null});
        assert!(extract_failed_jobs(&jobs).is_empty());
    }

    // extract_run_id tests
    #[test]
    fn extract_run_id_finds_first() {
        let runs = json!({
            "workflow_runs": [
                {"id": 123},
                {"id": 456},
            ]
        });
        assert_eq!(extract_run_id(&runs), Some(123));
    }

    #[test]
    fn extract_run_id_empty_array() {
        let runs = json!({"workflow_runs": []});
        assert_eq!(extract_run_id(&runs), None);
    }

    #[test]
    fn extract_run_id_missing_key() {
        let runs = json!({});
        assert_eq!(extract_run_id(&runs), None);
    }

    #[test]
    fn clean_ci_line_removes_timestamp() {
        let line = "2026-01-27T18:51:46.1029380Z      Failure/Error: some code";
        assert_eq!(clean_ci_line(line), "Failure/Error: some code");
    }

    #[test]
    fn clean_ci_line_preserves_line_without_timestamp() {
        let line = "  some regular line  ";
        assert_eq!(clean_ci_line(line), "some regular line");
    }

    #[test]
    fn clean_ci_line_handles_empty() {
        assert_eq!(clean_ci_line(""), "");
        assert_eq!(clean_ci_line("   "), "");
    }

    #[test]
    fn parse_test_failures_extracts_rspec_failures() {
        let logs = r#"
2026-01-27T18:51:46.1025638Z Failures:
2026-01-27T18:51:46.1026049Z
2026-01-27T18:51:46.1027821Z   1) MyClass does something
2026-01-27T18:51:46.1029380Z      Failure/Error: expect(result).to eq(expected)
2026-01-27T18:51:46.1167230Z        expected: 42
2026-01-27T18:51:46.1168761Z      # ./spec/my_class_spec.rb:10:in `block'
2026-01-27T18:51:46.1174151Z
2026-01-27T18:51:46.1253383Z Failed examples:
2026-01-27T18:51:46.1255271Z rspec ./spec/my_class_spec.rb:8 # MyClass does something
"#;
        let failures = parse_test_failures(logs);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].spec_file, "./spec/my_class_spec.rb:8");
        assert!(failures[0]
            .failure_text
            .contains("expect(result).to eq(expected)"));
        assert!(failures[0].failure_text.contains("expected: 42"));
    }

    #[test]
    fn parse_test_failures_handles_multiple_failures() {
        let logs = r#"
Failures:

  1) First test fails
     Failure/Error: assert false
       error one
     # ./spec/first_spec.rb:5

  2) Second test fails
     Failure/Error: raise "boom"
       error two
     # ./spec/second_spec.rb:10

Failed examples:

rspec ./spec/first_spec.rb:3 # First test fails
rspec ./spec/second_spec.rb:8 # Second test fails
"#;
        let failures = parse_test_failures(logs);
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].spec_file, "./spec/first_spec.rb:3");
        assert_eq!(failures[1].spec_file, "./spec/second_spec.rb:8");
        assert!(failures[0].failure_text.contains("assert false"));
        assert!(failures[1].failure_text.contains("raise \"boom\""));
    }

    #[test]
    fn parse_test_failures_handles_no_failures() {
        let logs = "All tests passed!\n0 failures";
        let failures = parse_test_failures(logs);
        assert!(failures.is_empty());
    }

    #[test]
    fn parse_test_failures_handles_empty_logs() {
        let failures = parse_test_failures("");
        assert!(failures.is_empty());
    }

    #[test]
    fn parse_test_failures_deduplicates() {
        let logs = r#"
Failures:

  1) Test fails
     Failure/Error: fail
     # ./spec/test_spec.rb:5

Failed examples:

rspec ./spec/test_spec.rb:3 # Test fails
rspec ./spec/test_spec.rb:3 # Test fails duplicate
"#;
        let failures = parse_test_failures(logs);
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn parse_test_failures_mock_error_format() {
        // Test the actual format from the CI logs
        let logs = r#"
2026-01-27T18:51:46.1025638Z Failures:
2026-01-27T18:51:46.1027821Z   1) PricesApiHelper pax value includes pax
2026-01-27T18:51:46.1029380Z      Failure/Error: found_lowest_prices += service.method
2026-01-27T18:51:46.1167230Z        #<InstanceDouble(Packages::Items)> received unexpected message :method
2026-01-27T18:51:46.1168761Z      # ./app/helpers/prices_api_helper.rb:62
2026-01-27T18:51:46.1253383Z Failed examples:
2026-01-27T18:51:46.1255271Z rspec ./spec/helpers/prices_api_helper_spec.rb:289 # PricesApiHelper pax value includes pax
"#;
        let failures = parse_test_failures(logs);
        assert_eq!(failures.len(), 1);
        assert_eq!(
            failures[0].spec_file,
            "./spec/helpers/prices_api_helper_spec.rb:289"
        );
        assert!(failures[0]
            .failure_text
            .contains("received unexpected message"));
    }

    #[test]
    fn parse_test_failures_code_only_when_error_is_stacktrace() {
        let logs = r#"
Failures:

  1) Test with stack trace only
     Failure/Error: some_method_call
     # ./spec/test_spec.rb:5

Failed examples:

rspec ./spec/test_spec.rb:3 # Test with stack trace only
"#;
        let failures = parse_test_failures(logs);
        assert_eq!(failures.len(), 1);
        // Should only have the code line since next line starts with #
        assert_eq!(failures[0].failure_text, "some_method_call");
    }

    #[test]
    fn parse_test_failures_handles_failures_section_only() {
        // Missing "Failed examples:" section
        let logs = r#"
Failures:

  1) Test fails
     Failure/Error: expect(1).to eq(2)
       expected: 2
     # ./spec/test_spec.rb:5
"#;
        let failures = parse_test_failures(logs);
        // No failed examples section means we can't extract spec files
        assert!(failures.is_empty());
    }

    #[test]
    fn parse_test_failures_handles_nested_spec_paths() {
        let logs = r#"
Failures:

  1) Deep path test
     Failure/Error: fail "deep"
       error msg

Failed examples:

rspec ./spec/features/admin/users/permissions_spec.rb:42 # Deep path test
"#;
        let failures = parse_test_failures(logs);
        assert_eq!(failures.len(), 1);
        assert_eq!(
            failures[0].spec_file,
            "./spec/features/admin/users/permissions_spec.rb:42"
        );
    }

    #[test]
    fn clean_ci_line_various_timestamps() {
        // Different timestamp formats from CI
        assert_eq!(
            clean_ci_line("2026-01-27T10:00:00.000Z some text"),
            "some text"
        );
        assert_eq!(clean_ci_line("2026-01-27T10:00:00.1234567Z text"), "text");
        assert_eq!(
            clean_ci_line("2020-12-31T23:59:59.9Z end of year"),
            "end of year"
        );
    }
}
