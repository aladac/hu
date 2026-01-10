use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::process::Command;

use crate::utils::{print_header, print_warning, spinner};

pub async fn get_config(profile: Option<&str>, region: &str) -> aws_config::SdkConfig {
    let mut builder = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new(region.to_string()));

    if let Some(profile_name) = profile {
        builder = builder.profile_name(profile_name);
    }

    builder.load().await
}

pub async fn check_session(config: &aws_config::SdkConfig) -> bool {
    let client = aws_sdk_sts::Client::new(config);
    client.get_caller_identity().send().await.is_ok()
}

pub fn sso_login(profile: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("aws");
    cmd.args(["sso", "login"]);

    if let Some(profile_name) = profile {
        cmd.args(["--profile", profile_name]);
    }

    let status = cmd.status().context("Failed to run aws sso login")?;

    if status.success() {
        Ok(())
    } else {
        bail!("AWS SSO login failed")
    }
}

// ==================== AWS Identity & Permissions ====================

#[derive(Debug)]
pub enum IdentityType {
    User(String),
    AssumedRole(String),
    FederatedUser(String),
    Unknown,
}

#[derive(Debug)]
pub struct IdentityInfo {
    pub account: String,
    pub arn: String,
    pub identity_type: IdentityType,
}

impl IdentityInfo {
    pub fn from_arn(arn: &str, account: &str) -> Self {
        let identity_type = if arn.contains(":user/") {
            let name = arn.split(":user/").last().unwrap_or("unknown").to_string();
            IdentityType::User(name)
        } else if arn.contains(":assumed-role/") {
            let parts: Vec<&str> = arn
                .split(":assumed-role/")
                .last()
                .unwrap_or("")
                .split('/')
                .collect();
            let role_name = parts.first().unwrap_or(&"unknown").to_string();
            IdentityType::AssumedRole(role_name)
        } else if arn.contains(":federated-user/") {
            let name = arn
                .split(":federated-user/")
                .last()
                .unwrap_or("unknown")
                .to_string();
            IdentityType::FederatedUser(name)
        } else {
            IdentityType::Unknown
        };

        Self {
            account: account.to_string(),
            arn: arn.to_string(),
            identity_type,
        }
    }

    pub fn type_name(&self) -> &str {
        match &self.identity_type {
            IdentityType::User(_) => "IAM User",
            IdentityType::AssumedRole(_) => "Assumed Role",
            IdentityType::FederatedUser(_) => "Federated User",
            IdentityType::Unknown => "Unknown",
        }
    }

    pub fn name(&self) -> &str {
        match &self.identity_type {
            IdentityType::User(n) => n,
            IdentityType::AssumedRole(n) => n,
            IdentityType::FederatedUser(n) => n,
            IdentityType::Unknown => "unknown",
        }
    }
}

pub async fn get_identity(config: &aws_config::SdkConfig) -> Result<IdentityInfo> {
    let sts = aws_sdk_sts::Client::new(config);
    let identity = sts
        .get_caller_identity()
        .send()
        .await
        .context("Failed to get caller identity")?;

    let arn = identity.arn().context("No ARN in identity response")?;
    let account = identity
        .account()
        .context("No account in identity response")?;

    Ok(IdentityInfo::from_arn(arn, account))
}

pub async fn whoami(config: &aws_config::SdkConfig) -> Result<()> {
    let spinner = spinner("Fetching AWS identity...");
    let identity = get_identity(config).await?;
    spinner.finish_and_clear();

    print_header("AWS Identity");
    println!("  {} {}", "Account:".dimmed(), identity.account.white());
    println!("  {} {}", "Type:".dimmed(), identity.type_name().cyan());
    println!("  {} {}", "ARN:".dimmed(), identity.arn.white());
    println!("  {} {}", "Name:".dimmed(), identity.name().cyan().bold());

    // Note: Policy fetching often fails due to IAM permissions
    // Could add --verbose flag to attempt policy lookup
    print_warning("Use AWS Console or `aws iam` CLI to view attached policies");

    println!();
    Ok(())
}

