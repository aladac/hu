use std::collections::HashMap;
use std::sync::LazyLock;

/// Nerd Font icons for file types, keyed by extension.
static EXTENSION_ICONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Languages
    m.insert("rs", "\u{e7a8}"); //
    m.insert("py", "\u{e73c}"); //
    m.insert("rb", "\u{e791}"); //
    m.insert("js", "\u{e74e}"); //
    m.insert("ts", "\u{e628}"); //
    m.insert("jsx", "\u{e7ba}"); //
    m.insert("tsx", "\u{e7ba}"); //
    m.insert("go", "\u{e627}"); //
    m.insert("java", "\u{e738}"); //
    m.insert("c", "\u{e61e}"); //
    m.insert("h", "\u{e61e}"); //
    m.insert("cpp", "\u{e61d}"); //
    m.insert("cc", "\u{e61d}"); //
    m.insert("hpp", "\u{e61d}"); //
    m.insert("cs", "\u{f81a}"); // 󰠚
    m.insert("swift", "\u{e755}"); //
    m.insert("kt", "\u{e634}"); //
    m.insert("dart", "\u{e798}"); //
    m.insert("lua", "\u{e620}"); //
    m.insert("php", "\u{e73d}"); //
    m.insert("sh", "\u{e795}"); //
    m.insert("bash", "\u{e795}"); //
    m.insert("zsh", "\u{e795}"); //
    m.insert("fish", "\u{e795}"); //
    m.insert("sql", "\u{e706}"); //
    m.insert("cu", "\u{e64b}"); //
    m.insert("cuh", "\u{e64b}"); //
    m.insert("r", "\u{f25d}"); //
    m.insert("scala", "\u{e737}"); //
    m.insert("elm", "\u{e62c}"); //
    m.insert("ex", "\u{e62d}"); //
    m.insert("exs", "\u{e62d}"); //
    m.insert("erl", "\u{e7b1}"); //
    m.insert("hs", "\u{e777}"); //
    m.insert("vim", "\u{e62b}"); //
    m.insert("zig", "\u{e6a9}"); //

    // Web
    m.insert("html", "\u{e736}"); //
    m.insert("htm", "\u{e736}"); //
    m.insert("css", "\u{e749}"); //
    m.insert("scss", "\u{e749}"); //
    m.insert("sass", "\u{e749}"); //
    m.insert("vue", "\u{e6a0}"); //
    m.insert("svelte", "\u{e697}"); //
    m.insert("wasm", "\u{e6a1}"); //

    // Data / Config
    m.insert("json", "\u{e60b}"); //
    m.insert("jsonl", "\u{e60b}"); //
    m.insert("yaml", "\u{e6a8}"); //
    m.insert("yml", "\u{e6a8}"); //
    m.insert("toml", "\u{e6b2}"); //
    m.insert("xml", "\u{e619}"); //
    m.insert("csv", "\u{f1c3}"); //
    m.insert("ini", "\u{e615}"); //
    m.insert("cfg", "\u{e615}"); //
    m.insert("conf", "\u{e615}"); //
    m.insert("env", "\u{e615}"); //

    // Docs
    m.insert("md", "\u{e73e}"); //
    m.insert("markdown", "\u{e73e}"); //
    m.insert("mdx", "\u{e73e}"); //
    m.insert("txt", "\u{f15c}"); //
    m.insert("pdf", "\u{f1c1}"); //
    m.insert("doc", "\u{f1c2}"); //
    m.insert("docx", "\u{f1c2}"); //
    m.insert("xls", "\u{f1c3}"); //
    m.insert("xlsx", "\u{f1c3}"); //
    m.insert("ppt", "\u{f1c4}"); //
    m.insert("pptx", "\u{f1c4}"); //

    // Images
    m.insert("png", "\u{f1c5}"); //
    m.insert("jpg", "\u{f1c5}"); //
    m.insert("jpeg", "\u{f1c5}"); //
    m.insert("gif", "\u{f1c5}"); //
    m.insert("webp", "\u{f1c5}"); //
    m.insert("bmp", "\u{f1c5}"); //
    m.insert("ico", "\u{f1c5}"); //
    m.insert("svg", "\u{f1c5}"); //
    m.insert("heic", "\u{f1c5}"); //

    // Audio
    m.insert("mp3", "\u{f1c7}"); //
    m.insert("wav", "\u{f1c7}"); //
    m.insert("flac", "\u{f1c7}"); //
    m.insert("ogg", "\u{f1c7}"); //
    m.insert("m4a", "\u{f1c7}"); //

    // Video
    m.insert("mp4", "\u{f1c8}"); //
    m.insert("mkv", "\u{f1c8}"); //
    m.insert("avi", "\u{f1c8}"); //
    m.insert("mov", "\u{f1c8}"); //
    m.insert("webm", "\u{f1c8}"); //

    // Archives
    m.insert("zip", "\u{f1c6}"); //
    m.insert("tar", "\u{f1c6}"); //
    m.insert("gz", "\u{f1c6}"); //
    m.insert("tgz", "\u{f1c6}"); //
    m.insert("bz2", "\u{f1c6}"); //
    m.insert("xz", "\u{f1c6}"); //
    m.insert("rar", "\u{f1c6}"); //
    m.insert("7z", "\u{f1c6}"); //
    m.insert("deb", "\u{e77d}"); //
    m.insert("rpm", "\u{e7bb}"); //

    // Compiled / Binary
    m.insert("o", "\u{e624}"); //
    m.insert("a", "\u{e624}"); //
    m.insert("so", "\u{e624}"); //
    m.insert("dylib", "\u{e624}"); //
    m.insert("dll", "\u{e624}"); //

    // Git
    m.insert("gitignore", "\u{e702}"); //
    m.insert("gitattributes", "\u{e702}");
    m.insert("gitmodules", "\u{e702}");

    // Docker
    m.insert("dockerfile", "\u{e7b0}"); //

    // Terraform
    m.insert("tf", "\u{e69a}"); //
    m.insert("tfvars", "\u{e69a}"); //

    // Databases
    m.insert("sqlite", "\u{e706}"); //
    m.insert("sqlite3", "\u{e706}"); //
    m.insert("db", "\u{e706}"); //

    // ML
    m.insert("onnx", "\u{e64b}"); //
    m.insert("pt", "\u{e64b}"); //
    m.insert("pth", "\u{e64b}"); //
    m.insert("safetensors", "\u{e64b}");

    // Fonts
    m.insert("ttf", "\u{f031}"); //
    m.insert("otf", "\u{f031}"); //
    m.insert("woff", "\u{f031}"); //
    m.insert("woff2", "\u{f031}"); //

    // Certs
    m.insert("pem", "\u{f084}"); //
    m.insert("crt", "\u{f084}"); //
    m.insert("key", "\u{f084}"); //

    // Lock
    m.insert("lock", "\u{f023}"); //

    // Rust ecosystem
    m.insert("rlib", "\u{e7a8}"); //
    m.insert("rmeta", "\u{e7a8}"); //
    m.insert("crate", "\u{e7a8}"); //

    // Python ecosystem
    m.insert("pyc", "\u{e73c}"); //
    m.insert("pyi", "\u{e73c}"); //
    m.insert("pyx", "\u{e73c}"); //
    m.insert("pxd", "\u{e73c}"); //

    // Ruby ecosystem
    m.insert("erb", "\u{e791}"); //
    m.insert("rbs", "\u{e791}"); //
    m.insert("rbi", "\u{e791}"); //
    m.insert("gemspec", "\u{e791}"); //
    m.insert("rake", "\u{e791}"); //

    // JS/TS variants
    m.insert("mjs", "\u{e74e}"); //
    m.insert("cjs", "\u{e74e}"); //
    m.insert("mts", "\u{e628}"); //
    m.insert("cts", "\u{e628}"); //

    m
});

