use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum GhCommand {
    /// List pull requests
    Prs,
    /// List workflow runs
    Runs,
    /// Show CI failures
    Failures,
    /// Check CI status
    Ci,
}
