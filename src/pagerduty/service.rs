//! PagerDuty service layer - business logic that returns data
//!
//! Functions in this module accept trait objects and return typed data.
//! They never print - that's the CLI layer's job.

use anyhow::{bail, Result};

use super::client::PagerDutyApi;
use super::config::{self, PagerDutyConfig};
use super::types::{Incident, IncidentStatus, Oncall, User};

/// Options for listing on-calls
#[derive(Debug, Default)]
pub struct OncallOptions {
    /// Filter by escalation policy ID
    pub policy_id: Option<String>,
    /// Filter by schedule ID
    pub schedule_id: Option<String>,
}

/// Options for listing incidents
#[derive(Debug)]
pub struct IncidentOptions {
    /// Filter by statuses
    pub statuses: Vec<IncidentStatus>,
    /// Maximum number of results
    pub limit: usize,
}

impl Default for IncidentOptions {
    fn default() -> Self {
        Self {
            statuses: vec![IncidentStatus::Triggered, IncidentStatus::Acknowledged],
            limit: 25,
        }
    }
}

/// Get current configuration
pub fn get_config() -> Result<PagerDutyConfig> {
    config::load_config()
}

/// Save API token
pub fn save_auth(token: &str) -> Result<()> {
    config::save_config(token)
}

/// Check if API is configured, return error if not
pub fn ensure_configured(config: &PagerDutyConfig) -> Result<()> {
    if !config.is_configured() {
        bail!(
            "PagerDuty not configured. Run: hu pagerduty auth <token>\n\
             Or set PAGERDUTY_API_TOKEN environment variable."
        );
    }
    Ok(())
}

/// List on-call users
pub async fn list_oncalls(api: &impl PagerDutyApi, opts: &OncallOptions) -> Result<Vec<Oncall>> {
    let policy_ids = opts.policy_id.as_ref().map(|p| vec![p.clone()]);
    let schedule_ids = opts.schedule_id.as_ref().map(|s| vec![s.clone()]);

    api.list_oncalls(schedule_ids.as_deref(), policy_ids.as_deref())
        .await
}

/// List incidents (alerts = triggered + acknowledged only)
pub async fn list_alerts(api: &impl PagerDutyApi, limit: usize) -> Result<Vec<Incident>> {
    let statuses = vec![IncidentStatus::Triggered, IncidentStatus::Acknowledged];
    api.list_incidents(&statuses, limit).await
}

/// List incidents with options
pub async fn list_incidents(
    api: &impl PagerDutyApi,
    opts: &IncidentOptions,
) -> Result<Vec<Incident>> {
    api.list_incidents(&opts.statuses, opts.limit).await
}

/// Get a single incident by ID
pub async fn get_incident(api: &impl PagerDutyApi, id: &str) -> Result<Incident> {
    api.get_incident(id).await
}

