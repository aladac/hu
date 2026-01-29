use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;

use super::cli::WebSearchArgs;
use super::fetch_html::extract_summary;
use crate::util::{load_credentials, BraveCredentials};

// ============================================================================
// Types
// ============================================================================

/// A single search result from Brave API
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub description: String,
}

/// Web results container from Brave API
#[derive(Debug, Deserialize)]
pub struct WebResults {
    #[serde(default)]
    pub results: Vec<SearchResult>,
}

/// Top-level Brave API response
#[derive(Debug, Deserialize)]
pub struct BraveSearchResponse {
    #[serde(default)]
    pub web: Option<WebResults>,
}

/// Fetched content for a search result
#[derive(Debug)]
pub struct FetchedResult {
    pub title: String,
    pub url: String,
    pub description: String,
    pub content: Option<String>,
}

// ============================================================================
// Client trait
// ============================================================================

/// Trait for Brave Search API operations
#[async_trait::async_trait]
pub trait BraveSearchApi {
    async fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>>;
}

/// Production client for Brave Search
pub struct BraveSearchClient {
    api_key: String,
    http: reqwest::Client,
}

impl BraveSearchClient {
    pub fn new(api_key: String) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("hu-cli/0.1")
            .build()
            .expect("Failed to build HTTP client");
        Self { api_key, http }
    }

    pub fn from_credentials(creds: &BraveCredentials) -> Self {
        Self::new(creds.api_key.clone())
    }
}

#[async_trait::async_trait]
impl BraveSearchApi for BraveSearchClient {
    async fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
            urlencoding::encode(query),
            count
        );

        let response = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await
            .context("Failed to call Brave Search API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Brave API error {}: {}", status, body);
        }

        let data: BraveSearchResponse = response
            .json()
            .await
            .context("Failed to parse Brave API response")?;

        Ok(data.web.map(|w| w.results).unwrap_or_default())
    }
}

// ============================================================================
// HTTP fetcher trait
// ============================================================================

/// Trait for fetching URL content
#[async_trait::async_trait]
pub trait HttpFetcher {
    async fn fetch(&self, url: &str) -> Result<String>;
}

/// Production HTTP fetcher
pub struct DefaultHttpFetcher {
    http: reqwest::Client,
}

impl Default for DefaultHttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultHttpFetcher {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("hu-cli/0.1")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");
        Self { http }
    }
}

#[async_trait::async_trait]
impl HttpFetcher for DefaultHttpFetcher {
    async fn fetch(&self, url: &str) -> Result<String> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch {}", url))?;

        response
            .text()
            .await
            .with_context(|| format!("Failed to read response from {}", url))
    }
}

// ============================================================================
// Service
// ============================================================================

/// Search and optionally fetch content from results
pub async fn search_and_fetch(
    api: &impl BraveSearchApi,
    fetcher: &impl HttpFetcher,
    query: &str,
    count: usize,
    fetch_content: bool,
) -> Result<Vec<FetchedResult>> {
    let results = api.search(query, count).await?;

    let mut fetched = Vec::new();
    for result in results.into_iter().take(count) {
        let content = if fetch_content {
            match fetcher.fetch(&result.url).await {
                Ok(html) => Some(extract_summary(&html)),
                Err(_) => None,
            }
        } else {
            None
        };

        fetched.push(FetchedResult {
            title: result.title,
            url: result.url,
            description: result.description,
            content,
        });
    }

    Ok(fetched)
}

/// Format results as markdown
pub fn format_results(results: &[FetchedResult], include_content: bool) -> String {
    let mut output = String::new();

    for (i, result) in results.iter().enumerate() {
        output.push_str(&format!("## {}. {}\n", i + 1, result.title));
        output.push_str(&format!("**URL:** {}\n\n", result.url));

        if !result.description.is_empty() {
            output.push_str(&format!("> {}\n\n", result.description));
        }

        if include_content {
            if let Some(content) = &result.content {
                output.push_str("### Content\n\n");
                output.push_str(content);
                output.push_str("\n\n");
            } else {
                output.push_str("*Content unavailable*\n\n");
            }
        }

        output.push_str("---\n\n");
    }

    output.trim_end().to_string()
}

