//! Installer trait + concrete implementations.
//!
//! The `Installer` trait is one of the two earned trait abstractions in
//! `hu setup` (per project doctrine §1 — ≥2 implementers exist or are likely):
//!
//! - [`BrewInstaller`] — wraps `brew list <pkg>` + `brew install <pkg>`
//! - `MiseInstaller` (Phase 2) — wraps `mise use -g <pkg@version>`
//!
//! Both use the [`Shell`] chokepoint for I/O. Installers consume `&impl Shell`
//! so callers stay generic (static dispatch).

// reason: trait + impls wired by Phase 1 chunk 1.3 (`hu setup pkgs`) and
// Phase 5 (`hu setup run`). Tests cover the surface now.
#![allow(dead_code)]

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::setup::types::Status;
use crate::util::shell::Shell;

/// Outcome of an `Installer::ensure` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResult {
    pub package: String,
    pub status: Status,
    pub note: String,
}

impl InstallResult {
    pub fn already(pkg: &str) -> Self {
        Self {
            package: pkg.to_string(),
            status: Status::Already,
            note: "already present".into(),
        }
    }

    pub fn installed(pkg: &str) -> Self {
        Self {
            package: pkg.to_string(),
            status: Status::Installed,
            note: "installed".into(),
        }
    }

    pub fn failed(pkg: &str, note: &str) -> Self {
        Self {
            package: pkg.to_string(),
            status: Status::Failed,
            note: note.into(),
        }
    }

    pub fn skipped(pkg: &str, note: &str) -> Self {
        Self {
            package: pkg.to_string(),
            status: Status::Skipped,
            note: note.into(),
        }
    }
}

/// A package installer for one delivery mechanism (brew, mise, apt, …).
#[async_trait]
pub trait Installer: Send + Sync {
    /// Short id ("brew", "mise") for status reporting.
    fn name(&self) -> &'static str;

    /// True when the package is currently installed.
    async fn check<S: Shell + ?Sized>(&self, shell: &S, package: &str) -> Result<bool>;

    /// Install the package. Implementations should be idempotent — calling
    /// install on a present package should be a no-op.
    async fn install<S: Shell + ?Sized>(&self, shell: &S, package: &str) -> Result<()>;

    /// Idempotent ensure: `check → skip-or-install → re-verify`.
    ///
    /// This is the primary entry point. It enforces the doctrine §9 contract:
    /// re-verifying after `install()` because exit 0 is not proof the side
    /// effect happened.
    async fn ensure<S: Shell + ?Sized>(&self, shell: &S, package: &str) -> InstallResult {
        match self.check(shell, package).await {
            Ok(true) => return InstallResult::already(package),
            Ok(false) => {}
            Err(e) => return InstallResult::failed(package, &format!("check failed: {}", e)),
        }
        if let Err(e) = self.install(shell, package).await {
            return InstallResult::failed(package, &format!("install failed: {}", e));
        }
        match self.check(shell, package).await {
            Ok(true) => InstallResult::installed(package),
            Ok(false) => {
                InstallResult::failed(package, "install reported success but check still fails")
            }
            Err(e) => InstallResult::failed(package, &format!("re-verify failed: {}", e)),
        }
    }
}

/// Homebrew installer. Works on macOS and Linux (linuxbrew).
pub struct BrewInstaller;

