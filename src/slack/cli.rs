use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum SlackCommand {
    /// List unread messages
    Messages,
    /// List channels
    Channels,
    /// Send a message
    Send {
        /// Channel name or ID
        channel: String,
        /// Message text
        message: String,
    },
}
