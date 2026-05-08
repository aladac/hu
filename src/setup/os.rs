//! Operating system detection for the setup module.
//!
//! Detects macOS via `cfg!(target_os)` and Linux distro via parsing
//! `/etc/os-release`. Distro string is lowercased for stable matching.

// reason: items used by Phase 0 chunk 0.4 (status table) and Phase 1+ (per-OS install paths).
// Tests cover them now; suppress dead_code until the first runtime caller wires in.
#![allow(dead_code)]

use std::path::Path;

use anyhow::Result;

/// Detected host operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Os {
    Mac,
    Linux { distro: String },
    Other { name: String },
}

impl Os {
    /// Detect the current host's OS.
    ///
    /// On macOS this is a constant. On Linux it reads `/etc/os-release` and
    /// extracts the `ID=` field. Falls back to `Os::Other` for unknown
    /// platforms.
    pub fn detect() -> Result<Self> {
        if cfg!(target_os = "macos") {
            return Ok(Os::Mac);
        }
        if cfg!(target_os = "linux") {
            let raw = std::fs::read_to_string("/etc/os-release")
                .unwrap_or_else(|_| String::from("ID=linux"));
            return Ok(Os::Linux {
                distro: parse_os_release_id(&raw),
            });
        }
        Ok(Os::Other {
            name: std::env::consts::OS.to_string(),
        })
    }

    /// Short label for status tables.
    pub fn label(&self) -> String {
        match self {
            Os::Mac => "macOS".to_string(),
            Os::Linux { distro } => format!("linux ({})", distro),
            Os::Other { name } => name.clone(),
        }
    }

    /// True when the host is macOS.
    pub fn is_macos(&self) -> bool {
        matches!(self, Os::Mac)
    }

    /// True when the host is any Linux distro.
    pub fn is_linux(&self) -> bool {
        matches!(self, Os::Linux { .. })
    }
}

/// Parse the `ID=` value out of an `/etc/os-release` blob.
///
/// Strips surrounding quotes and lowercases. Returns `"linux"` as a safe
/// fallback when no `ID=` line is present.
pub fn parse_os_release_id(raw: &str) -> String {
    raw.lines()
        .find_map(|line| line.strip_prefix("ID="))
        .map(strip_quotes)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "linux".to_string())
}

fn strip_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Whether `/etc/os-release` exists on disk (used by integration tests).
pub fn has_os_release_file() -> bool {
    Path::new("/etc/os-release").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ubuntu_id() {
        let raw = r#"NAME="Ubuntu"
ID=ubuntu
VERSION_ID="24.04"
"#;
        assert_eq!(parse_os_release_id(raw), "ubuntu");
    }

    #[test]
    fn parses_quoted_id() {
        let raw = "ID=\"debian\"\n";
        assert_eq!(parse_os_release_id(raw), "debian");
    }

    #[test]
    fn parses_single_quoted_id() {
        let raw = "ID='arch'\n";
        assert_eq!(parse_os_release_id(raw), "arch");
    }

    #[test]
    fn lowercases_id() {
        let raw = "ID=Fedora\n";
        assert_eq!(parse_os_release_id(raw), "fedora");
    }

    #[test]
    fn falls_back_when_no_id() {
        let raw = "NAME=Foo\nVERSION=1.0\n";
        assert_eq!(parse_os_release_id(raw), "linux");
    }

    #[test]
    fn empty_input_falls_back() {
        assert_eq!(parse_os_release_id(""), "linux");
    }

    #[test]
    fn label_for_macos() {
        assert_eq!(Os::Mac.label(), "macOS");
    }

    #[test]
    fn label_for_linux_includes_distro() {
        let os = Os::Linux {
            distro: "ubuntu".into(),
        };
        assert_eq!(os.label(), "linux (ubuntu)");
    }

    #[test]
    fn label_for_other() {
        let os = Os::Other {
            name: "freebsd".into(),
        };
        assert_eq!(os.label(), "freebsd");
    }

    #[test]
    fn predicates_match_variant() {
        assert!(Os::Mac.is_macos());
        assert!(!Os::Mac.is_linux());
        let linux = Os::Linux {
            distro: "ubuntu".into(),
        };
        assert!(linux.is_linux());
        assert!(!linux.is_macos());
    }

    #[test]
    fn detect_returns_a_known_variant() {
        let os = Os::detect().expect("detect should not fail on supported hosts");
        // We don't assert the specific variant — runs across mac + linux CI.
        assert!(os.is_macos() || os.is_linux() || matches!(os, Os::Other { .. }));
    }
}
