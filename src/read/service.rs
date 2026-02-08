use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::around::extract_lines_around;
use super::cli::ReadArgs;
use super::diff::git_diff;
use super::interface::extract_interface;
use super::outline::extract_outline;
use super::types::ReadOutput;

/// Run the read command - returns data, never prints
pub fn run(args: ReadArgs) -> Result<ReadOutput> {
    let path = resolve_path(&args.path)?;
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    if let Some(center) = args.around {
        // Lines around a specific line
        let (lines, total_lines) = extract_lines_around(&content, center, args.context);
        Ok(ReadOutput::Around {
            lines,
            center,
            total_lines,
        })
    } else if args.diff {
        // Git diff
        let commit = if args.commit == "HEAD" {
            None
        } else {
            Some(args.commit.as_str())
        };
        let diff = git_diff(path.to_str().unwrap_or(""), commit)?;
        Ok(ReadOutput::Diff(diff))
    } else if args.interface {
        // Public interface
        let items = extract_interface(&content, path.to_str().unwrap_or(""));
        Ok(ReadOutput::Interface(items))
    } else if args.outline {
        // File outline
        let outline = extract_outline(&content, path.to_str().unwrap_or(""));
        Ok(ReadOutput::Outline(outline))
    } else {
        // Full file content
        Ok(ReadOutput::Full(content))
    }
}

/// Resolve a path to absolute
fn resolve_path(path_str: &str) -> Result<std::path::PathBuf> {
    let path = Path::new(path_str);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let resolved = cwd.join(path);
        if !resolved.exists() {
            anyhow::bail!("File not found: {}", path_str);
        }
        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_path_absolute() {
        let result = resolve_path("/tmp");
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_path_relative() {
        // Cargo.toml exists in project root
        let result = resolve_path("Cargo.toml");
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_path_not_found() {
        let result = resolve_path("nonexistent_file_xyz.abc");
        assert!(result.is_err());
    }

    // Integration tests - verify correct ReadOutput variant is returned
    #[test]
    fn run_returns_outline() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: true,
            interface: false,
            around: None,
            context: 10,
            diff: false,
            commit: "HEAD".to_string(),
        };
        let result = run(args).unwrap();
        assert!(matches!(result, ReadOutput::Outline(_)));
    }

    #[test]
    fn run_returns_around() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: false,
            interface: false,
            around: Some(5),
            context: 3,
            diff: false,
            commit: "HEAD".to_string(),
        };
        let result = run(args).unwrap();
        assert!(matches!(result, ReadOutput::Around { .. }));
    }

    #[test]
    fn run_returns_full() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: false,
            interface: false,
            around: None,
            context: 10,
            diff: false,
            commit: "HEAD".to_string(),
        };
        let result = run(args).unwrap();
        assert!(matches!(result, ReadOutput::Full(_)));
    }

    #[test]
    fn run_returns_interface() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs").to_string(),
            outline: false,
            interface: true,
            around: None,
            context: 10,
            diff: false,
            commit: "HEAD".to_string(),
        };
        let result = run(args).unwrap();
        assert!(matches!(result, ReadOutput::Interface(_)));
    }

    #[test]
    fn run_returns_diff() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: false,
            interface: false,
            around: None,
            context: 10,
            diff: true,
            commit: "HEAD".to_string(),
        };
        let result = run(args).unwrap();
        assert!(matches!(result, ReadOutput::Diff(_)));
    }

    #[test]
    fn run_diff_specific_commit() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: false,
            interface: false,
            around: None,
            context: 10,
            diff: true,
            commit: "HEAD~1".to_string(),
        };
        // This may fail if HEAD~1 doesn't exist, but shouldn't panic
        let _ = run(args);
    }
}
