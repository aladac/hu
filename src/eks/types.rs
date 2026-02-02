//! EKS data types

use serde::{Deserialize, Serialize};

/// Kubernetes pod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    /// Pod name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Pod status (Running, Pending, etc.)
    pub status: String,
    /// Ready containers (e.g., "1/1")
    pub ready: String,
    /// Restart count
    pub restarts: u32,
    /// Age (e.g., "2d", "5h")
    pub age: String,
    /// Node name
    #[serde(default)]
    pub node: Option<String>,
}

/// Kubectl configuration
#[derive(Debug, Clone, Default)]
pub struct KubectlConfig {
    /// Kubeconfig context to use
    pub context: Option<String>,
    /// Namespace to use
    pub namespace: Option<String>,
}

/// Output format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Table format
    #[default]
    Table,
    /// JSON format
    Json,
}

/// Kubectl JSON output for pods
#[derive(Debug, Deserialize)]
pub struct PodList {
    /// List of items
    pub items: Vec<PodItem>,
}

/// Single pod item from kubectl JSON
#[derive(Debug, Deserialize)]
pub struct PodItem {
    /// Metadata
    pub metadata: PodMetadata,
    /// Spec
    #[serde(default)]
    pub spec: Option<PodSpec>,
    /// Status
    pub status: PodStatus,
}

/// Pod metadata
#[derive(Debug, Deserialize)]
pub struct PodMetadata {
    /// Pod name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Creation timestamp
    #[serde(rename = "creationTimestamp")]
    pub creation_timestamp: Option<String>,
}

/// Pod spec
#[derive(Debug, Deserialize, Default)]
pub struct PodSpec {
    /// Node name
    #[serde(rename = "nodeName")]
    pub node_name: Option<String>,
    /// Containers
    #[serde(default)]
    #[allow(dead_code)]
    pub containers: Vec<Container>,
}

/// Container spec
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Container {
    /// Container name
    pub name: String,
}

/// Pod status
#[derive(Debug, Deserialize)]
pub struct PodStatus {
    /// Phase (Running, Pending, Succeeded, Failed, Unknown)
    pub phase: String,
    /// Container statuses
    #[serde(rename = "containerStatuses", default)]
    pub container_statuses: Vec<ContainerStatus>,
}

/// Container status
#[derive(Debug, Deserialize)]
pub struct ContainerStatus {
    /// Container name
    #[allow(dead_code)]
    pub name: String,
    /// Ready state
    pub ready: bool,
    /// Restart count
    #[serde(rename = "restartCount")]
    pub restart_count: u32,
}

impl PodItem {
    /// Convert to simplified Pod struct
    pub fn to_pod(&self) -> Pod {
        let ready = self.ready_string();
        let restarts = self.total_restarts();
        let age = self.age_string();
        let node = self.spec.as_ref().and_then(|s| s.node_name.clone());

        Pod {
            name: self.metadata.name.clone(),
            namespace: self.metadata.namespace.clone(),
            status: self.status.phase.clone(),
            ready,
            restarts,
            age,
            node,
        }
    }

    /// Get ready string (e.g., "1/2")
    fn ready_string(&self) -> String {
        let total = self.status.container_statuses.len();
        let ready = self
            .status
            .container_statuses
            .iter()
            .filter(|c| c.ready)
            .count();
        format!("{}/{}", ready, total)
    }

    /// Get total restart count
    fn total_restarts(&self) -> u32 {
        self.status
            .container_statuses
            .iter()
            .map(|c| c.restart_count)
            .sum()
    }

