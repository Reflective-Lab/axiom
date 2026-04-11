//! Routing Configuration
//!
//! Typed structures for parsing delivery-policy.toml and routing alerts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Errors that can occur during config loading or validation
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Top-level delivery policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryPolicy {
    pub version: String,
    pub quiet_hours: Option<QuietHours>,
    pub rate_limiting: RateLimiting,
    pub routing_rules: Vec<RoutingRule>,
    pub gate_acknowledgments: Option<HashMap<String, Vec<String>>>,
}

/// Quiet hours configuration - suppress non-critical alerts during these hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub start_hour: u8,
    pub end_hour: u8,
    pub timezone: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiting {
    #[serde(default = "default_max_alerts_per_hour")]
    pub max_alerts_per_hour: u32,

    #[serde(default = "default_window_minutes")]
    pub window_minutes: u32,
}

fn default_max_alerts_per_hour() -> u32 {
    10
}

fn default_window_minutes() -> u32 {
    60
}

/// Individual routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub gate_id: String,
    pub severity: Option<String>,
    pub drift_code: Option<String>,
    pub recipients: Vec<String>,
    pub priority: Option<String>,
    pub owner: Option<String>,
    /// Optional Slack channel for Slack delivery (e.g., "#security-alerts")
    /// If specified, alerts matching this rule will be sent to this Slack channel
    /// via the configured Slack webhook.
    pub slack_channel: Option<String>,
}

/// Load delivery policy from TOML file
pub fn load_delivery_policy<P: AsRef<Path>>(path: P) -> Result<DeliveryPolicy, ConfigError> {
    let contents = fs::read_to_string(path)?;
    let policy: DeliveryPolicy = toml::from_str(&contents)?;
    validate_policy(&policy)?;
    Ok(policy)
}

