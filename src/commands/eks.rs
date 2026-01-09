use anyhow::{bail, Context, Result};
use aws_sdk_eks::types::Cluster;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crate::config::Settings;
use crate::utils::{print_error, print_header, print_success, spinner, ANSI_COLORS};

// ==================== Kubeconfig ====================

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Kubeconfig {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    clusters: Vec<KubeconfigCluster>,
    contexts: Vec<KubeconfigContext>,
    #[serde(rename = "current-context")]
    current_context: String,
    users: Vec<KubeconfigUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    preferences: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KubeconfigCluster {
    name: String,
    cluster: ClusterData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClusterData {
    #[serde(rename = "certificate-authority-data")]
    certificate_authority_data: String,
    server: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KubeconfigContext {
    name: String,
    context: ContextData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ContextData {
    cluster: String,
    user: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KubeconfigUser {
    name: String,
    user: UserData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserData {
    exec: ExecConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExecConfig {
    #[serde(rename = "apiVersion")]
    api_version: String,
    command: String,
    args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    env: Option<Vec<HashMap<String, String>>>,
    #[serde(rename = "interactiveMode", default, skip_serializing_if = "Option::is_none")]
    interactive_mode: Option<String>,
    #[serde(rename = "provideClusterInfo", default, skip_serializing_if = "Option::is_none")]
    provide_cluster_info: Option<bool>,
}

fn get_kubeconfig_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home).join(".kube").join("config"))
}

fn load_kubeconfig() -> Result<Kubeconfig> {
    let path = get_kubeconfig_path()?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read kubeconfig at {:?}", path))?;
        serde_yaml::from_str(&content).context("Failed to parse kubeconfig YAML")
    } else {
        Ok(Kubeconfig {
            api_version: "v1".to_string(),
            kind: "Config".to_string(),
            clusters: vec![],
            contexts: vec![],
            current_context: String::new(),
            users: vec![],
            preferences: None,
        })
    }
}

fn save_kubeconfig(config: &Kubeconfig) -> Result<()> {
    let path = get_kubeconfig_path()?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }

    let content = serde_yaml::to_string(config).context("Failed to serialize kubeconfig")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write kubeconfig to {:?}", path))?;
    Ok(())
}

async fn get_cluster_info(config: &aws_config::SdkConfig, cluster: &str) -> Result<Cluster> {
    let client = aws_sdk_eks::Client::new(config);
    let response = client
        .describe_cluster()
        .name(cluster)
        .send()
        .await
        .context("Failed to describe EKS cluster")?;

    response
        .cluster()
        .cloned()
        .context("No cluster info returned")
}

pub async fn update_kubeconfig(
    config: &aws_config::SdkConfig,
    cluster_name: &str,
    profile: Option<&str>,
    region: &str,
) -> Result<()> {
    let cluster = get_cluster_info(config, cluster_name).await?;

    let endpoint = cluster.endpoint().context("Cluster has no endpoint")?;
    let ca_data = cluster
        .certificate_authority()
        .and_then(|ca| ca.data())
        .context("Cluster has no CA data")?;
    let arn = cluster.arn().context("Cluster has no ARN")?;

    let mut kubeconfig = load_kubeconfig()?;

    let cluster_entry = KubeconfigCluster {
        name: arn.to_string(),
        cluster: ClusterData {
            certificate_authority_data: ca_data.to_string(),
            server: endpoint.to_string(),
        },
    };

    if let Some(existing) = kubeconfig.clusters.iter_mut().find(|c| c.name == arn) {
        *existing = cluster_entry;
    } else {
        kubeconfig.clusters.push(cluster_entry);
    }

    let mut exec_args = vec![
        "--region".to_string(),
        region.to_string(),
        "eks".to_string(),
        "get-token".to_string(),
        "--cluster-name".to_string(),
        cluster_name.to_string(),
        "--output".to_string(),
        "json".to_string(),
    ];

    if let Some(profile_name) = profile {
        exec_args.push("--profile".to_string());
        exec_args.push(profile_name.to_string());
    }

    let user_entry = KubeconfigUser {
        name: arn.to_string(),
        user: UserData {
            exec: ExecConfig {
                api_version: "client.authentication.k8s.io/v1beta1".to_string(),
                command: "aws".to_string(),
                args: exec_args,
                env: None,
                interactive_mode: Some("Never".to_string()),
                provide_cluster_info: None,
            },
        },
    };

    if let Some(existing) = kubeconfig.users.iter_mut().find(|u| u.name == arn) {
        *existing = user_entry;
    } else {
        kubeconfig.users.push(user_entry);
    }

    let context_entry = KubeconfigContext {
        name: arn.to_string(),
        context: ContextData {
            cluster: arn.to_string(),
            user: arn.to_string(),
        },
    };

    if let Some(existing) = kubeconfig.contexts.iter_mut().find(|c| c.name == arn) {
        *existing = context_entry;
    } else {
        kubeconfig.contexts.push(context_entry);
    }

    kubeconfig.current_context = arn.to_string();

    save_kubeconfig(&kubeconfig)?;
    Ok(())
}

// ==================== Pod Operations ====================

pub fn get_pods(namespace: &str, pattern: &str) -> Vec<String> {
    let output = crate::utils::run_cmd(&["kubectl", "get", "pod", "-n", namespace, "--no-headers"]);

    output
        .map(|s| {
            s.lines()
                .filter(|line| line.contains(pattern))
                .filter_map(|line| line.split_whitespace().next())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

pub fn display_pods(pods: &[String], env_name: &str, emoji: &str) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#").fg(Color::Cyan),
            Cell::new("Pod Name").fg(Color::Magenta),
            Cell::new("Short ID").fg(Color::DarkGrey),
        ]);

    for (i, pod) in pods.iter().enumerate() {
        let short_id = &pod[pod.len().saturating_sub(5)..];
        table.add_row(vec![
            Cell::new(i + 1).fg(Color::Cyan),
            Cell::new(pod).fg(Color::White),
            Cell::new(short_id).fg(Color::DarkGrey),
        ]);
    }

    println!();
    println!(
        "{}",
        format!("{} Matching Pods ({})", emoji, env_name.to_uppercase()).bold()
    );
    println!("{table}");
    println!();
}

pub fn exec_into_pod(
    pod: &str,
    namespace: &str,
    env_name: &str,
    emoji: &str,
    pod_type: &str,
    pod_num: usize,
) -> Result<()> {
    let prompt_label = format!("{}-{}-{}", env_name, pod_type, pod_num);

    print_header(&format!("Connecting to {}", pod.bright_cyan()));
    println!(
        "  {} {} {}",
        "Prompt:".dimmed(),
        emoji,
        prompt_label.cyan().bold()
    );
    println!();

    let ps1_cmd = format!(
        r#"export PS1="{} \[\033[1;36m\]{}\[\033[0m\] $ "; exec bash --norc --noprofile"#,
        emoji, prompt_label
    );

    let status = Command::new("kubectl")
        .args([
            "exec",
            "-it",
            pod,
            "-n",
            namespace,
            "--",
            "env",
            &format!("EKS_ENV={}", env_name),
            &format!("EKS_TYPE={}", pod_type),
            &format!("EKS_POD_NUM={}", pod_num),
            &format!("EKS_LABEL={}", prompt_label),
            &format!("EKS_EMOJI={}", emoji),
            "bash",
            "-c",
            &ps1_cmd,
        ])
        .status()
        .context("Failed to exec into pod")?;

    if !status.success() {
        bail!("kubectl exec failed");
    }

    Ok(())
}

// ==================== Pod Log Tailing ====================

fn tail_pod_log(
    pod: String,
    namespace: String,
    log_file: String,
    color: &'static str,
    running: Arc<AtomicBool>,
) {
    let short_id = &pod[pod.len().saturating_sub(5)..];
    let short_id = short_id.to_string();

    let child = Command::new("kubectl")
        .args([
            "exec", &pod, "-n", &namespace, "--", "tail", "-f", &log_file,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if !running.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(line) = line {
                let prefix = format!("[{}]", short_id);
                let colored_prefix = match color {
                    "red" => prefix.red(),
                    "green" => prefix.green(),
                    "yellow" => prefix.yellow(),
                    "blue" => prefix.blue(),
                    "magenta" => prefix.magenta(),
                    "cyan" => prefix.cyan(),
                    _ => prefix.white(),
                };
                println!("{} {}", colored_prefix, line);
            }
        }
    }

    let _ = child.kill();
}

pub fn tail_logs(pods: &[String], namespace: &str, log_file: &str) -> Result<()> {
    print_header(&format!("Tailing Logs: {}", log_file.bright_cyan()));
    println!(
        "  {} from {} pods",
        "Streaming".dimmed(),
        pods.len().to_string().green()
    );
    println!("  {} to stop", "Press Ctrl+C".yellow());
    println!();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
        println!("\n{}", "Stopping log tail...".yellow());
    })
    .context("Failed to set Ctrl+C handler")?;

    let mut handles = vec![];

    for (i, pod) in pods.iter().enumerate() {
        let pod = pod.clone();
        let namespace = namespace.to_string();
        let log_file = log_file.to_string();
        let color = ANSI_COLORS[i % ANSI_COLORS.len()];
        let running = running.clone();

        let handle = thread::spawn(move || {
            tail_pod_log(pod, namespace, log_file, color, running);
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

// ==================== Main EKS Command ====================

pub async fn run(
    aws_config: &aws_config::SdkConfig,
    settings: &Settings,
    env_name: &str,
    profile: Option<&str>,
    namespace: &str,
    pod_type: &str,
    pod_num: Option<usize>,
    log_file: Option<String>,
) -> Result<()> {
    let env_config = settings.get_env(env_name);
    let cluster = &env_config.cluster;
    let emoji = &env_config.emoji;

    // Update kubeconfig
    let spin = spinner(&format!("Updating kubeconfig for {}...", cluster));
    update_kubeconfig(aws_config, cluster, profile, &settings.aws.region).await?;
    spin.finish_and_clear();
    print_success(&format!("Connected to {}", cluster.bold()));

    // Get pods
    let spin = spinner(&format!(
        "Fetching pods matching '{}' in namespace '{}'...",
        pod_type, namespace
    ));
    let pods = get_pods(namespace, pod_type);
    spin.finish_and_clear();

    if pods.is_empty() {
        print_error(&format!("No pods found matching '{}'", pod_type));
        std::process::exit(1);
    }

    print_success(&format!("Found {} pods", pods.len()));
    display_pods(&pods, env_name, emoji);

    // Log mode
    if let Some(log_path) = log_file {
        return tail_logs(&pods, namespace, &log_path);
    }

    // No pod specified
    if pod_num.is_none() {
        println!("{}", "Next steps:".yellow().bold());
        println!(
            "  {} {}  Connect to a specific pod",
            "▸".blue(),
            "--pod <number>".cyan()
        );
        println!(
            "  {} {}           Tail logs from all pods",
            "▸".blue(),
            "--log".cyan()
        );
        println!();
        return Ok(());
    }

    // Validate and connect
    let pod_num = pod_num.unwrap();
    if pod_num < 1 || pod_num > pods.len() {
        print_error(&format!(
            "Invalid pod number '{}'. Choose 1-{}",
            pod_num,
            pods.len()
        ));
        std::process::exit(1);
    }

    let pod = &pods[pod_num - 1];
    exec_into_pod(pod, namespace, env_name, emoji, pod_type, pod_num)
}