/// Get current user info
pub async fn get_current_user(api: &impl PagerDutyApi) -> Result<User> {
    api.get_current_user().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pagerduty::types::{EscalationPolicy, Schedule, Service, Urgency};

    /// Mock PagerDuty API for testing
    struct MockApi {
        oncalls: Vec<Oncall>,
        incidents: Vec<Incident>,
        user: User,
    }

    impl MockApi {
        fn new() -> Self {
            Self {
                oncalls: vec![],
                incidents: vec![],
                user: User {
                    id: "USER123".to_string(),
                    name: Some("Test User".to_string()),
                    summary: None,
                    email: "test@example.com".to_string(),
                    html_url: "https://pagerduty.com/users/USER123".to_string(),
                },
            }
        }

        fn with_oncalls(mut self, oncalls: Vec<Oncall>) -> Self {
            self.oncalls = oncalls;
            self
        }

        fn with_incidents(mut self, incidents: Vec<Incident>) -> Self {
            self.incidents = incidents;
            self
        }
    }

    impl PagerDutyApi for MockApi {
        async fn get_current_user(&self) -> Result<User> {
            Ok(self.user.clone())
        }

        async fn list_oncalls(
            &self,
            _schedule_ids: Option<&[String]>,
            _escalation_policy_ids: Option<&[String]>,
        ) -> Result<Vec<Oncall>> {
            Ok(self.oncalls.clone())
        }

        async fn list_incidents(
            &self,
            statuses: &[IncidentStatus],
            limit: usize,
        ) -> Result<Vec<Incident>> {
            let filtered: Vec<Incident> = self
                .incidents
                .iter()
                .filter(|i| statuses.contains(&i.status))
                .take(limit)
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn get_incident(&self, id: &str) -> Result<Incident> {
            self.incidents
                .iter()
                .find(|i| i.id == id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Incident not found: {}", id))
        }

        async fn list_services(&self) -> Result<Vec<Service>> {
            Ok(vec![])
        }
    }

    fn make_oncall(user_name: &str, policy_name: &str) -> Oncall {
        Oncall {
            user: User {
                id: format!("U{}", user_name),
                name: Some(user_name.to_string()),
                summary: None,
                email: format!("{}@example.com", user_name.to_lowercase()),
                html_url: String::new(),
            },
            schedule: Some(Schedule {
                id: "SCHED1".to_string(),
                name: "Primary".to_string(),
                html_url: String::new(),
            }),
            escalation_policy: EscalationPolicy {
                id: "POL1".to_string(),
                name: policy_name.to_string(),
                html_url: String::new(),
            },
            escalation_level: 1,
            start: None,
            end: None,
        }
    }

    fn make_incident(id: &str, title: &str, status: IncidentStatus) -> Incident {
        Incident {
            id: id.to_string(),
            incident_number: 123,
            title: title.to_string(),
            status,
            urgency: Urgency::High,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            html_url: String::new(),
            service: Service {
                id: "SVC1".to_string(),
                name: "Test Service".to_string(),
                status: "active".to_string(),
                html_url: String::new(),
            },
            assignments: vec![],
        }
    }

    #[tokio::test]
    async fn list_oncalls_returns_data() {
        let api = MockApi::new().with_oncalls(vec![
            make_oncall("Alice", "Engineering"),
            make_oncall("Bob", "Platform"),
        ]);

        let result = list_oncalls(&api, &OncallOptions::default()).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].user.display_name(), "Alice");
    }

    #[tokio::test]
    async fn list_alerts_filters_statuses() {
        let api = MockApi::new().with_incidents(vec![
            make_incident("INC1", "Alert 1", IncidentStatus::Triggered),
            make_incident("INC2", "Alert 2", IncidentStatus::Resolved),
            make_incident("INC3", "Alert 3", IncidentStatus::Acknowledged),
        ]);

        let result = list_alerts(&api, 10).await.unwrap();
        assert_eq!(result.len(), 2); // Only triggered and acknowledged
        assert!(result.iter().all(|i| i.status != IncidentStatus::Resolved));
    }

    #[tokio::test]
    async fn list_incidents_respects_limit() {
        let api = MockApi::new().with_incidents(vec![
            make_incident("INC1", "Alert 1", IncidentStatus::Triggered),
            make_incident("INC2", "Alert 2", IncidentStatus::Triggered),
            make_incident("INC3", "Alert 3", IncidentStatus::Triggered),
        ]);

        let opts = IncidentOptions {
            statuses: vec![IncidentStatus::Triggered],
            limit: 2,
        };
        let result = list_incidents(&api, &opts).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn get_incident_returns_matching() {
        let api = MockApi::new().with_incidents(vec![
            make_incident("INC1", "Alert 1", IncidentStatus::Triggered),
            make_incident("INC2", "Alert 2", IncidentStatus::Resolved),
        ]);

        let result = get_incident(&api, "INC2").await.unwrap();
        assert_eq!(result.id, "INC2");
        assert_eq!(result.title, "Alert 2");
    }

    #[tokio::test]
    async fn get_incident_not_found() {
        let api = MockApi::new();
        let result = get_incident(&api, "MISSING").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_current_user_returns_data() {
        let api = MockApi::new();
        let result = get_current_user(&api).await.unwrap();
        assert_eq!(result.display_name(), "Test User");
    }

    #[test]
    fn ensure_configured_fails_without_token() {
        let config = PagerDutyConfig::default();
        let result = ensure_configured(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not configured"));
    }

    #[test]
    fn ensure_configured_succeeds_with_token() {
        let config = PagerDutyConfig {
            api_token: Some("test-token".to_string()),
            ..Default::default()
        };
        let result = ensure_configured(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn oncall_options_default() {
        let opts = OncallOptions::default();
        assert!(opts.policy_id.is_none());
        assert!(opts.schedule_id.is_none());
    }

    #[test]
    fn incident_options_default() {
        let opts = IncidentOptions::default();
        assert_eq!(opts.limit, 25);
        assert_eq!(opts.statuses.len(), 2);
    }
}
