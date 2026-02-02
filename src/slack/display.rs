//! Slack output formatting

use anyhow::{Context, Result};
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};
use regex::Regex;
use std::collections::HashMap;

use super::types::{OutputFormat, SlackChannel, SlackMessage, SlackSearchResult, SlackUser};

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Clean up Slack message text for display
/// - Converts <@U04H482TK6Z|Adam Ladachowski> to @Adam Ladachowski
/// - Converts <@U04H482TK6Z> to @username using lookup
/// - Converts <#C12345678|channel-name> to #channel-name
/// - Converts <URL|text> to text
fn clean_message_text(text: &str, user_lookup: &HashMap<String, String>) -> String {
    // Match Slack's special formatting: <...>
    let re = Regex::new(r"<([^>]+)>").unwrap();

    re.replace_all(text, |caps: &regex::Captures| {
        let content = &caps[1];

        if let Some(rest) = content.strip_prefix('@') {
            // User mention: <@U12345|Display Name> or <@U12345>
            if let Some((_, display_name)) = rest.split_once('|') {
                format!("@{}", display_name)
            } else {
                // No display name, look up user ID
                user_lookup
                    .get(rest)
                    .map(|name| format!("@{}", name))
                    .unwrap_or_else(|| format!("@{}", rest))
            }
        } else if let Some(rest) = content.strip_prefix('#') {
            // Channel mention: <#C12345|channel-name>
            if let Some((_, channel_name)) = rest.split_once('|') {
                format!("#{}", channel_name)
            } else {
                format!("#{}", rest)
            }
        } else if let Some(rest) = content.strip_prefix('!') {
            // Special mention: <!here>, <!channel>, <!everyone>
            format!("@{}", rest)
        } else if content.contains('|') {
            // URL with display text: <https://example.com|Example>
            let (_, display) = content.split_once('|').unwrap();
            display.to_string()
        } else {
            // Plain URL or other
            content.to_string()
        }
    })
    .to_string()
}

/// Format channel name for display
/// Converts mpdm-user1--user2--user3-1 to @user1, @user2, @user3
/// Converts user IDs like U04H482TK6Z to @username using lookup
fn format_channel_name(name: &str, user_lookup: &HashMap<String, String>) -> String {
    if name.starts_with("mpdm-") {
        // Multi-person DM: mpdm-user1--user2--user3-1
        let without_prefix = name.strip_prefix("mpdm-").unwrap_or(name);
        // Remove trailing -1, -2, etc.
        let without_suffix = without_prefix
            .rsplit_once('-')
            .map(|(rest, _)| rest)
            .unwrap_or(without_prefix);
        // Split on -- and format as @mentions
        let users: Vec<String> = without_suffix
            .split("--")
            .map(|u| format!("@{}", u))
            .collect();
        users.join(", ")
    } else if name.starts_with('U')
        && name.len() == 11
        && name.chars().all(|c| c.is_ascii_alphanumeric())
    {
        // User ID (DM): resolve to @username
        user_lookup
            .get(name)
            .map(|n| format!("@{}", n))
            .unwrap_or_else(|| "DM".to_string())
    } else {
        format!("#{}", name)
    }
}

/// Format Unix timestamp to readable date
fn format_timestamp(ts: &str) -> String {
    // Slack timestamps are like "1234567890.123456"
    ts.split('.')
        .next()
        .and_then(|s| s.parse::<i64>().ok())
        .and_then(|secs| chrono::DateTime::from_timestamp(secs, 0))
        .map_or_else(
            || ts.to_string(),
            |dt| dt.format("%Y-%m-%d %H:%M").to_string(),
        )
}

