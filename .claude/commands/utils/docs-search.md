Search docs index for matching sections.

## Usage

```bash
hu utils docs-search index.json "authentication"
hu utils docs-search index.json "error handling" -n 5
```

## Arguments

| Arg | Description |
|-----|-------------|
| `INDEX` | Path to index file (JSON, from `docs-index`) |
| `QUERY` | Search query (required) |

## Options

| Flag | Description |
|------|-------------|
| `-n, --limit` | Limit number of results |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu utils docs-index` | Build the index first |
| `hu utils docs-section <FILE> <HEADING>` | Extract matched section |
