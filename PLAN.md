# hu - Feature Plan

A unified CLI for dev workflows: Kubernetes pods, Jira tickets, GitHub PRs/Actions, and AWS pipelines.

## Current Features

- [x] AWS SSO login integration
- [x] EKS cluster connection (prod/dev/stg)
- [x] Pod discovery and filtering by type
- [x] Interactive shell access with custom prompts
- [x] Multi-pod log tailing with color-coded output

## Planned Features

### Jira Integration

- OAuth authentication flow
- Fetch ticket details by ID (`PROJ-123`)
- Display ticket summary, description, status, assignee
- Quick ticket search by project/sprint
- Open ticket in browser

### GitHub Integration

- List PRs for current branch or by number
- Show PR status, reviews, checks
- Monitor GitHub Actions workflow runs
- Display action logs and failure details
- Link PRs to Jira tickets (extract from branch name)

### AWS CodePipeline Integration

- List pipeline executions
- Show pipeline stage status (Source → Build → Deploy)
- Display execution history
- Quick link to AWS Console

### AWS Secrets Manager Integration

- List secrets by prefix/pattern
- Display secret value (with confirmation)
- Copy secret to clipboard
- Show secret metadata (last rotated, etc.)

## Architecture

```
src/
├── main.rs              # CLI entry, command routing
├── lib.rs               # Public API
├── cli/
│   ├── mod.rs           # Clap parser
│   └── commands/
│       ├── eks.rs       # Pod access (current functionality)
│       ├── jira.rs      # Ticket lookup
│       ├── gh.rs        # GitHub PRs and Actions
│       ├── pipeline.rs  # CodePipeline status
│       └── secret.rs    # Secrets Manager
├── auth/
│   ├── aws.rs           # SSO session management
│   ├── jira.rs          # OAuth flow
│   └── github.rs        # GitHub token
├── api/
│   ├── jira.rs          # Jira REST client
│   ├── github.rs        # GitHub API client
│   └── aws.rs           # AWS SDK wrappers
└── ui/
    ├── output.rs        # Consistent formatting
    ├── table.rs         # Table display
    └── progress.rs      # Spinners
```

## Command Examples

```bash
# EKS (existing)
hu eks                            # List pods
hu eks -p 1                       # Connect to pod
hu eks --log                      # Tail logs

# Jira
hu jira PROJ-123                  # Show ticket details
hu jira search "auth bug"         # Search tickets

# GitHub
hu gh pr                          # Show PR for current branch
hu gh pr 456                      # Show specific PR
hu gh actions                     # Show workflow runs
hu gh actions --watch             # Live monitor

# AWS Pipelines
hu pipeline                       # List recent executions
hu pipeline cms-deploy            # Show specific pipeline

# Secrets
hu secret list prod/              # List secrets by prefix
hu secret get prod/api-key        # Show secret value
```

## Dependencies to Add

- `oauth2` - Jira OAuth flow
- `octocrab` - GitHub API
- `aws-sdk-codepipeline` - Pipeline status
- `aws-sdk-secretsmanager` - Secrets access
- `keyring` - Secure token storage
- `open` - Open URLs in browser
