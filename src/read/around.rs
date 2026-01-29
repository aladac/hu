/// Extract lines around a center line with context
pub fn extract_lines_around(
    content: &str,
    center: usize,
    context: usize,
) -> (Vec<(usize, String)>, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    if total_lines == 0 || center == 0 {
        return (vec![], total_lines);
    }

    // Convert to 0-indexed
    let center_idx = center.saturating_sub(1);

    // Calculate range with clamping
    let start = center_idx.saturating_sub(context);
    let end = (center_idx + context + 1).min(total_lines);

    let result: Vec<(usize, String)> = lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| (start + i + 1, (*line).to_string()))
        .collect();

    (result, total_lines)
}

/// Format lines with line numbers and highlight center
pub fn format_lines_around(lines: &[(usize, String)], center: usize, total_lines: usize) -> String {
    if lines.is_empty() {
        return "No content".to_string();
    }

    let width = total_lines.to_string().len();
    let mut output = Vec::new();

    for (num, line) in lines {
        let marker = if *num == center { ">" } else { " " };
        output.push(format!(
            "{}{:>width$}: {}",
            marker,
            num,
            line,
            width = width
        ));
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_basic() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let (lines, total) = extract_lines_around(content, 3, 1);
        assert_eq!(total, 5);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], (2, "line2".to_string()));
        assert_eq!(lines[1], (3, "line3".to_string()));
        assert_eq!(lines[2], (4, "line4".to_string()));
    }

    #[test]
    fn extract_at_start() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let (lines, _) = extract_lines_around(content, 1, 2);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], (1, "line1".to_string()));
    }

    #[test]
    fn extract_at_end() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let (lines, _) = extract_lines_around(content, 5, 2);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[2], (5, "line5".to_string()));
    }

    #[test]
    fn extract_beyond_bounds() {
        let content = "line1\nline2\nline3";
        let (lines, _) = extract_lines_around(content, 2, 10);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn extract_empty_content() {
        let (lines, total) = extract_lines_around("", 1, 5);
        assert!(lines.is_empty());
        assert_eq!(total, 0);
    }

    #[test]
    fn extract_zero_center() {
        let content = "line1\nline2";
        let (lines, _) = extract_lines_around(content, 0, 5);
        assert!(lines.is_empty());
    }

    #[test]
    fn extract_zero_context() {
        let content = "line1\nline2\nline3";
        let (lines, _) = extract_lines_around(content, 2, 0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], (2, "line2".to_string()));
    }

    #[test]
    fn extract_single_line() {
        let content = "only line";
        let (lines, total) = extract_lines_around(content, 1, 5);
        assert_eq!(lines.len(), 1);
        assert_eq!(total, 1);
    }

    #[test]
    fn format_basic() {
        let lines = vec![
            (9, "line9".to_string()),
            (10, "line10".to_string()),
            (11, "line11".to_string()),
        ];
        let output = format_lines_around(&lines, 10, 11);
        assert!(output.contains(">10: line10"));
        assert!(output.contains(" 9: line9"));
        assert!(output.contains("11: line11"));
    }

    #[test]
    fn format_empty() {
        let lines: Vec<(usize, String)> = vec![];
        let output = format_lines_around(&lines, 1, 0);
        assert_eq!(output, "No content");
    }

    #[test]
    fn format_line_numbers_aligned() {
        let lines = vec![
            (1, "first".to_string()),
            (10, "tenth".to_string()),
            (100, "hundredth".to_string()),
        ];
        let output = format_lines_around(&lines, 10, 100);
        // All line numbers should be right-aligned with same width
        let output_lines: Vec<&str> = output.lines().collect();
        assert!(output_lines[0].starts_with("   1:"));
        assert!(output_lines[1].starts_with("> 10:"));
        assert!(output_lines[2].starts_with(" 100:"));
    }

    #[test]
    fn format_preserves_content() {
        let lines = vec![
            (1, "  indented content".to_string()),
            (2, "normal".to_string()),
        ];
        let output = format_lines_around(&lines, 1, 10);
        assert!(output.contains("  indented content"));
    }
}
