use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;
use std::time::Duration;

pub const ANSI_COLORS: [&str; 6] = ["red", "green", "yellow", "blue", "magenta", "cyan"];

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
        if let Some(home) = std::env::var("HOME").ok() {
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
