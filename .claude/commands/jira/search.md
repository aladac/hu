Search tickets using JQL.

## Usage

```bash
hu jira search "project = PROJ AND status = 'In Progress'"
hu jira search "assignee = currentUser() AND resolution = Unresolved"
hu jira search "text ~ 'login bug'"
```

## Arguments

| Arg | Description |
|-----|-------------|
| `QUERY` | JQL query string (required) |

## JQL Examples

| Query | Description |
|-------|-------------|
| `project = PROJ` | All tickets in project |
| `status = "In Progress"` | By status |
| `assignee = currentUser()` | My tickets |
| `text ~ "search term"` | Full text search |
| `created >= -7d` | Last 7 days |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu jira tickets` | My sprint tickets |
| `hu jira show <KEY>` | Ticket details |
