Authenticate with Jira via OAuth 2.0.

## Usage

```bash
hu jira auth
```

Opens browser for OAuth flow. Token is stored in `~/.config/hu/credentials.toml`.

## Setup

Requires Jira configuration in `~/.config/hu/settings.toml`:

```toml
[jira]
host = "yourorg.atlassian.net"
```
