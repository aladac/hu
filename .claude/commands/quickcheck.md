# Quick Check

Run all help commands to verify the CLI is working. No auth required.

## Instructions

Run these commands sequentially to verify the build works:

```bash
cargo build
```

Then test all help commands:

```bash
# Top-level
hu --help
hu --version

# install
hu install --help
hu install list --help
hu install run --help
hu install preview --help

# data
hu data --help
hu data sync --help
hu data config --help
hu data session --help
hu data session list --help
hu data stats --help
hu data search --help
hu data tools --help
hu data todos --help
hu data branches --help
hu data errors --help
hu data pricing --help

# context
hu context --help
hu context track --help
hu context check --help
hu context summary --help
hu context clear --help

# read
hu read --help

# utils
hu utils --help
hu utils fetch-html --help
hu utils grep --help
hu utils web-search --help
hu utils docs-index --help
hu utils docs-search --help
hu utils docs-section --help

# gh (auth required for actual use)
hu gh --help
hu gh login --help
hu gh prs --help
hu gh runs --help
hu gh failures --help
hu gh fix --help

# jira (auth required for actual use)
hu jira --help
hu jira auth --help
hu jira sprint --help
hu jira tickets --help
hu jira search --help
hu jira show --help
hu jira update --help

# slack (auth required for actual use)
hu slack --help
hu slack auth --help
hu slack config --help
hu slack channels --help
hu slack info --help
hu slack send --help
hu slack history --help
hu slack search --help
hu slack users --help
hu slack whoami --help
hu slack tidy --help

# pagerduty (auth required for actual use)
hu pagerduty --help
hu pagerduty auth --help
hu pagerduty config --help
hu pagerduty oncall --help
hu pagerduty alerts --help
hu pagerduty incidents --help
hu pagerduty show --help
hu pagerduty whoami --help

# sentry (auth required for actual use)
hu sentry --help
hu sentry auth --help
hu sentry config --help
hu sentry issues --help
hu sentry show --help
hu sentry events --help

# newrelic (auth required for actual use)
hu newrelic --help
hu newrelic auth --help
hu newrelic config --help
hu newrelic issues --help
hu newrelic incidents --help
hu newrelic query --help

# eks (auth required for actual use)
hu eks --help
hu eks list --help
hu eks exec --help
hu eks logs --help

# pipeline (auth required for actual use)
hu pipeline --help
hu pipeline list --help
hu pipeline status --help
hu pipeline history --help
```

Then test functional commands (no auth needed):

```bash
# install
hu install list

# data (run sync first if db empty)
hu data config
hu data sync

# context
hu context summary
hu context clear

# read
hu read -o src/main.rs

# utils
hu utils fetch-html https://example.com
hu utils grep "fn main" src/

# config commands (show auth status)
hu slack config
hu pagerduty config
hu sentry config
hu newrelic config
```
