Show who's currently on call.

## Usage

```bash
hu pagerduty oncall                    # All on-call schedules
hu pagerduty oncall -p POLICY_ID       # Filter by escalation policy
hu pagerduty oncall -s SCHEDULE_ID     # Filter by schedule
hu pagerduty oncall --json             # JSON output
```

## Options

| Flag | Description |
|------|-------------|
| `-p, --policy` | Filter by escalation policy ID |
| `-s, --schedule` | Filter by schedule ID |
| `--json` | Output as JSON |
