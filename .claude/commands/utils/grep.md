Smart grep with token-saving options.

## Usage

```bash
hu utils grep "pattern"                       # Search current dir
hu utils grep "pattern" src/                  # Search specific path
hu utils grep "pattern" -g "*.rs"             # Filter by glob
hu utils grep "pattern" --refs                # File:line refs only
hu utils grep "pattern" --unique              # Deduplicate matches
hu utils grep "pattern" --ranked              # Sort by relevance
hu utils grep "pattern" --signature           # Function signatures only
hu utils grep "pattern" -n 10                 # Limit results
```

## Arguments

| Arg | Description |
|-----|-------------|
| `PATTERN` | Regex pattern (required) |
| `PATH` | Search path (default: .) |

## Options

| Flag | Description |
|------|-------------|
| `--refs` | File:line references only (no content) |
| `--unique` | Deduplicate similar matches |
| `--ranked` | Sort by relevance (match density) |
| `--signature` | Show function/class signature only |
| `-n, --limit` | Limit number of results |
| `-g, --glob` | File glob pattern (e.g., "*.rs") |
| `-i, --ignore-case` | Case insensitive |
| `--hidden` | Include hidden files |
