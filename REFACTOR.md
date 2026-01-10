# Refactoring Analysis for hu CLI

**Analysis Date:** 2026-01-10  
**Total Lines of Code:** ~4,404 lines across 9 source files

## Executive Summary

The `hu` codebase is a moderately-sized CLI tool with several areas that could benefit from refactoring. The most significant issues are:
1. Large file sizes (`aws.rs` at 1230 lines, `main.rs` at 866 lines)
2. Duplicate code patterns across display functions
3. Some functions exceeding recommended complexity limits
4. Hardcoded magic numbers and strings scattered throughout

---

## 1. Code Smells

### 1.1 Large Files (High Priority)

| File | Lines | Recommendation |
|------|-------|----------------|
| `/src/aws.rs` | 1230 | Split into modules: `aws/identity.rs`, `aws/ec2.rs`, `aws/spawn.rs`, `aws/discovery.rs` |
| `/src/main.rs` | 866 | Extract command definitions to separate modules |
| `/src/jira.rs` | 705 | Split into `jira/api.rs`, `jira/display.rs`, `jira/oauth.rs` |
| `/src/commands/eks.rs` | 606 | Split into `eks/kubeconfig.rs`, `eks/pods.rs`, `eks/logs.rs` |

**Specific Locations:**
- `/src/aws.rs:1-1230` - Contains 6+ distinct responsibilities
- `/src/main.rs:1-866` - Contains CLI definitions, environment detection, session management, and dispatch logic
- `/src/jira.rs:1-705` - Mixes OAuth flow, API calls, and display logic

### 1.2 Long Functions (High Priority)

| Function | File:Line | Lines | Issue |
|----------|-----------|-------|-------|
| `spawn_instance` | `/src/aws.rs:654-772` | ~118 | Does too many things: IP detection, VPC lookup, AMI lookup, key creation, SG creation, instance launch, wait loop |
| `run` (eks) | `/src/commands/eks.rs:495-570` | ~75 | Multiple responsibilities: kubeconfig update, pod fetching, display, mode branching |
| `login` (jira) | `/src/jira.rs:86-145` | ~60 | OAuth flow should be extracted |
| `main` | `/src/main.rs:335-866` | ~530 | Giant match statement - extract handlers to separate functions |
| `display_workflow_runs` | `/src/github.rs:189-238` | ~50 | Could share code with `display_project_workflow_runs` |
| `view` (log) | `/src/commands/log.rs:24-116` | ~92 | Contains both static view and follow mode - split into two functions |

### 1.3 Feature Envy (Medium Priority)

Several functions in one module access data from another module more than their own:

- `/src/main.rs:462-530` - The `Commands::GitHub` handler reaches deep into `github::RunsFilter`, `github::RepoInfo`, and settings structures
- `/src/main.rs:379-425` - EKS command handlers repeatedly access `settings.get_env()`, `settings.kubernetes`, `settings.logging`

**Recommendation:** Create facade methods or builder patterns to reduce coupling.

### 1.4 Data Clumps (Medium Priority)

Same groups of parameters passed together repeatedly:

```rust
// Appears 5+ times in eks.rs and main.rs
(env_name, namespace, pod_type, profile, settings)

// Appears in multiple display functions
(run.status.as_str(), run.conclusion.as_deref())
```

**Locations:**
- `/src/commands/eks.rs:495` - `run()` has 8 parameters
- `/src/main.rs:379-460` - Same parameter destructuring repeated for each EKS subcommand

**Recommendation:** Create parameter structs:
```rust
struct EksContext<'a> {
    env_name: &'a str,
    namespace: &'a str,
    pod_type: &'a str,
    profile: Option<&'a str>,
}
```

---

## 2. Anti-Patterns

### 2.1 Magic Numbers and Strings (High Priority)

