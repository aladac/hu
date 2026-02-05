Analyze CI failures and output investigation context.

Usage:
```bash
hu gh fix                         # Current branch's latest failed run
hu gh fix --pr 123                # From specific PR
hu gh fix --run 456789            # From specific run ID
hu gh fix -b feature-branch       # From specific branch
hu gh fix -r owner/repo           # Specific repo
hu gh fix -j                      # JSON output
```

Options:
- `--pr` - PR number
- `--run` - Workflow run ID
- `-b, --branch` - Branch name
- `-r, --repo` - Repository in owner/repo format
- `-j, --json` - Output as JSON

Output (markdown):
- Fix report with repository, PR, and run info
- Per-failure: test file, language, source files to investigate, error text
- Re-run commands for each language (rspec, cargo test, pytest, jest)

Output (JSON with `-j`):
- Structured report with test_files, source_files, and failure details

Default Action: Analyze failures for current branch: `hu gh fix $ARGUMENTS`

Related Commands:
- `hu gh failures` - Extract raw test failures
- `hu gh runs` - List workflow runs
- `hu gh prs` - List open PRs
