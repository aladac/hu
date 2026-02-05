Authenticate with Slack (OAuth flow or direct token).

## Usage

```bash
hu slack auth                        # OAuth flow (opens browser)
hu slack auth -t xoxb-...            # Direct bot token
hu slack auth -u xoxp-...            # User token for search
hu slack auth -t xoxb-... -u xoxp-... # Both tokens
```

## Options

| Flag | Description |
|------|-------------|
| `-t, --token` | Bot token (skips OAuth flow) |
| `-u, --user-token` | User token for search API (xoxp-...) |
| `-p, --port` | Local server port for OAuth callback (default: 9877) |

Tokens stored in `~/.config/hu/credentials.toml`.
