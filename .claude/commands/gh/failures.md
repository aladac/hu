Extract test failures from failed CI runs.

Usage:
```bash
hu gh failures                # Latest failed run (current branch)
hu gh failures --pr 123       # From specific PR
hu gh failures -r owner/repo  # Specific repo
```

Options:
- `--pr` - PR number (defaults to current branch's PR)
- `-r, --repo` - Repository in owner/repo format

Default Action: Get failures from latest failed run on current branch: `hu gh failures $ARGUMENTS`

Output:
- File path and line number
- Test description
- Error message

Decision Point: After extracting failures, ask user whether to investigate or fix the failing tests.

Related Commands:
- `hu gh fix` - Analyze failures and get fix context
- `hu gh runs` - List workflow runs
- `hu gh prs` - List open PRs
