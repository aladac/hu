Show CodePipeline status (stages and actions).

## Usage

```bash
hu pipeline status my-pipeline
hu pipeline status my-pipeline -r us-west-2
hu pipeline status my-pipeline --json
```

## Arguments

| Arg | Description |
|-----|-------------|
| `NAME` | Pipeline name (required) |

## Options

| Flag | Description |
|------|-------------|
| `-r, --region` | AWS region |
| `--json` | Output as JSON |

## Output

Shows each stage and action with status (succeeded/failed/in-progress).
