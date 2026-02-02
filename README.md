# hu

Dev workflow CLI for Claude Code integration.

## Install

```bash
cargo install --path .
```

## Commands

```
hu jira        Jira operations (tickets, sprint, search)
hu gh          GitHub operations (prs, failures)
hu slack       Slack operations (messages, channels)
hu sentry      Sentry (issues, errors)
hu newrelic    NewRelic (incidents, queries)
hu utils       Utility commands (fetch-html, grep, web-search, docs)
hu context     Session context tracking
hu read        Smart file reading
```

### Jira

```bash
hu jira auth                   # OAuth 2.0 authentication
hu jira tickets                # List my tickets in current sprint
hu jira sprint                 # Show all issues in current sprint
hu jira search <query>         # Search tickets using JQL
hu jira show <ticket>          # Show ticket details
hu jira update <ticket>        # Update a ticket
```

### GitHub

```bash
hu gh login --token <PAT>      # Authenticate with PAT
hu gh prs                      # List your open PRs
hu gh failures                 # Extract test failures from CI
  --pr <number>                #   PR number (default: current branch)
  -r, --repo <owner/repo>      #   Repository
```

### Slack

```bash
hu slack auth                  # Authenticate with Slack
  --token <xoxb-...>           #   Bot token
  --user-token <xoxp-...>      #   User token (for search)
hu slack channels              # List channels
hu slack info <channel>        # Show channel details
hu slack send <channel> <msg>  # Send message
hu slack history <channel>     # Show message history
  --limit <n>                  #   Number of messages (default: 20)
hu slack search <query>        # Search messages
  -n, --count <n>              #   Max results (default: 20)
hu slack users                 # List users
hu slack config                # Show configuration status
hu slack whoami                # Show current user info
hu slack tidy                  # Mark channels as read if no mentions
  --dry-run                    #   Preview without marking
```

### Sentry

```bash
hu sentry auth <token>         # Set auth token
hu sentry config               # Show configuration status
hu sentry issues               # List unresolved issues
hu sentry show <id>            # Show issue details
hu sentry events <id>          # List events for an issue
```

### NewRelic

```bash
hu newrelic auth <key>         # Set API key
  --account <id>               #   Account ID (required)
hu newrelic config             # Show configuration status
hu newrelic issues             # List recent issues
  --limit <n>                  #   Max issues (default: 25)
hu newrelic incidents          # List recent incidents
  --limit <n>                  #   Max incidents (default: 25)
hu newrelic query <nrql>       # Run NRQL query
hu nr ...                      # Alias: nr -> newrelic
```

### Utils

```bash
# Fetch HTML and convert to markdown
hu utils fetch-html <url>
  -c, --content                # Extract main content only
  -s, --summary                # First N paragraphs + headings
  -l, --links                  # Extract links only
  -H, --headings               # Extract headings (outline)
  --selector <css>             # CSS selector (e.g., "article")
  -o, --output <file>          # Output to file

# Smart grep with token-saving
hu utils grep <pattern> [path]
  --refs                       # File:line references only
  --unique                     # Deduplicate similar matches
  --ranked                     # Sort by relevance
  -n, --limit <n>              # Limit results
  --signature                  # Function/class signature only
  -g, --glob <pattern>         # File glob (e.g., "*.rs")
  -i, --ignore-case            # Case insensitive

# Web search
hu utils web-search <query>
  -n, --results <n>            # Number of results (default: 3)
  -l, --list                   # Show results only (don't fetch)
  -o, --output <file>          # Output to file

# Documentation indexing
hu utils docs-index [path]     # Build heading index (JSON)
  -o, --output <file>          # Output index to file
hu utils docs-search <idx> <q> # Search docs index
  -n, --limit <n>              # Limit results
hu utils docs-section <f> <h>  # Extract section from markdown
```

### Context Tracking

Prevent duplicate file reads in Claude Code sessions:

```bash
hu context track <file>        # Mark file as loaded
hu context check <file>        # Check if already in context
hu context summary             # Show all tracked files
hu context clear               # Reset tracking
```

### Smart File Reading

Token-efficient file reading for AI agents:

```bash
hu read <file>
  -o, --outline                # Show functions, structs, classes
  -i, --interface              # Public API only
  -a, --around <line>          # Lines around line number
  -n, --context <n>            # Context lines (default: 10)
  -d, --diff                   # Git diff
  --commit <ref>               # Diff against commit (default: HEAD)
```

## Development

```bash
just check    # fmt + clippy (must pass)
just test     # run tests (must pass)
just build    # build release
```

## License

MIT