#[async_trait]
impl Installer for BrewInstaller {
    fn name(&self) -> &'static str {
        "brew"
    }

    async fn check<S: Shell + ?Sized>(&self, shell: &S, package: &str) -> Result<bool> {
        let out = shell
            .run("brew", &["list", "--versions", package])
            .await
            .with_context(|| format!("brew list {}", package))?;
        Ok(out.is_success())
    }

    async fn install<S: Shell + ?Sized>(&self, shell: &S, package: &str) -> Result<()> {
        let out = shell
            .run("brew", &["install", package])
            .await
            .with_context(|| format!("brew install {}", package))?;
        if !out.is_success() {
            anyhow::bail!(
                "brew install {} failed (exit {:?}): {}",
                package,
                out.status.code(),
                out.stderr.trim()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::shell::FakeShell;

    fn brew() -> BrewInstaller {
        BrewInstaller
    }

    #[tokio::test]
    async fn name_is_brew() {
        assert_eq!(brew().name(), "brew");
    }

    #[tokio::test]
    async fn check_returns_true_when_brew_list_succeeds() {
        let shell = FakeShell::new();
        shell.expect("brew", &["list", "--versions", "gh"], "gh 2.50.0\n", 0);
        assert!(brew().check(&shell, "gh").await.unwrap());
    }

    #[tokio::test]
    async fn check_returns_false_when_brew_list_fails() {
        let shell = FakeShell::new();
        shell.expect("brew", &["list", "--versions", "missing"], "", 1);
        assert!(!brew().check(&shell, "missing").await.unwrap());
    }

    #[tokio::test]
    async fn install_succeeds_on_zero_exit() {
        let shell = FakeShell::new();
        shell.expect("brew", &["install", "gh"], "Successfully installed gh\n", 0);
        brew().install(&shell, "gh").await.unwrap();
    }

    #[tokio::test]
    async fn install_errors_on_nonzero_exit() {
        let shell = FakeShell::new();
        shell.expect("brew", &["install", "broken"], "", 1);
        let err = brew().install(&shell, "broken").await.unwrap_err();
        assert!(err.to_string().contains("brew install broken failed"));
    }

    #[tokio::test]
    async fn ensure_skips_when_already_installed() {
        let shell = FakeShell::new();
        shell.expect("brew", &["list", "--versions", "gh"], "gh 2.50.0\n", 0);
        let result = brew().ensure(&shell, "gh").await;
        assert_eq!(result.status, Status::Already);
        // exactly one call: just the check
        assert_eq!(shell.calls().len(), 1);
    }

    #[tokio::test]
    async fn ensure_marks_failed_when_install_lies() {
        // Install reports success but post-install check still fails.
        // Doctrine §9: re-verify catches lies — exit 0 ≠ side effect happened.
        let shell = FakeShell::new();
        shell.expect("brew", &["list", "--versions", "jq"], "", 1);
        shell.expect("brew", &["install", "jq"], "Successfully installed jq\n", 0);
        let result = brew().ensure(&shell, "jq").await;
        assert_eq!(result.status, Status::Failed);
        assert!(result
            .note
            .contains("install reported success but check still fails"));
    }

    #[tokio::test]
    async fn ensure_installs_when_missing_then_re_verifies_green() {
        // Happy path: first check fails, install succeeds, second check passes.
        let shell = FakeShell::new();
        shell.expect_sequence(
            "brew",
            &["list", "--versions", "jq"],
            &[("", 1), ("jq 1.7\n", 0)],
        );
        shell.expect("brew", &["install", "jq"], "Successfully installed jq\n", 0);
        let result = brew().ensure(&shell, "jq").await;
        assert_eq!(result.status, Status::Installed);
        // Three calls: check → install → check
        assert_eq!(shell.calls().len(), 3);
    }

    #[tokio::test]
    async fn ensure_marks_failed_when_install_errors() {
        let shell = FakeShell::new();
        shell.expect("brew", &["list", "--versions", "broken"], "", 1);
        shell.expect("brew", &["install", "broken"], "", 1);
        let result = brew().ensure(&shell, "broken").await;
        assert_eq!(result.status, Status::Failed);
        assert!(result.note.contains("install failed"));
    }

    #[test]
    fn install_result_constructors() {
        assert_eq!(InstallResult::already("x").status, Status::Already);
        assert_eq!(InstallResult::installed("x").status, Status::Installed);
        assert_eq!(InstallResult::failed("x", "boom").status, Status::Failed);
        assert_eq!(
            InstallResult::skipped("x", "filtered").status,
            Status::Skipped
        );
    }
}