// ============================================================================
// Handler
// ============================================================================

/// Handle the `hu utils web-search` command
pub async fn run(args: WebSearchArgs) -> Result<()> {
    let creds = load_credentials()?;
    let brave = creds
        .brave
        .context("Brave API key not configured. Add [brave] section to credentials.toml")?;

    let client = BraveSearchClient::from_credentials(&brave);
    let fetcher = DefaultHttpFetcher::new();

    let fetch_content = !args.list;
    let results =
        search_and_fetch(&client, &fetcher, &args.query, args.results, fetch_content).await?;

    let output = format_results(&results, fetch_content);

    if let Some(path) = args.output {
        fs::write(&path, &output).with_context(|| format!("Failed to write to {}", path))?;
        eprintln!("Written to {}", path);
    } else {
        println!("{}", output);
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations for testing
    struct MockBraveApi {
        results: Vec<SearchResult>,
    }

    #[async_trait::async_trait]
    impl BraveSearchApi for MockBraveApi {
        async fn search(&self, _query: &str, count: usize) -> Result<Vec<SearchResult>> {
            Ok(self.results.iter().take(count).cloned().collect())
        }
    }

    struct MockFetcher {
        content: String,
    }

    #[async_trait::async_trait]
    impl HttpFetcher for MockFetcher {
        async fn fetch(&self, _url: &str) -> Result<String> {
            Ok(self.content.clone())
        }
    }

    struct FailingFetcher;

    #[async_trait::async_trait]
    impl HttpFetcher for FailingFetcher {
        async fn fetch(&self, url: &str) -> Result<String> {
            bail!("Failed to fetch {}", url)
        }
    }

    fn sample_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                title: "Result One".to_string(),
                url: "https://example.com/one".to_string(),
                description: "First result description".to_string(),
            },
            SearchResult {
                title: "Result Two".to_string(),
                url: "https://example.com/two".to_string(),
                description: "Second result description".to_string(),
            },
            SearchResult {
                title: "Result Three".to_string(),
                url: "https://example.com/three".to_string(),
                description: "".to_string(),
            },
        ]
    }

    #[test]
    fn search_result_debug() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://test.com".to_string(),
            description: "Desc".to_string(),
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("SearchResult"));
        assert!(debug.contains("Test"));
    }

    #[test]
    fn search_result_clone() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://test.com".to_string(),
            description: "Desc".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.title, result.title);
        assert_eq!(cloned.url, result.url);
    }

    #[test]
    fn web_results_debug() {
        let results = WebResults { results: vec![] };
        let debug = format!("{:?}", results);
        assert!(debug.contains("WebResults"));
    }

    #[test]
    fn brave_search_response_debug() {
        let response = BraveSearchResponse { web: None };
        let debug = format!("{:?}", response);
        assert!(debug.contains("BraveSearchResponse"));
    }

    #[test]
    fn fetched_result_debug() {
        let result = FetchedResult {
            title: "Test".to_string(),
            url: "https://test.com".to_string(),
            description: "Desc".to_string(),
            content: Some("Content".to_string()),
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("FetchedResult"));
    }

    #[test]
    fn brave_client_new() {
        let client = BraveSearchClient::new("test_key".to_string());
        assert_eq!(client.api_key, "test_key");
    }

    #[test]
    fn brave_client_from_credentials() {
        let creds = BraveCredentials {
            api_key: "creds_key".to_string(),
        };
        let client = BraveSearchClient::from_credentials(&creds);
        assert_eq!(client.api_key, "creds_key");
    }

    #[test]
    fn default_http_fetcher_new() {
        let fetcher = DefaultHttpFetcher::new();
        let _ = format!("{:?}", fetcher.http);
    }

    #[test]
    fn default_http_fetcher_default() {
        let fetcher = DefaultHttpFetcher::default();
        let _ = format!("{:?}", fetcher.http);
    }

    #[tokio::test]
    async fn search_and_fetch_without_content() {
        let api = MockBraveApi {
            results: sample_results(),
        };
        let fetcher = MockFetcher {
            content: "<p>Test</p>".to_string(),
        };

        let results = search_and_fetch(&api, &fetcher, "test", 2, false)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Result One");
        assert!(results[0].content.is_none());
    }

    #[tokio::test]
    async fn search_and_fetch_with_content() {
        let api = MockBraveApi {
            results: sample_results(),
        };
        let fetcher = MockFetcher {
            content: "<p>Fetched content here</p>".to_string(),
        };

        let results = search_and_fetch(&api, &fetcher, "test", 2, true)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].content.is_some());
        assert!(results[0].content.as_ref().unwrap().contains("Fetched"));
    }

    #[tokio::test]
    async fn search_and_fetch_handles_fetch_failure() {
        let api = MockBraveApi {
            results: sample_results(),
        };
        let fetcher = FailingFetcher;

        let results = search_and_fetch(&api, &fetcher, "test", 2, true)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].content.is_none());
    }

    #[tokio::test]
    async fn search_and_fetch_limits_results() {
        let api = MockBraveApi {
            results: sample_results(),
        };
        let fetcher = MockFetcher {
            content: "<p>Test</p>".to_string(),
        };

        let results = search_and_fetch(&api, &fetcher, "test", 1, false)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn format_results_list_mode() {
        let results = vec![
            FetchedResult {
                title: "Title One".to_string(),
                url: "https://one.com".to_string(),
                description: "Description one".to_string(),
                content: None,
            },
            FetchedResult {
                title: "Title Two".to_string(),
                url: "https://two.com".to_string(),
                description: "".to_string(),
                content: None,
            },
        ];

        let output = format_results(&results, false);
        assert!(output.contains("## 1. Title One"));
        assert!(output.contains("**URL:** https://one.com"));
        assert!(output.contains("> Description one"));
        assert!(output.contains("## 2. Title Two"));
        assert!(!output.contains("### Content"));
    }

    #[test]
    fn format_results_with_content() {
        let results = vec![FetchedResult {
            title: "Title".to_string(),
            url: "https://test.com".to_string(),
            description: "Desc".to_string(),
            content: Some("The actual content".to_string()),
        }];

        let output = format_results(&results, true);
        assert!(output.contains("### Content"));
        assert!(output.contains("The actual content"));
    }

    #[test]
    fn format_results_content_unavailable() {
        let results = vec![FetchedResult {
            title: "Title".to_string(),
            url: "https://test.com".to_string(),
            description: "Desc".to_string(),
            content: None,
        }];

        let output = format_results(&results, true);
        assert!(output.contains("*Content unavailable*"));
    }

    #[test]
    fn format_results_empty() {
        let results: Vec<FetchedResult> = vec![];
        let output = format_results(&results, false);
        assert!(output.is_empty());
    }

    #[test]
    fn brave_search_response_deserialize() {
        let json = r#"{"web": {"results": [{"title": "Test", "url": "https://t.com", "description": "D"}]}}"#;
        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.web.is_some());
        let web = response.web.unwrap();
        assert_eq!(web.results.len(), 1);
        assert_eq!(web.results[0].title, "Test");
    }

    #[test]
    fn brave_search_response_deserialize_empty() {
        let json = r#"{}"#;
        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.web.is_none());
    }

    #[test]
    fn brave_search_response_deserialize_empty_web() {
        let json = r#"{"web": {"results": []}}"#;
        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.web.is_some());
        assert!(response.web.unwrap().results.is_empty());
    }

    #[test]
    fn search_result_deserialize_missing_description() {
        let json = r#"{"title": "Test", "url": "https://t.com"}"#;
        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, "Test");
        assert_eq!(result.description, "");
    }
}
