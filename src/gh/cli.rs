use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum GhCommand {
    /// Authenticate with GitHub using a Personal Access Token
    Login(LoginArgs),
    /// List open pull requests authored by you
    Prs,
    /// List workflow runs
    Runs,
    /// Show CI failures
    Failures,
    /// Check CI status
    Ci,
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Personal Access Token (create at https://github.com/settings/tokens)
    #[arg(long, short)]
    pub token: String,
}
