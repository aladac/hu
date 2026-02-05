List events for a Sentry issue.

## Usage

```bash
hu sentry events 12345              # Last 25 events
hu sentry events 12345 -l 50        # More events
hu sentry events 12345 --json       # JSON output
```

## Arguments

| Arg | Description |
|-----|-------------|
| `ISSUE` | Issue ID or short ID (required) |

## Options

| Flag | Description |
|------|-------------|
| `-l, --limit` | Maximum events (default: 25) |
| `--json` | Output as JSON |
