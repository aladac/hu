//! Single chokepoint for external process I/O.
//!
//! Every shellout in `hu setup` (brew, mise, gh, op, stow, etc.) routes through
//! the [`Shell`] trait. The real impl uses `tokio::process::Command`; tests use
//! [`FakeShell`] which scripts (cmd, args) → output mappings.
//!
//! Per project doctrine (CLAUDE.md §1): one trait covers all process I/O so
//! every caller is testable without wrapping each binary in its own trait.

// reason: trait + RealShell wired by Phase 0 chunk 0.4 (status checker via `which`)
// and Phase 1+ installers. FakeShell exists only under cfg(test). Suppress
// dead_code until the first runtime caller lands.
#![allow(dead_code)]

use std::process::ExitStatus;

use anyhow::Result;
use async_trait::async_trait;

/// Result of running an external command.
#[derive(Debug, Clone)]
pub struct ShellOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl ShellOutput {
    pub fn is_success(&self) -> bool {
        self.status.success()
    }
}

/// Process I/O chokepoint.
#[async_trait]
pub trait Shell: Send + Sync {
    /// Run a command with arguments and return captured output.
    async fn run(&self, cmd: &str, args: &[&str]) -> Result<ShellOutput>;

    /// Convenience: returns true when the command exists on PATH.
    async fn which(&self, cmd: &str) -> bool {
        match self.run("which", &[cmd]).await {
            Ok(out) => out.is_success(),
            Err(_) => false,
        }
    }
}

/// Real implementation backed by `tokio::process::Command`.
pub struct RealShell;

#[async_trait]
impl Shell for RealShell {
    async fn run(&self, cmd: &str, args: &[&str]) -> Result<ShellOutput> {
        let output = tokio::process::Command::new(cmd)
            .args(args)
            .output()
            .await?;
        Ok(ShellOutput {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
mod fake {
    use super::*;
    use std::collections::HashMap;
    use std::os::unix::process::ExitStatusExt;
    use std::sync::Mutex;

    /// Scripted shell for tests. Maps `(cmd, args)` → canned output.
    ///
    /// Calls without a registered response default to exit code 127 (command
    /// not found) which mirrors real shell behaviour.
    pub struct FakeShell {
        responses: Mutex<HashMap<String, ShellOutput>>,
        sequences: Mutex<HashMap<String, Vec<ShellOutput>>>,
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl FakeShell {
        pub fn new() -> Self {
            Self {
                responses: Mutex::new(HashMap::new()),
                sequences: Mutex::new(HashMap::new()),
                calls: Mutex::new(Vec::new()),
            }
        }

        /// Register a response for a `(cmd, args)` invocation.
        pub fn expect(&self, cmd: &str, args: &[&str], stdout: &str, exit_code: i32) {
            let key = Self::key(cmd, args);
            let status = ExitStatus::from_raw(exit_code << 8);
            self.responses.lock().expect("fake shell mutex").insert(
                key,
                ShellOutput {
                    status,
                    stdout: stdout.to_string(),
                    stderr: String::new(),
                },
            );
        }

        /// Register an ordered sequence of responses for repeated calls to the
        /// same `(cmd, args)`. Each call pops the next response; exhausted
        /// sequences fall through to the steady-state `expect` value (or 127
        /// if none registered).
        ///
        /// Useful for `check → install → check` flows where the second check
        /// should observe the post-install state.
        pub fn expect_sequence(&self, cmd: &str, args: &[&str], outcomes: &[(&str, i32)]) {
            let key = Self::key(cmd, args);
            let queue: Vec<ShellOutput> = outcomes
                .iter()
                .map(|(stdout, exit_code)| ShellOutput {
                    status: ExitStatus::from_raw(*exit_code << 8),
                    stdout: stdout.to_string(),
                    stderr: String::new(),
                })
                .collect();
            self.sequences
                .lock()
                .expect("fake shell mutex")
                .insert(key, queue);
        }

        /// Recorded call log.
        pub fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().expect("fake shell mutex").clone()
        }

        fn key(cmd: &str, args: &[&str]) -> String {
            format!("{} {}", cmd, args.join(" "))
        }
    }

    #[async_trait]
    impl Shell for FakeShell {
        async fn run(&self, cmd: &str, args: &[&str]) -> Result<ShellOutput> {
            let key = Self::key(cmd, args);
            self.calls.lock().expect("fake shell mutex").push((
                cmd.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            // Sequence queue takes priority over single-response map.
            if let Some(queue) = self
                .sequences
                .lock()
                .expect("fake shell mutex")
                .get_mut(&key)
            {
                if !queue.is_empty() {
                    return Ok(queue.remove(0));
                }
            }
            let map = self.responses.lock().expect("fake shell mutex");
            match map.get(&key) {
                Some(out) => Ok(out.clone()),
                None => Ok(ShellOutput {
                    status: ExitStatus::from_raw(127 << 8),
                    stdout: String::new(),
                    stderr: format!("FakeShell: unscripted call: {}", key),
                }),
            }
        }
    }
}

#[cfg(test)]
pub use fake::FakeShell;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_shell_returns_scripted_output() {
        let shell = FakeShell::new();
        shell.expect("brew", &["list", "gh"], "gh\n", 0);
        let out = shell.run("brew", &["list", "gh"]).await.unwrap();
        assert!(out.is_success());
        assert_eq!(out.stdout, "gh\n");
    }

    #[tokio::test]
    async fn fake_shell_records_calls() {
        let shell = FakeShell::new();
        shell.expect("brew", &["list", "jq"], "", 0);
        let _ = shell.run("brew", &["list", "jq"]).await.unwrap();
        let calls = shell.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "brew");
        assert_eq!(calls[0].1, vec!["list", "jq"]);
    }

    #[tokio::test]
    async fn fake_shell_unscripted_returns_127() {
        let shell = FakeShell::new();
        let out = shell.run("nonexistent", &[]).await.unwrap();
        assert!(!out.is_success());
        assert_eq!(out.status.code(), Some(127));
        assert!(out.stderr.contains("unscripted call"));
    }

    #[tokio::test]
    async fn which_returns_true_when_command_exits_zero() {
        let shell = FakeShell::new();
        shell.expect("which", &["brew"], "/opt/homebrew/bin/brew\n", 0);
        assert!(shell.which("brew").await);
    }

    #[tokio::test]
    async fn which_returns_false_when_command_exits_nonzero() {
        let shell = FakeShell::new();
        shell.expect("which", &["nonexistent"], "", 1);
        assert!(!shell.which("nonexistent").await);
    }

    #[tokio::test]
    async fn shell_output_is_success_matches_status() {
        let ok = ShellOutput {
            status: std::os::unix::process::ExitStatusExt::from_raw(0),
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(ok.is_success());
        let bad = ShellOutput {
            status: std::os::unix::process::ExitStatusExt::from_raw(1 << 8),
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(!bad.is_success());
    }
}
