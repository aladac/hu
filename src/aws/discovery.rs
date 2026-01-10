//! AWS profile discovery and capability checking

use anyhow::{Context, Result};
use colored::Colorize;

use super::get_config;
use super::identity::IdentityInfo;
use crate::utils::{print_header, print_warning, spinner};

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
