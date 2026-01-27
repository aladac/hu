use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum JiraCommand {
    /// List tickets in current sprint
    Tickets,
    /// Show current sprint info
    Sprint,
    /// Search tickets
    Search {
        /// Search query
        query: String,
    },
    /// Show ticket details
    Show {
        /// Ticket key (e.g., PROJ-123)
        key: String,
    },
}