/// Validate delivery policy
pub fn validate_policy(policy: &DeliveryPolicy) -> Result<(), ConfigError> {
    // Validate quiet hours if present
    if let Some(ref qh) = policy.quiet_hours {
        if qh.start_hour >= 24 {
            return Err(ConfigError::ValidationError(
                format!("start_hour must be < 24, got {}", qh.start_hour)
            ));
        }
        if qh.end_hour >= 24 {
            return Err(ConfigError::ValidationError(
                format!("end_hour must be < 24, got {}", qh.end_hour)
            ));
        }
    }

    // Validate rate limiting
    if policy.rate_limiting.max_alerts_per_hour == 0 {
        return Err(ConfigError::ValidationError(
            "max_alerts_per_hour must be > 0".to_string()
        ));
    }
    if policy.rate_limiting.window_minutes == 0 {
        return Err(ConfigError::ValidationError(
            "window_minutes must be > 0".to_string()
        ));
    }

    // Validate routing rules
    if policy.routing_rules.is_empty() {
        return Err(ConfigError::ValidationError(
            "At least one routing rule is required".to_string()
        ));
    }

    for rule in &policy.routing_rules {
        if rule.recipients.is_empty() {
            return Err(ConfigError::ValidationError(
                format!("Routing rule for gate_id '{}' has no recipients", rule.gate_id)
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_minimal_config() {
        let toml_content = r#"
version = "1.0"

[rate_limiting]

[[routing_rules]]
gate_id = "G_PROMOTE_01"
recipients = ["team@example.com"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let policy = load_delivery_policy(temp_file.path()).unwrap();
        assert_eq!(policy.version, "1.0");
        assert_eq!(policy.rate_limiting.max_alerts_per_hour, 10); // default
        assert_eq!(policy.rate_limiting.window_minutes, 60); // default
        assert_eq!(policy.routing_rules.len(), 1);
    }

    #[test]
    fn test_parse_full_config() {
        let toml_content = r#"
version = "1.0"

[quiet_hours]
start_hour = 22
end_hour = 6
timezone = "America/New_York"

[rate_limiting]
max_alerts_per_hour = 5
window_minutes = 30

[[routing_rules]]
gate_id = "G_PROMOTE_01"
severity = "critical"
drift_code = "D_AUTH"
recipients = ["team@example.com", "oncall@example.com"]
priority = "high"
owner = "security-team"

[[routing_rules]]
gate_id = "G_PROMOTE_02"
recipients = ["team@example.com"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let policy = load_delivery_policy(temp_file.path()).unwrap();
        assert_eq!(policy.version, "1.0");

        let qh = policy.quiet_hours.unwrap();
        assert_eq!(qh.start_hour, 22);
        assert_eq!(qh.end_hour, 6);
        assert_eq!(qh.timezone, "America/New_York");

        assert_eq!(policy.rate_limiting.max_alerts_per_hour, 5);
        assert_eq!(policy.rate_limiting.window_minutes, 30);

        assert_eq!(policy.routing_rules.len(), 2);
        assert_eq!(policy.routing_rules[0].gate_id, "G_PROMOTE_01");
        assert_eq!(policy.routing_rules[0].recipients.len(), 2);
    }

    #[test]
    fn test_validation_quiet_hours() {
        let policy = DeliveryPolicy {
            version: "1.0".to_string(),
            quiet_hours: Some(QuietHours {
                start_hour: 25, // invalid
                end_hour: 6,
                timezone: "UTC".to_string(),
            }),
            rate_limiting: RateLimiting {
                max_alerts_per_hour: 10,
                window_minutes: 60,
            },
            routing_rules: vec![RoutingRule {
                gate_id: "G_TEST".to_string(),
                severity: None,
                drift_code: None,
                recipients: vec!["test@example.com".to_string()],
                priority: None,
                owner: None,
                slack_channel: None,
            }],
            gate_acknowledgments: None,
        };

        let result = validate_policy(&policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("start_hour"));
    }

    #[test]
    fn test_validation_rate_limiting() {
        let policy = DeliveryPolicy {
            version: "1.0".to_string(),
            quiet_hours: None,
            rate_limiting: RateLimiting {
                max_alerts_per_hour: 0, // invalid
                window_minutes: 60,
            },
            routing_rules: vec![RoutingRule {
                gate_id: "G_TEST".to_string(),
                severity: None,
                drift_code: None,
                recipients: vec!["test@example.com".to_string()],
                priority: None,
                owner: None,
                slack_channel: None,
            }],
            gate_acknowledgments: None,
        };

        let result = validate_policy(&policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_alerts_per_hour"));
    }

    #[test]
    fn test_validation_empty_rules() {
        let policy = DeliveryPolicy {
            version: "1.0".to_string(),
            quiet_hours: None,
            rate_limiting: RateLimiting {
                max_alerts_per_hour: 10,
                window_minutes: 60,
            },
            routing_rules: vec![], // empty
            gate_acknowledgments: None,
        };

        let result = validate_policy(&policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("At least one routing rule"));
    }

    #[test]
    fn test_routing_rule_with_slack_channel() {
        let toml_content = r##"
version = "1.0"

[rate_limiting]

[[routing_rules]]
gate_id = "G_PROMOTE_01"
recipients = ["team@example.com"]
slack_channel = "#security-alerts"
"##;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let policy = load_delivery_policy(temp_file.path()).unwrap();
        assert_eq!(policy.routing_rules.len(), 1);
        assert_eq!(
            policy.routing_rules[0].slack_channel,
            Some("#security-alerts".to_string())
        );
    }

    #[test]
    fn test_routing_rule_without_slack_channel() {
        // Test backward compatibility - rules without slack_channel should still parse
        let toml_content = r#"
version = "1.0"

[rate_limiting]

[[routing_rules]]
gate_id = "G_PROMOTE_01"
recipients = ["team@example.com"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let policy = load_delivery_policy(temp_file.path()).unwrap();
        assert_eq!(policy.routing_rules.len(), 1);
        assert!(policy.routing_rules[0].slack_channel.is_none());
    }
}
