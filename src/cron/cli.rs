use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum CronCommand {
    /// Add a scheduled job
    Add(AddArgs),
    /// List all cron jobs
    List(ListArgs),
    /// Remove a cron job
    Remove(RemoveArgs),
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Schedule: hourly, daily, weekly, monthly, reboot
    pub schedule: String,
    /// Command to run
    pub command: String,
    /// Output as JSON
    #[arg(long, short)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Show only hu-managed jobs
    #[arg(long)]
    pub hu_only: bool,
    /// Output as JSON
    #[arg(long, short)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Pattern to match (searches in command)
    pub pattern: String,
    /// Remove all matching jobs without confirmation
    #[arg(long, short)]
    pub force: bool,
    /// Output as JSON
    #[arg(long, short)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        cmd: CronCommand,
    }

    #[test]
    fn parse_add() {
        let cli = TestCli::try_parse_from(["test", "add", "daily", "hu gh sync ~/docs"]).unwrap();
        match cli.cmd {
            CronCommand::Add(args) => {
                assert_eq!(args.schedule, "daily");
                assert_eq!(args.command, "hu gh sync ~/docs");
                assert!(!args.json);
            }
            _ => panic!("expected Add"),
        }
    }

    #[test]
    fn parse_add_with_json() {
        let cli =
            TestCli::try_parse_from(["test", "add", "hourly", "echo test", "--json"]).unwrap();
        match cli.cmd {
            CronCommand::Add(args) => {
                assert!(args.json);
            }
            _ => panic!("expected Add"),
        }
    }

    #[test]
    fn parse_list() {
        let cli = TestCli::try_parse_from(["test", "list"]).unwrap();
        match cli.cmd {
            CronCommand::List(args) => {
                assert!(!args.hu_only);
                assert!(!args.json);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_list_hu_only() {
        let cli = TestCli::try_parse_from(["test", "list", "--hu-only"]).unwrap();
        match cli.cmd {
            CronCommand::List(args) => {
                assert!(args.hu_only);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_list_json() {
        let cli = TestCli::try_parse_from(["test", "list", "--json"]).unwrap();
        match cli.cmd {
            CronCommand::List(args) => {
                assert!(args.json);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_remove() {
        let cli = TestCli::try_parse_from(["test", "remove", "gh sync"]).unwrap();
        match cli.cmd {
            CronCommand::Remove(args) => {
                assert_eq!(args.pattern, "gh sync");
                assert!(!args.force);
            }
            _ => panic!("expected Remove"),
        }
    }

    #[test]
    fn parse_remove_force() {
        let cli = TestCli::try_parse_from(["test", "remove", "pattern", "--force"]).unwrap();
        match cli.cmd {
            CronCommand::Remove(args) => {
                assert!(args.force);
            }
            _ => panic!("expected Remove"),
        }
    }

    #[test]
    fn add_args_debug() {
        let args = AddArgs {
            schedule: "daily".to_string(),
            command: "test".to_string(),
            json: false,
        };
        let debug = format!("{:?}", args);
        assert!(debug.contains("daily"));
    }

    #[test]
    fn list_args_debug() {
        let args = ListArgs {
            hu_only: true,
            json: false,
        };
        let debug = format!("{:?}", args);
        assert!(debug.contains("hu_only: true"));
    }

    #[test]
    fn remove_args_debug() {
        let args = RemoveArgs {
            pattern: "test".to_string(),
            force: true,
            json: false,
        };
        let debug = format!("{:?}", args);
        assert!(debug.contains("force: true"));
    }
}
