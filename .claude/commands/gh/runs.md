List GitHub workflow runs.

Usage:
```bash
hu gh runs                        # All runs (default 20)
hu gh runs BFR-1234               # Find runs for a ticket
hu gh runs -b feature-branch      # Filter by branch
hu gh runs -s failure             # Filter by status
hu gh runs -n 10                  # Limit results
hu gh runs -r owner/repo          # Specific repo
hu gh runs -j                     # JSON output
```

Arguments:
- `TICKET` - Ticket key to find runs for (e.g. BFR-1234, optional)

Options:
- `-s, --status` - Filter: queued, in_progress, completed, success, failure
- `-b, --branch` - Filter by branch name
- `-r, --repo` - Repository in owner/repo format
- `-n, --limit` - Max results (default: 20)
- `-j, --json` - Output as JSON

Ticket Search: When a ticket key is provided, searches PRs matching that ticket in title/branch, then fetches runs for those branches.

Default Action: List recent workflow runs: `hu gh runs $ARGUMENTS`

Related Commands:
- `hu gh failures` - Extract test failures from CI
- `hu gh fix` - Analyze failures and get fix context
- `hu gh prs` - List open PRs
