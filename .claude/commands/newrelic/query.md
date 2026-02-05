Run a NRQL query against NewRelic.

## Usage

```bash
hu newrelic query "SELECT count(*) FROM Transaction SINCE 1 hour ago"
hu newrelic query "SELECT average(duration) FROM Transaction FACET appName" --json
```

## Arguments

| Arg | Description |
|-----|-------------|
| `NRQL` | NRQL query string (required) |

## Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

## NRQL Examples

| Query | Description |
|-------|-------------|
| `SELECT count(*) FROM Transaction SINCE 1 hour ago` | Transaction count |
| `SELECT average(duration) FROM Transaction FACET appName` | Avg duration by app |
| `SELECT * FROM NrAiIncident SINCE 1 day ago` | Recent AI incidents |
