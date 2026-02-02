//! EKS CLI commands

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum EksCommand {
    /// List pods in the cluster
    List {
        /// Namespace to list pods from
        #[arg(short, long)]
        namespace: Option<String>,

        /// List pods from all namespaces
        #[arg(short = 'A', long)]
        all_namespaces: bool,

        /// Kubeconfig context to use
        #[arg(short, long)]
        context: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Execute a command in a pod (interactive shell by default)
    Exec {
        /// Pod name
        pod: String,

        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,

        /// Container name (if pod has multiple containers)
        #[arg(short, long)]
        container: Option<String>,

        /// Kubeconfig context to use
        #[arg(long)]
        context: Option<String>,

        /// Command to run (default: /bin/sh)
        #[arg(last = true)]
        command: Vec<String>,
    },

    /// Tail logs from a pod
    Logs {
        /// Pod name
        pod: String,

        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,

        /// Container name (if pod has multiple containers)
        #[arg(short, long)]
        container: Option<String>,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Show logs from previous container instance
        #[arg(long)]
        previous: bool,

        /// Number of lines to show from the end
        #[arg(long)]
        tail: Option<usize>,

        /// Kubeconfig context to use
        #[arg(long)]
        context: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        cmd: EksCommand,
    }

    #[test]
    fn parses_list_basic() {
        let cli = TestCli::try_parse_from(["test", "list"]).unwrap();
        match cli.cmd {
            EksCommand::List {
                namespace,
                all_namespaces,
                context,
                json,
            } => {
                assert!(namespace.is_none());
                assert!(!all_namespaces);
                assert!(context.is_none());
                assert!(!json);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_list_with_namespace() {
        let cli = TestCli::try_parse_from(["test", "list", "-n", "kube-system"]).unwrap();
        match cli.cmd {
            EksCommand::List { namespace, .. } => {
                assert_eq!(namespace, Some("kube-system".to_string()));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_list_all_namespaces() {
        let cli = TestCli::try_parse_from(["test", "list", "-A"]).unwrap();
        match cli.cmd {
            EksCommand::List { all_namespaces, .. } => {
                assert!(all_namespaces);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_list_with_context() {
        let cli = TestCli::try_parse_from(["test", "list", "-c", "prod"]).unwrap();
        match cli.cmd {
            EksCommand::List { context, .. } => {
                assert_eq!(context, Some("prod".to_string()));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_list_json() {
        let cli = TestCli::try_parse_from(["test", "list", "--json"]).unwrap();
        match cli.cmd {
            EksCommand::List { json, .. } => {
                assert!(json);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn parses_exec_basic() {
        let cli = TestCli::try_parse_from(["test", "exec", "my-pod"]).unwrap();
        match cli.cmd {
            EksCommand::Exec {
                pod,
                namespace,
                container,
                command,
                ..
            } => {
                assert_eq!(pod, "my-pod");
                assert!(namespace.is_none());
                assert!(container.is_none());
                assert!(command.is_empty());
            }
            _ => panic!("Expected Exec command"),
        }
    }

    #[test]
    fn parses_exec_with_namespace() {
        let cli = TestCli::try_parse_from(["test", "exec", "my-pod", "-n", "prod"]).unwrap();
        match cli.cmd {
            EksCommand::Exec { namespace, .. } => {
                assert_eq!(namespace, Some("prod".to_string()));
            }
            _ => panic!("Expected Exec command"),
        }
    }

    #[test]
    fn parses_exec_with_container() {
        let cli = TestCli::try_parse_from(["test", "exec", "my-pod", "-c", "app"]).unwrap();
        match cli.cmd {
            EksCommand::Exec { container, .. } => {
                assert_eq!(container, Some("app".to_string()));
            }
            _ => panic!("Expected Exec command"),
        }
    }

    #[test]
    fn parses_exec_with_command() {
        let cli =
            TestCli::try_parse_from(["test", "exec", "my-pod", "--", "bash", "-c", "ls"]).unwrap();
        match cli.cmd {
            EksCommand::Exec { command, .. } => {
                assert_eq!(command, vec!["bash", "-c", "ls"]);
            }
            _ => panic!("Expected Exec command"),
        }
    }

    #[test]
    fn parses_logs_basic() {
        let cli = TestCli::try_parse_from(["test", "logs", "my-pod"]).unwrap();
        match cli.cmd {
            EksCommand::Logs {
                pod,
                follow,
                previous,
                tail,
                ..
            } => {
                assert_eq!(pod, "my-pod");
                assert!(!follow);
                assert!(!previous);
                assert!(tail.is_none());
            }
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn parses_logs_follow() {
        let cli = TestCli::try_parse_from(["test", "logs", "my-pod", "-f"]).unwrap();
        match cli.cmd {
            EksCommand::Logs { follow, .. } => {
                assert!(follow);
            }
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn parses_logs_previous() {
        let cli = TestCli::try_parse_from(["test", "logs", "my-pod", "--previous"]).unwrap();
        match cli.cmd {
            EksCommand::Logs { previous, .. } => {
                assert!(previous);
            }
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn parses_logs_tail() {
        let cli = TestCli::try_parse_from(["test", "logs", "my-pod", "--tail", "100"]).unwrap();
        match cli.cmd {
            EksCommand::Logs { tail, .. } => {
                assert_eq!(tail, Some(100));
            }
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn parses_logs_with_container() {
        let cli = TestCli::try_parse_from(["test", "logs", "my-pod", "-c", "sidecar"]).unwrap();
        match cli.cmd {
            EksCommand::Logs { container, .. } => {
                assert_eq!(container, Some("sidecar".to_string()));
            }
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn command_debug() {
        let cmd = EksCommand::List {
            namespace: None,
            all_namespaces: false,
            context: None,
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