/// Icon for directories.
const DIR_ICON: &str = "\u{f115}"; //

/// Icon for symlinks.
const SYMLINK_ICON: &str = "\u{f0c1}"; //

/// Icon for executables.
const EXEC_ICON: &str = "\u{f489}"; //

/// Icon for pipes/FIFOs.
const PIPE_ICON: &str = "\u{f0c1}"; //

/// Icon for sockets.
const SOCKET_ICON: &str = "\u{f1e6}"; //

/// Default icon for unknown file types.
const DEFAULT_ICON: &str = "\u{f15b}"; //

/// File type indicator from GNU ls --classify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Directory,
    Executable,
    Symlink,
    Pipe,
    Socket,
    Regular,
}

impl FileType {
    /// Parse from the classify indicator character appended by `ls --classify`.
    pub fn from_classify_char(c: char) -> Self {
        match c {
            '/' => Self::Directory,
            '*' => Self::Executable,
            '@' => Self::Symlink,
            '|' => Self::Pipe,
            '=' => Self::Socket,
            _ => Self::Regular,
        }
    }

    #[allow(dead_code)]
    pub fn icon(self) -> &'static str {
        match self {
            Self::Directory => DIR_ICON,
            Self::Executable => EXEC_ICON,
            Self::Symlink => SYMLINK_ICON,
            Self::Pipe => PIPE_ICON,
            Self::Socket => SOCKET_ICON,
            Self::Regular => DEFAULT_ICON,
        }
    }
}

