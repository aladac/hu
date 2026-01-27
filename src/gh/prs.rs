use anyhow::Result;

use super::client::GithubClient;
use super::types::CiStatus;

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Handle the `hu gh prs` command
pub async fn run() -> Result<()> {
    let client = GithubClient::new()?;
    let mut prs = client.list_user_prs().await?;

    if prs.is_empty() {
        println!("No open pull requests found.");
        return Ok(());
    }

    // Fetch CI status for each PR
    for pr in &mut prs {
        let parts: Vec<&str> = pr.repo_full_name.split('/').collect();
        if parts.len() == 2 {
            if let Ok(status) = client.get_ci_status(parts[0], parts[1], pr.number).await {
                pr.ci_status = Some(status);
            }
        }
    }

    print_prs_table(&prs);
    Ok(())
}

fn get_terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}

fn print_prs_table(prs: &[super::types::PullRequest]) {
    let term_width = get_terminal_width();

    // Calculate max link length
    let max_link_len = prs.iter().map(|p| p.html_url.len()).max().unwrap_or(40);

    // Layout: │ S │ Title... │ Link │
    // Borders take: 1 + 1 + 3 + 3 + 1 = 9 chars (│ S │ ... │ ... │)
    let status_col = 1;
    let border_overhead = 10; // "│ " + " │ " + " │ " + "│"

    let available = term_width.saturating_sub(border_overhead + status_col + max_link_len);
    let title_width = available.max(20);
    let link_width = max_link_len;

    // Top border
    println!(
        "┌───┬{}┬{}┐",
        "─".repeat(title_width + 2),
        "─".repeat(link_width + 2)
    );

    // Rows
    for pr in prs {
        let status_icon = match pr.ci_status.unwrap_or(CiStatus::Unknown) {
            CiStatus::Success => format!("{}{}{}", GREEN, "✓", RESET),
            CiStatus::Pending => format!("{}{}{}", YELLOW, "◐", RESET),
            CiStatus::Failed => format!("{}{}{}", RED, "✗", RESET),
            CiStatus::Unknown => format!("{}{}{}", GRAY, "○", RESET),
        };

        let title = truncate(&pr.title, title_width);
        let link = format!("{}{}{}", GRAY, &pr.html_url, RESET);

        println!(
            "│ {} │ {:<width$} │ {} │",
            status_icon,
            title,
            link,
            width = title_width
        );
    }

    // Bottom border
    println!(
        "└───┴{}┴{}┘",
        "─".repeat(title_width + 2),
        "─".repeat(link_width + 2)
    );
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello w…");
    }

    #[test]
    fn truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn status_icons_render() {
        let _ = format!("{}✓{}", GREEN, RESET);
        let _ = format!("{}◐{}", YELLOW, RESET);
        let _ = format!("{}✗{}", RED, RESET);
    }
}
