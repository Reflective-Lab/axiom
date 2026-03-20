//! Policy types and data structures.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Policy evaluation errors.
#[derive(Debug, Error)]
pub enum PolicyError {
    /// No matching rule found.
    #[error("no matching rule for method {method} and principal {principal}")]
    NoMatch { method: String, principal: String },

    /// Access denied by policy.
    #[error("access denied: {reason}")]
    Denied { reason: String },

    /// Invalid policy configuration.
    #[error("invalid policy: {0}")]
    Invalid(String),
}

/// Effect of a policy rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    /// Allow the action.
    Allow,
    /// Deny the action.
    #[default]
    Deny,
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Allow => write!(f, "allow"),
            Effect::Deny => write!(f, "deny"),
        }
    }
}

/// Principal specification for matching.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Principal {
    /// Roles that match this rule.
    #[serde(default)]
    pub roles: Vec<String>,

    /// Services that match this rule.
    #[serde(default)]
    pub services: Vec<String>,

    /// Specific user IDs that match.
    #[serde(default)]
    pub users: Vec<String>,
}

impl Principal {
    /// Check if this principal matches any criteria.
    pub fn is_empty(&self) -> bool {
        self.roles.is_empty() && self.services.is_empty() && self.users.is_empty()
    }

    /// Check if a user with given roles/service matches this principal.
    pub fn matches(&self, user_id: Option<&str>, roles: &[String], service_id: &str) -> bool {
        // Empty principal matches nothing (must have at least one criterion)
        if self.is_empty() {
            return false;
        }

        // Check user match
        if let Some(uid) = user_id {
            if self.users.iter().any(|u| u == uid || u == "*") {
                return true;
            }
        }

        // Check role match
        if self.roles.iter().any(|r| r == "*" || roles.contains(r)) {
            return true;
        }

        // Check service match
        if self.services.iter().any(|s| s == "*" || s == service_id) {
            return true;
        }

        false
    }
}

/// A policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Rule name for logging/debugging.
    pub name: String,

    /// Effect when this rule matches.
    pub effect: Effect,

    /// Principal specification.
    #[serde(default)]
    pub principals: Principal,

    /// Methods this rule applies to.
    /// Use "*" to match all methods.
    #[serde(default)]
    pub methods: Vec<String>,

    /// Optional condition expression (future use).
    #[serde(default)]
    pub condition: Option<String>,

    /// Rule priority (higher = evaluated first).
    #[serde(default)]
    pub priority: i32,
}

impl Rule {
    /// Check if this rule matches the given request.
    pub fn matches(
        &self,
        method: &str,
        user_id: Option<&str>,
        roles: &[String],
        service_id: &str,
    ) -> bool {
        // Check method match
        let method_matches =
            self.methods.is_empty() || self.methods.iter().any(|m| m == "*" || m == method);

        if !method_matches {
            return false;
        }

        // Check principal match
        self.principals.matches(user_id, roles, service_id)
    }
}

/// Complete policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Default effect when no rules match.
    #[serde(default)]
    pub default_effect: Effect,

    /// Policy rules (evaluated in priority order).
    #[serde(default)]
    pub rules: Vec<Rule>,

    /// Policy version for tracking changes.
    #[serde(default)]
    pub version: Option<String>,

    /// Policy description.
    #[serde(default)]
    pub description: Option<String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            default_effect: Effect::Deny,
            rules: Vec::new(),
            version: None,
            description: None,
        }
    }
}

impl Policy {
    /// Create an allow-all policy (for development).
    pub fn allow_all() -> Self {
        Self {
            default_effect: Effect::Allow,
            rules: Vec::new(),
            version: Some("dev".to_string()),
            description: Some("Development allow-all policy".to_string()),
        }
    }

