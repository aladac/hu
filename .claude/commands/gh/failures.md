Extract test failures from failed CI runs.

## Usage

```bash
hu gh failures                # Latest failed run (current branch)
hu gh failures --pr 123       # From specific PR
hu gh failures -r owner/repo  # Specific repo
```

## Options

| Flag | Description |
|------|-------------|
| `--pr` | PR number (defaults to current branch's PR) |
| `-r, --repo` | Repository in owner/repo format |

## Default Action

Get failures from latest failed run on current branch:

```bash
hu gh failures
```

## Output

For each failure:
- File path and line number
- Test description
- Error message

## Decision Point

After extracting failures, **ask user**:
- "Would you like me to investigate the failures?"
- "Should I try to fix the failing tests?"

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu gh prs` | List open PRs |
