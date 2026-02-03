//! Slack integration module
//!
//! Provides commands for interacting with Slack:
//! - Authenticate via OAuth browser flow
//! - List channels
//! - Get channel info
//! - Send messages
//! - View message history
//! - Search messages
//! - List users
//! - Show configuration status
//!
//! # Examples
//!
//! ```no_run
//! use hu::slack::{run, SlackCommands};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // List channels
//!     run(SlackCommands::Channels { json: false }).await?;
//!     Ok(())
//! }
//! ```

mod auth;
mod channels;
mod client;
mod config;
mod display;
mod handlers;
mod messages;
mod search;
mod tidy;
mod types;

use clap::Subcommand;

pub use handlers::run;

/// Slack subcommands
#[derive(Subcommand, Debug)]
pub enum SlackCommands {
    /// Authenticate with Slack (OAuth flow or direct token)
    Auth {
        /// Bot token to save directly (skips OAuth flow)
        #[arg(short, long)]
        token: Option<String>,
        /// User token for search API (xoxp-...)
        #[arg(short, long)]
        user_token: Option<String>,
        /// Local server port for OAuth callback
        #[arg(short, long, default_value = "9877")]
        port: u16,
    },
    /// List channels in the workspace
    Channels {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Show channel details
    Info {
        /// Channel name or ID (e.g., "#general" or "C12345678")
        channel: String,
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Send a message to a channel
    Send {
        /// Channel name or ID
        channel: String,
        /// Message text
        message: String,
    },
    /// Show message history for a channel
    History {
        /// Channel name or ID
        channel: String,
        /// Number of messages to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Search messages
    Search {
        /// Search query
        query: String,
        /// Maximum results to return
        #[arg(short = 'n', long, default_value = "20")]
        count: usize,
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// List users in the workspace
    Users {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Show Slack configuration status
    Config,
    /// Show current user info from token
    Whoami,
    /// Mark channels as read if no direct mentions
    Tidy {
        /// Dry run - show what would be marked without marking
        #[arg(short, long)]
        dry_run: bool,
    },
}

#[cfg(test)]
mod tests;
