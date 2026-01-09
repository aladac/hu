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
