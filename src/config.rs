use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Default settings.toml content with all options commented out
pub const DEFAULT_SETTINGS: &str = r#"# hu settings
# See settings.example.toml in the repo for full documentation

[aws]
region = "us-east-1"

# AWS profiles for different services
# Each profile is used for specific operations:
#   eks     - EKS/Kubernetes operations (pod listing, exec, log tailing)
#   general - General AWS operations (whoami, login)
#   ec2     - EC2 operations (future)
[aws.profiles]
eks = "eks"
general = "aws"
ec2 = "ec2"

# [kubernetes]
# namespace = "cms"
# pod_type = "web"

# [logging]
# log_path = "~/.config/hu/{env}.log"

# default_env = "dev"

[env.dev]
cluster = "eks-dev"
emoji = "🟢"
log_name = "development"

[env.stg]
cluster = "eks-stg"
emoji = "🟡"
log_name = "staging"

[env.prod]
cluster = "prod-eks"
emoji = "🔴"
log_name = "production"
"#;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub aws: AwsSettings,
    #[serde(default)]
    pub kubernetes: KubernetesSettings,
    #[serde(default)]
    pub logging: LoggingSettings,
    #[serde(default)]
    pub default_env: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, EnvConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AwsSettings {
    pub region: String,
    pub profiles: AwsProfiles,
}

impl Default for AwsSettings {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            profiles: AwsProfiles::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AwsProfiles {
    /// Profile for EKS/Kubernetes operations
    pub eks: Option<String>,
    /// Profile for general AWS operations (whoami, etc.)
    pub general: Option<String>,
    /// Profile for EC2 operations
    pub ec2: Option<String>,
}

impl Default for AwsProfiles {
    fn default() -> Self {
        Self {
            eks: Some("eks".to_string()),
            general: Some("aws".to_string()),
            ec2: Some("ec2".to_string()),
        }
    }
}

impl AwsProfiles {
    /// Get profile for EKS operations
    pub fn eks_profile(&self) -> Option<&str> {
        self.eks.as_deref()
    }

    /// Get profile for general AWS operations
    pub fn general_profile(&self) -> Option<&str> {
        self.general.as_deref()
    }

    /// Get profile for EC2 operations (future use)
    #[allow(dead_code)]
    pub fn ec2_profile(&self) -> Option<&str> {
        self.ec2.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct KubernetesSettings {
    pub namespace: String,
    pub pod_type: String,
}

impl Default for KubernetesSettings {
    fn default() -> Self {
        Self {
            namespace: "cms".to_string(),
            pod_type: "web".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvConfig {
    pub cluster: String,
    pub emoji: String,
    pub log_name: Option<String>,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            cluster: String::new(),
            emoji: "⚪".to_string(),
            log_name: None,
        }
    }
}

impl Settings {
    /// Get environment config, falling back to defaults for known envs
    pub fn get_env(&self, name: &str) -> EnvConfig {
        if let Some(config) = self.env.get(name) {
            return config.clone();
        }

        // Default configs for known environments
        match name {
            "prod" => EnvConfig {
                cluster: "prod-eks".to_string(),
                emoji: "🔴".to_string(),
                log_name: Some("production".to_string()),
            },
            "dev" => EnvConfig {
                cluster: "eks-dev".to_string(),
                emoji: "🟢".to_string(),
                log_name: Some("development".to_string()),
            },
            "stg" => EnvConfig {
                cluster: "eks-stg".to_string(),
                emoji: "🟡".to_string(),
                log_name: Some("staging".to_string()),
            },
            _ => EnvConfig {
                cluster: format!("eks-{}", name),
                emoji: "⚪".to_string(),
                log_name: None,
            },
        }
    }

    /// Get the default environment name
    pub fn default_env_name(&self) -> &str {
        self.default_env.as_deref().unwrap_or("dev")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingSettings {
    pub log_path: String,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            log_path: "~/.config/hu/{env}.log".to_string(),
        }
    }
}

/// Get the path to the settings file
pub fn settings_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    Ok(config_dir.join("hu").join("settings.toml"))
}

/// Ensure the settings file exists, creating it with defaults if not
pub fn ensure_settings_file() -> Result<PathBuf> {
    let path = settings_path()?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }
        std::fs::write(&path, DEFAULT_SETTINGS)
            .with_context(|| format!("Failed to write default settings to {:?}", path))?;
    }

    Ok(path)
}

/// Load settings from the config file
pub fn load_settings() -> Result<Settings> {
    let path = ensure_settings_file()?;
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read settings from {:?}", path))?;

    let settings: Settings =
        toml::from_str(&content).with_context(|| "Failed to parse settings.toml")?;

    Ok(settings)
}