| Location | Value | Context |
|----------|-------|---------|
| `/src/jira.rs:11` | `8765` | Hardcoded OAuth callback port |
| `/src/github.rs:85` | `(limit * 5).min(100)` | Fetch multiplier for filtering |
| `/src/github.rs:166` | `(limit / repos.len() as u32).max(5)` | Per-repo limit calculation |
| `/src/aws.rs:451` | `("maxResults", "100")` | Hardcoded pagination limit |
| `/src/commands/eks.rs:329` | `["web", "api", "worker", "celery", "redis", "nginx", "db"]` | Hardcoded pod type keywords |
| `/src/commands/log.rs:60` | `Duration::from_millis(100)` | Poll interval |
| `/src/aws.rs:728` | `60` iterations, `5` second sleep | Wait timeout calculation (300s total) |
| `/src/github.rs:222-228` | `55`, `52`, `25`, `22` | String truncation lengths |
| `/src/jira.rs:508-515` | `45`, `42`, `20`, `17` | String truncation lengths |

**Recommendation:** Extract to constants or configuration:
```rust
const OAUTH_CALLBACK_PORT: u16 = 8765;
const FETCH_MULTIPLIER: u32 = 5;
const MAX_FETCH_LIMIT: u32 = 100;
const DISPLAY_TITLE_MAX_LEN: usize = 55;
```

### 2.2 Hardcoded API URLs (Medium Priority)

| Location | URL |
|----------|-----|
| `/src/github.rs:5` | `https://api.github.com` |
| `/src/jira.rs:8-9` | `https://auth.atlassian.com/authorize`, `https://auth.atlassian.com/oauth/token` |
| `/src/jira.rs:204` | `https://api.atlassian.com/oauth/token/accessible-resources` |
| `/src/jira.rs:268-337` | Multiple `https://api.atlassian.com/ex/jira/...` endpoints |
| `/src/aws.rs:588` | `https://checkip.amazonaws.com` |

**Recommendation:** Centralize API base URLs as constants, potentially configurable.

### 2.3 Improper Error Handling (Medium Priority)

Silent failures and inconsistent error reporting:

| Location | Issue |
|----------|-------|
| `/src/github.rs:175-178` | Errors printed to stderr, returns empty vec instead of propagating |
| `/src/commands/eks.rs:432-447` | `tail_pod_log` silently returns on spawn failure |
| `/src/utils.rs:8-12` | `run_cmd` returns `Option` losing error context |
| `/src/aws.rs:767-773` | Instance IP timeout silently fails with bail |

**Recommendation:** 
- Use `Result` types consistently
- Log errors with context
- Consider a custom error enum for domain-specific errors

### 2.4 Tight Coupling (Medium Priority)

- `/src/main.rs` directly imports and calls functions from all modules
- Display functions in `github.rs` and `jira.rs` are tightly coupled to table formatting
- AWS operations in `main.rs` directly construct filter structs

**Recommendation:** Consider a command pattern or trait-based dispatch.

---

## 3. Large Units Analysis

### 3.1 Files Over 500 Lines

| File | Lines | Complexity |
|------|-------|------------|
| `/src/aws.rs` | 1230 | Very High - Contains identity, discovery, EC2 CRUD, spawn/kill operations |
| `/src/main.rs` | 866 | High - CLI parsing + all command dispatch |
| `/src/jira.rs` | 705 | High - OAuth + API + Display |
| `/src/commands/eks.rs` | 606 | High - Kubeconfig + Pods + Logs |

### 3.2 Deeply Nested Logic (Medium Priority)

| Location | Nesting Level | Context |
|----------|---------------|---------|
| `/src/main.rs:379-530` | 4+ levels | Match arms with nested if-let and option chains |
| `/src/aws.rs:702-745` | 4 levels | Instance wait loop with nested option unpacking |
| `/src/github.rs:94-128` | 3 levels | Filter chain in closure |
| `/src/commands/eks.rs:352-371` | 4 levels | Timestamp parsing with nested option chains |

**Recommendation:** Extract nested logic to helper functions, use early returns.

---

## 4. Unclear Code

### 4.1 Missing Documentation (Medium Priority)

Functions lacking documentation for complex logic:

| Location | Function | Needs |
|----------|----------|-------|
| `/src/commands/eks.rs:352-378` | `chrono_parse_timestamp` | Explain leap year handling limitations |
| `/src/aws.rs:99-109` | `IdentityInfo::from_arn` | Document ARN format expectations |
| `/src/aws.rs:539-549` | `generate_random_port` | Explain why hash-based, not true random |
| `/src/config.rs:160-180` | `AwsProfiles::profile_for` | Document capability lookup semantics |

