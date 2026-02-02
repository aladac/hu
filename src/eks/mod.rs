//! EKS pod management
//!
//! List pods, exec into pods, and tail logs.

mod cli;
mod display;
mod kubectl;
mod types;

use anyhow::Result;

pub use cli::EksCommand;
use types::{KubectlConfig, OutputFormat};

/// Run an EKS command
pub async fn run(cmd: EksCommand) -> Result<()> {
    match cmd {
        EksCommand::List {
            namespace,
            all_namespaces,
            context,
            json,
        } => cmd_list(namespace, all_namespaces, context, json),
        EksCommand::Exec {
            pod,
            namespace,
            container,
            context,
            command,
        } => cmd_exec(&pod, namespace, container, context, command),
        EksCommand::Logs {
            pod,
            namespace,
            container,
            follow,
            previous,
            tail,
            context,
        } => cmd_logs(&pod, namespace, container, follow, previous, tail, context),
    }
}

/// List pods
fn cmd_list(
    namespace: Option<String>,
    all_namespaces: bool,
    context: Option<String>,
    json: bool,
) -> Result<()> {
    let config = KubectlConfig {
        context,
        namespace: namespace.clone(),
    };

    let pods = kubectl::list_pods(&config, all_namespaces)?;

    let format = if json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    // Show namespace column if listing all namespaces or no specific namespace
    let show_namespace = all_namespaces || namespace.is_none();
    display::output_pods(&pods, format, show_namespace)?;

    Ok(())
}

/// Exec into a pod
fn cmd_exec(
    pod: &str,
    namespace: Option<String>,
    container: Option<String>,
    context: Option<String>,
    command: Vec<String>,
) -> Result<()> {
    let config = KubectlConfig { context, namespace };

    kubectl::exec_pod(&config, pod, container.as_deref(), &command)
}

/// Tail logs from a pod
#[allow(clippy::too_many_arguments)]
fn cmd_logs(
    pod: &str,
    namespace: Option<String>,
    container: Option<String>,
    follow: bool,
    previous: bool,
    tail: Option<usize>,
    context: Option<String>,
) -> Result<()> {
    let config = KubectlConfig { context, namespace };

    kubectl::tail_logs(&config, pod, container.as_deref(), follow, previous, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kubectl_config_from_options() {
        let config = KubectlConfig {
            context: Some("prod".to_string()),
            namespace: Some("default".to_string()),
        };
        assert_eq!(config.context, Some("prod".to_string()));
        assert_eq!(config.namespace, Some("default".to_string()));
    }
}
