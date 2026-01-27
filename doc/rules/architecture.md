# Architecture Principles

## Interface-Agnostic Logic (Critical)

**All business logic must be reusable across interfaces.** When implementing any feature, design it so the same logic can power:
- CLI commands
- REST API endpoints
- MCP server tools
- Future interfaces

```
src/
  lib.rs               # Library exports - the reusable core
  main.rs              # CLI interface (thin wrapper)

  # Core logic (interface-agnostic)
  core/
    mod.rs
    dashboard.rs       # Dashboard data aggregation
    types.rs           # Shared types returned by services

  # Services (business logic, no UI concerns)
  jira/
    service.rs         # JiraService - returns data, no formatting
  gh/
    service.rs         # GithubService
  pagerduty/
    service.rs         # PagerDutyService

  # Interfaces (thin wrappers over core)
  cli/
    mod.rs             # CLI-specific formatting, ratatui
  api/
    mod.rs             # REST API handlers (future)
  mcp/
    mod.rs             # MCP server tools (future)
```

**Service pattern:**
```rust
// jira/service.rs - Pure logic, no display
pub struct JiraService { client: JiraClient }

impl JiraService {
    /// Returns data, not formatted strings
    pub async fn get_sprint_tasks(&self, user: &str) -> Result<Vec<Task>> {
        self.client.get_tasks_for_user(user).await
    }

    pub async fn get_review_requests(&self, user: &str) -> Result<Vec<PullRequest>> {
        // ...
    }
}

// cli/jira.rs - CLI formatting
pub fn display_tasks(tasks: &[Task]) {
    let table = build_tasks_table(tasks);
    print_table(table);
}

// api/jira.rs - REST response (future)
pub fn tasks_to_json(tasks: &[Task]) -> JsonResponse {
    JsonResponse::ok(tasks)
}
```

**Rule:** If you can't easily add a REST endpoint that reuses your logic, refactor.

## Modular Dashboard Views

The CLI displays a dashboard with multiple independent views:

```
+-----------------------------------------------------+
| PR: 3 PRs to review                                 |
| Jira: 5 Jira tasks in current sprint                |
| PR: 2 open PRs awaiting review                      |
| Slack: 4 unread Slack messages                      |
| OnCall: You are ON-CALL until 6pm                   |
| Alert: 1 active PagerDuty alert                     |
| Sentry: 3 unresolved Sentry errors                  |
| NewRelic: 2 NewRelic incidents                      |
+-----------------------------------------------------+
```

**Each view is a separate module:**
```rust
// core/dashboard.rs
pub struct DashboardData {
    pub pr_reviews: Vec<PullRequest>,
    pub jira_tasks: Vec<Task>,
    pub open_prs: Vec<PullRequest>,
    pub slack_messages: Vec<Message>,
    pub oncall_status: OnCallStatus,
    pub pagerduty_alerts: Vec<Alert>,
    pub sentry_errors: Vec<Issue>,
    pub newrelic_incidents: Vec<Incident>,
}

impl DashboardData {
    /// Fetch all data concurrently
    pub async fn fetch(ctx: &AppContext) -> Result<Self> {
        let (pr_reviews, jira_tasks, open_prs, slack, oncall, alerts, errors, incidents) = tokio::join!(
            ctx.github.get_review_requests(),
            ctx.jira.get_sprint_tasks(),
            ctx.github.get_open_prs(),
            ctx.slack.get_unread_messages(),
            ctx.pagerduty.get_oncall_status(),
            ctx.pagerduty.get_active_alerts(),
            ctx.sentry.get_unresolved_issues(),
            ctx.newrelic.get_open_incidents(),
        );

        Ok(Self { /* ... */ })
    }
}

// Each view can be displayed independently
pub trait DashboardView {
    fn fetch(&self) -> Result<ViewData>;
    fn render(&self, data: &ViewData) -> Widget;
}
```

**Views are composable:**
```rust
// User can configure which views to show
hu dashboard                    # All views
hu dashboard --only jira,gh     # Just Jira and GitHub
hu dashboard --except slack     # Everything except Slack
```

## Separation of Concerns

- CLI parsing in `main.rs`
- Business logic in services (interface-agnostic)
- Display/formatting in interface modules (cli/, api/)
- Data fetching in clients

## Dependency Direction

```
Interfaces (cli, api, mcp)
        |
        v
    Services (business logic)
        |
        v
    Clients (API calls)
        |
        v
    Types (data structures)
```

- Interfaces depend on services
- Services depend on clients and types
- Clients depend on types
- Types depend on nothing
