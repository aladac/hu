use serde::{Deserialize, Serialize};

/// CI check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CiStatus {
    Success,
    Pending,
    Failed,
    #[default]
    Unknown,
}

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
    #[serde(skip)]
    pub ci_status: Option<CiStatus>,
}

/// A test failure extracted from CI logs
#[derive(Debug, Clone)]
pub struct TestFailure {
    /// The spec file path (e.g., "spec/models/user_spec.rb")
    pub spec_file: String,
    /// The failure message/output
    pub failure_text: String,
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
            ci_status: None,
        };

        let json = serde_json::to_string(&pr).unwrap();
        assert!(json.contains("Fix bug"));
        assert!(json.contains("org/repo"));
    }

    #[test]
    fn pull_request_deserializes() {
        let json = r#"{
            "number": 456,
            "title": "Add feature",
            "html_url": "https://github.com/org/repo/pull/456",
            "state": "open",
            "repo_full_name": "org/repo",
            "created_at": "2024-01-15T10:00:00Z",
            "updated_at": "2024-01-15T12:00:00Z"
        }"#;

        let pr: PullRequest = serde_json::from_str(json).unwrap();
        assert_eq!(pr.number, 456);
        assert_eq!(pr.title, "Add feature");
        assert!(pr.ci_status.is_none());
    }

    #[test]
    fn ci_status_default_is_unknown() {
        let status = CiStatus::default();
        assert_eq!(status, CiStatus::Unknown);
    }

    #[test]
    fn ci_status_equality() {
        assert_eq!(CiStatus::Success, CiStatus::Success);
        assert_eq!(CiStatus::Pending, CiStatus::Pending);
        assert_eq!(CiStatus::Failed, CiStatus::Failed);
        assert_eq!(CiStatus::Unknown, CiStatus::Unknown);
        assert_ne!(CiStatus::Success, CiStatus::Failed);
    }

    #[test]
    fn ci_status_clone() {
        let status = CiStatus::Success;
        let cloned = status;
        assert_eq!(status, cloned);
    }

    #[test]
    fn ci_status_debug_format() {
        let debug_str = format!("{:?}", CiStatus::Pending);
        assert!(debug_str.contains("Pending"));
    }

    #[test]
    fn test_failure_clone() {
        let failure = TestFailure {
            spec_file: "./spec/test_spec.rb:10".to_string(),
            failure_text: "expected true, got false".to_string(),
        };
        let cloned = failure.clone();
        assert_eq!(cloned.spec_file, failure.spec_file);
        assert_eq!(cloned.failure_text, failure.failure_text);
    }

    #[test]
    fn test_failure_debug_format() {
        let failure = TestFailure {
            spec_file: "./spec/test_spec.rb:10".to_string(),
            failure_text: "error".to_string(),
        };
        let debug_str = format!("{:?}", failure);
        assert!(debug_str.contains("TestFailure"));
        assert!(debug_str.contains("spec_file"));
    }

    #[test]
    fn pull_request_clone() {
        let pr = PullRequest {
            number: 123,
            title: "Test".to_string(),
            html_url: "https://github.com/a/b/pull/123".to_string(),
            state: "open".to_string(),
            repo_full_name: "a/b".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            ci_status: Some(CiStatus::Success),
        };
        let cloned = pr.clone();
        assert_eq!(cloned.number, pr.number);
        assert_eq!(cloned.ci_status, pr.ci_status);
    }

    #[test]
    fn pull_request_debug_format() {
        let pr = PullRequest {
            number: 1,
            title: "T".to_string(),
            html_url: "u".to_string(),
            state: "open".to_string(),
            repo_full_name: "r".to_string(),
            created_at: "c".to_string(),
            updated_at: "u".to_string(),
            ci_status: None,
        };
        let debug_str = format!("{:?}", pr);
        assert!(debug_str.contains("PullRequest"));
    }
}
