List all CodePipeline pipelines.

## Usage

```bash
hu pipeline list                  # Default region
hu pipeline list -r us-west-2     # Specific region
hu pipeline list --json           # JSON output
```

## Options

| Flag | Description |
|------|-------------|
| `-r, --region` | AWS region |
| `--json` | Output as JSON |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu pipeline status <NAME>` | Pipeline stage details |
| `hu pipeline history <NAME>` | Execution history |
