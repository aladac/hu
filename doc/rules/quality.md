# Code Quality Rules

## No Magic Numbers

```rust
// BAD
if s.len() > 55 { ... }
Duration::from_millis(100)

// GOOD
const DISPLAY_TITLE_MAX_LEN: usize = 55;
const POLL_INTERVAL_MS: u64 = 100;
```

## No Hardcoded URLs

```rust
// Centralize API base URLs
const GITHUB_API_BASE: &str = "https://api.github.com";
const JIRA_AUTH_URL: &str = "https://auth.atlassian.com";
```

## Function Complexity

- Avoid nesting deeper than 3 levels
- Use early returns to reduce nesting
- Extract nested logic to helper functions

## Parameter Limits

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

# Error Handling

## Use Result Consistently

```rust
// BAD - loses error context
fn run_cmd(cmd: &str) -> Option<String>

// GOOD - preserves error info
fn run_cmd(cmd: &str) -> Result<String>
```

## Custom Error Types

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

## No Silent Failures

- Always propagate errors with `?`
- Log errors with context before handling
- Never return empty collections on error without logging

---

# Avoid Duplication

## Common Patterns to Extract

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