// ==================== Profile Discovery ====================

/// List all AWS profiles from ~/.aws/config
pub fn list_aws_profiles() -> Result<Vec<String>> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let config_path = home.join(".aws").join("config");

    if !config_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {:?}", config_path))?;

    let mut profiles = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];
            if section == "default" {
                profiles.push("default".to_string());
            } else if let Some(name) = section.strip_prefix("profile ") {
                profiles.push(name.to_string());
            }
        }
    }

    Ok(profiles)
}

/// Capabilities discovered for a profile (all read-only operations)
#[derive(Debug)]
pub struct ProfileCapabilities {
    pub profile: String,
    pub valid: bool,
    pub identity: Option<IdentityInfo>,
    pub eks_clusters: Option<Vec<String>>,
    pub ec2_accessible: Option<bool>,
    pub s3_bucket_count: Option<usize>,
    pub pipeline_count: Option<usize>,
}

/// Check what a profile can do (read-only operations only)
pub async fn check_profile_capabilities(profile: &str, region: &str) -> ProfileCapabilities {
    let config = get_config(Some(profile), region).await;

    // Check identity first
    let sts = aws_sdk_sts::Client::new(&config);
    let identity_result = sts.get_caller_identity().send().await;

    let (valid, identity) = match identity_result {
        Ok(id) => {
            let arn = id.arn().unwrap_or("");
            let account = id.account().unwrap_or("");
            (true, Some(IdentityInfo::from_arn(arn, account)))
        }
        Err(_) => (false, None),
    };

    if !valid {
        return ProfileCapabilities {
            profile: profile.to_string(),
            valid: false,
            identity: None,
            eks_clusters: None,
            ec2_accessible: None,
            s3_bucket_count: None,
            pipeline_count: None,
        };
    }

    // Check EKS (list clusters - read only)
    let eks_clusters = {
        let eks = aws_sdk_eks::Client::new(&config);
        match eks.list_clusters().send().await {
            Ok(resp) => Some(resp.clusters().to_vec()),
            Err(_) => None,
        }
    };

    // Check EC2 (describe regions - basic read check)
    let ec2_accessible = {
        let ec2 = aws_sdk_ec2::Client::new(&config);
        match ec2.describe_regions().send().await {
            Ok(_) => Some(true),
            Err(_) => Some(false),
        }
    };

    // Check S3 (list buckets - read only, count only)
    let s3_bucket_count = {
        let s3 = aws_sdk_s3::Client::new(&config);
        match s3.list_buckets().send().await {
            Ok(resp) => Some(resp.buckets().len()),
            Err(_) => None,
        }
    };

    // Check CodePipeline (list pipelines - read only, count only)
    let pipeline_count = {
        let cp = aws_sdk_codepipeline::Client::new(&config);
        match cp.list_pipelines().send().await {
            Ok(resp) => Some(resp.pipelines().len()),
            Err(_) => None,
        }
    };

    ProfileCapabilities {
        profile: profile.to_string(),
        valid,
        identity,
        eks_clusters,
        ec2_accessible,
        s3_bucket_count,
        pipeline_count,
    }
}

/// Discover all AWS profiles and their capabilities
pub async fn discover(region: &str, show_all: bool, json_output: bool) -> Result<()> {
    let profiles = list_aws_profiles()?;

    if profiles.is_empty() {
        print_warning("No AWS profiles found in ~/.aws/config");
        return Ok(());
    }

    if !json_output {
        print_header(&format!(
            "AWS Profile Discovery ({} profiles)",
            profiles.len()
        ));
        println!();
    }

    let mut results = Vec::new();

    for profile in &profiles {
        if !json_output {
            let spin = spinner(&format!("Checking profile: {}...", profile));
            let caps = check_profile_capabilities(profile, region).await;
            spin.finish_and_clear();
            results.push(caps);
        } else {
            let caps = check_profile_capabilities(profile, region).await;
            results.push(caps);
        }
    }

    if json_output {
        print_discovery_json(&results, show_all);
    } else {
        print_discovery_table(&results, show_all);
    }

    Ok(())
}

