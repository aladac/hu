use anyhow::Result;

use super::cli::FailuresArgs;
use super::client::{parse_test_failures, GithubApi, GithubClient};
use super::helpers::{get_current_repo, is_test_job, parse_owner_repo};

#[cfg(test)]
mod tests;

/// Handle the `hu gh failures` command
#[cfg(not(tarpaulin_include))]
pub async fn run(args: FailuresArgs) -> Result<()> {
    let client = GithubClient::new()?;

    // Get repo info from args or current directory
    let (owner, repo) = if let Some(repo_arg) = &args.repo {
        parse_owner_repo(repo_arg)?
    } else {
        get_current_repo()?
    };

    // If PR specified, use PR-based flow; otherwise get latest repo failures
    if let Some(pr_number) = args.pr {
        process_pr_failures(&client, &owner, &repo, pr_number).await
    } else {
        process_repo_failures(&client, &owner, &repo).await
    }
}

/// Process failures for a specific PR (testable)
pub async fn process_pr_failures(
    client: &impl GithubApi,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<()> {
    eprintln!(
        "Fetching failures for PR #{} in {}/{}...",
        pr_number, owner, repo
    );

    // Get the PR's branch name
    let branch = client.get_pr_branch(owner, repo, pr_number).await?;

    // Get the latest failed workflow run for this branch
    let run_id = client
        .get_latest_failed_run_for_branch(owner, repo, &branch)
        .await?;

    let run_id = match run_id {
        Some(id) => id,
        None => {
            println!("No failed workflow runs found for PR #{}.", pr_number);
            return Ok(());
        }
    };

    process_run_failures(client, owner, repo, run_id).await
}

/// Process failures for the latest failed run in the repo (testable)
pub async fn process_repo_failures(client: &impl GithubApi, owner: &str, repo: &str) -> Result<()> {
    eprintln!("Fetching latest failures in {}/{}...", owner, repo);

    // Get the latest failed workflow run for the repo
    let run_id = client.get_latest_failed_run(owner, repo).await?;

    let run_id = match run_id {
        Some(id) => id,
        None => {
            println!("No failed workflow runs found in {}/{}.", owner, repo);
            return Ok(());
        }
    };

    process_run_failures(client, owner, repo, run_id).await
}

/// Process failures for a specific workflow run (shared logic)
async fn process_run_failures(
    client: &impl GithubApi,
    owner: &str,
    repo: &str,
    run_id: u64,
) -> Result<()> {
    // Get failed jobs in that run
    let failed_jobs = client.get_failed_jobs(owner, repo, run_id).await?;

    if failed_jobs.is_empty() {
        println!("No failed jobs found in run {}.", run_id);
        return Ok(());
    }

    // Only process test-related jobs (rspec, jest, etc.)
    let test_jobs: Vec<_> = failed_jobs
        .into_iter()
        .filter(|(_, name)| is_test_job(name))
        .collect();

    if test_jobs.is_empty() {
        println!("No test-related job failures found.");
        return Ok(());
    }

    let mut all_failures = Vec::new();

    for (job_id, job_name) in test_jobs {
        eprintln!("Fetching logs for job: {}", job_name);

        match client.get_job_logs(owner, repo, job_id).await {
            Ok(logs) => {
                let failures = parse_test_failures(&logs);
                all_failures.extend(failures);
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch logs for {}: {}", job_name, e);
            }
        }
    }

    if all_failures.is_empty() {
        println!("No test failures found in logs.");
        return Ok(());
    }

    // Output in a format useful for Claude
    println!("\n# Test Failures\n");
    for failure in &all_failures {
        println!("## {}\n", failure.spec_file);
        println!("```");
        println!("{}", failure.failure_text);
        println!("```\n");
    }

    // Also output the rspec commands to rerun
    println!("# Rerun Commands\n");
    println!("```bash");
    for failure in &all_failures {
        println!("bundle exec rspec {}", failure.spec_file);
    }
    println!("```");

    Ok(())
}
