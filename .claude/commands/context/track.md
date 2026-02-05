Track file(s) as loaded in context.

## Usage

```bash
hu context track src/main.rs
hu context track src/main.rs src/lib.rs src/cli.rs
```

## Arguments

| Arg | Description |
|-----|-------------|
| `PATHS...` | File path(s) to track (one or more) |

Prevents duplicate file reads in a session.

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu context check <PATH>` | Check if already tracked |
| `hu context summary` | Show all tracked files |
| `hu context clear` | Clear tracking |
