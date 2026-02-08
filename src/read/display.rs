//! Display formatting for read output (CLI-only)

use super::around::format_lines_around;
use super::diff::format_diff;
use super::types::{FileOutline, OutlineItem, ReadOutput};

/// Format ReadOutput for CLI display
pub fn format(output: &ReadOutput) -> String {
    match output {
        ReadOutput::Full(content) => content.clone(),
        ReadOutput::Outline(outline) => format_outline(outline),
        ReadOutput::Interface(items) => format_interface(items),
        ReadOutput::Around {
            lines,
            center,
            total_lines,
        } => format_lines_around(lines, *center, *total_lines),
        ReadOutput::Diff(diff) => format_diff(diff),
    }
}

/// Format outline for display
fn format_outline(outline: &FileOutline) -> String {
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
fn format_interface(items: &[OutlineItem]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::types::ItemKind;

    #[test]
    fn format_full_content() {
        let output = ReadOutput::Full("hello\nworld".to_string());
        let formatted = format(&output);
        assert_eq!(formatted, "hello\nworld");
    }

    #[test]
    fn format_empty_outline() {
        let output = ReadOutput::Outline(FileOutline::new());
        let formatted = format(&output);
        assert_eq!(formatted, "No outline items found");
    }

    #[test]
    fn format_outline_with_items() {
        let mut outline = FileOutline::new();
        outline.push(OutlineItem::new(
            10,
            "pub fn test()".to_string(),
            0,
            ItemKind::Function,
        ));
        let output = ReadOutput::Outline(outline);
        let formatted = format(&output);
        assert!(formatted.contains("fn pub fn test()"));
        assert!(formatted.contains(":10"));
    }

    #[test]
    fn format_nested_outline() {
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
        let output = ReadOutput::Outline(outline);
        let formatted = format(&output);
        let lines: Vec<&str> = formatted.lines().collect();
        assert!(lines[0].starts_with("impl"));
        assert!(lines[1].starts_with("  fn")); // Indented
    }

    #[test]
    fn format_empty_interface() {
        let output = ReadOutput::Interface(vec![]);
        let formatted = format(&output);
        assert_eq!(formatted, "No public interface items found");
    }

    #[test]
    fn format_interface_with_items() {
        let items = vec![OutlineItem::new(
            10,
            "pub fn test()".to_string(),
            0,
            ItemKind::Function,
        )];
        let output = ReadOutput::Interface(items);
        let formatted = format(&output);
        assert!(formatted.contains("fn pub fn test()"));
        assert!(formatted.contains(":L10"));
    }

    #[test]
    fn format_around_lines() {
        let output = ReadOutput::Around {
            lines: vec![
                (9, "line9".to_string()),
                (10, "line10".to_string()),
                (11, "line11".to_string()),
            ],
            center: 10,
            total_lines: 11, // width is 2, so format is ">10: line10"
        };
        let formatted = format(&output);
        assert!(formatted.contains(">10: line10"));
        assert!(formatted.contains(" 9: line9"));
    }

    #[test]
    fn format_diff_content() {
        let output = ReadOutput::Diff("+added line".to_string());
        let formatted = format(&output);
        assert!(formatted.contains("+added line"));
        // Should have green color for additions
        assert!(formatted.contains("\x1b[32m"));
    }

    #[test]
    fn format_diff_no_changes() {
        let output = ReadOutput::Diff("No changes".to_string());
        let formatted = format(&output);
        assert_eq!(formatted, "No changes");
    }
}
