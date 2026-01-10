//! EC2 instance spawning and cleanup operations

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::{
    print_header, print_info, print_success, spinner, EC2_KILL_MAX_WAIT_ITERATIONS,
    EC2_SPAWN_MAX_WAIT_ITERATIONS, EC2_SPAWN_WAIT_INTERVAL_SECS,
};

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

    let key_material = resp.key_material().context("No key material in response")?;

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
pub async fn spawn_instance(
    config: &aws_config::SdkConfig,
    spawn_cfg: &SpawnConfig,
) -> Result<SpawnedInstance> {
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
    let sg_id =
        create_temp_security_group(&ec2, &vpc_id, &sg_name, ssh_port, &public_ports, &my_ip)
            .await?;
    spin.finish_and_clear();

    // Launch instance
    let spin = spinner(&format!(
        "Launching {} instance...",
        spawn_cfg.instance_type
    ));
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
    for _ in 0..EC2_SPAWN_MAX_WAIT_ITERATIONS {
        tokio::time::sleep(tokio::time::Duration::from_secs(
            EC2_SPAWN_WAIT_INTERVAL_SECS,
        ))
        .await;

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
    println!();
    print_header("EC2 Instance Spawned");
    println!();
    println!(
        "  {} {}",
        "Instance ID:".dimmed(),
        instance.instance_id.cyan()
    );
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
    use crate::utils::print_error;

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
    for _ in 0..EC2_KILL_MAX_WAIT_ITERATIONS {
        tokio::time::sleep(tokio::time::Duration::from_secs(
            EC2_SPAWN_WAIT_INTERVAL_SECS,
        ))
        .await;

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
        let key_path =
            dirs::home_dir().map(|h| h.join(".hu").join("keys").join(format!("{}.pem", key)));
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
