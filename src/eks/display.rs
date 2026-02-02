//! EKS output formatting

use anyhow::{Context, Result};
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};

use super::types::{OutputFormat, Pod};

/// Get color for pod status
fn status_color(status: &str) -> Color {
    match status {
        "Running" => Color::Green,
        "Pending" => Color::Yellow,
        "Succeeded" => Color::Cyan,
        "Failed" => Color::Red,
        "Unknown" => Color::DarkGrey,
        _ => Color::White,
    }
}

/// Output pods list
pub fn output_pods(pods: &[Pod], format: OutputFormat, show_namespace: bool) -> Result<()> {
    match format {
        OutputFormat::Table => {
            if pods.is_empty() {
                println!("No pods found.");
                return Ok(());
            }

            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_content_arrangement(ContentArrangement::Dynamic);

            if show_namespace {
                table.set_header(vec![
                    "NAMESPACE",
                    "NAME",
                    "READY",
                    "STATUS",
                    "RESTARTS",
                    "AGE",
                ]);
            } else {
                table.set_header(vec!["NAME", "READY", "STATUS", "RESTARTS", "AGE"]);
            }

            for pod in pods {
                if show_namespace {
                    table.add_row(vec![
                        Cell::new(&pod.namespace),
                        Cell::new(&pod.name).fg(Color::Cyan),
                        Cell::new(&pod.ready),
                        Cell::new(&pod.status).fg(status_color(&pod.status)),
                        Cell::new(pod.restarts.to_string()),
                        Cell::new(&pod.age),
                    ]);
                } else {
                    table.add_row(vec![
                        Cell::new(&pod.name).fg(Color::Cyan),
                        Cell::new(&pod.ready),
                        Cell::new(&pod.status).fg(status_color(&pod.status)),
                        Cell::new(pod.restarts.to_string()),
                        Cell::new(&pod.age),
                    ]);
                }
            }

            println!("{table}");
            println!("\n{} pods", pods.len());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(pods).context("Failed to serialize pods")?;
            println!("{json}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_color_running() {
        assert_eq!(status_color("Running"), Color::Green);
    }

    #[test]
    fn status_color_pending() {
        assert_eq!(status_color("Pending"), Color::Yellow);
    }

    #[test]
    fn status_color_succeeded() {
        assert_eq!(status_color("Succeeded"), Color::Cyan);
    }

    #[test]
    fn status_color_failed() {
        assert_eq!(status_color("Failed"), Color::Red);
    }

    #[test]
    fn status_color_unknown() {
        assert_eq!(status_color("Unknown"), Color::DarkGrey);
    }

    #[test]
    fn status_color_other() {
        assert_eq!(status_color("CrashLoopBackOff"), Color::White);
    }

    #[test]
    fn output_pods_empty() {
        let result = output_pods(&[], OutputFormat::Table, false);
        assert!(result.is_ok());
    }

    #[test]
    fn output_pods_table() {
        let pods = vec![Pod {
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            ready: "1/1".to_string(),
            restarts: 0,
            age: "1d".to_string(),
            node: None,
        }];
        let result = output_pods(&pods, OutputFormat::Table, false);
        assert!(result.is_ok());
    }

    #[test]
    fn output_pods_table_with_namespace() {
        let pods = vec![Pod {
            name: "test-pod".to_string(),
            namespace: "kube-system".to_string(),
            status: "Running".to_string(),
            ready: "1/1".to_string(),
            restarts: 0,
            age: "1d".to_string(),
            node: None,
        }];
        let result = output_pods(&pods, OutputFormat::Table, true);
        assert!(result.is_ok());
    }

    #[test]
    fn output_pods_json() {
        let pods = vec![Pod {
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            ready: "1/1".to_string(),
            restarts: 0,
            age: "1d".to_string(),
            node: None,
        }];
        let result = output_pods(&pods, OutputFormat::Json, false);
        assert!(result.is_ok());
    }

    #[test]
    fn output_pods_json_empty() {
        let result = output_pods(&[], OutputFormat::Json, false);
        assert!(result.is_ok());
    }
}
