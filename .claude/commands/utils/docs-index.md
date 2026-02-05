Build heading index for markdown files.

## Usage

```bash
hu utils docs-index                       # Index current directory
hu utils docs-index doc/                  # Index specific directory
hu utils docs-index -o index.json         # Save index to file
```

## Arguments

| Arg | Description |
|-----|-------------|
| `PATH` | Directory to index (default: .) |

## Options

| Flag | Description |
|------|-------------|
| `-o, --output` | Output index to file (JSON) |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu utils docs-search <INDEX> <QUERY>` | Search the index |
| `hu utils docs-section <FILE> <HEADING>` | Extract a section |
