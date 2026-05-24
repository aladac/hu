use clap::{Parser, Subcommand};

use crate::context::ContextCommand;
use crate::cron::CronCommand;
use crate::data::DataCommand;
use crate::docs::DocsCommand;
use crate::install::InstallCommand;
use crate::mcp::McpCommand;
use crate::newrelic::NewRelicCommand;
use crate::read::ReadArgs;
use crate::setup::SetupCommand;
use crate::shell::ShellCommand;
use crate::utils::UtilsCommand;

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
    /// NewRelic (incidents, queries)
    #[command(name = "newrelic", alias = "nr")]
    NewRelic {
        #[command(subcommand)]
        cmd: Option<NewRelicCommand>,
    },

    /// Utility commands (fetch-html, grep)
    Utils {
        #[command(subcommand)]
        cmd: Option<UtilsCommand>,
    },

    /// Session context tracking (prevent duplicate file reads)
    Context {
        #[command(subcommand)]
        cmd: Option<ContextCommand>,
    },

    /// Smart file reading (outline, interface, around, diff)
    Read(ReadArgs),

    /// Claude Code session data (sync, stats, search)
    Data {
        #[command(subcommand)]
        cmd: Option<DataCommand>,
    },

    /// Install hu hooks and commands to Claude Code
    Install {
        #[command(subcommand)]
        cmd: Option<InstallCommand>,
    },

    /// Documentation management (add, get, list, remove, sync)
    Docs {
        #[command(subcommand)]
        cmd: Option<DocsCommand>,
    },

    /// Cron job management (add, list, remove)
    Cron {
        #[command(subcommand)]
        cmd: Option<CronCommand>,
    },

    /// Shell command wrappers (ls, etc.)
    Shell {
        #[command(subcommand)]
        cmd: Option<ShellCommand>,
    },

    /// MCP server for Claude Code integration
    Mcp {
        #[command(subcommand)]
        cmd: Option<McpCommand>,
    },

    /// Universal fresh-host bootstrap (packages, dotfiles, ssh)
    Setup {
        #[command(subcommand)]
        cmd: Option<SetupCommand>,
    },
}
