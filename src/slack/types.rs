//! Slack data types and structures

use serde::{Deserialize, Serialize};

/// Slack channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackChannel {
    /// Channel ID (e.g., "C12345678")
    pub id: String,
    /// Channel name (without #)
    pub name: String,
    /// Whether this is a private channel
    pub is_private: bool,
    /// Whether the bot is a member of this channel
    pub is_member: bool,
    /// Channel topic
    pub topic: Option<String>,
    /// Channel purpose
    pub purpose: Option<String>,
    /// Number of members
    pub num_members: Option<u32>,
    /// Creation timestamp
    pub created: i64,
}

/// Slack message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    /// Message type (usually "message")
    #[serde(rename = "type")]
    pub msg_type: String,
    /// User ID who sent the message
    pub user: Option<String>,
    /// Message text
    pub text: String,
    /// Timestamp (unique ID for the message)
    pub ts: String,
    /// Thread timestamp (if this is a reply)
    pub thread_ts: Option<String>,
    /// Number of replies in thread
    pub reply_count: Option<u32>,
    /// User display name (enriched after fetch)
    #[serde(skip_deserializing)]
    pub username: Option<String>,
}

/// Slack user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackUser {
    /// User ID
    pub id: String,
    /// Team ID
    pub team_id: Option<String>,
    /// Username (handle without @)
    pub name: String,
    /// Display name
    pub real_name: Option<String>,
    /// Whether this is a bot
    pub is_bot: bool,
    /// Whether this user is deleted
    pub deleted: bool,
    /// User's timezone
    pub tz: Option<String>,
}

/// Search result match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackSearchMatch {
    /// Channel where the message was posted
    pub channel: SlackSearchChannel,
    /// User ID who posted
    pub user: Option<String>,
    /// Username who posted
    pub username: Option<String>,
    /// Message text
    pub text: String,
    /// Timestamp
    pub ts: String,
    /// Permalink to the message
    pub permalink: Option<String>,
}

/// Channel info in search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackSearchChannel {
    /// Channel ID
    pub id: String,
    /// Channel name
    pub name: String,
}

/// Search results container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackSearchResult {
    /// Total matches found
    pub total: u32,
    /// Matches returned
    pub matches: Vec<SlackSearchMatch>,
}

/// Output format for Slack commands
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Human-readable table format
    #[default]
    Table,
    /// JSON format for scripting
    Json,
}