fn print_discovery_table(results: &[ProfileCapabilities], show_all: bool) {
    for caps in results {
        if !show_all && !caps.valid {
            continue;
        }

        if caps.valid {
            let identity = caps.identity.as_ref().unwrap();
            println!("  {} {}", "Profile:".dimmed(), caps.profile.cyan().bold());
            println!("    {} {}", "Account:".dimmed(), identity.account.white());
            println!(
                "    {} {} ({})",
                "Identity:".dimmed(),
                identity.name().white(),
                identity.type_name().cyan()
            );

            // EKS
            if let Some(clusters) = &caps.eks_clusters {
                if clusters.is_empty() {
                    println!("    {} {}", "EKS:".dimmed(), "no clusters".yellow());
                } else {
                    println!(
                        "    {} {} clusters ({})",
                        "EKS:".dimmed(),
                        clusters.len().to_string().green(),
                        clusters.join(", ").white()
                    );
                }
            } else {
                println!("    {} {}", "EKS:".dimmed(), "no access".red());
            }

            // EC2
            match caps.ec2_accessible {
                Some(true) => println!("    {} {}", "EC2:".dimmed(), "accessible".green()),
                Some(false) => println!("    {} {}", "EC2:".dimmed(), "no access".red()),
                None => println!("    {} {}", "EC2:".dimmed(), "unknown".yellow()),
            }

            // S3
            if let Some(count) = caps.s3_bucket_count {
                println!(
                    "    {} {} buckets",
                    "S3:".dimmed(),
                    count.to_string().green()
                );
            } else {
                println!("    {} {}", "S3:".dimmed(), "no access".red());
            }

            // Pipelines
            if let Some(count) = caps.pipeline_count {
                println!(
                    "    {} {} pipelines",
                    "Pipelines:".dimmed(),
                    count.to_string().green()
                );
            } else {
                println!("    {} {}", "Pipelines:".dimmed(), "no access".red());
            }
        } else {
            println!(
                "  {} {} {}",
                "Profile:".dimmed(),
                caps.profile.cyan().bold(),
                "(EXPIRED)".red()
            );
            println!(
                "    {} aws sso login --profile {}",
                "Run:".dimmed(),
                caps.profile.yellow()
            );
        }
        println!();
    }

    // Summary
    let valid_count = results.iter().filter(|c| c.valid).count();
    let expired_count = results.len() - valid_count;

    println!(
        "  {} {} valid, {} expired",
        "Summary:".dimmed(),
        valid_count.to_string().green(),
        expired_count.to_string().red()
    );
}

