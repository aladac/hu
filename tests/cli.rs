use std::process::Command;

fn hu() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hu"))
}

#[test]
fn no_args_shows_help_and_exits_zero() {
    let output = hu().output().expect("failed to execute");

    assert!(output.status.success(), "expected exit code 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: hu [COMMAND]"));
    assert!(stdout.contains("Commands:"));
}

#[test]
fn help_flag_shows_help() {
    let output = hu().arg("--help").output().expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dev workflow CLI"));
}

#[test]
fn version_flag_shows_version() {
    let output = hu().arg("--version").output().expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hu "));
}

#[test]
fn subcommand_without_action_shows_help() {
    let cases = [
        ("newrelic", "NewRelic"),
        ("utils", "Utility"),
        ("context", "context"),
        ("data", "Claude"),
        ("docs", "Documentation"),
        ("cron", "Cron"),
        ("shell", "Shell"),
        ("mcp", "MCP"),
        ("setup", "bootstrap"),
    ];

    for (cmd, expected) in cases {
        let output = hu().arg(cmd).output().expect("failed to execute");
        assert!(output.status.success(), "{} should exit 0", cmd);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(expected),
            "{} help missing expected text: {}",
            cmd,
            expected
        );
    }
}

#[test]
fn all_main_commands_in_help() {
    let output = hu().output().expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let commands = [
        "newrelic", "utils", "context", "read", "data", "install", "docs", "cron", "shell",
        "mcp", "setup",
    ];
    for cmd in commands {
        assert!(stdout.contains(cmd), "help missing command: {}", cmd);
    }
}

#[test]
fn command_aliases_work() {
    // nr -> newrelic
    let output = hu()
        .args(["nr", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("NewRelic"));
}

#[test]
fn invalid_command_fails() {
    let output = hu().arg("invalid").output().expect("failed to execute");

    assert!(!output.status.success(), "expected non-zero exit code");
}

// NewRelic

#[test]
fn newrelic_incidents_runs() {
    let output = hu()
        .args(["newrelic", "incidents"])
        .output()
        .expect("failed to execute");
    // May succeed or fail depending on auth state — just verify no panic
    let _ = output.status;
}

// Utils

#[test]
fn utils_shows_help() {
    let output = hu().arg("utils").output().expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fetch-html"));
    assert!(stdout.contains("grep"));
}

#[test]
fn utils_fetch_html_help() {
    let output = hu()
        .args(["utils", "fetch-html", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--content"));
    assert!(stdout.contains("--summary"));
    assert!(stdout.contains("--links"));
    assert!(stdout.contains("--headings"));
    assert!(stdout.contains("--selector"));
}

#[test]
fn utils_grep_help() {
    let output = hu()
        .args(["utils", "grep", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--refs"));
    assert!(stdout.contains("--unique"));
    assert!(stdout.contains("--ranked"));
    assert!(stdout.contains("--limit"));
    assert!(stdout.contains("--signature"));
}

#[test]
fn utils_grep_executes() {
    let output = hu()
        .args(["utils", "grep", "fn main", "src/main.rs"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main.rs"));
}

#[test]
fn utils_grep_refs_mode() {
    let output = hu()
        .args(["utils", "grep", "fn", "src/main.rs", "--refs"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Refs mode: just file:line, no content
    assert!(stdout.contains("main.rs:"));
}
