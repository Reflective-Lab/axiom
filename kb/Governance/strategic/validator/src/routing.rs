//! Alert Routing Logic
//!
//! Implements routing rules with quiet hours, rate limiting, and delivery decisions.

use crate::routing_config::DeliveryPolicy;
use governor::{Quota, RateLimiter};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Mutex;

/// Type alias for rate limiter with recipient keys
pub type KeyedRateLimiter = HashMap<String, RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>;

/// Alert throttler with per-recipient rate limiting
pub struct AlertThrottler {
    limiters: Mutex<HashMap<String, RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>>,
    max_per_hour: u32,
}

impl AlertThrottler {
    /// Create new throttler with configured rate limit
    pub fn new(max_per_hour: u32) -> Self {
        Self {
            limiters: Mutex::new(HashMap::new()),
            max_per_hour,
        }
    }

    /// Check if recipient can receive another alert (respects rate limit)
    ///
    /// Returns `true` if the alert should be delivered, `false` if rate-limited.
    pub fn check_recipient(&self, recipient: &str) -> bool {
        let mut limiters = self.limiters.lock().unwrap();

        let limiter = limiters.entry(recipient.to_string()).or_insert_with(|| {
            let quota = Quota::per_hour(NonZeroU32::new(self.max_per_hour).unwrap());
            RateLimiter::direct(quota)
        });

        limiter.check().is_ok()
    }
}

/// Delivery decision result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryDecision {
    /// Deliver the alert
    Deliver,
    /// Suppressed due to quiet hours
    SuppressedQuietHours,
    /// Suppressed due to rate limiting
    SuppressedRateLimited,
    /// No routing rule matches
    NoRoute,
}

/// Check if current hour falls within quiet hours
///
/// Handles wraparound (e.g., 22:00-06:00 spans midnight).
///
/// # Arguments
///
/// * `current_hour` - Current hour (0-23)
/// * `start_hour` - Quiet hours start (0-23)
/// * `end_hour` - Quiet hours end (0-23)
///
/// # Returns
///
/// `true` if current hour is within quiet hours
pub fn is_quiet_hours(current_hour: u8, start_hour: u8, end_hour: u8) -> bool {
    if start_hour < end_hour {
        // Normal range (e.g., 8:00-17:00)
        current_hour >= start_hour && current_hour < end_hour
    } else {
        // Wraparound range (e.g., 22:00-06:00)
        current_hour >= start_hour || current_hour < end_hour
    }
}

/// Find recipients for an alert based on routing rules
///
/// Matches rules by `gate_id`, severity, and `drift_code`.
/// Returns recipients from the highest-priority matching rule.
///
/// # Arguments
///
/// * `policy` - Delivery policy configuration
/// * `gate_id` - Gate identifier
/// * `severity` - Optional severity level
/// * `drift_code` - Optional drift code
///
/// # Returns
///
/// Vector of recipient email addresses (empty if no match)
pub fn find_recipients(
    policy: &DeliveryPolicy,
    gate_id: &str,
    severity: Option<&str>,
    drift_code: Option<&str>,
) -> Vec<String> {
    // Priority order: exact > partial > fallback
    let mut exact_matches = Vec::new();
    let mut partial_matches = Vec::new();
    let mut fallback_matches = Vec::new();

    for rule in &policy.routing_rules {
        // Check gate_id match
        if rule.gate_id != gate_id {
            continue;
        }

        // Categorize by specificity
        let severity_match = rule.severity.as_deref() == severity || rule.severity.is_none();
        let drift_match = rule.drift_code.as_deref() == drift_code || rule.drift_code.is_none();

        if rule.severity.is_some() && rule.drift_code.is_some() {
            // Exact match: both severity and drift_code specified
            if severity_match && drift_match {
                exact_matches.push(rule);
            }
        } else if rule.severity.is_some() || rule.drift_code.is_some() {
            // Partial match: one of severity or drift_code specified
            if severity_match && drift_match {
                partial_matches.push(rule);
            }
        } else {
            // Fallback: neither severity nor drift_code specified
            fallback_matches.push(rule);
        }
    }

    // Return first matching rule in priority order
    if let Some(rule) = exact_matches.first() {
        rule.recipients.clone()
    } else if let Some(rule) = partial_matches.first() {
        rule.recipients.clone()
    } else if let Some(rule) = fallback_matches.first() {
        rule.recipients.clone()
    } else {
        Vec::new()
    }
}