/// Get icon for a filename based on its extension.
pub fn icon_for_name(name: &str) -> &'static str {
    // Strip classify indicator if present
    let clean = name.trim_end_matches(['/', '*', '@', '|', '=']);

    let ext = clean.rsplit('.').next().unwrap_or("");
    EXTENSION_ICONS
        .get(ext.to_lowercase().as_str())
        .copied()
        .unwrap_or(DEFAULT_ICON)
}

/// Get icon for a filename with its file type.
pub fn icon_for_entry(name: &str, file_type: FileType) -> &'static str {
    match file_type {
        FileType::Directory => DIR_ICON,
        FileType::Executable => EXEC_ICON,
        FileType::Symlink => SYMLINK_ICON,
        FileType::Pipe => PIPE_ICON,
        FileType::Socket => SOCKET_ICON,
        FileType::Regular => icon_for_name(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_directory() {
        assert_eq!(FileType::from_classify_char('/'), FileType::Directory);
    }

    #[test]
    fn classify_executable() {
        assert_eq!(FileType::from_classify_char('*'), FileType::Executable);
    }

    #[test]
    fn classify_symlink() {
        assert_eq!(FileType::from_classify_char('@'), FileType::Symlink);
    }

    #[test]
    fn classify_pipe() {
        assert_eq!(FileType::from_classify_char('|'), FileType::Pipe);
    }

    #[test]
    fn classify_socket() {
        assert_eq!(FileType::from_classify_char('='), FileType::Socket);
    }

    #[test]
    fn classify_regular() {
        assert_eq!(FileType::from_classify_char(' '), FileType::Regular);
        assert_eq!(FileType::from_classify_char('x'), FileType::Regular);
    }

    #[test]
    fn file_type_icons() {
        assert_eq!(FileType::Directory.icon(), DIR_ICON);
        assert_eq!(FileType::Executable.icon(), EXEC_ICON);
        assert_eq!(FileType::Symlink.icon(), SYMLINK_ICON);
        assert_eq!(FileType::Pipe.icon(), PIPE_ICON);
        assert_eq!(FileType::Socket.icon(), SOCKET_ICON);
        assert_eq!(FileType::Regular.icon(), DEFAULT_ICON);
    }

    #[test]
    fn icon_for_name_rust() {
        assert_eq!(icon_for_name("main.rs"), "\u{e7a8}");
    }

    #[test]
    fn icon_for_name_python() {
        assert_eq!(icon_for_name("script.py"), "\u{e73c}");
    }

    #[test]
    fn icon_for_name_unknown() {
        assert_eq!(icon_for_name("unknown.xyz123"), DEFAULT_ICON);
    }

    #[test]
    fn icon_for_name_no_extension() {
        assert_eq!(icon_for_name("Makefile"), DEFAULT_ICON);
    }

    #[test]
    fn icon_for_name_strips_classify() {
        // File with classify indicator
        assert_eq!(icon_for_name("main.rs*"), "\u{e7a8}");
        // icon_for_name only knows extensions, not types — "src" has no extension
        assert_eq!(icon_for_name("src/"), DEFAULT_ICON);
    }

    #[test]
    fn icon_for_entry_dir_overrides_ext() {
        // Even if name looks like a file, dir type wins
        assert_eq!(
            icon_for_entry("something.rs", FileType::Directory),
            DIR_ICON
        );
    }

    #[test]
    fn icon_for_entry_regular_uses_ext() {
        assert_eq!(icon_for_entry("main.rs", FileType::Regular), "\u{e7a8}");
    }

    #[test]
    fn icon_for_entry_exec_overrides_ext() {
        assert_eq!(icon_for_entry("script.sh", FileType::Executable), EXEC_ICON);
    }

    #[test]
    fn extension_icons_populated() {
        assert!(EXTENSION_ICONS.len() > 100);
    }

    #[test]
    fn icon_for_name_case_insensitive() {
        // Upper case extension should still match
        assert_eq!(icon_for_name("FILE.RS"), "\u{e7a8}");
        assert_eq!(icon_for_name("DATA.JSON"), "\u{e60b}");
    }
}
