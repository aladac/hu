use crossterm::style::Color;
use std::collections::HashMap;

/// File type color mapping using crossterm colors (via ratatui).
pub struct FileColors {
    extensions: HashMap<String, Color>,
    directory: Color,
    symlink: Color,
    executable: Color,
    pipe: Color,
    socket: Color,
}

impl Default for FileColors {
    fn default() -> Self {
        Self::new()
    }
}

impl FileColors {
    pub fn new() -> Self {
        let mut ext = HashMap::new();

        // Languages
        for e in ["rs", "rlib", "rmeta", "crate"] {
            ext.insert(e.into(), rgb(0xCE, 0x42, 0x2B)); // Rust copper
        }
        for e in ["py", "pyc", "pyi", "pyx", "pxd"] {
            ext.insert(e.into(), rgb(0x37, 0x76, 0xAB)); // Python blue
        }
        for e in ["rb", "erb", "rbs", "rbi", "gemspec", "rake"] {
            ext.insert(e.into(), rgb(0xCC, 0x34, 0x2D)); // Ruby red
        }
        for e in ["js", "mjs", "cjs", "jsx"] {
            ext.insert(e.into(), rgb(0xF7, 0xDF, 0x1E)); // JS yellow
        }
        for e in ["ts", "mts", "cts", "tsx"] {
            ext.insert(e.into(), rgb(0x31, 0x78, 0xC6)); // TS blue
        }
        ext.insert("go".into(), rgb(0x00, 0xAD, 0xD8));
        for e in ["java", "jar"] {
            ext.insert(e.into(), rgb(0xED, 0x8B, 0x00));
        }
        for e in ["c", "h"] {
            ext.insert(e.into(), rgb(0xA8, 0xB9, 0xCC));
        }
        for e in ["cpp", "cc", "cxx", "hpp"] {
            ext.insert(e.into(), rgb(0x00, 0x59, 0x9C));
        }
        for e in ["sh", "bash", "zsh", "fish"] {
            ext.insert(e.into(), rgb(0x4E, 0xAA, 0x25));
        }
        ext.insert("lua".into(), rgb(0x00, 0x00, 0x80));
        ext.insert("swift".into(), rgb(0xFA, 0x73, 0x43));
        ext.insert("dart".into(), rgb(0x01, 0x75, 0xC2));
        ext.insert("sql".into(), rgb(0xE3, 0x8C, 0x00));
        for e in ["cu", "cuh"] {
            ext.insert(e.into(), rgb(0x76, 0xB9, 0x00));
        }
        for e in ["kt", "kts"] {
            ext.insert(e.into(), rgb(0x7F, 0x52, 0xFF));
        }
        ext.insert("cs".into(), rgb(0x51, 0x2B, 0xD4));
        ext.insert("php".into(), rgb(0x77, 0x7B, 0xB4));

        // Web
        for e in ["html", "htm"] {
            ext.insert(e.into(), rgb(0xE3, 0x4F, 0x26));
        }
        ext.insert("css".into(), rgb(0x15, 0x72, 0xB6));
        for e in ["scss", "sass"] {
            ext.insert(e.into(), rgb(0xCF, 0x64, 0x9A));
        }
        ext.insert("vue".into(), rgb(0x4F, 0xC0, 0x8D));
        ext.insert("svelte".into(), rgb(0xFF, 0x3E, 0x00));
        for e in ["wasm", "wat"] {
            ext.insert(e.into(), rgb(0x65, 0x4F, 0xF0));
        }

        // Data
        for e in ["json", "jsonl"] {
            ext.insert(e.into(), rgb(0xCB, 0xCB, 0x41));
        }
        for e in ["yaml", "yml"] {
            ext.insert(e.into(), rgb(0xCB, 0x17, 0x1E));
        }
        ext.insert("toml".into(), rgb(0x9C, 0x41, 0x21));
        ext.insert("xml".into(), rgb(0xF8, 0x00, 0x00));
        ext.insert("csv".into(), rgb(0x23, 0x73, 0x46));
        for e in ["proto", "pb"] {
            ext.insert(e.into(), rgb(0x42, 0x85, 0xF4));
        }

        // Docs
        for e in ["md", "markdown", "mdx"] {
            ext.insert(e.into(), rgb(0xFF, 0xFF, 0xFF));
        }
        ext.insert("txt".into(), rgb(0xAA, 0xAA, 0xAA));
        ext.insert("pdf".into(), rgb(0xFF, 0x00, 0x00));
        for e in ["doc", "docx"] {
            ext.insert(e.into(), rgb(0x2B, 0x57, 0x9A));
        }
        for e in ["xls", "xlsx"] {
            ext.insert(e.into(), rgb(0x21, 0x73, 0x46));
        }

        // Config / VCS
        for e in ["gitignore", "gitattributes", "gitmodules"] {
            ext.insert(e.into(), rgb(0xF0, 0x50, 0x32));
        }
        for e in ["ini", "cfg", "conf"] {
            ext.insert(e.into(), rgb(0x6D, 0x80, 0x86));
        }
        ext.insert("plist".into(), rgb(0x99, 0x99, 0x99));
        ext.insert("dockerfile".into(), rgb(0x24, 0x96, 0xED));
        for e in ["tf", "tfvars"] {
            ext.insert(e.into(), rgb(0x7B, 0x42, 0xBC));
        }

        // Images
        for e in ["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "heic"] {
            ext.insert(e.into(), rgb(0xFF, 0x69, 0xB4));
        }
        ext.insert("svg".into(), rgb(0xFF, 0xB1, 0x3B));

        // Audio/Video
        for e in ["mp3", "wav", "flac", "ogg", "m4a"] {
            ext.insert(e.into(), rgb(0x1D, 0xB9, 0x54));
        }
        for e in ["mp4", "mkv", "avi", "mov", "webm"] {
            ext.insert(e.into(), rgb(0xFF, 0x00, 0x00));
        }

        // Archives
        for e in ["zip", "tar", "gz", "tgz", "bz2", "xz", "rar", "7z"] {
            ext.insert(e.into(), rgb(0xF9, 0xE2, 0xAF));
        }
        ext.insert("deb".into(), rgb(0xA8, 0x00, 0x30));
        ext.insert("rpm".into(), rgb(0xEE, 0x00, 0x00));

        // Compiled
        for e in ["o", "a"] {
            ext.insert(e.into(), rgb(0x6D, 0x6D, 0x6D));
        }
        for e in ["so", "dylib", "dll"] {
            ext.insert(e.into(), rgb(0x5C, 0x6B, 0xC0));
        }
        for e in ["exe", "bin"] {
            ext.insert(e.into(), rgb(0x00, 0xFF, 0x00));
        }

        // Databases
        for e in ["sqlite", "sqlite3", "db"] {
            ext.insert(e.into(), rgb(0x00, 0x3B, 0x57));
        }

        // ML
        ext.insert("onnx".into(), rgb(0x80, 0x80, 0x80));
        for e in ["pt", "pth"] {
            ext.insert(e.into(), rgb(0xEE, 0x4C, 0x2C));
        }
        ext.insert("safetensors".into(), rgb(0x00, 0xB4, 0xD8));

        // Fonts
        for e in ["ttf", "otf", "woff", "woff2"] {
            ext.insert(e.into(), rgb(0xCC, 0xCC, 0xCC));
        }

        // Certs
        for e in ["pem", "crt", "cer", "key"] {
            ext.insert(e.into(), rgb(0xFF, 0xD7, 0x00));
        }

        // Lock
        ext.insert("lock".into(), rgb(0x80, 0x80, 0x80));

        Self {
            extensions: ext,
            directory: rgb(0x5C, 0x9D, 0xFF),
            symlink: rgb(0x00, 0xFF, 0xFF),
            executable: rgb(0x00, 0xFF, 0x00),
            pipe: Color::Magenta,
            socket: Color::Magenta,
        }
    }

