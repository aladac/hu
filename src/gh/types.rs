use serde::{Deserialize, Serialize};

/// Pull request data for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub state: String,
    pub repo_full_name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_request_serializes() {
        let pr = PullRequest {
            number: 123,
            title: "Fix bug".to_string(),
            html_url: "https://github.com/org/repo/pull/123".to_string(),
            state: "open".to_string(),
            repo_full_name: "org/repo".to_string(),
            created_at: "2024-01-15T10:00:00Z".to_string(),
            updated_at: "2024-01-15T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&pr).unwrap();
        assert!(json.contains("Fix bug"));
        assert!(json.contains("org/repo"));
    }
}
