Fetch a URL and convert HTML to markdown.

## Usage

```bash
hu utils fetch-html <URL>                    # Basic fetch
hu utils fetch-html <URL> -c                 # Extract main content only
hu utils fetch-html <URL> -s                 # Summary (headings + first paragraphs)
hu utils fetch-html <URL> -l                 # Extract links only
hu utils fetch-html <URL> -H                 # Extract headings only
hu utils fetch-html <URL> --selector "article"  # CSS selector
hu utils fetch-html <URL> -o out.md          # Write to file
hu utils fetch-html <URL> -r                 # Raw (no filtering)
```

## Arguments

| Arg | Description |
|-----|-------------|
| `URL` | URL to fetch (required) |

## Options

| Flag | Description |
|------|-------------|
| `-c, --content` | Extract main content (strip nav, footer, scripts, ads) |
| `-s, --summary` | Summary (first N paragraphs + headings) |
| `-l, --links` | Extract links only [text](url) |
| `-H, --headings` | Extract headings only (document outline) |
| `--selector` | CSS selector (e.g., "article", "main", ".content") |
| `-o, --output` | Write to file instead of stdout |
| `-r, --raw` | Raw markdown without filtering |

## Default Action

Fetch URL and output full markdown:

```bash
hu utils fetch-html "$ARGUMENTS"
```
