use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::around::{extract_lines_around, format_lines_around};
use super::cli::ReadArgs;
use super::diff::{format_diff, git_diff};
use super::interface::extract_interface;
use super::outline::extract_outline;
use super::types::{FileOutline, ItemKind, OutlineItem};

/// Run the read command
pub fn run(args: ReadArgs) -> Result<()> {
    let path = resolve_path(&args.path)?;
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    if let Some(center) = args.around {
        // Show lines around a specific line
        let (lines, total) = extract_lines_around(&content, center, args.context);
        let output = format_lines_around(&lines, center, total);
        println!("{}", output);
    } else if args.diff {
        // Show git diff
        let commit = if args.commit == "HEAD" {
            None
        } else {
            Some(args.commit.as_str())
        };
        let diff = git_diff(path.to_str().unwrap_or(""), commit)?;
        let output = format_diff(&diff);
        println!("{}", output);
    } else if args.interface {
        // Show public interface
        let items = extract_interface(&content, path.to_str().unwrap_or(""));
        let output = format_interface(&items);
        println!("{}", output);
    } else if args.outline {
        // Show file outline
        let outline = extract_outline(&content, path.to_str().unwrap_or(""));
        let output = format_outline(&outline);
        println!("{}", output);
    } else {
        // Full file content
        print!("{}", content);
    }

    Ok(())
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

/// Format outline for display
pub fn format_outline(outline: &FileOutline) -> String {
    if outline.is_empty() {
        return "No outline items found".to_string();
    }

    let mut output = Vec::new();

    for item in &outline.items {
        let indent = "  ".repeat(item.level);
        let icon = item.kind.icon();
        let line_info = format!(":{}", item.line);
        output.push(format!("{}{} {}{}", indent, icon, item.text, line_info));
    }

    output.join("\n")
}

/// Format interface for display
pub fn format_interface(items: &[OutlineItem]) -> String {
    if items.is_empty() {
        return "No public interface items found".to_string();
    }

    let mut output = Vec::new();

    for item in items {
        let indent = "  ".repeat(item.level);
        let icon = item.kind.icon();
        output.push(format!("{}{} {} :L{}", indent, icon, item.text, item.line));
    }

    output.join("\n")
}

/// Format outline item kind as icon/prefix
fn _format_kind(kind: &ItemKind) -> &'static str {
    kind.icon()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_outline_empty() {
        let outline = FileOutline::new();
        let output = format_outline(&outline);
        assert_eq!(output, "No outline items found");
    }

    #[test]
    fn format_outline_single() {
        let mut outline = FileOutline::new();
        outline.push(OutlineItem::new(
            10,
            "pub fn test()".to_string(),
            0,
            ItemKind::Function,
        ));
        let output = format_outline(&outline);
        assert!(output.contains("fn pub fn test()"));
        assert!(output.contains(":10"));
    }

    #[test]
    fn format_outline_nested() {
        let mut outline = FileOutline::new();
        outline.push(OutlineItem::new(
            1,
            "impl Config".to_string(),
            0,
            ItemKind::Impl,
        ));
        outline.push(OutlineItem::new(
            2,
            "pub fn new()".to_string(),
            1,
            ItemKind::Function,
        ));
        let output = format_outline(&outline);
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines[0].starts_with("impl"));
        assert!(lines[1].starts_with("  fn")); // Indented
    }

    #[test]
    fn format_outline_markdown() {
        let mut outline = FileOutline::new();
        outline.push(OutlineItem::new(
            1,
            "Title".to_string(),
            0,
            ItemKind::Heading(1),
        ));
        outline.push(OutlineItem::new(
            5,
            "Section".to_string(),
            1,
            ItemKind::Heading(2),
        ));
        let output = format_outline(&outline);
        assert!(output.contains("# Title"));
        assert!(output.contains("  ## Section"));
    }

    #[test]
    fn format_interface_empty() {
        let items: Vec<OutlineItem> = vec![];
        let output = format_interface(&items);
        assert_eq!(output, "No public interface items found");
    }

    #[test]
    fn format_interface_single() {
        let items = vec![OutlineItem::new(
            10,
            "pub fn test()".to_string(),
            0,
            ItemKind::Function,
        )];
        let output = format_interface(&items);
        assert!(output.contains("fn pub fn test()"));
        assert!(output.contains(":L10"));
    }

    #[test]
    fn format_interface_multiple() {
        let items = vec![
            OutlineItem::new(1, "pub struct Config".to_string(), 0, ItemKind::Struct),
            OutlineItem::new(5, "pub fn new()".to_string(), 0, ItemKind::Function),
        ];
        let output = format_interface(&items);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
    }

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

    // Integration tests
    #[test]
    fn run_outline_cargo_toml() {
        // Cargo.toml is markdown-ish but won't have Rust outline
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: true,
            interface: false,
            around: None,
            context: 10,
            diff: false,
            commit: "HEAD".to_string(),
        };
        // Should not error, even if outline is empty
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn run_around_cargo_toml() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: false,
            interface: false,
            around: Some(5),
            context: 3,
            diff: false,
            commit: "HEAD".to_string(),
        };
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn run_full_cargo_toml() {
        let args = ReadArgs {
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string(),
            outline: false,
            interface: false,
            around: None,
            context: 10,
            diff: false,
            commit: "HEAD".to_string(),
        };
        let result = run(args);
        assert!(result.is_ok());
    }
}
