# Project Structure

## File Size Limits
- **Maximum 400 lines per file** - Split larger files into modules
- **Maximum 50 lines per function** - Extract helpers for longer functions
- No file should contain more than 2-3 distinct responsibilities

## Scalable CLI Layout

Pattern: `hu <command> <subcommand>` (e.g., `hu jira list`, `hu gh prs`)

```
src/
  main.rs              # CLI entry, top-level dispatch only (~50 lines)
  lib.rs               # Re-exports all modules
  cli.rs               # Top-level CLI struct with #[command(flatten)]
  errors.rs            # Shared error types

  # Each command is a self-contained module
  jira/
    mod.rs             # pub use, Jira enum with subcommands
    cli.rs             # #[derive(Subcommand)] enum JiraCommand
    list.rs            # `hu jira list` handler
    show.rs            # `hu jira show` handler
    types.rs           # Jira-specific types
    client.rs          # API client

  gh/
    mod.rs
    cli.rs             # #[derive(Subcommand)] enum GhCommand
    prs.rs             # `hu gh prs` handler
    runs.rs            # `hu gh runs` handler
    types.rs
    client.rs

  # Utilities (name by purpose, not "shared")
  util/
    mod.rs
    http.rs            # HTTP client setup, retries
    config.rs          # Config loading/saving
    output.rs          # Output format handling (--json, --table)
    fmt.rs             # Humanization: time_ago, duration, bytes, inflection
    ui/
      mod.rs           # UI re-exports
      table.rs         # Ratatui table helpers
      progress.rs      # Progress bar/spinner helpers
      status.rs        # Status badges, icons, styles
```

## Implementation Workflow

**Never assume a command will be simple.** Always follow this order:

1. **Base infrastructure first** — Create the foundation before any handler
2. **Then subcommand handler** — One file per subcommand, using the base

**For a new command module** (`hu slack`):
```
Step 1: src/slack/mod.rs      # Module definition
Step 2: src/slack/types.rs    # Data structures
Step 3: src/slack/client.rs   # API client (if needed)
Step 4: src/slack/config.rs   # Config loading (if needed)
Step 5: src/slack/display.rs  # Output formatting (if needed)
Step 6: src/slack/list.rs     # First subcommand handler
```

**For a new subcommand** in existing module (`hu jira sprint`):
```
Step 1: src/jira/sprint.rs    # Handler file
Step 2: Update src/jira/mod.rs and CLI enum
```

**Why base-first:**
- Forces you to understand the API/domain before writing handlers
- Base files are reused by all subcommands — get them right first
- Handlers become thin and focused when infrastructure exists
- Avoids refactoring handlers later to extract shared code

## Reusable Patterns → `util/` First

If implementing something that looks reusable, **start in `util/`**, not in the command module:

| Feature | Put in `util/` | NOT in |
|---------|----------------|--------|
| `--json` / `--table` output flag | `util/output.rs` | `jira/display.rs` |
| Tables (ratatui) | `util/ui/table.rs` | `gh/display.rs` |
| Colored status badges | `util/ui/status.rs` | `gh/display.rs` |
| Progress spinners | `util/ui/progress.rs` | `jira/tickets.rs` |
| Time ago, durations | `util/fmt.rs` | `gh/runs.rs` |
| Byte size formatting | `util/fmt.rs` | `jira/tickets.rs` |
| Pluralize/inflection | `util/fmt.rs` | `jira/display.rs` |
| Config file loading | `util/config.rs` | `jira/config.rs` |
| HTTP client with retries | `util/http.rs` | `jira/client.rs` |

**Rule:** When in doubt, put it in `util/`. Moving from `util/` to module-specific is easy; extracting from module to `util/` later is a refactor.

## Adding a New Command Module

To add `hu slack <subcommand>`:

1. Create `src/slack/mod.rs`:
```rust
mod cli;
mod list;
mod send;

pub use cli::SlackCommand;
```

2. Create `src/slack/cli.rs`:
```rust
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SlackCommand {
    /// List channels
    List,
    /// Send a message
    Send { channel: String, message: String },
}

impl SlackCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::List => list::run().await,
            Self::Send { channel, message } => send::run(&channel, &message).await,
        }
    }
}
```

3. Add to `src/cli.rs`:
```rust
use crate::slack::SlackCommand;

#[derive(Subcommand)]
pub enum Command {
    /// Jira operations
    Jira {
        #[command(subcommand)]
        cmd: JiraCommand,
    },
    /// GitHub operations
    Gh {
        #[command(subcommand)]
        cmd: GhCommand,
    },
    /// Slack operations  // <- ADD
    Slack {
        #[command(subcommand)]
        cmd: SlackCommand,
    },
}
```

4. Add match arm in `main.rs`:
```rust
Command::Slack { cmd } => cmd.run().await,
```

## Module Isolation Rules

- Each command module owns its CLI definition, types, and handlers
- Modules only import from `util/` and standard library
- No cross-imports between command modules (jira/ never imports from gh/)
- If two modules need the same code, extract to `util/`

## Internal Module Structure

Separate base infrastructure from subcommand handlers:

```
jira/
  # Base infrastructure (shared within module)
  mod.rs             # Re-exports, CLI enum definition
  client.rs          # API client, HTTP calls
  config.rs          # Module-specific config loading
  types.rs           # Data structures, API responses
  display.rs         # Table formatting, output helpers
  auth.rs            # Authentication flow

  # Subcommand handlers (one file per subcommand)
  sprint.rs          # `hu jira sprint` → uses client, types, display
  tickets.rs         # `hu jira tickets` → uses client, types, display

gh/
  mod.rs
  client.rs          # GitHub API client
  types.rs

  # Handlers
  prs.rs             # `hu gh prs`
  runs.rs            # `hu gh runs`
  failures.rs        # `hu gh failures`

git/
  mod.rs

  # Handlers (no client needed - shells out to git)
  branch.rs          # `hu git branch`
  commit.rs          # `hu git commit`
```

**Base files** (infrastructure):
| File | Purpose |
|------|---------|
| `client.rs` | API client, HTTP requests |
| `config.rs` | Load/save module config |
| `types.rs` | Structs for API responses |
| `display.rs` | Format output, tables |
| `auth.rs` | OAuth, tokens, login flow |

**Handler files** (one per subcommand):
| File | Purpose |
|------|---------|
| `{subcommand}.rs` | Single handler, imports base files |

**Handler file pattern:**
```rust
// src/jira/sprint.rs
use super::{client::JiraClient, display, types::Sprint};

pub async fn run(client: &JiraClient) -> anyhow::Result<()> {
    let sprints = client.get_sprints().await?;
    display::print_sprints(&sprints);
    Ok(())
}
```

**When to split further:**
- Handler file > 200 lines → extract helpers to `{subcommand}/mod.rs` + subfiles
- Shared logic between 2+ handlers → extract to base file or new `helpers.rs`

## Module Organization

- Group by command, not by type
- Each module is self-contained and independently testable
- Use `mod.rs` for clean re-exports
- Keep `main.rs` minimal—just CLI parsing and dispatch
