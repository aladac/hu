use super::*;
use client::SlackClient;
use config::{OAuthConfig, SlackConfig};
use reqwest::Client;

fn make_unconfigured_client() -> SlackClient {
    let config = SlackConfig {
        oauth: OAuthConfig {
            client_id: None,
            client_secret: None,
            bot_token: None,
            user_token: None,
            team_id: None,
            team_name: None,
        },
        default_channel: String::new(),
        is_configured: false,
    };
    let http = Client::builder().build().unwrap();
    SlackClient::with_config(config, http)
}

fn make_configured_client() -> SlackClient {
    let config = SlackConfig {
        oauth: OAuthConfig {
            client_id: None,
            client_secret: None,
            bot_token: Some("xoxb-test-token".to_string()),
            user_token: Some("xoxp-test-token".to_string()),
            team_id: Some("T12345".to_string()),
            team_name: Some("Test Team".to_string()),
        },
        default_channel: "#general".to_string(),
        is_configured: true,
    };
    let http = Client::builder().build().unwrap();
    SlackClient::with_config(config, http)
}

#[test]
fn test_check_configured_when_not_configured() {
    let client = make_unconfigured_client();
    let result = handlers::check_configured(&client);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not configured"));
}

#[test]
fn test_check_configured_when_configured() {
    let client = make_configured_client();
    let result = handlers::check_configured(&client);
    assert!(result.is_ok());
}

#[test]
fn test_slack_commands_debug() {
    let cmd = SlackCommands::Channels { json: false };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Channels"));
}

#[test]
fn test_slack_commands_auth_debug() {
    let cmd = SlackCommands::Auth {
        token: Some("xoxb-test".to_string()),
        user_token: None,
        port: 9877,
    };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Auth"));
    assert!(debug.contains("9877"));
}

#[test]
fn test_slack_commands_info_debug() {
    let cmd = SlackCommands::Info {
        channel: "#general".to_string(),
        json: true,
    };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Info"));
    assert!(debug.contains("#general"));
}

#[test]
fn test_slack_commands_send_debug() {
    let cmd = SlackCommands::Send {
        channel: "#test".to_string(),
        message: "Hello".to_string(),
    };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Send"));
    assert!(debug.contains("Hello"));
}

#[test]
fn test_slack_commands_history_debug() {
    let cmd = SlackCommands::History {
        channel: "#dev".to_string(),
        limit: 50,
        json: false,
    };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("History"));
    assert!(debug.contains("50"));
}

#[test]
fn test_slack_commands_search_debug() {
    let cmd = SlackCommands::Search {
        query: "deploy".to_string(),
        count: 20,
        json: true,
    };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Search"));
    assert!(debug.contains("deploy"));
}

#[test]
fn test_slack_commands_users_debug() {
    let cmd = SlackCommands::Users { json: false };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Users"));
}

#[test]
fn test_slack_commands_config_debug() {
    let cmd = SlackCommands::Config;
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Config"));
}

#[test]
fn test_slack_commands_whoami_debug() {
    let cmd = SlackCommands::Whoami;
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Whoami"));
}

#[test]
fn test_slack_commands_tidy_debug() {
    let cmd = SlackCommands::Tidy { dry_run: true };
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Tidy"));
    assert!(debug.contains("true"));
}

#[test]
fn test_output_format_reexport() {
    // Verify OutputFormat is accessible via types module
    let format = types::OutputFormat::Table;
    assert!(matches!(format, types::OutputFormat::Table));
    let format = types::OutputFormat::Json;
    assert!(matches!(format, types::OutputFormat::Json));
}
