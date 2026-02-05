Mark Slack channels as read if no direct mentions.

## Usage

```bash
hu slack tidy           # Mark channels as read
hu slack tidy -d        # Dry run (show what would be marked)
```

## Options

| Flag | Description |
|------|-------------|
| `-d, --dry-run` | Show what would be marked without marking |

Skips channels with unread direct mentions to you.
