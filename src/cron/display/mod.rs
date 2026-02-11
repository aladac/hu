use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};

use super::types::CronJob;

#[cfg(test)]
mod tests;

/// Format job list as a pretty table
pub fn format_jobs(jobs: &[CronJob], json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(jobs).unwrap_or_else(|_| "[]".to_string());
    }

    if jobs.is_empty() {
        return "No cron jobs found".to_string();
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("#").fg(Color::DarkGrey),
            Cell::new("Schedule").fg(Color::DarkGrey),
            Cell::new("Time").fg(Color::DarkGrey),
            Cell::new("Command").fg(Color::DarkGrey),
            Cell::new("").fg(Color::DarkGrey), // hu marker
        ]);

    for (i, job) in jobs.iter().enumerate() {
        let schedule_display = job.schedule_name.as_deref().unwrap_or("-").to_string();

        let time_display = job.describe_time();

        let hu_marker = if job.is_hu_job {
            Cell::new("hu").fg(Color::Cyan)
        } else {
            Cell::new("")
        };

        let command_display = truncate_command(&job.command, 50);

        table.add_row(vec![
            Cell::new(i + 1).fg(Color::DarkGrey),
            Cell::new(schedule_display).fg(Color::Green),
            Cell::new(time_display).fg(Color::Yellow),
            Cell::new(command_display),
            hu_marker,
        ]);
    }

    table.to_string()
}

/// Format a single added job
pub fn format_added(job: &CronJob, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(job).unwrap_or_else(|_| "{}".to_string());
    }

    format!(
        "\x1b[32m\u{2713}\x1b[0m Added {} job: {} {}",
        job.schedule_name.as_deref().unwrap_or("cron"),
        job.expression,
        truncate_command(&job.command, 40)
    )
}

/// Format removed jobs
pub fn format_removed(jobs: &[CronJob], json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(jobs).unwrap_or_else(|_| "[]".to_string());
    }

    if jobs.is_empty() {
        return "No matching jobs found".to_string();
    }

    let mut output = format!(
        "\x1b[32m\u{2713}\x1b[0m Removed {} job{}:\n",
        jobs.len(),
        if jobs.len() == 1 { "" } else { "s" }
    );

    for job in jobs {
        output.push_str(&format!(
            "  - {} {}\n",
            job.expression,
            truncate_command(&job.command, 50)
        ));
    }

    output.trim_end().to_string()
}

/// Truncate a command string for display
fn truncate_command(cmd: &str, max_len: usize) -> String {
    if cmd.len() <= max_len {
        cmd.to_string()
    } else {
        format!("{}...", &cmd[..max_len - 3])
    }
}
