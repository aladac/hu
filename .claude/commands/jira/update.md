Update a Jira ticket.

## Usage

```bash
hu jira update PROJ-123 --status "In Progress"
hu jira update PROJ-123 --assign me
hu jira update PROJ-123 --summary "New title"
```

## Arguments

| Arg | Description |
|-----|-------------|
| `KEY` | Ticket key, e.g., PROJ-123 (required) |

## Options

| Flag | Description |
|------|-------------|
| `--summary` | New summary/title |
| `--status` | New status (triggers transition) |
| `--assign` | Assign to user (account ID or "me") |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu jira show <KEY>` | View ticket details first |
