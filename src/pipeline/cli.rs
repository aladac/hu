//! Pipeline CLI commands

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum PipelineCommand {
    /// List all pipelines
    List {
        /// AWS region
        #[arg(short, long)]
        region: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show pipeline status (stages and actions)
    Status {
        /// Pipeline name
        name: String,

        /// AWS region
        #[arg(short, long)]
        region: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show pipeline execution history
    History {
        /// Pipeline name
        name: String,

        /// AWS region
        #[arg(short, long)]
        region: Option<String>,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

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
        cmd: PipelineCommand,
    }

    #[test]
    fn parses_list_basic() {
        let cli = TestCli::try_parse_from(["test", "list"]).unwrap();
        match cli.cmd {
            PipelineCommand::List { region, json } => {
                assert!(region.is_none());
                assert!(!json);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_list_with_region() {
        let cli = TestCli::try_parse_from(["test", "list", "-r", "us-west-2"]).unwrap();
        match cli.cmd {
            PipelineCommand::List { region, .. } => {
                assert_eq!(region, Some("us-west-2".to_string()));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_list_json() {
        let cli = TestCli::try_parse_from(["test", "list", "--json"]).unwrap();
        match cli.cmd {
            PipelineCommand::List { json, .. } => {
                assert!(json);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_status_basic() {
        let cli = TestCli::try_parse_from(["test", "status", "my-pipeline"]).unwrap();
        match cli.cmd {
            PipelineCommand::Status { name, region, json } => {
                assert_eq!(name, "my-pipeline");
                assert!(region.is_none());
                assert!(!json);
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn parses_status_with_region() {
        let cli =
            TestCli::try_parse_from(["test", "status", "my-pipeline", "-r", "eu-west-1"]).unwrap();
        match cli.cmd {
            PipelineCommand::Status { region, .. } => {
                assert_eq!(region, Some("eu-west-1".to_string()));
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn parses_status_json() {
        let cli = TestCli::try_parse_from(["test", "status", "my-pipeline", "--json"]).unwrap();
        match cli.cmd {
            PipelineCommand::Status { json, .. } => {
                assert!(json);
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn parses_history_basic() {
        let cli = TestCli::try_parse_from(["test", "history", "my-pipeline"]).unwrap();
        match cli.cmd {
            PipelineCommand::History {
                name, limit, json, ..
            } => {
                assert_eq!(name, "my-pipeline");
                assert_eq!(limit, 10); // default
                assert!(!json);
            }
            _ => panic!("Expected History command"),
        }
    }

    #[test]
    fn parses_history_with_limit() {
        let cli = TestCli::try_parse_from(["test", "history", "my-pipeline", "-l", "25"]).unwrap();
        match cli.cmd {
            PipelineCommand::History { limit, .. } => {
                assert_eq!(limit, 25);
            }
            _ => panic!("Expected History command"),
        }
    }

    #[test]
    fn parses_history_with_region() {
        let cli = TestCli::try_parse_from(["test", "history", "my-pipeline", "-r", "ap-south-1"])
            .unwrap();
        match cli.cmd {
            PipelineCommand::History { region, .. } => {
                assert_eq!(region, Some("ap-south-1".to_string()));
            }
            _ => panic!("Expected History command"),
        }
    }

    #[test]
    fn parses_history_json() {
        let cli = TestCli::try_parse_from(["test", "history", "my-pipeline", "--json"]).unwrap();
        match cli.cmd {
            PipelineCommand::History { json, .. } => {
                assert!(json);
            }
            _ => panic!("Expected History command"),
        }
    }

    #[test]
    fn command_debug() {
        let cmd = PipelineCommand::List {
            region: None,
            json: false,
        };
        let debug = format!("{:?}", cmd);
        assert!(debug.contains("List"));
    }

    #[test]
    fn command_has_help() {
        let mut cmd = TestCli::command();
        let help = cmd.render_help();
        assert!(!help.to_string().is_empty());
    }
}
