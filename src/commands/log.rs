use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::utils::{colorize_log_line, expand_tilde, print_header, LOG_POLL_INTERVAL_MS};

pub fn view(
    path: &str,
    follow: bool,
    lines: usize,
    grep: Option<&str>,
    colorize: bool,
) -> Result<()> {
    let path = expand_tilde(path);
    let path = PathBuf::from(&path);

    if !path.exists() {
        bail!("Log file not found: {:?}", path);
    }

    print_header(&format!(
        "Log: {}",
        path.display().to_string().bright_cyan()
    ));

    if follow {
        println!("  {} to stop", "Press Ctrl+C".yellow());
        println!();

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            r.store(false, Ordering::Relaxed);
            println!("\n{}", "Stopped.".yellow());
            std::process::exit(0);
        })
        .context("Failed to set Ctrl+C handler")?;

        // First show last N lines
        let file = std::fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
        let start = all_lines.len().saturating_sub(lines);

        for line in &all_lines[start..] {
            if let Some(pattern) = grep {
                if !line.contains(pattern) {
                    continue;
                }
            }
            if colorize {
                println!("{}", colorize_log_line(line));
            } else {
                println!("{}", line);
            }
        }

        // Now tail
        let mut last_size = std::fs::metadata(&path)?.len();
        let mut last_pos = last_size;

        while running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(LOG_POLL_INTERVAL_MS));

            let current_size = std::fs::metadata(&path)
                .map(|m| m.len())
                .unwrap_or(last_size);

            if current_size > last_pos {
                let file = std::fs::File::open(&path)?;
                let mut reader = BufReader::new(file);
                std::io::Seek::seek(&mut reader, std::io::SeekFrom::Start(last_pos))?;

                for line in reader.lines().map_while(Result::ok) {
                    if let Some(pattern) = grep {
                        if !line.contains(pattern) {
                            continue;
                        }
                    }
                    if colorize {
                        println!("{}", colorize_log_line(&line));
                    } else {
                        println!("{}", line);
                    }
                }

                last_pos = current_size;
            } else if current_size < last_size {
                // File was truncated/rotated
                last_pos = 0;
            }
            last_size = current_size;
        }
    } else {
        // Just show last N lines
        let file = std::fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
        let start = all_lines.len().saturating_sub(lines);

        for line in &all_lines[start..] {
            if let Some(pattern) = grep {
                if !line.contains(pattern) {
                    continue;
                }
            }
            if colorize {
                println!("{}", colorize_log_line(line));
            } else {
                println!("{}", line);
            }
        }

        println!();
        println!(
            "{} {} lines (use {} to follow)",
            "Showing last".dimmed(),
            (all_lines.len() - start).to_string().cyan(),
            "-f".yellow()
        );
    }

    Ok(())
}
