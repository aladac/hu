use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ContextCommand {
    /// Track a file as loaded in context
    Track(TrackArgs),
    /// Check if a file is in context
    Check(CheckArgs),
    /// Show summary of all tracked files
    Summary,
    /// Clear all tracked files
    Clear,
}

#[derive(Debug, Args)]
pub struct TrackArgs {
    /// File path(s) to track
    #[arg(required = true)]
    pub paths: Vec<String>,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// File path(s) to check
    #[arg(required = true)]
    pub paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        cmd: ContextCommand,
    }

    #[test]
    fn parse_track_single() {
        let cli = TestCli::try_parse_from(["test", "track", "file.rs"]).unwrap();
        if let ContextCommand::Track(args) = cli.cmd {
            assert_eq!(args.paths, vec!["file.rs"]);
        } else {
            panic!("Expected Track");
        }
    }

    #[test]
    fn parse_track_multiple() {
        let cli = TestCli::try_parse_from(["test", "track", "a.rs", "b.rs", "c.rs"]).unwrap();
        if let ContextCommand::Track(args) = cli.cmd {
            assert_eq!(args.paths, vec!["a.rs", "b.rs", "c.rs"]);
        } else {
            panic!("Expected Track");
        }
    }

    #[test]
    fn parse_check_single() {
        let cli = TestCli::try_parse_from(["test", "check", "file.rs"]).unwrap();
        if let ContextCommand::Check(args) = cli.cmd {
            assert_eq!(args.paths, vec!["file.rs"]);
        } else {
            panic!("Expected Check");
        }
    }

    #[test]
    fn parse_check_multiple() {
        let cli = TestCli::try_parse_from(["test", "check", "a.rs", "b.rs"]).unwrap();
        if let ContextCommand::Check(args) = cli.cmd {
            assert_eq!(args.paths, vec!["a.rs", "b.rs"]);
        } else {
            panic!("Expected Check");
        }
    }

    #[test]
    fn parse_summary() {
        let cli = TestCli::try_parse_from(["test", "summary"]).unwrap();
        assert!(matches!(cli.cmd, ContextCommand::Summary));
    }

    #[test]
    fn parse_clear() {
        let cli = TestCli::try_parse_from(["test", "clear"]).unwrap();
        assert!(matches!(cli.cmd, ContextCommand::Clear));
    }

    #[test]
    fn track_requires_path() {
        let result = TestCli::try_parse_from(["test", "track"]);
        assert!(result.is_err());
    }

    #[test]
    fn check_requires_path() {
        let result = TestCli::try_parse_from(["test", "check"]);
        assert!(result.is_err());
    }

    #[test]
    fn track_args_debug() {
        let args = TrackArgs {
            paths: vec!["a.rs".to_string()],
        };
        let debug = format!("{:?}", args);
        assert!(debug.contains("TrackArgs"));
    }

    #[test]
    fn check_args_debug() {
        let args = CheckArgs {
            paths: vec!["a.rs".to_string()],
        };
        let debug = format!("{:?}", args);
        assert!(debug.contains("CheckArgs"));
    }

    #[test]
    fn context_command_debug() {
        let cmd = ContextCommand::Summary;
        let debug = format!("{:?}", cmd);
        assert!(debug.contains("Summary"));
    }
}
