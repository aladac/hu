List recent NewRelic issues.

## Usage

```bash
hu newrelic issues               # Last 25 issues
hu newrelic issues -l 50         # More results
hu newrelic issues --json        # JSON output
```

## Options

| Flag | Description |
|------|-------------|
| `-l, --limit` | Maximum results (default: 25) |
| `--json` | Output as JSON |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu newrelic incidents` | List incidents |
| `hu newrelic query <NRQL>` | Run NRQL query |
