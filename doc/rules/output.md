# Output Standards

## UI with Ratatui

Ratatui provides unified UI components. Put wrappers in `util/ui/`:

```
util/
  ui/
    mod.rs           # Re-exports
    table.rs         # Table helpers
    progress.rs      # Progress bar/spinner helpers
    status.rs        # Status badges, icons
    prompt.rs        # User prompts (pair with dialoguer)
```

**Table pattern:**
```rust
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Table, Row, Cell},
};

pub fn tickets_table(tickets: &[Ticket]) -> Table<'_> {
    let header = Row::new(vec!["Key", "Summary", "Status"])
        .style(Style::default().bold());

    let rows = tickets.iter().map(|t| {
        Row::new(vec![
            Cell::from(t.key.as_str()),
            Cell::from(t.summary.as_str()),
            Cell::from(t.status.as_str()).style(status_style(&t.status)),
        ])
    });

    Table::new(rows, [Constraint::Length(12), Constraint::Min(30), Constraint::Length(15)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Tickets"))
}
```

**Progress pattern:**
```rust
use ratatui::widgets::{Gauge, Block, Borders};

pub fn progress_gauge(percent: u16, label: &str) -> Gauge<'_> {
    Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(label))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(percent)
}
```

**Status badge pattern:**
```rust
pub fn status_style(status: &str) -> Style {
    match status {
        "success" | "done" => Style::default().fg(Color::Green),
        "failure" | "error" => Style::default().fg(Color::Red),
        "pending" | "in_progress" => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    }
}

pub fn status_icon(status: &str) -> &'static str {
    match status {
        "success" | "done" => "Y",
        "failure" | "error" => "X",
        "pending" => "o",
        "in_progress" => "*",
        _ => "-",
    }
}
```

**Markdown rendering:**
```rust
use tui_markdown::from_str;
use ratatui::widgets::Paragraph;

// Render Jira description, PR body, etc.
let markdown = r#"
## Summary
Fix the login redirect issue.

### Changes
- Updated auth middleware
- Added session validation

### Testing
- [x] Unit tests pass
- [ ] Integration tests pending
"#;

let text = from_str(markdown);
let widget = Paragraph::new(text)
    .block(Block::default().borders(Borders::ALL).title("Description"));

frame.render_widget(widget, area);
```

Useful for:
- Jira ticket descriptions
- PR/issue bodies from GitHub
- Slack messages with formatting
- Help text and documentation

**Simple output (non-interactive):**
```rust
use std::io::{self, stdout};
use ratatui::{prelude::*, Terminal};
use crossterm::{execute, terminal::*};

pub fn print_table(table: Table) -> io::Result<()> {
    // For non-interactive output, render once to stdout
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.draw(|frame| {
        frame.render_widget(table, frame.area());
    })?;
    Ok(())
}
```

**When to use ratatui vs plain print:**
| Use ratatui | Use `println!` |
|-------------|----------------|
| Tables | Simple single-line output |
| Progress bars | Error messages |
| Colored status | Debug info |
| Interactive TUI | JSON output (`--json`) |

## Pretty by Default

**All CLI output must be visually polished.** Use ratatui for everything:

```
+-- Jira Sprint Tasks ----------------------------------------------+
| #   | Key        | Summary                     | Status           |
|-----+------------+-----------------------------+------------------|
| 1   | PROJ-123   | Fix login redirect          | Y Done           |
| 2   | PROJ-456   | Add caching layer           | * Progress       |
| 3   | PROJ-789   | Update dependencies         | o Todo           |
+-----+------------+-----------------------------+------------------+
```

## Color Conventions

| Element | Color | Usage |
|---------|-------|-------|
| Success/Done | Green | Completed, passing, resolved |
| Warning/In Progress | Yellow | Pending, in progress, needs attention |
| Error/Critical | Red | Failed, errors, urgent alerts |
| Info/Neutral | Cyan/Blue | Informational, links, counts |
| Muted | Gray | Secondary info, timestamps |

## Status Icons

```rust
// util/ui/status.rs
pub fn status_icon(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "done" | "success" | "resolved" | "closed" => "Y",
        "in_progress" | "running" | "pending" => "*",
        "todo" | "open" | "new" => "o",
        "failed" | "error" | "critical" => "X",
        "warning" | "acknowledged" => "!",
        "blocked" => "#",
        _ => "-",
    }
}

pub fn status_color(status: &str) -> Color {
    match status.to_lowercase().as_str() {
        "done" | "success" | "resolved" => Color::Green,
        "in_progress" | "running" => Color::Yellow,
        "todo" | "open" => Color::White,
        "failed" | "error" | "critical" => Color::Red,
        "warning" | "acknowledged" => Color::Yellow,
        _ => Color::Gray,
    }
}
```

## Dashboard Summary Style

```rust
// Counts with color-coded severity
pub fn render_dashboard_line(icon: &str, count: usize, label: &str, style: Style) -> Line {
    Line::from(vec![
        Span::raw(format!(" {} ", icon)),
        Span::styled(format!("{}", count), style.bold()),
        Span::raw(format!(" {}", label)),
    ])
}

// Example output:
// ! 3 active alerts        <- Red if count > 0
// Y 0 active alerts        <- Green if count == 0
```

## Output Rules

- Tables for any list > 2 items
- Colors for all status indicators
- Icons for quick visual scanning
- Consistent spacing and alignment
- Borders for grouped content
- No plain `println!` for user-facing output

## Configuration

### Linting Setup

Create `clippy.toml`:
```toml
cognitive-complexity-threshold = 15
too-many-arguments-threshold = 5
```

Create `rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
```

### CI Setup

- Run `cargo fmt --check`
- Run `cargo clippy -- -D warnings`
- Run `cargo test`
- Consider `cargo deny` for dependency auditing

## Documentation

### Required Docs

- Complex algorithms
- Public API functions
- Non-obvious behavior
- ARN/URL format expectations

### Variable Naming

```rust
// BAD
let s = parse_timestamp(input);
let cli_profile = args.profile;

// GOOD
let timestamp_str = parse_timestamp(input);
let aws_profile_override = args.profile;
```
