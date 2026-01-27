use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum DashboardCommand {
    /// Show full dashboard
    Show,
    /// Refresh dashboard data
    Refresh,
}
