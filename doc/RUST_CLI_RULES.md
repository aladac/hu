# Rust CLI Project Rules

Best practices distilled from project analysis and refactoring experience.

**Style Philosophy:** When patterns diverge, prefer Ruby/Python idioms—readability over cleverness, flat over nested, explicit over implicit.

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

## 2. Style Conventions (Ruby/Python Influence)

### Naming Patterns
```rust
// Predicates: use is_, has_, can_ prefixes (Python style)
fn is_empty(&self) -> bool
fn has_permissions(&self) -> bool
fn can_connect(&self) -> bool

// NOT: empty(), permissions(), connectable()

// Mutating methods: use imperative verbs
fn clear(&mut self)        // not reset_to_empty
fn save(&mut self)         // not persist_to_disk
fn update(&mut self)       // not apply_changes

// Constructors: prefer new(), from_*, parse_*
fn new() -> Self
fn from_path(p: &Path) -> Result<Self>
fn parse(s: &str) -> Result<Self>
```

### Flat Over Nested
```rust
// BAD - deeply nested
fn process(data: Option<Vec<Item>>) -> Result<()> {
    if let Some(items) = data {
        if !items.is_empty() {
            for item in items {
                if item.is_valid() {
                    // finally do something
                }
            }
        }
    }
    Ok(())
}

// GOOD - early returns, flat structure (Python style)
fn process(data: Option<Vec<Item>>) -> Result<()> {
    let items = match data {
        Some(v) if !v.is_empty() => v,
        _ => return Ok(()),
    };

    for item in items.iter().filter(|i| i.is_valid()) {
        // do something
    }
    Ok(())
}
```

### Iterators Over Loops
Prefer functional iterator chains (Ruby/Python comprehension style):
```rust
// BAD - imperative loop
let mut results = Vec::new();
for item in items {
    if item.is_active() {
        results.push(item.name.clone());
    }
}

// GOOD - iterator chain
let results: Vec<_> = items
    .iter()
    .filter(|i| i.is_active())
    .map(|i| i.name.clone())
    .collect();
```

### Method Chaining
Build fluent APIs where appropriate (Ruby style):
```rust
// Builder pattern
let config = Config::new()
    .with_timeout(30)
    .with_retries(3)
    .build()?;

// NOT
let mut config = Config::new();
config.set_timeout(30);
config.set_retries(3);
let config = config.build()?;
```

### Explicit Over Implicit
```rust
// BAD - implicit behavior
fn fetch(url: &str) -> Data  // What if it fails? Panics?

// GOOD - explicit about what can happen
fn fetch(url: &str) -> Result<Data, FetchError>

// BAD - magic boolean
process(data, true, false)

// GOOD - named parameters via struct or enum
process(data, Mode::Async, Validate::Skip)
```

### Sensible Defaults (Convention Over Configuration)
```rust
// Provide good defaults, allow override
#[derive(Default)]
pub struct Options {
    pub timeout: Option<Duration>,  // None = use default
    pub retries: u32,               // Default via Default trait
}

impl Options {
    pub fn timeout_or_default(&self) -> Duration {
        self.timeout.unwrap_or(Duration::from_secs(30))
    }
}
```

### Debug-Friendly Types
Always derive Debug, implement Display for user-facing types:
```rust
#[derive(Debug, Clone)]  // Always derive Debug
pub struct Pod {
    pub name: String,
    pub status: Status,
}

impl std::fmt::Display for Pod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.status)
    }
}
```

---

## 3. Code Quality Rules

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

## 4. Error Handling

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

## 5. Avoid Duplication

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

## 6. Testing

### No Inline Tests
**Never use `#[cfg(test)]` modules in source files.**

```rust
// BAD - inline test module
// src/parser.rs
pub fn parse(input: &str) -> Result<Data> { ... }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() { ... }
}

// GOOD - separate test file
// src/parser.rs
pub fn parse(input: &str) -> Result<Data> { ... }

// tests/parser_tests.rs
use hu::parse;
#[test]
fn test_parse() { ... }
```

**Why external tests:**
- Keeps source files focused on implementation
- Tests only access public API (better encapsulation)
- Easier to find and navigate tests
- Cleaner compilation units
- Forces better module design

### Test Directory Structure
```
tests/
  unit_tests/
    mod.rs
    parser.rs        # Unit tests for src/parser.rs
    config.rs        # Unit tests for src/config.rs
  integration_tests/
    mod.rs
    cli.rs           # End-to-end CLI tests
    api.rs           # API integration tests
  fixtures/
    sample.json      # Test data files
  snapshots/         # insta snapshot files (auto-generated)
```

### Test File Naming
- `tests/unit_tests/{module}.rs` - mirrors `src/{module}.rs`
- `tests/integration_tests/{feature}.rs` - tests features end-to-end
- Entry points: `tests/unit.rs`, `tests/integration.rs`

### Test Types
- **Unit tests** - Test individual functions, use mocks
- **Integration tests** - Test CLI commands, real I/O
- **Snapshot tests** - Use `insta` for output formatting
- **Benchmarks** - Use `criterion` in `benches/`

### Before Refactoring
1. Ensure existing tests pass
2. Add tests for functions being extracted
3. Consider property-based tests for parsing

### Test Commands
```bash
just check      # fmt + clippy
just test       # run all tests
cargo insta review  # review snapshot changes
```

---

## 7. Dependencies

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

## 8. Configuration

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

## 9. Documentation

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

## 10. Architecture Principles

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

## 11. Metrics Targets

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

**Structure:**
- [ ] Files under 400 lines
- [ ] Functions under 50 lines
- [ ] Nesting depth ≤ 3 levels

**Style (Ruby/Python):**
- [ ] Predicates use `is_`, `has_`, `can_` prefixes
- [ ] Iterator chains over imperative loops
- [ ] Early returns to flatten logic
- [ ] Builder pattern for complex construction
- [ ] All types derive `Debug`

**Quality:**
- [ ] No magic numbers (use constants)
- [ ] No hardcoded URLs
- [ ] All errors propagated with context
- [ ] Common patterns extracted to helpers

**Testing:**
- [ ] Tests in `tests/` directory (no inline `#[cfg(test)]`)
- [ ] Snapshot tests for output formatting

**Tooling:**
- [ ] `clippy.toml` and `rustfmt.toml` configured
- [ ] CI runs fmt, clippy, and tests
