List active PagerDuty alerts (triggered + acknowledged).

## Usage

```bash
hu pagerduty alerts              # Active alerts (default 25)
hu pagerduty alerts -l 50        # More results
hu pagerduty alerts --json       # JSON output
```

## Options

| Flag | Description |
|------|-------------|
| `-l, --limit` | Maximum number to show (default: 25) |
| `--json` | Output as JSON |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu pagerduty incidents` | List incidents with filters |
| `hu pagerduty show <ID>` | Incident details |