    /// Create a deny-all policy.
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// Get rules sorted by priority (highest first).
    pub fn sorted_rules(&self) -> Vec<&Rule> {
        let mut rules: Vec<&Rule> = self.rules.iter().collect();
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_display() {
        assert_eq!(Effect::Allow.to_string(), "allow");
        assert_eq!(Effect::Deny.to_string(), "deny");
    }

    #[test]
    fn test_effect_default() {
        assert_eq!(Effect::default(), Effect::Deny);
    }

    #[test]
    fn test_principal_empty() {
        let p = Principal::default();
        assert!(p.is_empty());
    }

    #[test]
    fn test_principal_matches_role() {
        let p = Principal {
            roles: vec!["admin".to_string()],
            ..Default::default()
        };
        assert!(p.matches(None, &["admin".to_string()], "some-service"));
        assert!(!p.matches(None, &["user".to_string()], "some-service"));
    }

    #[test]
    fn test_principal_matches_service() {
        let p = Principal {
            services: vec!["api-gateway".to_string()],
            ..Default::default()
        };
        assert!(p.matches(None, &[], "api-gateway"));
        assert!(!p.matches(None, &[], "other-service"));
    }

    #[test]
    fn test_principal_matches_user() {
        let p = Principal {
            users: vec!["user-123".to_string()],
            ..Default::default()
        };
        assert!(p.matches(Some("user-123"), &[], "some-service"));
        assert!(!p.matches(Some("user-456"), &[], "some-service"));
        assert!(!p.matches(None, &[], "some-service"));
    }

    #[test]
    fn test_principal_matches_wildcard() {
        let p = Principal {
            roles: vec!["*".to_string()],
            ..Default::default()
        };
        assert!(p.matches(None, &["anything".to_string()], "any-service"));
    }

    #[test]
    fn test_rule_matches_method() {
        let rule = Rule {
            name: "test".to_string(),
            effect: Effect::Allow,
            principals: Principal {
                roles: vec!["admin".to_string()],
                ..Default::default()
            },
            methods: vec!["/service/Method".to_string()],
            condition: None,
            priority: 0,
        };

        assert!(rule.matches("/service/Method", None, &["admin".to_string()], "svc"));
        assert!(!rule.matches("/service/Other", None, &["admin".to_string()], "svc"));
    }

    #[test]
    fn test_rule_matches_wildcard_method() {
        let rule = Rule {
            name: "test".to_string(),
            effect: Effect::Allow,
            principals: Principal {
                services: vec!["*".to_string()],
                ..Default::default()
            },
            methods: vec!["*".to_string()],
            condition: None,
            priority: 0,
        };

        assert!(rule.matches("/any/Method", None, &[], "any-service"));
    }

    #[test]
    fn test_policy_sorted_rules() {
        let policy = Policy {
            rules: vec![
                Rule {
                    name: "low".to_string(),
                    effect: Effect::Deny,
                    principals: Principal::default(),
                    methods: vec![],
                    condition: None,
                    priority: 1,
                },
                Rule {
                    name: "high".to_string(),
                    effect: Effect::Allow,
                    principals: Principal::default(),
                    methods: vec![],
                    condition: None,
                    priority: 100,
                },
            ],
            ..Default::default()
        };

        let sorted = policy.sorted_rules();
        assert_eq!(sorted[0].name, "high");
        assert_eq!(sorted[1].name, "low");
    }

    #[test]
    fn test_policy_allow_all() {
        let policy = Policy::allow_all();
        assert_eq!(policy.default_effect, Effect::Allow);
    }

    #[test]
    fn test_policy_deny_all() {
        let policy = Policy::deny_all();
        assert_eq!(policy.default_effect, Effect::Deny);
    }

    // Negative tests
    #[test]
    fn test_empty_principal_matches_nothing() {
        let p = Principal::default();
        assert!(!p.matches(Some("user"), &["role".to_string()], "service"));
    }

    #[test]
    fn test_rule_no_principal_match() {
        let rule = Rule {
            name: "test".to_string(),
            effect: Effect::Allow,
            principals: Principal {
                roles: vec!["admin".to_string()],
                ..Default::default()
            },
            methods: vec!["*".to_string()],
            condition: None,
            priority: 0,
        };

        // Method matches but principal doesn't
        assert!(!rule.matches("/any/Method", None, &["user".to_string()], "svc"));
    }
}