/// Find owner for a gate
///
/// Looks up the owner field from routing rules matching the `gate_id`.
///
/// # Arguments
///
/// * `policy` - Delivery policy configuration
/// * `gate_id` - Gate identifier
///
/// # Returns
///
/// Owner string if found, None otherwise
pub fn find_owner(policy: &DeliveryPolicy, gate_id: &str) -> Option<String> {
    policy
        .routing_rules
        .iter()
        .find(|rule| rule.gate_id == gate_id)
        .and_then(|rule| rule.owner.clone())
}

/// Find Slack channel for an alert based on routing rules
///
/// Matches rules by `gate_id`, severity, and `drift_code`.
/// Returns `slack_channel` from the highest-priority matching rule.
/// Uses same priority logic as `find_recipients`: exact > partial > fallback.
///
/// # Arguments
///
/// * `policy` - Delivery policy configuration
/// * `gate_id` - Gate identifier
/// * `severity` - Optional severity level
/// * `drift_code` - Optional drift code
///
/// # Returns
///
/// Slack channel name if found, None otherwise
pub fn find_slack_channel(
    policy: &DeliveryPolicy,
    gate_id: &str,
    severity: Option<&str>,
    drift_code: Option<&str>,
) -> Option<String> {
    // Priority order: exact > partial > fallback
    let mut exact_matches = Vec::new();
    let mut partial_matches = Vec::new();
    let mut fallback_matches = Vec::new();

    for rule in &policy.routing_rules {
        // Check gate_id match
        if rule.gate_id != gate_id {
            continue;
        }

        // Skip rules without slack_channel configured
        if rule.slack_channel.is_none() {
            continue;
        }

        // Categorize by specificity
        let severity_match = rule.severity.as_deref() == severity || rule.severity.is_none();
        let drift_match = rule.drift_code.as_deref() == drift_code || rule.drift_code.is_none();

        if rule.severity.is_some() && rule.drift_code.is_some() {
            // Exact match: both severity and drift_code specified
            if severity_match && drift_match {
                exact_matches.push(rule);
            }
        } else if rule.severity.is_some() || rule.drift_code.is_some() {
            // Partial match: one of severity or drift_code specified
            if severity_match && drift_match {
                partial_matches.push(rule);
            }
        } else {
            // Fallback: neither severity nor drift_code specified
            fallback_matches.push(rule);
        }
    }

    // Return slack_channel from first matching rule in priority order
    if let Some(rule) = exact_matches.first() {
        rule.slack_channel.clone()
    } else if let Some(rule) = partial_matches.first() {
        rule.slack_channel.clone()
    } else if let Some(rule) = fallback_matches.first() {
        rule.slack_channel.clone()
    } else {
        None
    }
}

/// Get people who must acknowledge a specific gate
///
/// Returns the list of people who must acknowledge alerts for the given gate.
///
/// # Arguments
///
/// * `policy` - Delivery policy configuration
/// * `gate_id` - Gate identifier to look up
///
/// # Returns
///
/// Vector of email addresses (empty if gate not found or no acknowledgments configured)
pub fn get_gate_acknowledgments(policy: &DeliveryPolicy, gate_id: &str) -> Vec<String> {
    policy
        .gate_acknowledgments
        .as_ref()
        .and_then(|acks| acks.get(gate_id))
        .cloned()
        .unwrap_or_default()
}