fn print_discovery_json(results: &[ProfileCapabilities], show_all: bool) {
    let filtered: Vec<_> = results.iter().filter(|c| show_all || c.valid).collect();

    let json_results: Vec<serde_json::Value> = filtered
        .iter()
        .map(|caps| {
            serde_json::json!({
                "profile": caps.profile,
                "valid": caps.valid,
                "account": caps.identity.as_ref().map(|i| &i.account),
                "identity_type": caps.identity.as_ref().map(|i| i.type_name()),
                "identity_name": caps.identity.as_ref().map(|i| i.name()),
                "eks_clusters": caps.eks_clusters,
                "ec2_accessible": caps.ec2_accessible,
                "s3_bucket_count": caps.s3_bucket_count,
                "pipeline_count": caps.pipeline_count,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_results).unwrap());
}

// ==================== EC2 Operations ====================

#[derive(Debug)]
pub struct Ec2Instance {
    pub instance_id: String,
    pub name: Option<String>,
    pub instance_type: String,
    pub state: String,
    pub private_ip: Option<String>,
    pub environment: Option<String>,
}

pub struct Ec2Filter {
    pub env: Option<String>,
    pub name_filter: Option<String>,
    pub show_all: bool,
    pub stopped_only: bool,
}

pub async fn list_instances(region: &str, filter: &Ec2Filter) -> Result<Vec<Ec2Instance>> {
    let config = get_config(None, region).await;
    let ec2 = aws_sdk_ec2::Client::new(&config);

    let resp = ec2
        .describe_instances()
        .send()
        .await
        .context("Failed to describe EC2 instances")?;

    let mut instances = Vec::new();

    for reservation in resp.reservations() {
        for instance in reservation.instances() {
            let instance_id = instance.instance_id().unwrap_or("").to_string();
            let instance_type = instance
                .instance_type()
                .map(|t| t.as_str().to_string())
                .unwrap_or_default();
            let state = instance
                .state()
                .and_then(|s| s.name())
                .map(|n| n.as_str().to_string())
                .unwrap_or_default();
            let private_ip = instance.private_ip_address().map(|s| s.to_string());

            // Extract tags
            let mut name = None;
            let mut environment = None;
            for tag in instance.tags() {
                match tag.key() {
                    Some("Name") => name = tag.value().map(|s| s.to_string()),
                    Some("Environment") => environment = tag.value().map(|s| s.to_string()),
                    _ => {}
                }
            }

            instances.push(Ec2Instance {
                instance_id,
                name,
                instance_type,
                state,
                private_ip,
                environment,
            });
        }
    }

    // Apply filters
    let filtered: Vec<Ec2Instance> = instances
        .into_iter()
        .filter(|i| {
            // Filter terminated unless --all
            if !filter.show_all && i.state == "terminated" {
                return false;
            }

            // Filter unnamed unless --all
            if !filter.show_all && i.name.is_none() {
                return false;
            }

            // Filter by stopped_only
            if filter.stopped_only && i.state != "stopped" {
                return false;
            }

            // Filter by environment
            if let Some(env) = &filter.env {
                if i.environment.as_deref() != Some(env.as_str()) {
                    return false;
                }
            }

            // Filter by name pattern
            if let Some(pattern) = &filter.name_filter {
                if let Some(name) = &i.name {
                    if !name.to_lowercase().contains(&pattern.to_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        })
        .collect();

    Ok(filtered)
}

pub fn display_instances(instances: &[Ec2Instance]) {
    use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};

    if instances.is_empty() {
        print_warning("No instances found");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#").fg(Color::Yellow),
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("State").fg(Color::White),
            Cell::new("Type").fg(Color::Magenta),
            Cell::new("IP").fg(Color::Green),
            Cell::new("Env").fg(Color::Blue),
        ]);

    for (idx, instance) in instances.iter().enumerate() {
        let state_color = match instance.state.as_str() {
            "running" => Color::Green,
            "stopped" => Color::Red,
            "pending" | "stopping" => Color::Yellow,
            _ => Color::DarkGrey,
        };

        let name = instance
            .name
            .as_deref()
            .unwrap_or("-")
            .to_string();
        let name_display = if name.len() > 30 {
            format!("{}...", &name[..27])
        } else {
            name
        };

        table.add_row(vec![
            Cell::new(idx + 1).fg(Color::Yellow),
            Cell::new(&name_display).fg(Color::White),
            Cell::new(&instance.state).fg(state_color),
            Cell::new(&instance.instance_type).fg(Color::DarkGrey),
            Cell::new(instance.private_ip.as_deref().unwrap_or("-")).fg(Color::DarkGrey),
            Cell::new(instance.environment.as_deref().unwrap_or("-")).fg(Color::DarkGrey),
        ]);
    }

    println!();
    print_header(&format!("EC2 Instances ({})", instances.len()));
    println!("{table}");
    println!();
}

pub fn ssm_connect(instances: &[Ec2Instance], num: usize) -> Result<()> {
    use crate::utils::print_error;

    if num == 0 || num > instances.len() {
        print_error(&format!(
            "Invalid instance number. Choose 1-{}",
            instances.len()
        ));
        return Ok(());
    }

    let instance = &instances[num - 1];

    if instance.state != "running" {
        print_error(&format!(
            "Instance '{}' is {} (must be running)",
            instance.name.as_deref().unwrap_or(&instance.instance_id),
            instance.state
        ));
        return Ok(());
    }

    let name = instance.name.as_deref().unwrap_or(&instance.instance_id);
    println!(
        "{}",
        format!("Connecting to {} ({})...", name, instance.instance_id).dimmed()
    );

    let status = std::process::Command::new("aws")
        .args([
            "ssm",
            "start-session",
            "--target",
            &instance.instance_id,
            "--document-name",
            "AWS-StartInteractiveCommand",
            "--parameters",
            "command=[\"bash -l\"]",
        ])
        .status()
        .context("Failed to start SSM session")?;

    if !status.success() {
        print_error("SSM session failed. Ensure the instance has SSM agent and proper IAM role.");
    }

    Ok(())
}

// ==================== EC2 Spawn Operations ====================

use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for spawning a temporary EC2 instance
pub struct SpawnConfig {
    pub instance_type: String,
    pub ami: Option<String>,
    pub my_ip: Option<String>,
    pub public_ports: Vec<u16>,
}

/// Result of spawning an EC2 instance
#[derive(Debug)]
pub struct SpawnedInstance {
    pub instance_id: String,
    pub public_ip: String,
    pub ssh_port: u16,
    pub public_ports: Vec<u16>,
    pub key_name: String,
    pub key_path: String,
    pub security_group_id: String,
}

/// Generate a random high port (1024-65535)
fn generate_random_port() -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    let hash = hasher.finish();

    ((hash % 64511) + 1024) as u16
}

/// Get public IP via external API
pub async fn get_my_public_ip() -> Result<String> {
    let client = reqwest::Client::new();
    let ip = client
        .get("https://checkip.amazonaws.com")
        .send()
        .await
        .context("Failed to fetch public IP")?
        .text()
        .await
        .context("Failed to read public IP response")?
        .trim()
        .to_string();
    Ok(ip)
}

/// Get default VPC ID
async fn get_default_vpc(ec2: &aws_sdk_ec2::Client) -> Result<String> {
    let resp = ec2
        .describe_vpcs()
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("isDefault")
                .values("true")
                .build(),
        )
        .send()
        .await
        .context("Failed to describe VPCs")?;

    resp.vpcs()
        .first()
        .and_then(|v| v.vpc_id().map(|s| s.to_string()))
        .context("No default VPC found")
}

/// Get latest Amazon Linux 2023 ARM AMI
async fn get_latest_al2023_arm_ami(ec2: &aws_sdk_ec2::Client) -> Result<String> {
    let resp = ec2
        .describe_images()
        .owners("amazon")
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("name")
                .values("al2023-ami-2023*-arm64")
                .build(),
        )
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("state")
                .values("available")
                .build(),
        )
        .send()
        .await
        .context("Failed to describe AMIs")?;

    // Sort by creation date and get the latest
    let mut images: Vec<_> = resp.images().to_vec();
    images.sort_by(|a, b| {
        b.creation_date()
            .unwrap_or("")
            .cmp(a.creation_date().unwrap_or(""))
    });

    images
        .first()
        .and_then(|i| i.image_id().map(|s| s.to_string()))
        .context("No Amazon Linux 2023 ARM AMI found")
}

