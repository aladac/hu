List Sentry issues.

## Usage

```bash
hu sentry issues                          # All issues (default 25)
hu sentry issues -p myproject             # Filter by project
hu sentry issues -q "is:unresolved"       # Search query
hu sentry issues -l 50 --json             # More results, JSON
```

## Options

| Flag | Description |
|------|-------------|
| `-p, --project` | Filter by project |
| `-q, --query` | Search query (Sentry search syntax) |
| `-l, --limit` | Maximum results (default: 25) |
| `--json` | Output as JSON |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu sentry show <ISSUE>` | Issue details |
| `hu sentry events <ISSUE>` | Events for an issue |
