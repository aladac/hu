# hu - Feature Plan

A unified CLI for dev workflows: Kubernetes pods, Jira tickets, GitHub PRs/Actions, and AWS pipelines.

## !!! READ-ONLY TOOL !!!

**This tool is for STATUS REPORTING and LOG VIEWING only.**
**NO write, execute, modify, create, or delete operations.**

## Current Features

- [x] AWS SSO login integration
- [x] EKS cluster connection (prod/dev/stg)
- [x] Pod discovery and filtering by type
- [x] Interactive shell access with custom prompts
- [x] Multi-pod log tailing with color-coded output
- [x] Jira OAuth 2.0 authentication
- [x] Jira ticket lookup and search
- [x] Jira project listing
- [x] GitHub token authentication
- [x] GitHub Actions workflow runs listing
- [x] GitHub Actions filtering by actor, workflow name
- [x] Config-based defaults (repo, actor, workflow)

## Project Configuration

Link a Jira project to GitHub repos for unified workflow tracking.

### Config File (`~/.config/hu/project.json`)

```json
{
  "default_project": "BFR",
  "projects": {
    "BFR": {
      "name": "Booking Flow Rewrite",
      "jira_key": "BFR",
      "github_repos": [
        "fusetechnologies/fuseignited-cms"
      ],
      "github_actor": "aladac",
      "github_workflow": "Rspec tests for ruby on rails development",
      "pipeline": "cms"
    }
  }
}
```

## Planned Features

### Dashboard Summary (`hu status` or `hu`)

Display a unified view of your current work:

```
BFR - Booking Flow Rewrite
==========================

Jira Tasks (assigned to you):
  1. BFR-7423  In Progress   Add Allianz and Protect insurance
  2. BFR-7353  In Review     Calculate package full amount
  3. BFR-3838  To Do         Extract Checkout Parameters
  ... (top 10)

GitHub PRs (open):
  3 open PRs on fusetechnologies/fuseignited-cms
  #5721  BFR-7423-protect-insurance    Ready for review
  #5718  BFR-7353-calculate-package    Changes requested
  #5715  BFR-3838-extract-checkout     Draft

GitHub Actions (Rspec by aladac):
  2 running, 8 successful, 2 failed
  #31567  running   BFR-7279-Package-...  2m 34s
  #31566  success   BFR-7353-calculat...  5m ago

Pipeline (cms):
  Status: Succeeded
  Last run: 2h ago
```

### Command Structure

```bash
# Dashboard (default)
hu                                # Show project dashboard
hu status                         # Same as above
hu status -p INF                  # Different project

# Jira (existing + enhancements)
hu jira                           # List assigned tasks (default project)
hu jira BFR-123                   # Show ticket details
hu jira search "auth bug"         # Search tickets

# GitHub (existing + enhancements)
hu gh runs                        # Workflow runs (config defaults)
hu gh runs --ok                   # Only successful runs
hu gh prs                         # List open PRs for project repos
hu gh prs --mine                  # PRs by current user

# Pipeline (new)
hu pipeline                       # Show default pipeline status
hu pipeline cms                   # Specific pipeline
hu pipeline --logs                # View execution logs

# EKS (existing)
hu eks                            # List pods
hu eks -p 1                       # Connect to pod
hu eks --log                      # Tail logs
```

### Linking Jira to GitHub

- Extract Jira ticket key from branch names (e.g., `BFR-7423-protect-insurance`)
- Match GitHub PRs/Actions to Jira tickets
- Show related PRs when viewing a Jira ticket
- Show Jira ticket details when viewing a PR

### AWS CodePipeline Integration (READ-ONLY)

- List pipeline executions
- Show pipeline stage status (Source -> Build -> Deploy)
- Display execution history
- View execution logs

## Architecture

```
src/
├── main.rs              # CLI entry, command routing
├── aws.rs               # SSO session management
├── jira.rs              # Jira OAuth + API client
├── github.rs            # GitHub token + API client
├── pipeline.rs          # CodePipeline status (planned)
├── project.rs           # Project config management (planned)
├── dashboard.rs         # Unified status view (planned)
├── utils.rs             # Shared utilities
└── commands/
    └── eks.rs           # Pod access
```

## Config Locations

- `~/.config/hu/settings.toml` - General settings
- `~/.config/hu/github.json` - GitHub token + defaults
- `~/.config/hu/jira_token.json` - Jira OAuth tokens
- `~/.config/hu/project.json` - Project mappings (planned)

## Dependencies

Current:
- `clap` - CLI argument parsing
- `reqwest` - HTTP client (GitHub, Jira APIs)
- `tokio` - Async runtime
- `serde` - Serialization
- `colored` - Terminal colors
- `comfy-table` - Table display
- `indicatif` - Progress spinners
- `anyhow/thiserror` - Error handling

Planned:
- `aws-sdk-codepipeline` - Pipeline status
