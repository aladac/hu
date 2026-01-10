use anyhow::{Context, Result};
use colored::{ColoredString, Colorize};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

// ==================== Constants ====================

pub const ANSI_COLORS: [&str; 6] = ["red", "green", "yellow", "blue", "magenta", "cyan"];

/// OAuth callback port for Jira authentication
pub const OAUTH_CALLBACK_PORT: u16 = 8765;

/// Fetch multiplier for filtering GitHub workflow runs
pub const GITHUB_FETCH_MULTIPLIER: u32 = 5;

/// Maximum fetch limit for GitHub API
pub const GITHUB_MAX_FETCH_LIMIT: u32 = 100;

/// Display truncation lengths
pub const DISPLAY_TITLE_MAX_LEN: usize = 55;
pub const DISPLAY_TITLE_TRUNCATE_AT: usize = 52;
pub const DISPLAY_BRANCH_MAX_LEN: usize = 25;
pub const DISPLAY_BRANCH_TRUNCATE_AT: usize = 22;
pub const DISPLAY_NAME_MAX_LEN: usize = 30;
pub const DISPLAY_NAME_TRUNCATE_AT: usize = 27;
pub const DISPLAY_SUMMARY_MAX_LEN: usize = 45;
pub const DISPLAY_SUMMARY_TRUNCATE_AT: usize = 42;
pub const DISPLAY_EPIC_MAX_LEN: usize = 20;
pub const DISPLAY_EPIC_TRUNCATE_AT: usize = 17;

/// Project workflow display lengths (shorter for multi-column)
pub const PROJECT_TITLE_MAX_LEN: usize = 45;
pub const PROJECT_TITLE_TRUNCATE_AT: usize = 42;
pub const PROJECT_BRANCH_MAX_LEN: usize = 20;
pub const PROJECT_BRANCH_TRUNCATE_AT: usize = 17;
pub const PROJECT_REPO_MAX_LEN: usize = 10;
pub const PROJECT_REPO_TRUNCATE_AT: usize = 7;

/// EC2 spawn timeout settings
pub const EC2_SPAWN_MAX_WAIT_ITERATIONS: u32 = 60;
pub const EC2_SPAWN_WAIT_INTERVAL_SECS: u64 = 5;
pub const EC2_KILL_MAX_WAIT_ITERATIONS: u32 = 30;

/// Log tail poll interval
pub const LOG_POLL_INTERVAL_MS: u64 = 100;

/// Pagination limits
pub const JIRA_MAX_RESULTS: &str = "100";
pub const GITHUB_PER_REPO_MIN_LIMIT: u32 = 5;

pub fn run_cmd(cmd: &[&str]) -> Option<String> {
    Command::new(cmd[0])
        .args(&cmd[1..])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

pub fn print_header(text: &str) {
    println!();
    println!("{}", format!("━━━ {} ━━━", text).bright_blue().bold());
    println!();
}

pub fn print_info(text: &str) {
    println!("{} {}", "ℹ".blue(), text);
}

pub fn print_success(text: &str) {
    println!("{} {}", "✓".green(), text);
}

pub fn print_warning(text: &str) {
    println!("{} {}", "⚠".yellow(), text);
}

pub fn print_error(text: &str) {
    eprintln!("{} {}", "✗".red(), text);
}

pub fn spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

// ==================== String Helpers ====================

/// Truncate a string to a maximum length, adding "..." if truncated.
/// Uses (max_len, truncate_at) where truncate_at should be max_len - 3.
pub fn truncate(s: &str, max_len: usize, truncate_at: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..truncate_at])
    } else {
        s.to_string()
    }
}

// ==================== Table Helpers ====================

/// Header definition for create_table
pub struct TableHeader<'a> {
    pub name: &'a str,
    pub color: Color,
}

impl<'a> TableHeader<'a> {
    pub fn new(name: &'a str, color: Color) -> Self {
        Self { name, color }
    }
}

/// Create a consistently styled table with the given headers.
pub fn create_table(headers: &[TableHeader]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::new(h.name).fg(h.color))
        .collect();

    table.set_header(header_cells);
    table
}

// ==================== Workflow Status Helpers ====================

/// Get a colored status icon for GitHub workflow runs.
/// Returns the appropriate icon based on status and conclusion.
pub fn workflow_status_icon(status: &str, conclusion: Option<&str>) -> ColoredString {
    match (status, conclusion) {
        ("completed", Some("success")) => "✓".green(),
        ("completed", Some("failure")) => "✗".red(),
        ("completed", Some("cancelled")) => "⊘".dimmed(),
        ("in_progress", _) => "●".yellow(),
        ("queued", _) | ("waiting", _) => "○".blue(),
        _ => "?".white(),
    }
}

/// Colorize a log line based on its content (ERROR, WARN, DEBUG).
pub fn colorize_log_line(line: &str) -> String {
    if line.contains("ERROR") || line.contains("error") || line.contains("Error") {
        line.red().to_string()
    } else if line.contains("WARN") || line.contains("warn") || line.contains("Warning") {
        line.yellow().to_string()
    } else if line.contains("DEBUG") || line.contains("debug") {
        line.dimmed().to_string()
    } else {
        line.to_string()
    }
}

// ==================== Config Helpers ====================

/// Get the path to a config file in the hu config directory.
pub fn get_config_path(filename: &str) -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    Ok(config_dir.join("hu").join(filename))
}

/// Load a JSON config file, returning default if it doesn't exist.
pub fn load_json_config<T: Default + DeserializeOwned>(filename: &str) -> Result<T> {
    let path = get_config_path(filename)?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))
    } else {
        Ok(T::default())
    }
}

/// Save a config object to a JSON file.
pub fn save_json_config<T: Serialize>(filename: &str, config: &T) -> Result<()> {
    let path = get_config_path(filename)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
    }
    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write config file: {:?}", path))?;
    Ok(())
}
