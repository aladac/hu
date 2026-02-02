//! Slack HTTP client
//!
//! Handles API requests with Bot token authentication.

use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;

use super::config::{load_config, SlackConfig};

const SLACK_API_URL: &str = "https://slack.com/api";

/// Slack API client
pub struct SlackClient {
    config: SlackConfig,
    http: Client,
}

impl SlackClient {
    /// Create a new Slack client
    pub fn new() -> Result<Self> {
        let config = load_config()?;
        let http = Client::builder()
            .user_agent("hu-cli/0.1.0")
            .no_proxy()
            .build()
            .map_err(|e| anyhow::anyhow!(format!("Failed to create HTTP client: {}", e)))?;
        Ok(Self { config, http })
    }

    /// Get a reference to the current config
    #[must_use]
    pub const fn config(&self) -> &SlackConfig {
        &self.config
    }

    /// Get the bot token
    fn bot_token(&self) -> Result<&str> {
        self.config
            .oauth
            .bot_token
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("bot_token not configured".to_string()))
    }

    /// Get the user token (required for search API)
    fn user_token(&self) -> Result<&str> {
        self.config.oauth.user_token.as_deref().ok_or_else(|| {
            anyhow::anyhow!("user_token not configured (required for search)".to_string())
        })
    }

    /// Make a GET request to the Slack API
    pub async fn get<T: DeserializeOwned>(&self, method: &str) -> Result<T> {
        let url = format!("{}/{}", SLACK_API_URL, method);
        let token = self.bot_token()?;

        // debug!("GET {}", url);

        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a GET request with query parameters
    pub async fn get_with_params<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}/{}", SLACK_API_URL, method);
        let token = self.bot_token()?;

        // debug!("GET {} with params {:?}", url, params);

        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .query(params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a GET request using user token (required for search API)
    pub async fn get_with_user_token<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}/{}", SLACK_API_URL, method);
        let token = self.user_token()?;

        // debug!("GET {} with user token, params {:?}", url, params);

        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .query(params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the Slack API
    pub async fn post<T, B>(&self, method: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: serde::Serialize + Sync,
    {
        let url = format!("{}/{}", SLACK_API_URL, method);
        let token = self.bot_token()?;

        // debug!("POST {}", url);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json; charset=utf-8")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request using user token (required for conversations.mark)
    pub async fn post_with_user_token<T, B>(&self, method: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: serde::Serialize + Sync,
    {
        let url = format!("{}/{}", SLACK_API_URL, method);
        let token = self.user_token()?;

        // debug!("POST {} with user token", url);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json; charset=utf-8")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle API response and check for Slack-specific errors
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        let text = response.text().await?;

        // Slack returns { "ok": false, "error": "..." } for API errors
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Parse error: {}: {}", e, &text[..text.len().min(200)]))?;

        if let Some(ok) = value.get("ok").and_then(serde_json::Value::as_bool) {
            if !ok {
                let error = value
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                return Err(anyhow::anyhow!(error.to_string()));
            }
        }

        serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Parse error: {}: {}", e, &text[..text.len().min(200)]))
    }
}
