//! PagerDuty CLI commands

use clap::{Subcommand, ValueEnum};

/// Incident status filter
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StatusFilter {
    /// Only triggered incidents
    Triggered,
    /// Only acknowledged incidents
    Acknowledged,
    /// Only resolved incidents
    Resolved,
    /// Triggered and acknowledged (active)
    Active,
}

#[derive(Debug, Subcommand)]
pub enum PagerDutyCommand {
    /// Show configuration status
    Config,

    /// Set API token
    Auth {
        /// PagerDuty API token
        token: String,
    },

    /// Show who's currently on call
    Oncall {
        /// Filter by escalation policy ID
        #[arg(short = 'p', long)]
        policy: Option<String>,

        /// Filter by schedule ID
        #[arg(short, long)]
        schedule: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List active alerts (triggered + acknowledged incidents)
    Alerts {
        /// Maximum number to show
        #[arg(short, long, default_value = "25")]
        limit: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List incidents with filters
    Incidents {
        /// Filter by status
        #[arg(short, long, value_enum)]
        status: Option<StatusFilter>,

        /// Maximum number to show
        #[arg(short, long, default_value = "25")]
        limit: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show incident details
    Show {
        /// Incident ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show current user info
    Whoami {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        cmd: PagerDutyCommand,
    }

    #[test]
    fn parses_config() {
        let cli = TestCli::try_parse_from(["test", "config"]).unwrap();
        assert!(matches!(cli.cmd, PagerDutyCommand::Config));
    }

    #[test]
    fn parses_auth() {
        let cli = TestCli::try_parse_from(["test", "auth", "my-token"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Auth { token } => assert_eq!(token, "my-token"),
            _ => panic!("Expected Auth command"),
        }
    }

    #[test]
    fn parses_oncall_no_args() {
        let cli = TestCli::try_parse_from(["test", "oncall"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Oncall {
                policy,
                schedule,
                json,
            } => {
                assert!(policy.is_none());
                assert!(schedule.is_none());
                assert!(!json);
            }
            _ => panic!("Expected Oncall command"),
        }
    }

    #[test]
    fn parses_oncall_with_policy() {
        let cli = TestCli::try_parse_from(["test", "oncall", "-p", "EP123"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Oncall { policy, .. } => {
                assert_eq!(policy, Some("EP123".to_string()));
            }
            _ => panic!("Expected Oncall command"),
        }
    }

    #[test]
    fn parses_oncall_with_schedule() {
        let cli = TestCli::try_parse_from(["test", "oncall", "--schedule", "S456"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Oncall { schedule, .. } => {
                assert_eq!(schedule, Some("S456".to_string()));
            }
            _ => panic!("Expected Oncall command"),
        }
    }

    #[test]
    fn parses_oncall_json() {
        let cli = TestCli::try_parse_from(["test", "oncall", "--json"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Oncall { json, .. } => assert!(json),
            _ => panic!("Expected Oncall command"),
        }
    }

    #[test]
    fn parses_alerts_default_limit() {
        let cli = TestCli::try_parse_from(["test", "alerts"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Alerts { limit, json } => {
                assert_eq!(limit, 25);
                assert!(!json);
            }
            _ => panic!("Expected Alerts command"),
        }
    }

    #[test]
    fn parses_alerts_custom_limit() {
        let cli = TestCli::try_parse_from(["test", "alerts", "-l", "50"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Alerts { limit, .. } => assert_eq!(limit, 50),
            _ => panic!("Expected Alerts command"),
        }
    }

    #[test]
    fn parses_incidents_no_filter() {
        let cli = TestCli::try_parse_from(["test", "incidents"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Incidents {
                status,
                limit,
                json,
            } => {
                assert!(status.is_none());
                assert_eq!(limit, 25);
                assert!(!json);
            }
            _ => panic!("Expected Incidents command"),
        }
    }

    #[test]
    fn parses_incidents_status_triggered() {
        let cli = TestCli::try_parse_from(["test", "incidents", "-s", "triggered"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Incidents { status, .. } => {
                assert!(matches!(status, Some(StatusFilter::Triggered)));
            }
            _ => panic!("Expected Incidents command"),
        }
    }

    #[test]
    fn parses_incidents_status_acknowledged() {
        let cli =
            TestCli::try_parse_from(["test", "incidents", "--status", "acknowledged"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Incidents { status, .. } => {
                assert!(matches!(status, Some(StatusFilter::Acknowledged)));
            }
            _ => panic!("Expected Incidents command"),
        }
    }

    #[test]
    fn parses_incidents_status_resolved() {
        let cli = TestCli::try_parse_from(["test", "incidents", "-s", "resolved"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Incidents { status, .. } => {
                assert!(matches!(status, Some(StatusFilter::Resolved)));
            }
            _ => panic!("Expected Incidents command"),
        }
    }

    #[test]
    fn parses_incidents_status_active() {
        let cli = TestCli::try_parse_from(["test", "incidents", "-s", "active"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Incidents { status, .. } => {
                assert!(matches!(status, Some(StatusFilter::Active)));
            }
            _ => panic!("Expected Incidents command"),
        }
    }

    #[test]
    fn parses_show() {
        let cli = TestCli::try_parse_from(["test", "show", "INC123"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Show { id, json } => {
                assert_eq!(id, "INC123");
                assert!(!json);
            }
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn parses_show_json() {
        let cli = TestCli::try_parse_from(["test", "show", "INC123", "--json"]).unwrap();
        match cli.cmd {
            PagerDutyCommand::Show { id, json } => {
                assert_eq!(id, "INC123");
                assert!(json);
            }
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn status_filter_debug() {
        let filter = StatusFilter::Triggered;
        let debug = format!("{:?}", filter);
        assert!(debug.contains("Triggered"));
    }

    #[test]
    fn status_filter_clone() {
        let filter = StatusFilter::Active;
        let cloned = filter;
        assert!(matches!(cloned, StatusFilter::Active));
    }

    #[test]
    fn command_debug() {
        let cmd = PagerDutyCommand::Config;
        let debug = format!("{:?}", cmd);
        assert!(debug.contains("Config"));
    }

    #[test]
    fn command_has_help() {
        // Verify help text is generated without panic
        let mut cmd = TestCli::command();
        let help = cmd.render_help();
        assert!(!help.to_string().is_empty());
    }
}
