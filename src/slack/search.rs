//! Slack message search
//!
//! Search messages across channels.

use anyhow::Result;
use serde::Deserialize;

use super::client::SlackClient;
use super::types::{SlackSearchChannel, SlackSearchMatch, SlackSearchResult};

/// Response from search.messages API
#[derive(Deserialize)]
struct SearchResponse {
    messages: MessagesContainer,
}

/// Container for search matches
#[derive(Deserialize)]
struct MessagesContainer {
    total: u32,
    matches: Vec<MatchResponse>,
}

/// Raw match data from API
#[derive(Deserialize)]
struct MatchResponse {
    channel: ChannelResponse,
    user: Option<String>,
    username: Option<String>,
    text: String,
    ts: String,
    permalink: Option<String>,
}

/// Channel info in search response
#[derive(Deserialize)]
struct ChannelResponse {
    id: String,
    name: String,
}

impl From<MatchResponse> for SlackSearchMatch {
    fn from(r: MatchResponse) -> Self {
        Self {
            channel: SlackSearchChannel {
                id: r.channel.id,
                name: r.channel.name,
            },
            user: r.user,
            username: r.username,
            text: r.text,
            ts: r.ts,
            permalink: r.permalink,
        }
    }
}

/// Search messages across the workspace (requires user token)
pub async fn search_messages(
    client: &SlackClient,
    query: &str,
    count: usize,
) -> Result<SlackSearchResult> {
    let count_str = count.to_string();
    let response: SearchResponse = client
        .get_with_user_token(
            "search.messages",
            &[
                ("query", query),
                ("count", &count_str),
                ("sort", "timestamp"),
                ("sort_dir", "desc"),
            ],
        )
        .await?;

    Ok(SlackSearchResult {
        total: response.messages.total,
        matches: response
            .messages
            .matches
            .into_iter()
            .map(SlackSearchMatch::from)
            .collect(),
    })
}
