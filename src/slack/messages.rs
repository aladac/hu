//! Slack message operations
//!
//! Send messages and retrieve message history.

use anyhow::Result;
use serde::Deserialize;

use super::client::SlackClient;
use super::types::SlackMessage;

/// Response from conversations.history API
#[derive(Deserialize)]
struct HistoryResponse {
    messages: Vec<MessageResponse>,
}

/// Response from chat.postMessage API
#[derive(Deserialize)]
struct PostMessageResponse {
    ts: String,
    channel: String,
}

/// Raw message data from API
#[derive(Deserialize)]
struct MessageResponse {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    user: Option<String>,
    text: Option<String>,
    ts: String,
    thread_ts: Option<String>,
    reply_count: Option<u32>,
}

impl From<MessageResponse> for SlackMessage {
    fn from(r: MessageResponse) -> Self {
        Self {
            msg_type: r.msg_type.unwrap_or_else(|| "message".to_string()),
            user: r.user,
            text: r.text.unwrap_or_default(),
            ts: r.ts,
            thread_ts: r.thread_ts,
            reply_count: r.reply_count,
            username: None,
        }
    }
}

/// Get message history for a channel
pub async fn get_history(
    client: &SlackClient,
    channel_id: &str,
    limit: usize,
) -> Result<Vec<SlackMessage>> {
    let limit_str = limit.to_string();
    let response: HistoryResponse = client
        .get_with_params(
            "conversations.history",
            &[("channel", channel_id), ("limit", &limit_str)],
        )
        .await?;

    let messages: Vec<SlackMessage> = response
        .messages
        .into_iter()
        .map(SlackMessage::from)
        .collect();

    Ok(messages)
}

/// Send a message to a channel
pub async fn send_message(
    client: &SlackClient,
    channel_id: &str,
    text: &str,
) -> Result<(String, String), anyhow::Error> {
    let body = serde_json::json!({
        "channel": channel_id,
        "text": text,
    });

    let response: PostMessageResponse = client.post("chat.postMessage", &body).await?;

    Ok((response.channel, response.ts))
}
