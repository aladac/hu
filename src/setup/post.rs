//! Phase 6 post-install checks.
//!
//! Runs after `pkgs → dotfiles → ssh` to verify the host is in a usable
//! state and start any services that need explicit kick. Each check
//! produces a `StatusRow`; nothing here is destructive.

// reason: post checks called from `hu setup run` orchestrator.
#![allow(dead_code)]

use crate::setup::config::SetupConfig;
use crate::setup::display::StatusRow;
use crate::setup::types::Status;
use crate::util::shell::Shell;

/// Run all post-install checks. Currently:
/// - `brew services start postgresql` (only if configured + brew has it)
/// - `gh auth status` — read-only
/// - `ssh -T git@github.com` — smoke (exit 1 with greeting = success)
pub async fn run<S: Shell + ?Sized>(shell: &S, config: &SetupConfig) -> Vec<StatusRow> {
    let mut rows = Vec::new();

    if config.packages.brew.iter().any(|p| p == "postgresql") {
        rows.push(start_postgres(shell).await);
    }
    rows.push(check_gh_auth(shell).await);
    rows.push(smoke_github_ssh(shell).await);

    rows
}

async fn start_postgres<S: Shell + ?Sized>(shell: &S) -> StatusRow {
    let out = shell
        .run("brew", &["services", "info", "postgresql", "--json"])
        .await;
    let already_running = match out {
        Ok(out) if out.is_success() => {
            out.stdout.contains("\"running\":true") || out.stdout.contains("\"running\": true")
        }
        _ => false,
    };
    if already_running {
        return StatusRow::new("post", "postgresql", Status::Already)
            .with_note("brew service already running");
    }
    match shell
        .run("brew", &["services", "start", "postgresql"])
        .await
    {
        Ok(out) if out.is_success() => StatusRow::new("post", "postgresql", Status::Installed)
            .with_note("brew service started"),
        Ok(out) => StatusRow::new("post", "postgresql", Status::Failed).with_note(&format!(
            "brew services start failed (exit {:?}): {}",
            out.status.code(),
            out.stderr.trim()
        )),
        Err(e) => StatusRow::new("post", "postgresql", Status::Failed)
            .with_note(&format!("shell errored: {}", e)),
    }
}

async fn check_gh_auth<S: Shell + ?Sized>(shell: &S) -> StatusRow {
    match shell.run("gh", &["auth", "status"]).await {
        Ok(out) if out.is_success() => {
            StatusRow::new("post", "gh-auth", Status::Already).with_note("gh signed in")
        }
        Ok(_) => StatusRow::new("post", "gh-auth", Status::Failed)
            .with_note("not signed in — run `gh auth login`"),
        Err(e) => StatusRow::new("post", "gh-auth", Status::Failed)
            .with_note(&format!("shell errored: {}", e)),
    }
}

async fn smoke_github_ssh<S: Shell + ?Sized>(shell: &S) -> StatusRow {
    // `ssh -T git@github.com` exits 1 with a greeting on success — odd but
    // documented. We grep stderr for "successfully authenticated" instead.
    match shell
        .run(
            "ssh",
            &[
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "git@github.com",
            ],
        )
        .await
    {
        Ok(out) => {
            let combined = format!("{}{}", out.stdout, out.stderr);
            if combined.contains("successfully authenticated") {
                StatusRow::new("post", "github-ssh", Status::Already)
                    .with_note("ssh -T git@github.com authenticated")
            } else {
                StatusRow::new("post", "github-ssh", Status::Failed).with_note(&format!(
                    "no auth confirmation (exit {:?})",
                    out.status.code()
                ))
            }
        }
        Err(e) => StatusRow::new("post", "github-ssh", Status::Failed)
            .with_note(&format!("shell errored: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::shell::FakeShell;

    fn cfg_with_postgres(include: bool) -> SetupConfig {
        let mut cfg = SetupConfig::default();
        cfg.packages.brew = if include {
            vec!["gh".into(), "postgresql".into()]
        } else {
            vec!["gh".into()]
        };
        cfg
    }

    #[tokio::test]
    async fn run_skips_postgres_when_not_configured() {
        let shell = FakeShell::new();
        // gh + ssh expected; postgres should NOT be invoked
        shell.expect("gh", &["auth", "status"], "Logged in to github.com\n", 0);
        shell.expect(
            "ssh",
            &[
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "git@github.com",
            ],
            "Hi user! You've successfully authenticated.\n",
            1,
        );
        let cfg = cfg_with_postgres(false);
        let rows = run(&shell, &cfg).await;
        assert_eq!(rows.len(), 2);
        assert!(!rows.iter().any(|r| r.name == "postgresql"));
    }

    #[tokio::test]
    async fn run_starts_postgres_when_not_running() {
        let shell = FakeShell::new();
        shell.expect(
            "brew",
            &["services", "info", "postgresql", "--json"],
            "[{\"running\":false}]",
            0,
        );
        shell.expect(
            "brew",
            &["services", "start", "postgresql"],
            "Started postgresql\n",
            0,
        );
        shell.expect("gh", &["auth", "status"], "Logged in\n", 0);
        shell.expect(
            "ssh",
            &[
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "git@github.com",
            ],
            "successfully authenticated.\n",
            1,
        );
        let cfg = cfg_with_postgres(true);
        let rows = run(&shell, &cfg).await;
        assert_eq!(rows.len(), 3);
        let pg = rows.iter().find(|r| r.name == "postgresql").unwrap();
        assert_eq!(pg.status, Status::Installed);
    }

    #[tokio::test]
    async fn run_marks_postgres_already_when_running() {
        let shell = FakeShell::new();
        shell.expect(
            "brew",
            &["services", "info", "postgresql", "--json"],
            "[{\"running\":true,\"name\":\"postgresql\"}]",
            0,
        );
        shell.expect("gh", &["auth", "status"], "Logged in\n", 0);
        shell.expect(
            "ssh",
            &[
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "git@github.com",
            ],
            "successfully authenticated.\n",
            1,
        );
        let cfg = cfg_with_postgres(true);
        let rows = run(&shell, &cfg).await;
        let pg = rows.iter().find(|r| r.name == "postgresql").unwrap();
        assert_eq!(pg.status, Status::Already);
    }

    #[tokio::test]
    async fn check_gh_auth_marks_failed_when_not_signed_in() {
        let shell = FakeShell::new();
        shell.expect("gh", &["auth", "status"], "", 1);
        let row = check_gh_auth(&shell).await;
        assert_eq!(row.status, Status::Failed);
        assert!(row.note.contains("gh auth login"));
    }

    #[tokio::test]
    async fn smoke_ssh_marks_already_when_greeting_present() {
        let shell = FakeShell::new();
        shell.expect(
            "ssh",
            &[
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "git@github.com",
            ],
            "Hi aladac! You've successfully authenticated.\n",
            1,
        );
        let row = smoke_github_ssh(&shell).await;
        assert_eq!(row.status, Status::Already);
    }

    #[tokio::test]
    async fn smoke_ssh_marks_failed_without_greeting() {
        let shell = FakeShell::new();
        shell.expect(
            "ssh",
            &[
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "git@github.com",
            ],
            "Permission denied",
            255,
        );
        let row = smoke_github_ssh(&shell).await;
        assert_eq!(row.status, Status::Failed);
        assert!(row.note.contains("no auth confirmation"));
    }
}
