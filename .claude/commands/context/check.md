Check if file(s) are already in context.

## Usage

```bash
hu context check src/main.rs
hu context check src/main.rs src/lib.rs
```

## Arguments

| Arg | Description |
|-----|-------------|
| `PATHS...` | File path(s) to check (one or more) |

Returns whether each file has been tracked in the current session.
