List PagerDuty incidents with filters.

## Usage

```bash
hu pagerduty incidents                     # All recent incidents
hu pagerduty incidents -s triggered        # Only triggered
hu pagerduty incidents -s acknowledged     # Only acknowledged
hu pagerduty incidents -s active           # Triggered + acknowledged
hu pagerduty incidents -s resolved         # Only resolved
hu pagerduty incidents -l 50 --json        # More results, JSON
```

## Options

| Flag | Description |
|------|-------------|
| `-s, --status` | Filter: triggered, acknowledged, resolved, active |
| `-l, --limit` | Maximum number to show (default: 25) |
| `--json` | Output as JSON |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu pagerduty alerts` | Active alerts only |
| `hu pagerduty show <ID>` | Incident details |
