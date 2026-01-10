//! EC2 instance operations (listing, display, SSM connection)

use anyhow::{Context, Result};
use colored::Colorize;
use comfy_table::{Cell, Color};

use super::get_config;
use crate::utils::{
    create_table, print_header, print_warning, truncate, TableHeader, DISPLAY_NAME_MAX_LEN,
    DISPLAY_NAME_TRUNCATE_AT,
};

/// Information about an EC2 instance
#[derive(Debug)]
pub struct Ec2Instance {
    pub instance_id: String,
    pub name: Option<String>,
    pub instance_type: String,
    pub state: String,
    pub private_ip: Option<String>,
    pub environment: Option<String>,
}

/// Filter options for listing EC2 instances
pub struct Ec2Filter {
    pub env: Option<String>,
    pub name_filter: Option<String>,
    pub show_all: bool,
    pub stopped_only: bool,
}

/// List EC2 instances with optional filtering
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

/// Display EC2 instances in a formatted table
pub fn display_instances(instances: &[Ec2Instance]) {
    if instances.is_empty() {
        print_warning("No instances found");
        return;
    }

    let mut table = create_table(&[
        TableHeader::new("#", Color::Yellow),
        TableHeader::new("Name", Color::Cyan),
        TableHeader::new("State", Color::White),
        TableHeader::new("Type", Color::Magenta),
        TableHeader::new("IP", Color::Green),
        TableHeader::new("Env", Color::Blue),
    ]);

    for (idx, instance) in instances.iter().enumerate() {
        let state_color = match instance.state.as_str() {
            "running" => Color::Green,
            "stopped" => Color::Red,
            "pending" | "stopping" => Color::Yellow,
            _ => Color::DarkGrey,
        };

        let name = instance.name.as_deref().unwrap_or("-");
        let name_display = truncate(name, DISPLAY_NAME_MAX_LEN, DISPLAY_NAME_TRUNCATE_AT);

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

/// Connect to an EC2 instance via SSM
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
