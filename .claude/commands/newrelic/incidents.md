List recent NewRelic incidents.

## Usage

```bash
hu newrelic incidents               # Last 25 incidents
hu newrelic incidents -l 50         # More results
hu newrelic incidents --json        # JSON output
```

## Options

| Flag | Description |
|------|-------------|
| `-l, --limit` | Maximum results (default: 25) |
| `--json` | Output as JSON |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu newrelic issues` | List issues |
| `hu newrelic query <NRQL>` | Run NRQL query |
