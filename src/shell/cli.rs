use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ShellCommand {
    /// List directory contents (GNU ls passthrough with icons)
    Ls(LsArgs),
    /// Show disk filesystem usage
    Df(DfArgs),
}

#[derive(Debug, Args)]
pub struct LsArgs {
    /// Arguments passed through to GNU ls
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    pub args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct DfArgs {
    /// Output as JSON
    #[arg(short, long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        cmd: ShellCommand,
    }

    #[test]
    fn parse_ls_no_args() {
        let cli = TestCli::try_parse_from(["test", "ls"]).unwrap();
        match cli.cmd {
            ShellCommand::Ls(args) => {
                assert!(args.args.is_empty());
            }
            _ => panic!("Expected Ls command"),
        }
    }

    #[test]
    fn parse_ls_with_flags() {
        let cli = TestCli::try_parse_from(["test", "ls", "-la"]).unwrap();
        match cli.cmd {
            ShellCommand::Ls(args) => {
                assert_eq!(args.args, vec!["-la"]);
            }
            _ => panic!("Expected Ls command"),
        }
    }

    #[test]
    fn parse_ls_with_path() {
        let cli = TestCli::try_parse_from(["test", "ls", "/tmp"]).unwrap();
        match cli.cmd {
            ShellCommand::Ls(args) => {
                assert_eq!(args.args, vec!["/tmp"]);
            }
            _ => panic!("Expected Ls command"),
        }
    }

    #[test]
    fn parse_ls_mixed_args() {
        let cli = TestCli::try_parse_from(["test", "ls", "-la", "--color=never", "/home"]).unwrap();
        match cli.cmd {
            ShellCommand::Ls(args) => {
                assert_eq!(args.args, vec!["-la", "--color=never", "/home"]);
            }
            _ => panic!("Expected Ls command"),
        }
    }

    #[test]
    fn parse_ls_long_flags() {
        let cli = TestCli::try_parse_from(["test", "ls", "--all", "--sort=size"]).unwrap();
        match cli.cmd {
            ShellCommand::Ls(args) => {
                assert_eq!(args.args, vec!["--all", "--sort=size"]);
            }
            _ => panic!("Expected Ls command"),
        }
    }

    #[test]
    fn parse_df_default() {
        let cli = TestCli::try_parse_from(["test", "df"]).unwrap();
        match cli.cmd {
            ShellCommand::Df(args) => {
                assert!(!args.json);
            }
            _ => panic!("Expected Df command"),
        }
    }

    #[test]
    fn parse_df_json() {
        let cli = TestCli::try_parse_from(["test", "df", "--json"]).unwrap();
        match cli.cmd {
            ShellCommand::Df(args) => {
                assert!(args.json);
            }
            _ => panic!("Expected Df command"),
        }
    }

    #[test]
    fn parse_df_json_short() {
        let cli = TestCli::try_parse_from(["test", "df", "-j"]).unwrap();
        match cli.cmd {
            ShellCommand::Df(args) => {
                assert!(args.json);
            }
            _ => panic!("Expected Df command"),
        }
    }
}
