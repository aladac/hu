Show message history for a Slack channel.

## Usage

```bash
hu slack history "#general"           # Last 20 messages
hu slack history "#general" -l 50     # Last 50 messages
hu slack history "#general" -j        # JSON output
```

## Arguments

| Arg | Description |
|-----|-------------|
| `CHANNEL` | Channel name or ID (required) |

## Options

| Flag | Description |
|------|-------------|
| `-l, --limit` | Number of messages (default: 20) |
| `-j, --json` | Output as JSON |