### 4.2 Confusing Variable Names (Low Priority)

| Location | Name | Suggestion |
|----------|------|------------|
| `/src/main.rs:335` | `cli_profile` | `aws_profile_override` |
| `/src/commands/eks.rs:312` | `s` | `timestamp_str` |
| `/src/github.rs:145-150` | `filter_actor`, `filter_workflow` | `cloned_actor`, `cloned_workflow` (clarify they're cloned for async) |
| `/src/aws.rs:727` | `_` | Use descriptive `_attempt` or just remove iterator variable |

### 4.3 Complex Control Flow (Medium Priority)

| Location | Issue |
|----------|-------|
| `/src/main.rs:462-530` | GitHub command has complex branching for single vs multi-repo mode |
| `/src/commands/log.rs:24-116` | `view()` handles both modes in one function with deeply nested conditionals |

---

## 5. Dead Code

### 5.1 Unused Items (Low Priority)

| Location | Item | Status |
|----------|------|--------|
| `/src/config.rs:174` | `ec2_profile()` method | Marked `#[allow(dead_code)]` - consider removing or using |
| `/src/utils.rs:5` | `ANSI_COLORS` constant | Only used in `eks.rs` - consider moving |

### 5.2 Potentially Unused Struct Fields

| Location | Field | Usage |
|----------|-------|-------|
| `/src/jira.rs:219` | `SearchResult::total` | Only used in display, often `None` |
| `/src/config.rs:248` | `EnvConfig::emoji` | Used but could be Option since fallback exists |

---

## 6. Duplicate Code

### 6.1 Display Function Patterns (High Priority)

Almost identical table setup code appears in:
- `/src/github.rs:189-205` (`display_workflow_runs`)
- `/src/github.rs:241-262` (`display_project_workflow_runs`)
- `/src/jira.rs:285-310` (`display_projects`)
- `/src/jira.rs:458-490` (`display_search_results`)
- `/src/aws.rs:368-395` (`display_instances`)
- `/src/commands/eks.rs:274-302` (`display_pods`)

**Common Pattern:**
```rust
let mut table = Table::new();
table
    .load_preset(UTF8_FULL)
    .apply_modifier(UTF8_ROUND_CORNERS)
    .set_header(vec![...]);
// ... add rows
println!("{table}");
```

**Recommendation:** Create a `TableBuilder` helper in `utils.rs`:
```rust
pub fn create_table(headers: &[(&str, Color)]) -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL).apply_modifier(UTF8_ROUND_CORNERS);
    // ... set headers
    table
}
```

### 6.2 Status Icon Matching (Medium Priority)

Identical status-to-icon matching in:
- `/src/github.rs:207-215`
- `/src/github.rs:266-274`

**Recommendation:** Extract to shared function:
```rust
fn workflow_status_icon(status: &str, conclusion: Option<&str>) -> ColoredString
```

### 6.3 Config Loading Pattern (Medium Priority)

Similar patterns in:
- `/src/github.rs:18-32` (`get_github_config_path`, `load_github_config`, `save_github_config`)
- `/src/jira.rs:40-60` (`get_jira_token_path`, `load_jira_config`, `save_jira_config`)

**Recommendation:** Create a generic config helper:
```rust
fn load_json_config<T: Default + DeserializeOwned>(filename: &str) -> Result<T>
fn save_json_config<T: Serialize>(filename: &str, config: &T) -> Result<()>
```

### 6.4 String Truncation (Low Priority)

Same truncation pattern appears 8+ times:
```rust
if s.len() > MAX {
    format!("{}...", &s[..MAX-3])
} else {
    s.clone()
}
```

**Locations:** `/src/github.rs:222,228,281,287`, `/src/jira.rs:504,508,515`, `/src/aws.rs:383`

**Recommendation:** Add to `utils.rs`:
```rust
pub fn truncate(s: &str, max_len: usize) -> String
```

### 6.5 Log Line Colorization (Low Priority)

Colorization logic could be shared between:
- `/src/commands/log.rs:15-22` (`colorize_log_line`)
- `/src/commands/eks.rs:437-448` (inline in `tail_pod_log`)

---

## 7. Architecture Issues

### 7.1 Module Organization (High Priority)

Current flat structure doesn't scale well:

```
src/
  aws.rs          # 1230 lines - too large
  github.rs       # 484 lines
  jira.rs         # 705 lines
  config.rs       # 310 lines
  main.rs         # 866 lines - too large
  utils.rs        # 59 lines
  commands/
    eks.rs        # 606 lines
    log.rs        # 142 lines
    mod.rs        # 2 lines
```

**Recommended Structure:**
```
src/
  main.rs              # CLI parsing only (~200 lines)
  config.rs            # Settings types
  utils.rs             # Shared utilities
  
  aws/
    mod.rs             # Re-exports
    identity.rs        # whoami, get_identity
    discovery.rs       # profile discovery
    ec2/
      mod.rs
      list.rs          # list_instances, display_instances
      spawn.rs         # spawn_instance, kill_instance
      
  github/
    mod.rs
    api.rs             # API calls
    display.rs         # Table formatting
    
  jira/
    mod.rs
    oauth.rs           # OAuth flow
    api.rs             # API calls
    display.rs         # Table formatting
    
  commands/
    mod.rs
    eks/
      mod.rs
      kubeconfig.rs    # Kubeconfig management
      pods.rs          # Pod operations
      logs.rs          # Log tailing
    log.rs             # Local log viewing
```

### 7.2 Dependency Direction (Medium Priority)

- `main.rs` depends on everything (expected for CLI entry point)
- `commands/eks.rs` depends on `config`, `utils`, and AWS SDK
- `github.rs` and `jira.rs` are independent (good)

**Issue:** No clear abstraction layer between CLI and business logic.

**Recommendation:** Consider a service layer:
```rust
// services/eks_service.rs
pub struct EksService { /* ... */ }
impl EksService {
    pub async fn connect(&self, env: &str, pod_num: usize) -> Result<()>;
    pub async fn tail_logs(&self, env: &str, pattern: &str) -> Result<()>;
}
```

### 7.3 Configuration Coupling (Medium Priority)

`Settings` struct is passed around extensively. Consider:
- Extracting environment-specific config into smaller focused structs
- Using dependency injection for testability

---

## 8. Prioritized Improvement Plan

### Phase 1: Quick Wins (1-2 hours each)

1. **Extract magic numbers to constants** - All files
2. **Create `truncate()` helper** - `/src/utils.rs`
3. **Create `create_table()` helper** - `/src/utils.rs`
4. **Extract `workflow_status_icon()` function** - `/src/github.rs`

### Phase 2: Moderate Refactoring (2-4 hours each)

5. **Split `/src/aws.rs`** into `aws/identity.rs`, `aws/ec2.rs`, `aws/spawn.rs`, `aws/discovery.rs`
6. **Extract EKS command handlers** from `main.rs` to `commands/eks/handlers.rs`
7. **Create generic JSON config helpers** - reduce duplication in github.rs and jira.rs
8. **Split `spawn_instance()`** into smaller functions
9. **Create `EksContext` parameter struct**

### Phase 3: Architectural Improvements (4-8 hours each)

10. **Introduce service layer** for EKS operations
11. **Split `/src/jira.rs`** into `jira/oauth.rs`, `jira/api.rs`, `jira/display.rs`
12. **Improve error handling** with custom error types
13. **Add missing documentation** for complex functions

---

## 9. Metrics Summary

| Metric | Current | Target |
|--------|---------|--------|
| Largest file | 1230 lines | <400 lines |
| Longest function | ~530 lines | <50 lines |
| Files with >500 lines | 4 | 0 |
| Magic numbers | ~20 | 0 (all in constants) |
| Duplicate code blocks | ~8 patterns | <3 |

---

## 10. Testing Considerations

Before refactoring:
1. Ensure existing tests pass (`just check`)
2. Add tests for any extracted functions
3. Consider property-based tests for parsing functions

Test files found:
- `/tests/unit.rs`
- `/tests/integration.rs`
- `/tests/unit_tests/` (environment, common modules)
- `/tests/integration_tests/` (CLI tests)

---

*This analysis was generated by automated code review. Manual inspection may reveal additional issues or context that changes priorities.*