/// Determine if alert should be delivered
///
/// Combines routing, quiet hours, and rate limiting checks.
///
/// # Arguments
///
/// * `policy` - Delivery policy configuration
/// * `gate_id` - Gate identifier
/// * `severity` - Optional severity level
/// * `drift_code` - Optional drift code
/// * `current_hour` - Current hour (0-23) for quiet hours check
/// * `throttler` - Alert throttler for rate limiting
/// * `recipient` - Recipient email address
///
/// # Returns
///
/// `DeliveryDecision` indicating whether to deliver or suppress
pub fn should_deliver(
    policy: &DeliveryPolicy,
    gate_id: &str,
    severity: Option<&str>,
    drift_code: Option<&str>,
    current_hour: u8,
    throttler: &AlertThrottler,
    recipient: &str,
) -> DeliveryDecision {
    // Find matching recipients
    let recipients = find_recipients(policy, gate_id, severity, drift_code);
    if !recipients.contains(&recipient.to_string()) {
        return DeliveryDecision::NoRoute;
    }

    // Check quiet hours (skip for critical severity)
    if let Some(ref qh) = policy.quiet_hours {
        if severity != Some("critical") && is_quiet_hours(current_hour, qh.start_hour, qh.end_hour) {
            return DeliveryDecision::SuppressedQuietHours;
        }
    }

    // Check rate limiting
    if !throttler.check_recipient(recipient) {
        return DeliveryDecision::SuppressedRateLimited;
    }

    DeliveryDecision::Deliver
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing_config::{QuietHours, RateLimiting, RoutingRule};

    fn create_test_policy() -> DeliveryPolicy {
        DeliveryPolicy {
            version: "1.0".to_string(),
            quiet_hours: Some(QuietHours {
                start_hour: 22,
                end_hour: 6,
                timezone: "UTC".to_string(),
            }),
            rate_limiting: RateLimiting {
                max_alerts_per_hour: 10,
                window_minutes: 60,
            },
            routing_rules: vec![
                // Exact match: gate + severity + drift
                RoutingRule {
                    gate_id: "G_PROMOTE_01".to_string(),
                    severity: Some("critical".to_string()),
                    drift_code: Some("D_AUTH".to_string()),
                    recipients: vec!["exact@example.com".to_string()],
                    priority: Some("high".to_string()),
                    owner: Some("security-team".to_string()),
                    slack_channel: None,
                },
                // Partial match: gate + severity only
                RoutingRule {
                    gate_id: "G_PROMOTE_01".to_string(),
                    severity: Some("high".to_string()),
                    drift_code: None,
                    recipients: vec!["partial@example.com".to_string()],
                    priority: None,
                    owner: Some("security-team".to_string()),
                    slack_channel: None,
                },
                // Fallback: gate only
                RoutingRule {
                    gate_id: "G_PROMOTE_01".to_string(),
                    severity: None,
                    drift_code: None,
                    recipients: vec!["fallback@example.com".to_string()],
                    priority: None,
                    owner: Some("security-team".to_string()),
                    slack_channel: None,
                },
                // Different gate
                RoutingRule {
                    gate_id: "G_PROMOTE_02".to_string(),
                    severity: None,
                    drift_code: None,
                    recipients: vec!["gate2@example.com".to_string()],
                    priority: None,
                    owner: Some("platform-team".to_string()),
                    slack_channel: None,
                },
            ],
            gate_acknowledgments: Some(HashMap::from([
                ("G_PROMOTE_03".to_string(), vec!["security-team@example.com".to_string(), "compliance@example.com".to_string()]),
            ])),
        }
    }

    fn create_test_policy_with_slack() -> DeliveryPolicy {
        DeliveryPolicy {
            version: "1.0".to_string(),
            quiet_hours: None,
            rate_limiting: RateLimiting {
                max_alerts_per_hour: 10,
                window_minutes: 60,
            },
            routing_rules: vec![
                // Exact match with Slack channel
                RoutingRule {
                    gate_id: "G_PROMOTE_01".to_string(),
                    severity: Some("critical".to_string()),
                    drift_code: Some("D_AUTH".to_string()),
                    recipients: vec!["exact@example.com".to_string()],
                    priority: Some("high".to_string()),
                    owner: Some("security-team".to_string()),
                    slack_channel: Some("#security-critical".to_string()),
                },
                // Partial match with Slack channel
                RoutingRule {
                    gate_id: "G_PROMOTE_01".to_string(),
                    severity: Some("high".to_string()),
                    drift_code: None,
                    recipients: vec!["partial@example.com".to_string()],
                    priority: None,
                    owner: Some("security-team".to_string()),
                    slack_channel: Some("#security-high".to_string()),
                },
                // Fallback with Slack channel
                RoutingRule {
                    gate_id: "G_PROMOTE_01".to_string(),
                    severity: None,
                    drift_code: None,
                    recipients: vec!["fallback@example.com".to_string()],
                    priority: None,
                    owner: Some("security-team".to_string()),
                    slack_channel: Some("#security-default".to_string()),
                },
                // Gate without Slack channel
                RoutingRule {
                    gate_id: "G_PROMOTE_02".to_string(),
                    severity: None,
                    drift_code: None,
                    recipients: vec!["gate2@example.com".to_string()],
                    priority: None,
                    owner: Some("platform-team".to_string()),
                    slack_channel: None,
                },
            ],
            gate_acknowledgments: None,
        }
    }

    #[test]
    fn test_is_quiet_hours_normal_range() {
        // 8:00-17:00
        assert!(!is_quiet_hours(7, 8, 17), "Before quiet hours");
        assert!(is_quiet_hours(8, 8, 17), "Start of quiet hours");
        assert!(is_quiet_hours(12, 8, 17), "Middle of quiet hours");
        assert!(is_quiet_hours(16, 8, 17), "End of quiet hours - 1");
        assert!(!is_quiet_hours(17, 8, 17), "After quiet hours");
    }

    #[test]
    fn test_is_quiet_hours_wraparound() {
        // 22:00-06:00 (overnight)
        assert!(is_quiet_hours(22, 22, 6), "Start of quiet hours");
        assert!(is_quiet_hours(23, 22, 6), "Late night");
        assert!(is_quiet_hours(0, 22, 6), "Midnight");
        assert!(is_quiet_hours(3, 22, 6), "Early morning");
        assert!(is_quiet_hours(5, 22, 6), "End of quiet hours - 1");
        assert!(!is_quiet_hours(6, 22, 6), "After quiet hours");
        assert!(!is_quiet_hours(12, 22, 6), "Midday");
        assert!(!is_quiet_hours(21, 22, 6), "Before quiet hours");
    }

    #[test]
    fn test_find_recipients_exact_match() {
        let policy = create_test_policy();
        let recipients = find_recipients(
            &policy,
            "G_PROMOTE_01",
            Some("critical"),
            Some("D_AUTH"),
        );
        assert_eq!(recipients, vec!["exact@example.com"]);
    }

    #[test]
    fn test_find_recipients_partial_match() {
        let policy = create_test_policy();
        let recipients = find_recipients(
            &policy,
            "G_PROMOTE_01",
            Some("high"),
            None,
        );
        assert_eq!(recipients, vec!["partial@example.com"]);
    }

    #[test]
    fn test_find_recipients_fallback_match() {
        let policy = create_test_policy();
        let recipients = find_recipients(
            &policy,
            "G_PROMOTE_01",
            None,
            None,
        );
        assert_eq!(recipients, vec!["fallback@example.com"]);
    }

    #[test]
    fn test_find_recipients_no_match() {
        let policy = create_test_policy();
        let recipients = find_recipients(
            &policy,
            "G_NONEXISTENT",
            None,
            None,
        );
        assert!(recipients.is_empty());
    }

    #[test]
    fn test_find_owner() {
        let policy = create_test_policy();
        assert_eq!(
            find_owner(&policy, "G_PROMOTE_01"),
            Some("security-team".to_string())
        );
        assert_eq!(
            find_owner(&policy, "G_PROMOTE_02"),
            Some("platform-team".to_string())
        );
        assert_eq!(find_owner(&policy, "G_NONEXISTENT"), None);
    }

    #[test]
    fn test_get_gate_acknowledgments() {
        let policy = create_test_policy();
        let acks = get_gate_acknowledgments(&policy, "G_PROMOTE_03");
        assert_eq!(acks, vec!["security-team@example.com", "compliance@example.com"]);
    }

    #[test]
    fn test_get_gate_acknowledgments_empty() {
        let mut policy = create_test_policy();
        policy.gate_acknowledgments = None;
        let acks = get_gate_acknowledgments(&policy, "G_PROMOTE_03");
        assert!(acks.is_empty());
    }

    #[test]
    fn test_get_gate_acknowledgments_gate_not_found() {
        let policy = create_test_policy();
        let acks = get_gate_acknowledgments(&policy, "G_NONEXISTENT");
        assert!(acks.is_empty());
    }

    #[test]
    fn test_throttler_allows_initial_requests() {
        let throttler = AlertThrottler::new(10);
        assert!(throttler.check_recipient("test@example.com"));
    }

    #[test]
    fn test_throttler_enforces_limit() {
        let throttler = AlertThrottler::new(2);

        // First two should succeed
        assert!(throttler.check_recipient("test@example.com"));
        assert!(throttler.check_recipient("test@example.com"));

        // Third should fail (rate limited)
        assert!(!throttler.check_recipient("test@example.com"));
    }

    #[test]
    fn test_throttler_per_recipient() {
        let throttler = AlertThrottler::new(1);

        // Each recipient gets their own limit
        assert!(throttler.check_recipient("user1@example.com"));
        assert!(throttler.check_recipient("user2@example.com"));

        // Second attempt for user1 should fail
        assert!(!throttler.check_recipient("user1@example.com"));

        // But user2's second should also fail
        assert!(!throttler.check_recipient("user2@example.com"));
    }

    #[test]
    fn test_should_deliver_success() {
        let policy = create_test_policy();
        let throttler = AlertThrottler::new(10);

        let decision = should_deliver(
            &policy,
            "G_PROMOTE_01",
            Some("critical"),
            Some("D_AUTH"),
            12, // noon
            &throttler,
            "exact@example.com",
        );

        assert_eq!(decision, DeliveryDecision::Deliver);
    }

    #[test]
    fn test_should_deliver_quiet_hours() {
        let policy = create_test_policy();
        let throttler = AlertThrottler::new(10);

        let decision = should_deliver(
            &policy,
            "G_PROMOTE_01",
            Some("high"), // non-critical
            None,
            23, // 11 PM - quiet hours
            &throttler,
            "partial@example.com",
        );

        assert_eq!(decision, DeliveryDecision::SuppressedQuietHours);
    }

    #[test]
    fn test_should_deliver_quiet_hours_critical_override() {
        let policy = create_test_policy();
        let throttler = AlertThrottler::new(10);

        let decision = should_deliver(
            &policy,
            "G_PROMOTE_01",
            Some("critical"), // critical overrides quiet hours
            Some("D_AUTH"),
            23, // 11 PM - quiet hours
            &throttler,
            "exact@example.com",
        );

        assert_eq!(decision, DeliveryDecision::Deliver);
    }

    #[test]
    fn test_should_deliver_rate_limited() {
        let policy = create_test_policy();
        let throttler = AlertThrottler::new(1);

        // First delivery succeeds
        let decision1 = should_deliver(
            &policy,
            "G_PROMOTE_01",
            None,
            None,
            12,
            &throttler,
            "fallback@example.com",
        );
        assert_eq!(decision1, DeliveryDecision::Deliver);

        // Second delivery rate-limited
        let decision2 = should_deliver(
            &policy,
            "G_PROMOTE_01",
            None,
            None,
            12,
            &throttler,
            "fallback@example.com",
        );
        assert_eq!(decision2, DeliveryDecision::SuppressedRateLimited);
    }

    #[test]
    fn test_should_deliver_no_route() {
        let policy = create_test_policy();
        let throttler = AlertThrottler::new(10);

        let decision = should_deliver(
            &policy,
            "G_PROMOTE_01",
            None,
            None,
            12,
            &throttler,
            "nonexistent@example.com", // not in recipients list
        );

        assert_eq!(decision, DeliveryDecision::NoRoute);
    }

    #[test]
    fn test_find_slack_channel_exact_match() {
        let policy = create_test_policy_with_slack();
        let channel = find_slack_channel(
            &policy,
            "G_PROMOTE_01",
            Some("critical"),
            Some("D_AUTH"),
        );
        assert_eq!(channel, Some("#security-critical".to_string()));
    }

    #[test]
    fn test_find_slack_channel_partial_match() {
        let policy = create_test_policy_with_slack();
        let channel = find_slack_channel(
            &policy,
            "G_PROMOTE_01",
            Some("high"),
            None,
        );
        assert_eq!(channel, Some("#security-high".to_string()));
    }

    #[test]
    fn test_find_slack_channel_fallback() {
        let policy = create_test_policy_with_slack();
        let channel = find_slack_channel(
            &policy,
            "G_PROMOTE_01",
            None,
            None,
        );
        assert_eq!(channel, Some("#security-default".to_string()));
    }

    #[test]
    fn test_find_slack_channel_none_configured() {
        let policy = create_test_policy_with_slack();
        // G_PROMOTE_02 has no slack_channel configured
        let channel = find_slack_channel(
            &policy,
            "G_PROMOTE_02",
            None,
            None,
        );
        assert_eq!(channel, None);
    }

    #[test]
    fn test_find_slack_channel_no_match() {
        let policy = create_test_policy_with_slack();
        let channel = find_slack_channel(
            &policy,
            "G_NONEXISTENT",
            None,
            None,
        );
        assert_eq!(channel, None);
    }

    #[test]
    fn test_find_slack_channel_original_policy_no_slack() {
        // Test with original policy that has no Slack channels configured
        let policy = create_test_policy();
        let channel = find_slack_channel(
            &policy,
            "G_PROMOTE_01",
            Some("critical"),
            Some("D_AUTH"),
        );
        assert_eq!(channel, None);
    }
}
