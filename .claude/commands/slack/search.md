Search Slack messages.

## Usage

```bash
hu slack search "deployment error"
hu slack search "from:@jane in:#ops" -n 50
hu slack search "deployment" -j
```

## Arguments

| Arg | Description |
|-----|-------------|
| `QUERY` | Search query (required) |

## Options

| Flag | Description |
|------|-------------|
| `-n, --count` | Maximum results (default: 20) |
| `-j, --json` | Output as JSON |

Requires a user token (`xoxp-...`) set via `hu slack auth -u`.
