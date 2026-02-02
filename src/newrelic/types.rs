//! New Relic data types

use serde::{Deserialize, Serialize};

/// New Relic incident
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Incident {
    /// Incident ID
    pub incident_id: String,
    /// Title
    pub title: String,
    /// Priority (CRITICAL, HIGH, MEDIUM, LOW)
    #[serde(default)]
    pub priority: String,
    /// State (CREATED, ACTIVATED, CLOSED)
    #[serde(default)]
    pub state: String,
    /// Account ID
    pub account_ids: Vec<i64>,
    /// Created at timestamp
    #[serde(default)]
    pub created_at: Option<i64>,
    /// Closed at timestamp
    #[serde(default)]
    pub closed_at: Option<i64>,
}

/// New Relic issue (groups incidents)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// Issue ID
    pub issue_id: String,
    /// Title
    pub title: Vec<String>,
    /// Priority
    #[serde(default)]
    pub priority: String,
    /// State
    #[serde(default)]
    pub state: String,
    /// Entity names
    #[serde(default)]
    pub entity_names: Vec<String>,
    /// Created at
    pub created_at: Option<i64>,
    /// Closed at
    pub closed_at: Option<i64>,
    /// Activated at
    pub activated_at: Option<i64>,
}

/// Output format
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_debug() {
        let incident = Incident {
            incident_id: "INC123".to_string(),
            title: "Test Incident".to_string(),
            priority: "CRITICAL".to_string(),
            state: "CLOSED".to_string(),
            account_ids: vec![12345],
            created_at: Some(1704067200000),
            closed_at: None,
        };
        let debug = format!("{:?}", incident);
        assert!(debug.contains("Incident"));
        assert!(debug.contains("INC123"));
    }

    #[test]
    fn test_incident_clone() {
        let incident = Incident {
            incident_id: "INC123".to_string(),
            title: "Test".to_string(),
            priority: "HIGH".to_string(),
            state: "ACTIVATED".to_string(),
            account_ids: vec![1, 2, 3],
            created_at: None,
            closed_at: None,
        };
        let cloned = incident.clone();
        assert_eq!(cloned.incident_id, incident.incident_id);
        assert_eq!(cloned.account_ids, incident.account_ids);
    }

    #[test]
    fn test_issue_debug() {
        let issue = Issue {
            issue_id: "ISS123".to_string(),
            title: vec!["Test Issue".to_string()],
            priority: "MEDIUM".to_string(),
            state: "CREATED".to_string(),
            entity_names: vec!["service-a".to_string()],
            created_at: Some(1704067200000),
            closed_at: None,
            activated_at: None,
        };
        let debug = format!("{:?}", issue);
        assert!(debug.contains("Issue"));
        assert!(debug.contains("ISS123"));
    }

    #[test]
    fn test_issue_clone() {
        let issue = Issue {
            issue_id: "ISS123".to_string(),
            title: vec!["Title 1".to_string(), "Title 2".to_string()],
            priority: "LOW".to_string(),
            state: "CLOSED".to_string(),
            entity_names: vec![],
            created_at: None,
            closed_at: Some(1704067200000),
            activated_at: Some(1704067100000),
        };
        let cloned = issue.clone();
        assert_eq!(cloned.issue_id, issue.issue_id);
        assert_eq!(cloned.title, issue.title);
    }

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::default();
        assert!(matches!(format, OutputFormat::Table));
    }

    #[test]
    fn test_output_format_clone() {
        let format = OutputFormat::Json;
        let cloned = format.clone();
        assert!(matches!(cloned, OutputFormat::Json));
    }

    #[test]
    fn test_output_format_debug() {
        let format = OutputFormat::Table;
        let debug = format!("{:?}", format);
        assert_eq!(debug, "Table");
    }

    #[test]
    fn test_incident_serde_default() {
        // Test that serde default works for optional fields
        let json = r#"{"incidentId":"INC1","title":"Test","accountIds":[1]}"#;
        let incident: Incident = serde_json::from_str(json).unwrap();
        assert_eq!(incident.priority, "");
        assert_eq!(incident.state, "");
        assert!(incident.created_at.is_none());
    }

    #[test]
    fn test_issue_serde_default() {
        let json = r#"{"issueId":"ISS1","title":["Test"]}"#;
        let issue: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.priority, "");
        assert_eq!(issue.state, "");
        assert!(issue.entity_names.is_empty());
    }
}