    /// Get age string from creation timestamp
    fn age_string(&self) -> String {
        let Some(ts) = &self.metadata.creation_timestamp else {
            return "-".to_string();
        };

        let Ok(created) = chrono::DateTime::parse_from_rfc3339(ts) else {
            return "-".to_string();
        };

        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(created);

        if duration.num_days() > 0 {
            format!("{}d", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}m", duration.num_minutes())
        } else {
            format!("{}s", duration.num_seconds())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pod_debug() {
        let pod = Pod {
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            ready: "1/1".to_string(),
            restarts: 0,
            age: "1d".to_string(),
            node: Some("node-1".to_string()),
        };
        let debug = format!("{:?}", pod);
        assert!(debug.contains("test-pod"));
    }

    #[test]
    fn pod_clone() {
        let pod = Pod {
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            ready: "1/1".to_string(),
            restarts: 0,
            age: "1d".to_string(),
            node: None,
        };
        let cloned = pod.clone();
        assert_eq!(cloned.name, pod.name);
    }

    #[test]
    fn kubectl_config_default() {
        let config = KubectlConfig::default();
        assert!(config.context.is_none());
        assert!(config.namespace.is_none());
    }

    #[test]
    fn output_format_default() {
        let format = OutputFormat::default();
        assert_eq!(format, OutputFormat::Table);
    }

    #[test]
    fn parse_pod_list() {
        let json = r#"{
            "items": [
                {
                    "metadata": {
                        "name": "my-pod",
                        "namespace": "default",
                        "creationTimestamp": "2026-01-01T00:00:00Z"
                    },
                    "status": {
                        "phase": "Running",
                        "containerStatuses": [
                            {"name": "main", "ready": true, "restartCount": 2}
                        ]
                    }
                }
            ]
        }"#;

        let pod_list: PodList = serde_json::from_str(json).unwrap();
        assert_eq!(pod_list.items.len(), 1);

        let pod = pod_list.items[0].to_pod();
        assert_eq!(pod.name, "my-pod");
        assert_eq!(pod.namespace, "default");
        assert_eq!(pod.status, "Running");
        assert_eq!(pod.ready, "1/1");
        assert_eq!(pod.restarts, 2);
    }

    #[test]
    fn parse_pod_list_multiple_containers() {
        let json = r#"{
            "items": [
                {
                    "metadata": {
                        "name": "multi-container",
                        "namespace": "prod"
                    },
                    "status": {
                        "phase": "Running",
                        "containerStatuses": [
                            {"name": "app", "ready": true, "restartCount": 1},
                            {"name": "sidecar", "ready": false, "restartCount": 3}
                        ]
                    }
                }
            ]
        }"#;

        let pod_list: PodList = serde_json::from_str(json).unwrap();
        let pod = pod_list.items[0].to_pod();
        assert_eq!(pod.ready, "1/2");
        assert_eq!(pod.restarts, 4);
    }

    #[test]
    fn parse_pod_list_with_node() {
        let json = r#"{
            "items": [
                {
                    "metadata": {
                        "name": "my-pod",
                        "namespace": "default"
                    },
                    "spec": {
                        "nodeName": "node-abc123"
                    },
                    "status": {
                        "phase": "Running",
                        "containerStatuses": []
                    }
                }
            ]
        }"#;

        let pod_list: PodList = serde_json::from_str(json).unwrap();
        let pod = pod_list.items[0].to_pod();
        assert_eq!(pod.node, Some("node-abc123".to_string()));
    }

    #[test]
    fn parse_pod_list_no_node() {
        let json = r#"{
            "items": [
                {
                    "metadata": {
                        "name": "pending-pod",
                        "namespace": "default"
                    },
                    "status": {
                        "phase": "Pending",
                        "containerStatuses": []
                    }
                }
            ]
        }"#;

        let pod_list: PodList = serde_json::from_str(json).unwrap();
        let pod = pod_list.items[0].to_pod();
        assert!(pod.node.is_none());
    }

    #[test]
    fn age_string_no_timestamp() {
        let item = PodItem {
            metadata: PodMetadata {
                name: "test".to_string(),
                namespace: "default".to_string(),
                creation_timestamp: None,
            },
            spec: None,
            status: PodStatus {
                phase: "Running".to_string(),
                container_statuses: vec![],
            },
        };
        let pod = item.to_pod();
        assert_eq!(pod.age, "-");
    }

    #[test]
    fn age_string_invalid_timestamp() {
        let item = PodItem {
            metadata: PodMetadata {
                name: "test".to_string(),
                namespace: "default".to_string(),
                creation_timestamp: Some("not-a-date".to_string()),
            },
            spec: None,
            status: PodStatus {
                phase: "Running".to_string(),
                container_statuses: vec![],
            },
        };
        let pod = item.to_pod();
        assert_eq!(pod.age, "-");
    }

    #[test]
    fn pod_serialize() {
        let pod = Pod {
            name: "test".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            ready: "1/1".to_string(),
            restarts: 0,
            age: "1h".to_string(),
            node: None,
        };
        let json = serde_json::to_string(&pod).unwrap();
        assert!(json.contains("test"));
    }
}
