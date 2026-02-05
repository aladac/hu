Show CodePipeline execution history.

## Usage

```bash
hu pipeline history my-pipeline              # Last 10 executions
hu pipeline history my-pipeline -l 25        # More results
hu pipeline history my-pipeline --json       # JSON output
```

## Arguments

| Arg | Description |
|-----|-------------|
| `NAME` | Pipeline name (required) |

## Options

| Flag | Description |
|------|-------------|
| `-r, --region` | AWS region |
| `-l, --limit` | Maximum results (default: 10) |
| `--json` | Output as JSON |
