# Rust CLI Project Rules

Best practices distilled from project analysis and refactoring experience.

---

## 1. Project Structure

### File Size Limits
- **Maximum 400 lines per file** - Split larger files into modules
- **Maximum 50 lines per function** - Extract helpers for longer functions
- No file should contain more than 2-3 distinct responsibilities

### Recommended Layout
```
src/
  main.rs              # CLI parsing only (~200 lines)
  lib.rs               # Library exports (if dual bin/lib)
  errors.rs            # Custom error types

  feature/
    mod.rs             # Re-exports
    types.rs           # Data structures
    api.rs             # API/external calls
    display.rs         # Table formatting, output

  commands/
    mod.rs
    subcommand/
      mod.rs
      handlers.rs      # Command handlers
```

### Module Organization
- Group by feature, not by type
- Each module should have a single responsibility
- Use `mod.rs` for clean re-exports

---

## 2. Code Quality Rules

### No Magic Numbers
```rust
// BAD
if s.len() > 55 { ... }
Duration::from_millis(100)

// GOOD
const DISPLAY_TITLE_MAX_LEN: usize = 55;
const POLL_INTERVAL_MS: u64 = 100;
```

### No Hardcoded URLs
```rust
// Centralize API base URLs
const GITHUB_API_BASE: &str = "https://api.github.com";
const JIRA_AUTH_URL: &str = "https://auth.atlassian.com";
```

### Function Complexity
- Avoid nesting deeper than 3 levels
- Use early returns to reduce nesting
- Extract nested logic to helper functions

### Parameter Limits
- Maximum 4-5 parameters per function
- Use parameter structs for related data:
```rust
struct Context<'a> {
    env: &'a str,
    namespace: &'a str,
    profile: Option<&'a str>,
}
```

---

## 3. Error Handling

### Use Result Consistently
```rust
// BAD - loses error context
fn run_cmd(cmd: &str) -> Option<String>

// GOOD - preserves error info
fn run_cmd(cmd: &str) -> Result<String>
```

### Custom Error Types
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("API request failed: {0}")]
    Api(#[from] reqwest::Error),

    #[error("Config not found: {path}")]
    ConfigNotFound { path: PathBuf },
}
```

### No Silent Failures
- Always propagate errors with `?`
- Log errors with context before handling
- Never return empty collections on error without logging

---

## 4. Avoid Duplication

### Common Patterns to Extract

**Table Creation:**
```rust
pub fn create_table(headers: &[(&str, Color)]) -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
         .apply_modifier(UTF8_ROUND_CORNERS);
    // ... set headers
    table
}
```

**String Truncation:**
```rust
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
```

**Config Loading:**
```rust
fn load_json_config<T: Default + DeserializeOwned>(path: &Path) -> Result<T>
fn save_json_config<T: Serialize>(path: &Path, config: &T) -> Result<()>
```

**Status Icons:**
```rust
fn status_icon(status: &str, conclusion: Option<&str>) -> ColoredString
```

---

## 5. Testing

### Test Organization
- Tests in `tests/` directory, not inline
- Unit tests: `tests/unit_tests/`
- Integration tests: `tests/integration_tests/`
- Use snapshot tests (`insta`) for output formatting
- Add benchmarks for performance-critical code

### Before Refactoring
1. Ensure existing tests pass
2. Add tests for functions being extracted
3. Consider property-based tests for parsing

### Test Commands
```bash
just check      # fmt + clippy
just test       # run all tests
```

---

## 6. Dependencies

### Core Crates
- **clap** (derive) - CLI parsing
- **colored** - Terminal colors
- **comfy-table** - Table display
- **indicatif** - Progress spinners
- **anyhow** - Application errors
- **thiserror** - Library errors
- **serde** + **serde_json** - Serialization
- **tokio** - Async runtime (if needed)
- **tracing** - Structured logging

### Dev Dependencies
- **insta** - Snapshot testing
- **criterion** - Benchmarks
- **tempfile** - Test fixtures

---

## 7. Configuration

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

---

## 8. Documentation

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

---

## 9. Architecture Principles

### Separation of Concerns
- CLI parsing in `main.rs`
- Business logic in feature modules
- Display/formatting separate from data fetching

### Dependency Direction
- Commands depend on services
- Services depend on types
- Types depend on nothing

### Consider Service Layer
```rust
pub struct EksService { /* ... */ }
impl EksService {
    pub async fn connect(&self, env: &str, pod: usize) -> Result<()>;
    pub async fn tail_logs(&self, env: &str, pattern: &str) -> Result<()>;
}
```

---

## 10. Metrics Targets

| Metric | Target |
|--------|--------|
| Max file size | <400 lines |
| Max function size | <50 lines |
| Max nesting depth | 3 levels |
| Max parameters | 5 |
| Magic numbers | 0 (use constants) |
| Duplicate code blocks | <3 patterns |

---

## Quick Checklist

- [ ] Files under 400 lines
- [ ] Functions under 50 lines
- [ ] No magic numbers
- [ ] No hardcoded URLs
- [ ] All errors propagated with context
- [ ] Common patterns extracted to helpers
- [ ] Tests in `tests/` directory
- [ ] `clippy.toml` and `rustfmt.toml` configured
- [ ] CI runs fmt, clippy, and tests
