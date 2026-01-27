use clap::{Parser, Subcommand};

use crate::dashboard::DashboardCommand;
use crate::eks::EksCommand;
use crate::gh::GhCommand;
use crate::jira::JiraCommand;
use crate::newrelic::NewRelicCommand;
use crate::pagerduty::PagerDutyCommand;
use crate::sentry::SentryCommand;
use crate::slack::SlackCommand;

#[derive(Parser)]
#[command(name = "hu")]
#[command(about = "Dev workflow CLI", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Dev dashboard (PRs, tasks, alerts, oncall)
    Dashboard {
        #[command(subcommand)]
        cmd: Option<DashboardCommand>,
    },

    /// Jira operations (tickets, sprint, search)
    Jira {
        #[command(subcommand)]
        cmd: Option<JiraCommand>,
    },

    /// GitHub operations (prs, runs, failures)
    Gh {
        #[command(subcommand)]
        cmd: Option<GhCommand>,
    },

    /// Slack operations (messages, channels)
    Slack {
        #[command(subcommand)]
        cmd: Option<SlackCommand>,
    },

    /// PagerDuty (oncall, alerts)
    #[command(name = "pagerduty", alias = "pd")]
    PagerDuty {
        #[command(subcommand)]
        cmd: Option<PagerDutyCommand>,
    },

    /// Sentry (issues, errors)
    Sentry {
        #[command(subcommand)]
        cmd: Option<SentryCommand>,
    },

    /// NewRelic (incidents, queries)
    #[command(name = "newrelic", alias = "nr")]
    NewRelic {
        #[command(subcommand)]
        cmd: Option<NewRelicCommand>,
    },

    /// EKS pod access (list, exec, logs)
    Eks {
        #[command(subcommand)]
        cmd: Option<EksCommand>,
    },
}
