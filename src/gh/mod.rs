//! GitHub integration
//!
//! # CLI Usage
//! Use [`run_command`] for CLI commands that format and print output.
//!
//! # Programmatic Usage (MCP/HTTP)
//! Use the reusable functions that return typed data:
//! - [`list_user_prs`] - List open PRs by current user
//! - [`get_ci_status`] - Get CI status for a PR
//! - [`list_workflow_runs`] - List workflow runs
//! - [`search_prs`] - Search PRs by title/branch

mod auth;
mod cli;
mod client;
mod failures;
mod fix;
mod helpers;
mod login;
mod prs;
mod runs;
mod service;
mod sync;
mod types;

use anyhow::Result;

pub use cli::GhCommand;
pub use types::{CiStatus, PullRequest, RunsQuery, WorkflowRun};

/// Run a GitHub command (CLI entry point - formats and prints)
#[cfg(not(tarpaulin_include))]
pub async fn run_command(cmd: GhCommand) -> anyhow::Result<()> {
    match cmd {
        GhCommand::Login(args) => login::run(args).await,
        GhCommand::Prs => prs::run().await,
        GhCommand::Failures(args) => failures::run(args).await,
        GhCommand::Fix(args) => fix::run(args).await,
        GhCommand::Runs(args) => runs::run(args).await,
        GhCommand::Sync(args) => sync::run(args),
    }
}

// ============================================================================
// Reusable functions for MCP/HTTP - return typed data, never print
// ============================================================================

/// List open PRs authored by the current user (for MCP/HTTP)
#[allow(dead_code)]
pub async fn list_user_prs() -> Result<Vec<PullRequest>> {
    let client = service::create_client()?;
    service::list_user_prs(&client).await
}

/// Get CI status for a PR (for MCP/HTTP)
#[allow(dead_code)]
pub async fn get_ci_status(owner: &str, repo: &str, pr_number: u64) -> Result<CiStatus> {
    let client = service::create_client()?;
    service::get_ci_status(&client, owner, repo, pr_number).await
}

/// List workflow runs for a repository (for MCP/HTTP)
#[allow(dead_code)]
pub async fn list_workflow_runs(query: &RunsQuery<'_>) -> Result<Vec<WorkflowRun>> {
    let client = service::create_client()?;
    service::list_workflow_runs(&client, query).await
}

/// Search PRs by title/branch (for MCP/HTTP)
#[allow(dead_code)]
pub async fn search_prs(owner: &str, repo: &str, query: &str) -> Result<Vec<PullRequest>> {
    let client = service::create_client()?;
    service::search_prs(&client, owner, repo, query).await
}

/// Find PR number for a branch (for MCP/HTTP)
#[allow(dead_code)]
pub async fn find_pr_for_branch(owner: &str, repo: &str, branch: &str) -> Result<Option<u64>> {
    let client = service::create_client()?;
    service::find_pr_for_branch(&client, owner, repo, branch).await
}

/// Get failed jobs for a workflow run (for MCP/HTTP)
#[allow(dead_code)]
pub async fn get_failed_jobs(owner: &str, repo: &str, run_id: u64) -> Result<Vec<(u64, String)>> {
    let client = service::create_client()?;
    service::get_failed_jobs(&client, owner, repo, run_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gh_command_exported() {
        // Verify GhCommand is accessible
        let _ = std::any::type_name::<GhCommand>();
    }
}
