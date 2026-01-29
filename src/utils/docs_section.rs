use anyhow::{Context, Result};
use regex::Regex;
use std::fs;

/// Extract a section from markdown content by heading
pub fn extract_section(content: &str, heading: &str) -> Option<String> {
    let heading_lower = heading.to_lowercase();
    let heading_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let mut section_start: Option<(usize, u8)> = None;
    let mut section_end: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = heading_re.captures(line) {
            let level = caps.get(1).unwrap().as_str().len() as u8;
            let text = caps.get(2).unwrap().as_str();

            if let Some((_, start_level)) = section_start {
                // We're in a section - check if this heading ends it
                if level <= start_level {
                    section_end = Some(i);
                    break;
                }
            } else if text.to_lowercase() == heading_lower
                || text.to_lowercase().contains(&heading_lower)
            {
                // Found the section
                section_start = Some((i, level));
            }
        }
    }

    // If we found the start but not the end, section goes to end of file
    if let Some((start, _)) = section_start {
        let end = section_end.unwrap_or(lines.len());
        let section_lines: Vec<&str> = lines[start..end].to_vec();
        return Some(section_lines.join("\n"));
    }

    None
}

/// Extract a section from a file by heading
pub fn extract_section_from_file(path: &str, heading: &str) -> Result<String> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?;

    extract_section(&content, heading)
        .ok_or_else(|| anyhow::anyhow!("Section not found: {}", heading))
}

/// Extract a section by line range
#[cfg(test)]
pub fn extract_lines(content: &str, start: usize, end: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    // Convert 1-indexed line numbers to 0-indexed array indices
    // start is inclusive, end is exclusive
    let start_idx = start.saturating_sub(1).min(lines.len());
    let end_idx = end.saturating_sub(1).min(lines.len());

    if start_idx >= end_idx {
        return String::new();
    }

    lines[start_idx..end_idx].join("\n")
}

/// Extract a section from a file by line range
#[cfg(test)]
pub fn extract_lines_from_file(path: &str, start: usize, end: usize) -> Result<String> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?;
    Ok(extract_lines(&content, start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONTENT: &str = r#"# Main Title

Introduction paragraph.

## First Section

First section content.
More content here.

### Nested Section

Nested content.

## Second Section

Second section content.

### Another Nested

More nested.

## Third Section

Final content.
"#;

    #[test]
    fn extract_section_h1() {
        let section = extract_section(TEST_CONTENT, "Main Title").unwrap();
        assert!(section.starts_with("# Main Title"));
        // H1 section should include everything until another H1 (none here)
        // or end of file
    }

    #[test]
    fn extract_section_h2() {
        let section = extract_section(TEST_CONTENT, "First Section").unwrap();
        assert!(section.starts_with("## First Section"));
        assert!(section.contains("First section content"));
        assert!(section.contains("### Nested Section"));
        // Should NOT include "## Second Section"
        assert!(!section.contains("## Second Section"));
    }

    #[test]
    fn extract_section_h3() {
        let section = extract_section(TEST_CONTENT, "Nested Section").unwrap();
        assert!(section.starts_with("### Nested Section"));
        assert!(section.contains("Nested content"));
        // Should end at "## Second Section"
        assert!(!section.contains("## Second Section"));
    }

    #[test]
    fn extract_section_last() {
        let section = extract_section(TEST_CONTENT, "Third Section").unwrap();
        assert!(section.starts_with("## Third Section"));
        assert!(section.contains("Final content"));
    }

    #[test]
    fn extract_section_case_insensitive() {
        let section = extract_section(TEST_CONTENT, "first section").unwrap();
        assert!(section.starts_with("## First Section"));
    }

    #[test]
    fn extract_section_partial_match() {
        let section = extract_section(TEST_CONTENT, "Nested").unwrap();
        // Should match first "Nested Section"
        assert!(section.starts_with("### Nested Section"));
    }

    #[test]
    fn extract_section_not_found() {
        let section = extract_section(TEST_CONTENT, "Nonexistent");
        assert!(section.is_none());
    }

    #[test]
    fn extract_section_empty_content() {
        let section = extract_section("", "Any");
        assert!(section.is_none());
    }

    #[test]
    fn extract_section_no_headings() {
        let content = "Just some text\nNo headings here\n";
        let section = extract_section(content, "Test");
        assert!(section.is_none());
    }

    #[test]
    fn extract_lines_basic() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let section = extract_lines(content, 2, 4);
        assert_eq!(section, "line2\nline3");
    }

    #[test]
    fn extract_lines_from_start() {
        let content = "line1\nline2\nline3";
        let section = extract_lines(content, 1, 2);
        assert_eq!(section, "line1");
    }

    #[test]
    fn extract_lines_to_end() {
        let content = "line1\nline2\nline3";
        let section = extract_lines(content, 2, 100);
        assert_eq!(section, "line2\nline3");
    }

    #[test]
    fn extract_lines_out_of_bounds() {
        let content = "line1\nline2";
        let section = extract_lines(content, 10, 20);
        assert_eq!(section, "");
    }

    #[test]
    fn extract_lines_invalid_range() {
        let content = "line1\nline2\nline3";
        let section = extract_lines(content, 5, 2);
        assert_eq!(section, "");
    }

    #[test]
    fn extract_lines_empty_content() {
        let section = extract_lines("", 1, 10);
        assert_eq!(section, "");
    }

    // File-based tests
    #[test]
    fn extract_section_from_file_cargo_toml() {
        // Cargo.toml doesn't have markdown headings, so this should fail
        let result = extract_section_from_file(
            concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
            "dependencies",
        );
        assert!(result.is_err());
    }

    #[test]
    fn extract_section_from_file_not_found() {
        let result = extract_section_from_file("/nonexistent/file.md", "Test");
        assert!(result.is_err());
    }

    #[test]
    fn extract_lines_from_file_cargo_toml() {
        let result =
            extract_lines_from_file(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"), 1, 5);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn extract_lines_from_file_not_found() {
        let result = extract_lines_from_file("/nonexistent/file.md", 1, 10);
        assert!(result.is_err());
    }
}
