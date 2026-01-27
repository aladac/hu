# Style Conventions (Ruby/Python Influence)

## Naming Patterns

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

## Flat Over Nested

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

## Iterators Over Loops

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

## Method Chaining

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

## Explicit Over Implicit

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

## Sensible Defaults (Convention Over Configuration)

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

## Debug-Friendly Types

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
