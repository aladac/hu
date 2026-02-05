Web search using Brave Search API.

## Usage

```bash
hu utils web-search "rust async patterns"         # Search + fetch top 3
hu utils web-search "rust async patterns" -l       # List results only
hu utils web-search "rust async patterns" -n 5     # Fetch top 5
hu utils web-search "rust async patterns" -o out.md # Save to file
```

## Arguments

| Arg | Description |
|-----|-------------|
| `QUERY` | Search query (required) |

## Options

| Flag | Description |
|------|-------------|
| `-n, --results` | Number of results to fetch content from (default: 3) |
| `-l, --list` | Only show search results (don't fetch content) |
| `-o, --output` | Output to file instead of stdout |

Requires Brave Search API key in `~/.config/hu/credentials.toml`.