/// Create a temporary SSH key pair and save to ~/.hu/keys/
async fn create_temp_keypair(ec2: &aws_sdk_ec2::Client, key_name: &str) -> Result<String> {
    let resp = ec2
        .create_key_pair()
        .key_name(key_name)
        .key_type(aws_sdk_ec2::types::KeyType::Ed25519)
        .send()
        .await
        .context("Failed to create key pair")?;

    let key_material = resp
        .key_material()
        .context("No key material in response")?;

    // Save to ~/.hu/keys/
    let hu_dir = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".hu")
        .join("keys");

    std::fs::create_dir_all(&hu_dir).context("Failed to create ~/.hu/keys directory")?;

    let key_path = hu_dir.join(format!("{}.pem", key_name));
    std::fs::write(&key_path, key_material).context("Failed to write key file")?;

    // Set permissions to 400
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o400))
            .context("Failed to set key file permissions")?;
    }

    Ok(key_path.to_string_lossy().to_string())
}

/// Create a temporary security group with custom SSH port and public ports
async fn create_temp_security_group(
    ec2: &aws_sdk_ec2::Client,
    vpc_id: &str,
    sg_name: &str,
    ssh_port: u16,
    public_ports: &[u16],
    my_ip: &str,
) -> Result<String> {
    // Create security group
    let public_desc = if public_ports.is_empty() {
        String::new()
    } else {
        format!(", Public:{:?}", public_ports)
    };
    let resp = ec2
        .create_security_group()
        .group_name(sg_name)
        .description(format!(
            "Temporary hu-spawned instance (SSH:{}{})",
            ssh_port, public_desc
        ))
        .vpc_id(vpc_id)
        .send()
        .await
        .context("Failed to create security group")?;

    let sg_id = resp
        .group_id()
        .context("No security group ID in response")?
        .to_string();

    let cidr = format!("{}/32", my_ip);

    // Add SSH rule (my IP only)
    ec2.authorize_security_group_ingress()
        .group_id(&sg_id)
        .ip_permissions(
            aws_sdk_ec2::types::IpPermission::builder()
                .ip_protocol("tcp")
                .from_port(ssh_port as i32)
                .to_port(ssh_port as i32)
                .ip_ranges(
                    aws_sdk_ec2::types::IpRange::builder()
                        .cidr_ip(&cidr)
                        .description("SSH from my IP")
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .context("Failed to add SSH ingress rule")?;

    // Add public port rules
    for port in public_ports {
        ec2.authorize_security_group_ingress()
            .group_id(&sg_id)
            .ip_permissions(
                aws_sdk_ec2::types::IpPermission::builder()
                    .ip_protocol("tcp")
                    .from_port(*port as i32)
                    .to_port(*port as i32)
                    .ip_ranges(
                        aws_sdk_ec2::types::IpRange::builder()
                            .cidr_ip("0.0.0.0/0")
                            .description(format!("Port {} public", port))
                            .build(),
                    )
                    .build(),
            )
            .send()
            .await
            .with_context(|| format!("Failed to add port {} ingress rule", port))?;
    }

    Ok(sg_id)
}

/// Generate user data script to configure SSH on custom port
fn generate_user_data(ssh_port: u16) -> String {
    let script = format!(
        r#"#!/bin/bash
# Configure SSH on custom port
sed -i 's/#Port 22/Port {}/' /etc/ssh/sshd_config
sed -i 's/Port 22/Port {}/' /etc/ssh/sshd_config
# SELinux port context (Amazon Linux)
semanage port -a -t ssh_port_t -p tcp {} 2>/dev/null || true
# Restart SSH
systemctl restart sshd
"#,
        ssh_port, ssh_port, ssh_port
    );

    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(script.as_bytes())
}

/// Spawn an EC2 instance with random SSH/HTTPS ports
pub async fn spawn_instance(config: &aws_config::SdkConfig, spawn_cfg: &SpawnConfig) -> Result<SpawnedInstance> {
    use crate::utils::{print_info, print_success};

    let ec2 = aws_sdk_ec2::Client::new(config);

    // Generate random SSH port
    let ssh_port = generate_random_port();
    let public_ports = spawn_cfg.public_ports.clone();

    if public_ports.is_empty() {
        print_info(&format!("SSH port: {} (random, your IP only)", ssh_port));
    } else {
        print_info(&format!(
            "SSH port: {} (your IP only), Public ports: {:?}",
            ssh_port, public_ports
        ));
    }

    // Get public IP
    let spin = spinner("Detecting your public IP...");
    let my_ip = match &spawn_cfg.my_ip {
        Some(ip) => ip.clone(),
        None => get_my_public_ip().await?,
    };
    spin.finish_and_clear();
    print_info(&format!("Your IP: {}", my_ip));

    // Get default VPC
    let spin = spinner("Finding default VPC...");
    let vpc_id = get_default_vpc(&ec2).await?;
    spin.finish_and_clear();

    // Get AMI
    let spin = spinner("Finding latest Amazon Linux 2023 ARM AMI...");
    let ami_id = match &spawn_cfg.ami {
        Some(ami) => ami.clone(),
        None => get_latest_al2023_arm_ami(&ec2).await?,
    };
    spin.finish_and_clear();
    print_info(&format!("AMI: {}", ami_id));

    // Generate unique names
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let key_name = format!("hu-{}", timestamp);
    let sg_name = format!("hu-temp-{}", timestamp);

    // Create key pair
    let spin = spinner("Creating SSH key pair...");
    let key_path = create_temp_keypair(&ec2, &key_name).await?;
    spin.finish_and_clear();
    print_success(&format!("Key saved: {}", key_path));

    // Create security group
    let spin = spinner("Creating security group...");
    let sg_id = create_temp_security_group(&ec2, &vpc_id, &sg_name, ssh_port, &public_ports, &my_ip).await?;
    spin.finish_and_clear();

    // Launch instance
    let spin = spinner(&format!("Launching {} instance...", spawn_cfg.instance_type));
    let user_data = generate_user_data(ssh_port);

    let public_ports_str = public_ports
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let run_resp = ec2
        .run_instances()
        .image_id(&ami_id)
        .instance_type(aws_sdk_ec2::types::InstanceType::from(
            spawn_cfg.instance_type.as_str(),
        ))
        .key_name(&key_name)
        .security_group_ids(&sg_id)
        .user_data(&user_data)
        .min_count(1)
        .max_count(1)
        .tag_specifications(
            aws_sdk_ec2::types::TagSpecification::builder()
                .resource_type(aws_sdk_ec2::types::ResourceType::Instance)
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("Name")
                        .value(format!("hu-spawned-{}", timestamp))
                        .build(),
                )
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("hu-managed")
                        .value("true")
                        .build(),
                )
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("hu-key-name")
                        .value(&key_name)
                        .build(),
                )
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("hu-sg-id")
                        .value(&sg_id)
                        .build(),
                )
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("hu-ssh-port")
                        .value(ssh_port.to_string())
                        .build(),
                )
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("hu-public-ports")
                        .value(&public_ports_str)
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .context("Failed to launch instance")?;

    let instance_id = run_resp
        .instances()
        .first()
        .and_then(|i| i.instance_id().map(|s| s.to_string()))
        .context("No instance ID in response")?;

    spin.finish_and_clear();
    print_info(&format!("Instance launched: {}", instance_id));

    // Wait for instance to be running
    let spin = spinner("Waiting for instance to be running...");
    let mut public_ip = String::new();
    for _ in 0..60 {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let desc_resp = ec2
            .describe_instances()
            .instance_ids(&instance_id)
            .send()
            .await?;

        if let Some(reservation) = desc_resp.reservations().first() {
            if let Some(instance) = reservation.instances().first() {
                let state = instance
                    .state()
                    .and_then(|s| s.name())
                    .map(|n| n.as_str())
                    .unwrap_or("");

                if state == "running" {
                    if let Some(ip) = instance.public_ip_address() {
                        public_ip = ip.to_string();
                        break;
                    }
                }
            }
        }
    }
    spin.finish_and_clear();

    if public_ip.is_empty() {
        bail!("Instance did not get a public IP within timeout");
    }

    Ok(SpawnedInstance {
        instance_id,
        public_ip,
        ssh_port,
        public_ports,
        key_name,
        key_path,
        security_group_id: sg_id,
    })
}

