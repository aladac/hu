use super::colors::FileColors;
use super::types::{icon_for_entry, FileType};
use crossterm::style::{Attribute, Color, Stylize};
use ratatui::style::Style;
use std::sync::LazyLock;

static FILE_COLORS: LazyLock<FileColors> = LazyLock::new(FileColors::new);

/// Process raw GNU ls output and add icons + colors.
/// Handles both single-column (-1, default piped) and long (-l) formats.
pub fn enhance_output(raw: &str, is_long: bool) -> String {
    if raw.is_empty() {
        return String::new();
    }

    raw.lines()
        .map(|line| {
            if is_long {
                enhance_long_line(line)
            } else {
                enhance_simple_line(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Enhance a single-column line: "filename" or "filename/" etc.
fn enhance_simple_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Skip "total NNN" lines
    if trimmed.starts_with("total ") {
        return line.to_string();
    }

    let (name, file_type) = parse_name_and_type(trimmed);
    let icon = icon_for_entry(name, file_type);
    let color = color_for_type(name, file_type);

    format!("{} {}", icon, colorize(name, color, file_type))
}

/// Enhance a long-listing line: "drwxr-xr-x 2 user group 4.0K Feb 18 12:00 dirname/"
fn enhance_long_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // "total NNN" header
    if trimmed.starts_with("total ") {
        return line.to_string();
    }

    // Split at the filename. In long format, the filename is the last field.
    // For symlinks it looks like: "name -> target"
    // We find the filename by splitting on whitespace and taking from column 8+
    // (permissions, nlink, owner, group, size, month, day, time/year, name...)
    let parts: Vec<&str> = trimmed.splitn(9, char::is_whitespace).collect();
    if parts.len() < 9 {
        return line.to_string();
    }

    let prefix = &trimmed[..trimmed.len() - parts[8].len()];
    let name_part = parts[8];

    // Handle symlink: "name -> target"
    let display_name = if let Some(arrow_pos) = name_part.find(" -> ") {
        let link_name = &name_part[..arrow_pos];
        let target = &name_part[arrow_pos..];
        let (name, _file_type) = parse_name_and_type(link_name);
        let icon = icon_for_entry(name, FileType::Symlink);
        let color = FILE_COLORS.symlink();
        format!(
            "{} {}{}",
            icon,
            colorize(name, color, FileType::Symlink),
            target.with(Color::DarkGrey)
        )
    } else {
        let (name, file_type) = parse_name_and_type(name_part);
        let icon = icon_for_entry(name, file_type);
        let color = color_for_type(name, file_type);
        format!("{} {}", icon, colorize(name, color, file_type))
    };

    format!("{}{}", prefix, display_name)
}

/// Parse a filename and determine its FileType from classify indicator.
fn parse_name_and_type(name: &str) -> (&str, FileType) {
    if name.is_empty() {
        return (name, FileType::Regular);
    }

    let last_char = name.chars().last().unwrap_or(' ');
    let file_type = FileType::from_classify_char(last_char);

    if file_type != FileType::Regular {
        // Strip the classify indicator
        let clean = &name[..name.len() - last_char.len_utf8()];
        (clean, file_type)
    } else {
        (name, FileType::Regular)
    }
}

/// Get the crossterm color for a given file type and name.
fn color_for_type(name: &str, file_type: FileType) -> Color {
    match file_type {
        FileType::Directory => FILE_COLORS.directory(),
        FileType::Symlink => FILE_COLORS.symlink(),
        FileType::Executable => FILE_COLORS.executable(),
        FileType::Pipe => FILE_COLORS.pipe(),
        FileType::Socket => FILE_COLORS.socket(),
        FileType::Regular => {
            let ext = name.rsplit('.').next().unwrap_or("");
            let color = FILE_COLORS.for_extension(ext);
            if matches!(color, Color::Reset) {
                Color::White
            } else {
                color
            }
        }
    }
}

/// Apply color and attributes via crossterm Stylize.
fn colorize(text: &str, color: Color, file_type: FileType) -> String {
    let styled = text.with(color);
    match file_type {
        FileType::Directory | FileType::Executable => styled.attribute(Attribute::Bold).to_string(),
        _ => styled.to_string(),
    }
}

/// Get a ratatui Style for a file type (used for programmatic access).
#[allow(dead_code)]
pub fn style_for_type(file_type: FileType) -> Style {
    let color = match file_type {
        FileType::Directory => convert_color(FILE_COLORS.directory()),
        FileType::Symlink => convert_color(FILE_COLORS.symlink()),
        FileType::Executable => convert_color(FILE_COLORS.executable()),
        FileType::Pipe => convert_color(FILE_COLORS.pipe()),
        FileType::Socket => convert_color(FILE_COLORS.socket()),
        FileType::Regular => ratatui::style::Color::White,
    };

    let style = Style::default().fg(color);
    match file_type {
        FileType::Directory | FileType::Executable => {
            style.add_modifier(ratatui::style::Modifier::BOLD)
        }
        _ => style,
    }
}

/// Convert crossterm Color to ratatui Color.
#[allow(dead_code)]
fn convert_color(c: Color) -> ratatui::style::Color {
    match c {
        Color::Rgb { r, g, b } => ratatui::style::Color::Rgb(r, g, b),
        Color::Magenta => ratatui::style::Color::Magenta,
        Color::Reset => ratatui::style::Color::Reset,
        _ => ratatui::style::Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhance_empty() {
        assert_eq!(enhance_output("", false), "");
        assert_eq!(enhance_output("", true), "");
    }

    #[test]
    fn enhance_simple_file() {
        let out = enhance_simple_line("main.rs");
        assert!(out.contains("main.rs"));
        // Should contain the Rust icon
        assert!(out.contains('\u{e7a8}'));
    }

    #[test]
    fn enhance_simple_directory() {
        let out = enhance_simple_line("src/");
        assert!(out.contains("src"));
        // Should contain directory icon
        assert!(out.contains('\u{f115}'));
    }

    #[test]
    fn enhance_simple_executable() {
        let out = enhance_simple_line("run*");
        assert!(out.contains("run"));
    }

    #[test]
    fn enhance_simple_symlink() {
        let out = enhance_simple_line("link@");
        assert!(out.contains("link"));
    }

    #[test]
    fn enhance_simple_empty_line() {
        assert_eq!(enhance_simple_line(""), "");
        assert_eq!(enhance_simple_line("  "), "");
    }

    #[test]
    fn enhance_total_line_passthrough() {
        let out = enhance_simple_line("total 42");
        assert_eq!(out, "total 42");
    }

    #[test]
    fn enhance_long_total_passthrough() {
        let out = enhance_long_line("total 128");
        assert_eq!(out, "total 128");
    }

    #[test]
    fn enhance_long_regular_file() {
        let line = "-rw-r--r-- 1 chi staff 1.2K Feb 18 12:00 main.rs";
        let out = enhance_long_line(line);
        assert!(out.contains("main.rs"));
        assert!(out.contains('\u{e7a8}')); // Rust icon
    }

    #[test]
    fn enhance_long_directory() {
        let line = "drwxr-xr-x 5 chi staff 160B Feb 18 12:00 src/";
        let out = enhance_long_line(line);
        assert!(out.contains("src"));
        assert!(out.contains('\u{f115}')); // Dir icon
    }

    #[test]
    fn enhance_long_symlink() {
        let line = "lrwxr-xr-x 1 chi staff 24B Feb 18 12:00 link -> /target/path";
        let out = enhance_long_line(line);
        assert!(out.contains("link"));
        assert!(out.contains("target"));
    }

    #[test]
    fn enhance_long_short_line_passthrough() {
        let out = enhance_long_line("short");
        assert_eq!(out, "short");
    }

    #[test]
    fn enhance_long_empty_passthrough() {
        assert_eq!(enhance_long_line(""), "");
    }

    #[test]
    fn enhance_output_multi_line() {
        let raw = "src/\nmain.rs\nCargo.toml";
        let out = enhance_output(raw, false);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn enhance_output_long_multi() {
        let raw = "total 8\n-rw-r--r-- 1 chi staff 100B Feb 18 12:00 file.txt";
        let out = enhance_output(raw, true);
        assert!(out.contains("total 8"));
        assert!(out.contains("file.txt"));
    }

    #[test]
    fn parse_name_regular() {
        let (name, ft) = parse_name_and_type("main.rs");
        assert_eq!(name, "main.rs");
        assert_eq!(ft, FileType::Regular);
    }

    #[test]
    fn parse_name_directory() {
        let (name, ft) = parse_name_and_type("src/");
        assert_eq!(name, "src");
        assert_eq!(ft, FileType::Directory);
    }

    #[test]
    fn parse_name_executable() {
        let (name, ft) = parse_name_and_type("script*");
        assert_eq!(name, "script");
        assert_eq!(ft, FileType::Executable);
    }

    #[test]
    fn parse_name_symlink() {
        let (name, ft) = parse_name_and_type("link@");
        assert_eq!(name, "link");
        assert_eq!(ft, FileType::Symlink);
    }

    #[test]
    fn parse_name_pipe() {
        let (name, ft) = parse_name_and_type("fifo|");
        assert_eq!(name, "fifo");
        assert_eq!(ft, FileType::Pipe);
    }

    #[test]
    fn parse_name_socket() {
        let (name, ft) = parse_name_and_type("sock=");
        assert_eq!(name, "sock");
        assert_eq!(ft, FileType::Socket);
    }

    #[test]
    fn parse_name_empty() {
        let (name, ft) = parse_name_and_type("");
        assert_eq!(name, "");
        assert_eq!(ft, FileType::Regular);
    }

    #[test]
    fn color_for_type_directory() {
        let c = color_for_type("src", FileType::Directory);
        assert!(matches!(c, Color::Rgb { .. }));
    }

    #[test]
    fn color_for_type_regular_with_ext() {
        let c = color_for_type("main.rs", FileType::Regular);
        assert!(matches!(c, Color::Rgb { .. }));
    }

    #[test]
    fn color_for_type_regular_unknown_ext() {
        let c = color_for_type("file.xyz123", FileType::Regular);
        assert_eq!(c, Color::White);
    }

    #[test]
    fn colorize_bold_for_dirs() {
        let out = colorize("src", Color::Blue, FileType::Directory);
        // Should contain ANSI bold
        assert!(out.contains("\x1b["));
    }

    #[test]
    fn colorize_no_bold_for_regular() {
        let out = colorize("file.txt", Color::White, FileType::Regular);
        assert!(out.contains("file.txt"));
    }

    #[test]
    fn style_for_type_coverage() {
        // Just ensure all variants produce a style
        let variants = [
            FileType::Directory,
            FileType::Symlink,
            FileType::Executable,
            FileType::Pipe,
            FileType::Socket,
            FileType::Regular,
        ];
        for ft in variants {
            let _style = style_for_type(ft);
        }
    }

    #[test]
    fn convert_color_rgb() {
        let c = convert_color(Color::Rgb { r: 255, g: 0, b: 0 });
        assert!(matches!(c, ratatui::style::Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn convert_color_magenta() {
        let c = convert_color(Color::Magenta);
        assert!(matches!(c, ratatui::style::Color::Magenta));
    }

    #[test]
    fn convert_color_reset() {
        let c = convert_color(Color::Reset);
        assert!(matches!(c, ratatui::style::Color::Reset));
    }

    #[test]
    fn convert_color_other() {
        let c = convert_color(Color::Red);
        assert!(matches!(c, ratatui::style::Color::White));
    }
}
