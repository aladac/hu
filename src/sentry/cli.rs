use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum SentryCommand {
    /// List unresolved issues
    Issues,
    /// Show issue details
    Show {
        /// Issue ID
        id: String,
    },
}