/// Display spawned instance information
pub fn display_spawned_instance(instance: &SpawnedInstance) {
    use crate::utils::print_header;

    println!();
    print_header("EC2 Instance Spawned");
    println!();
    println!("  {} {}", "Instance ID:".dimmed(), instance.instance_id.cyan());
    println!("  {} {}", "Public IP:".dimmed(), instance.public_ip.green());
    println!(
        "  {} {} {}",
        "SSH Port:".dimmed(),
        instance.ssh_port.to_string().yellow(),
        "(your IP only)".dimmed()
    );
    if !instance.public_ports.is_empty() {
        let ports_str = instance
            .public_ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {} {} {}",
            "Public Ports:".dimmed(),
            ports_str.green(),
            "(0.0.0.0/0)".dimmed()
        );
    }
    println!("  {} {}", "Key File:".dimmed(), instance.key_path.white());
    println!("  {} {}", "Key Name:".dimmed(), instance.key_name.dimmed());
    println!(
        "  {} {}",
        "Security Group:".dimmed(),
        instance.security_group_id.dimmed()
    );
    println!();
    println!("  {}", "Connect:".dimmed());
    println!(
        "    {}",
        format!(
            "ssh -i {} -p {} ec2-user@{}",
            instance.key_path, instance.ssh_port, instance.public_ip
        )
        .green()
    );
    println!();
    println!("  {}", "Cleanup:".dimmed());
    println!(
        "    {}",
        format!("hu ec2 kill {}", instance.instance_id).red()
    );
    println!();
}