    pub fn for_extension(&self, ext: &str) -> Color {
        self.extensions
            .get(&ext.to_lowercase())
            .copied()
            .unwrap_or(Color::Reset)
    }

    pub fn directory(&self) -> Color {
        self.directory
    }

    pub fn symlink(&self) -> Color {
        self.symlink
    }

    pub fn executable(&self) -> Color {
        self.executable
    }

    pub fn pipe(&self) -> Color {
        self.pipe
    }

    pub fn socket(&self) -> Color {
        self.socket
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_helper() {
        let c = rgb(255, 0, 0);
        assert!(matches!(c, Color::Rgb { r: 255, g: 0, b: 0 }));
    }

    #[test]
    fn file_colors_rust() {
        let colors = FileColors::new();
        assert!(matches!(colors.for_extension("rs"), Color::Rgb { .. }));
    }

    #[test]
    fn file_colors_python() {
        let colors = FileColors::new();
        assert!(matches!(colors.for_extension("py"), Color::Rgb { .. }));
    }

    #[test]
    fn file_colors_case_insensitive() {
        let colors = FileColors::new();
        let upper = colors.for_extension("RS");
        let lower = colors.for_extension("rs");
        assert!(matches!(upper, Color::Rgb { .. }));
        assert!(matches!(lower, Color::Rgb { .. }));
    }

    #[test]
    fn file_colors_unknown() {
        let colors = FileColors::new();
        assert!(matches!(
            colors.for_extension("xyz123unknown"),
            Color::Reset
        ));
    }

    #[test]
    fn file_colors_special_types() {
        let colors = FileColors::new();
        assert!(matches!(colors.directory(), Color::Rgb { .. }));
        assert!(matches!(colors.symlink(), Color::Rgb { .. }));
        assert!(matches!(colors.executable(), Color::Rgb { .. }));
        assert!(matches!(colors.pipe(), Color::Magenta));
        assert!(matches!(colors.socket(), Color::Magenta));
    }

    #[test]
    fn default_trait() {
        let colors = FileColors::default();
        assert!(matches!(colors.for_extension("rs"), Color::Rgb { .. }));
    }
}
