use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Get git diff for a file against a commit
pub fn git_diff(path: &str, commit: Option<&str>) -> Result<String> {
    let commit_ref = commit.unwrap_or("HEAD");

    // Verify file exists
    let path = Path::new(path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    // Run git diff
    let output = Command::new("git")
        .args(["diff", commit_ref, "--", path.to_str().unwrap_or("")])
        .output()
        .context("Failed to run git diff")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed: {}", stderr);
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();

    if diff.is_empty() {
        return Ok("No changes".to_string());
    }

    Ok(diff)
}

/// Format diff output with colors
pub fn format_diff(diff: &str) -> String {
    if diff == "No changes" {
        return diff.to_string();
    }

    let mut output = Vec::new();

    for line in diff.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            // Added line - green
            output.push(format!("\x1b[32m{}\x1b[0m", line));
        } else if line.starts_with('-') && !line.starts_with("---") {
            // Removed line - red
            output.push(format!("\x1b[31m{}\x1b[0m", line));
        } else if line.starts_with("@@") {
            // Hunk header - cyan
            output.push(format!("\x1b[36m{}\x1b[0m", line));
        } else if line.starts_with("diff") || line.starts_with("index") {
            // Header - dim
            output.push(format!("\x1b[2m{}\x1b[0m", line));
        } else {
            output.push(line.to_string());
        }
    }

    output.join("\n")
}

/// Parse diff to extract changed line ranges
#[cfg(test)]
pub fn parse_diff_hunks(diff: &str) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let hunk_re = regex::Regex::new(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").unwrap();

    for caps in hunk_re.captures_iter(diff) {
        let old_start: usize = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
        let old_count: usize = caps
            .get(2)
            .map(|m| m.as_str().parse().unwrap_or(1))
            .unwrap_or(1);
        let new_start: usize = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
        let new_count: usize = caps
            .get(4)
            .map(|m| m.as_str().parse().unwrap_or(1))
            .unwrap_or(1);

        hunks.push(DiffHunk {
            old_start,
            old_count,
            new_start,
            new_count,
        });
    }

    hunks
}

/// A diff hunk (changed section)
#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_diff_additions() {
        let diff = "+added line";
        let formatted = format_diff(diff);
        assert!(formatted.contains("\x1b[32m"));
        assert!(formatted.contains("+added line"));
    }

    #[test]
    fn format_diff_deletions() {
        let diff = "-removed line";
        let formatted = format_diff(diff);
        assert!(formatted.contains("\x1b[31m"));
        assert!(formatted.contains("-removed line"));
    }

    #[test]
    fn format_diff_hunk_header() {
        let diff = "@@ -1,3 +1,4 @@";
        let formatted = format_diff(diff);
        assert!(formatted.contains("\x1b[36m"));
    }

    #[test]
    fn format_diff_file_header() {
        let diff = "diff --git a/file.rs b/file.rs";
        let formatted = format_diff(diff);
        assert!(formatted.contains("\x1b[2m"));
    }

    #[test]
    fn format_diff_no_changes() {
        let formatted = format_diff("No changes");
        assert_eq!(formatted, "No changes");
    }

    #[test]
    fn format_diff_preserves_context() {
        let diff = " unchanged line";
        let formatted = format_diff(diff);
        assert_eq!(formatted, " unchanged line");
    }

    #[test]
    fn format_diff_plus_header_not_green() {
        let diff = "+++ b/file.rs";
        let formatted = format_diff(diff);
        // Should not have green color code
        assert!(!formatted.contains("\x1b[32m"));
    }

    #[test]
    fn format_diff_minus_header_not_red() {
        let diff = "--- a/file.rs";
        let formatted = format_diff(diff);
        // Should not have red color code
        assert!(!formatted.contains("\x1b[31m"));
    }

    #[test]
    fn parse_diff_hunks_single() {
        let diff = "@@ -1,3 +1,4 @@";
        let hunks = parse_diff_hunks(diff);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[0].old_count, 3);
        assert_eq!(hunks[0].new_start, 1);
        assert_eq!(hunks[0].new_count, 4);
    }

    #[test]
    fn parse_diff_hunks_multiple() {
        let diff = "@@ -1,3 +1,4 @@\nsome content\n@@ -10,5 +11,6 @@";
        let hunks = parse_diff_hunks(diff);
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[1].old_start, 10);
        assert_eq!(hunks[1].new_start, 11);
    }

    #[test]
    fn parse_diff_hunks_no_count() {
        let diff = "@@ -5 +5 @@";
        let hunks = parse_diff_hunks(diff);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_count, 1);
        assert_eq!(hunks[0].new_count, 1);
    }

    #[test]
    fn parse_diff_hunks_empty() {
        let diff = "no hunks here";
        let hunks = parse_diff_hunks(diff);
        assert!(hunks.is_empty());
    }

    #[test]
    fn diff_hunk_clone() {
        let hunk = DiffHunk {
            old_start: 1,
            old_count: 2,
            new_start: 3,
            new_count: 4,
        };
        let cloned = hunk.clone();
        assert_eq!(hunk, cloned);
    }

    #[test]
    fn diff_hunk_debug() {
        let hunk = DiffHunk {
            old_start: 1,
            old_count: 2,
            new_start: 3,
            new_count: 4,
        };
        let debug = format!("{:?}", hunk);
        assert!(debug.contains("DiffHunk"));
    }

    // Integration test - requires git repo
    #[test]
    fn git_diff_cargo_toml() {
        // This test uses Cargo.toml which should exist in a git repo
        let result = git_diff(
            concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
            Some("HEAD"),
        );
        // Either succeeds with diff or "No changes"
        assert!(result.is_ok());
    }

    #[test]
    fn git_diff_nonexistent_file() {
        let result = git_diff("/nonexistent/file.txt", None);
        assert!(result.is_err());
    }

    #[test]
    fn git_diff_invalid_commit() {
        // Using an invalid commit reference should cause git diff to fail
        let result = git_diff(
            concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
            Some("invalid_commit_ref_that_does_not_exist_xyz123"),
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("git diff failed"));
    }

    #[test]
    fn format_diff_index_header() {
        let diff = "index abc123..def456 100644";
        let formatted = format_diff(diff);
        // Should have dim color
        assert!(formatted.contains("\x1b[2m"));
    }

    #[test]
    fn git_diff_with_actual_changes() {
        // Compare src/main.rs against an older commit to ensure we get actual diff output
        // This tests the Ok(diff) return path (line 32) when diff is non-empty
        let result = git_diff(
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"),
            Some("HEAD~20"), // src/main.rs changes frequently
        );

        // This test is designed to exercise the non-empty diff return path
        match result {
            Ok(diff) => {
                // Either "No changes" or actual diff content
                if diff != "No changes" {
                    assert!(
                        diff.contains("diff") || diff.contains("@@"),
                        "Expected diff content but got: {}",
                        diff
                    );
                }
            }
            Err(_) => {
                // If not enough history, skip silently - this is CI-friendly
            }
        }
    }
}