/// Kill a spawned instance and cleanup resources
pub async fn kill_instance(config: &aws_config::SdkConfig, instance_id: &str) -> Result<()> {
    use crate::utils::{print_error, print_info, print_success};

    let ec2 = aws_sdk_ec2::Client::new(config);

    // Get instance info to find associated resources
    let spin = spinner("Getting instance info...");
    let desc_resp = ec2
        .describe_instances()
        .instance_ids(instance_id)
        .send()
        .await
        .context("Failed to describe instance")?;
    spin.finish_and_clear();

    let instance = desc_resp
        .reservations()
        .first()
        .and_then(|r| r.instances().first())
        .context("Instance not found")?;

    // Extract resource info from tags
    let mut key_name = None;
    let mut sg_id = None;

    for tag in instance.tags() {
        match tag.key() {
            Some("hu-key-name") => key_name = tag.value().map(|s| s.to_string()),
            Some("hu-sg-id") => sg_id = tag.value().map(|s| s.to_string()),
            _ => {}
        }
    }

    // Terminate instance
    let spin = spinner("Terminating instance...");
    ec2.terminate_instances()
        .instance_ids(instance_id)
        .send()
        .await
        .context("Failed to terminate instance")?;
    spin.finish_and_clear();
    print_success(&format!("Instance {} terminated", instance_id));

    // Wait for instance to be terminated before deleting security group
    let spin = spinner("Waiting for termination...");
    for _ in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let desc_resp = ec2
            .describe_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        if let Some(reservation) = desc_resp.reservations().first() {
            if let Some(inst) = reservation.instances().first() {
                let state = inst
                    .state()
                    .and_then(|s| s.name())
                    .map(|n| n.as_str())
                    .unwrap_or("");

                if state == "terminated" {
                    break;
                }
            }
        }
    }
    spin.finish_and_clear();

    // Delete key pair
    if let Some(key) = &key_name {
        let spin = spinner("Deleting key pair...");
        match ec2.delete_key_pair().key_name(key).send().await {
            Ok(_) => {
                spin.finish_and_clear();
                print_success(&format!("Key pair {} deleted", key));
            }
            Err(e) => {
                spin.finish_and_clear();
                print_error(&format!("Failed to delete key pair: {}", e));
            }
        }

        // Also delete local key file
        let key_path = dirs::home_dir()
            .map(|h| h.join(".hu").join("keys").join(format!("{}.pem", key)));
        if let Some(path) = key_path {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    print_error(&format!("Failed to delete local key file: {}", e));
                } else {
                    print_info(&format!("Deleted local key: {}", path.display()));
                }
            }
        }
    }

    // Delete security group
    if let Some(sg) = &sg_id {
        let spin = spinner("Deleting security group...");
        match ec2.delete_security_group().group_id(sg).send().await {
            Ok(_) => {
                spin.finish_and_clear();
                print_success(&format!("Security group {} deleted", sg));
            }
            Err(e) => {
                spin.finish_and_clear();
                print_error(&format!("Failed to delete security group: {}", e));
            }
        }
    }

    println!();
    print_success("Cleanup complete");
    Ok(())
}
