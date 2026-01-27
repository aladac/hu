use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum PagerDutyCommand {
    /// Show oncall status
    Oncall,
    /// List active alerts
    Alerts,
    /// List incidents
    Incidents,
}