/// Output channels list
pub fn output_channels(channels: &[SlackChannel], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => {
            if channels.is_empty() {
                println!("No channels found.");
                return Ok(());
            }

            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec!["Name", "Type", "Members", "Topic"]);

            for channel in channels {
                let channel_type = if channel.is_private {
                    "private"
                } else {
                    "public"
                };
                let members = channel
                    .num_members
                    .map_or_else(|| "-".to_string(), |n| n.to_string());
                let topic = channel.topic.as_deref().unwrap_or("-");

                table.add_row(vec![
                    Cell::new(format!("#{}", channel.name)).fg(Color::Cyan),
                    Cell::new(channel_type),
                    Cell::new(members),
                    Cell::new(truncate(topic, 40)),
                ]);
            }

            println!("{table}");
            println!("\n{} channels", channels.len());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(channels)
                .context("Failed to serialize channels to JSON")?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Output channel detail
pub fn output_channel_detail(channel: &SlackChannel, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => {
            println!("{}", "-".repeat(60));
            println!("#{} ({})", channel.name, channel.id);
            println!("{}", "-".repeat(60));
            println!(
                "Type:    {}",
                if channel.is_private {
                    "private"
                } else {
                    "public"
                }
            );
            println!("Member:  {}", if channel.is_member { "yes" } else { "no" });
            if let Some(n) = channel.num_members {
                println!("Members: {}", n);
            }
            if let Some(ref topic) = channel.topic {
                println!("\nTopic: {}", topic);
            }
            if let Some(ref purpose) = channel.purpose {
                println!("\nPurpose: {}", purpose);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(channel)
                .context("Failed to serialize channel to JSON")?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Output message history
pub fn output_messages(
    messages: &[SlackMessage],
    channel_name: &str,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Table => {
            if messages.is_empty() {
                println!("No messages found.");
                return Ok(());
            }

            println!("Messages in #{}", channel_name);
            println!("{}", "-".repeat(60));

            for msg in messages.iter().rev() {
                let time = format_timestamp(&msg.ts);
                let user = msg
                    .username
                    .as_deref()
                    .or(msg.user.as_deref())
                    .unwrap_or("unknown");
                let thread = msg
                    .reply_count
                    .map_or(String::new(), |n| format!(" [{} replies]", n));

                println!("[{}] {}: {}{}", time, user, msg.text, thread);
            }

            println!("\n{} messages", messages.len());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(messages)
                .context("Failed to serialize messages to JSON")?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Output search results
pub fn output_search_results(
    results: &SlackSearchResult,
    format: OutputFormat,
    user_lookup: &HashMap<String, String>,
) -> Result<()> {
    match format {
        OutputFormat::Table => {
            if results.matches.is_empty() {
                println!("No messages found.");
                return Ok(());
            }

            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec!["Channel", "User", "Time", "Message"]);

            for m in &results.matches {
                let time = format_timestamp(&m.ts);
                let user = m.username.as_deref().unwrap_or("-");
                let channel = format_channel_name(&m.channel.name, user_lookup);
                let text = clean_message_text(&m.text, user_lookup);

                table.add_row(vec![
                    Cell::new(&channel).fg(Color::Cyan),
                    Cell::new(user),
                    Cell::new(time),
                    Cell::new(truncate(&text, 50)),
                ]);
            }

            println!("{table}");
            println!(
                "\nShowing {} of {} matches",
                results.matches.len(),
                results.total
            );
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(results)
                .context("Failed to serialize search results to JSON")?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Output users list
pub fn output_users(users: &[SlackUser], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => {
            if users.is_empty() {
                println!("No users found.");
                return Ok(());
            }

            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec!["Username", "Name", "Timezone"]);

            for user in users {
                let name = user.real_name.as_deref().unwrap_or("-");
                let tz = user.tz.as_deref().unwrap_or("-");

                table.add_row(vec![
                    Cell::new(format!("@{}", user.name)).fg(Color::Cyan),
                    Cell::new(name),
                    Cell::new(tz),
                ]);
            }

            println!("{table}");
            println!("\n{} users", users.len());
        }
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(users).context("Failed to serialize users to JSON")?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Output config status
pub fn output_config_status(
    is_configured: bool,
    has_user_token: bool,
    team_name: Option<&str>,
    default_channel: &str,
) {
    println!("Slack Configuration");
    println!("{}", "-".repeat(40));
    println!("Bot token:  {}", if is_configured { "Yes" } else { "No" });
    println!(
        "User token: {}",
        if has_user_token {
            "Yes (search enabled)"
        } else {
            "No (search disabled)"
        }
    );
    if let Some(name) = team_name {
        println!("Workspace:  {}", name);
    }
    if !default_channel.is_empty() {
        println!("Default:    {}", default_channel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_very_short_max() {
        assert_eq!(truncate("hello", 3), "...");
    }

    #[test]
    fn test_clean_message_text_user_mention_with_display() {
        let lookup = HashMap::new();
        assert_eq!(
            clean_message_text("<@U12345|John Doe>", &lookup),
            "@John Doe"
        );
    }

    #[test]
    fn test_clean_message_text_user_mention_with_lookup() {
        let mut lookup = HashMap::new();
        lookup.insert("U12345".to_string(), "johndoe".to_string());
        assert_eq!(clean_message_text("<@U12345>", &lookup), "@johndoe");
    }

    #[test]
    fn test_clean_message_text_user_mention_without_lookup() {
        let lookup = HashMap::new();
        assert_eq!(clean_message_text("<@U12345>", &lookup), "@U12345");
    }

    #[test]
    fn test_clean_message_text_channel_mention() {
        let lookup = HashMap::new();
        assert_eq!(clean_message_text("<#C12345|general>", &lookup), "#general");
    }

    #[test]
    fn test_clean_message_text_channel_mention_no_name() {
        let lookup = HashMap::new();
        assert_eq!(clean_message_text("<#C12345>", &lookup), "#C12345");
    }

    #[test]
    fn test_clean_message_text_special_mention() {
        let lookup = HashMap::new();
        assert_eq!(clean_message_text("<!here>", &lookup), "@here");
        assert_eq!(clean_message_text("<!channel>", &lookup), "@channel");
        assert_eq!(clean_message_text("<!everyone>", &lookup), "@everyone");
    }

    #[test]
    fn test_clean_message_text_url_with_display() {
        let lookup = HashMap::new();
        assert_eq!(
            clean_message_text("<https://example.com|Example Site>", &lookup),
            "Example Site"
        );
    }

    #[test]
    fn test_clean_message_text_plain_url() {
        let lookup = HashMap::new();
        assert_eq!(
            clean_message_text("<https://example.com>", &lookup),
            "https://example.com"
        );
    }

    #[test]
    fn test_clean_message_text_mixed() {
        let mut lookup = HashMap::new();
        lookup.insert("U12345".to_string(), "bob".to_string());
        assert_eq!(
            clean_message_text("Hey <@U12345>, check <#C99999|dev>!", &lookup),
            "Hey @bob, check #dev!"
        );
    }

    #[test]
    fn test_format_channel_name_regular() {
        let lookup = HashMap::new();
        assert_eq!(format_channel_name("general", &lookup), "#general");
    }

    #[test]
    fn test_format_channel_name_mpdm() {
        let lookup = HashMap::new();
        assert_eq!(
            format_channel_name("mpdm-alice--bob--charlie-1", &lookup),
            "@alice, @bob, @charlie"
        );
    }

    #[test]
    fn test_format_channel_name_user_id_with_lookup() {
        let mut lookup = HashMap::new();
        lookup.insert("U04H482TK6Z".to_string(), "alice".to_string());
        assert_eq!(format_channel_name("U04H482TK6Z", &lookup), "@alice");
    }

    #[test]
    fn test_format_channel_name_user_id_without_lookup() {
        let lookup = HashMap::new();
        assert_eq!(format_channel_name("U04H482TK6Z", &lookup), "DM");
    }

    #[test]
    fn test_format_timestamp_valid() {
        // 2024-01-01 00:00:00 UTC
        let result = format_timestamp("1704067200.123456");
        assert_eq!(result, "2024-01-01 00:00");
    }

    #[test]
    fn test_format_timestamp_no_decimal() {
        let result = format_timestamp("1704067200");
        assert_eq!(result, "2024-01-01 00:00");
    }

    #[test]
    fn test_format_timestamp_invalid() {
        let result = format_timestamp("invalid");
        assert_eq!(result, "invalid");
    }

    #[test]
    fn test_output_channels_empty() {
        // Just verify it doesn't panic
        let channels: Vec<SlackChannel> = vec![];
        let result = output_channels(&channels, OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_channels_json() {
        let channels = vec![SlackChannel {
            id: "C12345".to_string(),
            name: "general".to_string(),
            is_private: false,
            is_member: true,
            topic: Some("General discussion".to_string()),
            purpose: None,
            num_members: Some(100),
            created: 1704067200,
        }];
        let result = output_channels(&channels, OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_channel_detail_table() {
        let channel = SlackChannel {
            id: "C12345".to_string(),
            name: "general".to_string(),
            is_private: true,
            is_member: false,
            topic: Some("Topic".to_string()),
            purpose: Some("Purpose".to_string()),
            num_members: Some(50),
            created: 1704067200,
        };
        let result = output_channel_detail(&channel, OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_messages_empty() {
        let messages: Vec<SlackMessage> = vec![];
        let result = output_messages(&messages, "general", OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_messages_json() {
        let messages = vec![SlackMessage {
            msg_type: "message".to_string(),
            user: Some("U12345".to_string()),
            text: "Hello world".to_string(),
            ts: "1704067200.123456".to_string(),
            thread_ts: None,
            reply_count: Some(5),
            username: Some("alice".to_string()),
        }];
        let result = output_messages(&messages, "general", OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_users_empty() {
        let users: Vec<SlackUser> = vec![];
        let result = output_users(&users, OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_users_json() {
        let users = vec![SlackUser {
            id: "U12345".to_string(),
            team_id: Some("T12345".to_string()),
            name: "alice".to_string(),
            real_name: Some("Alice Smith".to_string()),
            is_bot: false,
            deleted: false,
            tz: Some("America/New_York".to_string()),
        }];
        let result = output_users(&users, OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_search_results_empty() {
        let results = SlackSearchResult {
            total: 0,
            matches: vec![],
        };
        let lookup = HashMap::new();
        let result = output_search_results(&results, OutputFormat::Table, &lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_search_results_json() {
        use crate::slack::types::{SlackSearchChannel, SlackSearchMatch};
        let results = SlackSearchResult {
            total: 1,
            matches: vec![SlackSearchMatch {
                channel: SlackSearchChannel {
                    id: "C12345".to_string(),
                    name: "general".to_string(),
                },
                user: Some("U12345".to_string()),
                username: Some("alice".to_string()),
                text: "Hello world".to_string(),
                ts: "1704067200.123456".to_string(),
                permalink: Some("https://slack.com/...".to_string()),
            }],
        };
        let lookup = HashMap::new();
        let result = output_search_results(&results, OutputFormat::Json, &lookup);
        assert!(result.is_ok());
    }
}
