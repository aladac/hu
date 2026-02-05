Extract a section from a markdown file by heading.

## Usage

```bash
hu utils docs-section README.md "Installation"
hu utils docs-section doc/api.md "Authentication"
```

## Arguments

| Arg | Description |
|-----|-------------|
| `FILE` | Markdown file path (required) |
| `HEADING` | Section heading to extract (required) |

Returns the content under the specified heading, up to the next heading of equal or higher level.
