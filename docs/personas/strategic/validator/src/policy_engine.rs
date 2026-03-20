//! Cedar policy engine integration for converge-personas authorization
//!
//! This module provides a wrapper around the cedar-policy crate for evaluating
//! authorization requests against Cedar policies and schemas. It handles:
//! - Loading schemas and policy sets from files
//! - Parsing authorization requests with principal/action/resource/context
//! - Evaluating requests and returning structured results
//! - Validating policies against schemas

use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request, Schema, Validator,
    ValidationMode,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

/// Authority violation details for actionable error messages (AUTH-04)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityViolation {
    /// Principal who attempted the action
    pub principal_id: String,
    /// Team the principal belongs to ("Core" | "Extended")
    pub principal_team: String,
    /// Principal's authority tier
    pub principal_tier: String,
    /// Gate being accessed
    pub gate_id: String,
    /// Gate's risk classification
    pub gate_risk_class: String,
    /// Type of authority violation
    pub violation_type: ViolationType,
}

/// Types of authority violations (AUTH-02, AUTH-03)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationType {
    /// AUTH-03: Extended team cannot approve high-risk gates
    ExtendedOnHighRiskGate,
    /// AUTH-02: Extended team not in gate's elevated_approvers
    ExtendedNotElevated,
    /// General authority insufficient
    InsufficientAuthority,
}

impl AuthorityViolation {
    /// Build actionable error message from violation details
    pub fn to_message(&self) -> String {
        match self.violation_type {
            ViolationType::ExtendedOnHighRiskGate => {
                format!(
                    "Extended team member '{}' cannot approve high-risk gate '{}'. \
                     High-risk gates require Core team authority (Blocking-by-Policy tier). \
                     Contact: founder, legal-counsel, security-auditor, ethics-safety-officer.",
                    self.principal_id, self.gate_id
                )
            }
            ViolationType::ExtendedNotElevated => {
                format!(
                    "Extended team member '{}' cannot approve gate '{}' without elevation. \
                     This gate requires Core team approval or specific elevation configuration. \
                     Request elevation via GATES.md elevated_blocking_evals or contact Core team.",
                    self.principal_id, self.gate_id
                )
            }
            ViolationType::InsufficientAuthority => {
                format!(
                    "User '{}' ({} team, {} authority) lacks required authority for gate '{}' ({}). \
                     This approval requires Blocking-by-Policy authority tier.",
                    self.principal_id,
                    self.principal_team,
                    self.principal_tier,
                    self.gate_id,
                    self.gate_risk_class
                )
            }
        }
    }
}

/// Errors that can occur during policy engine operations
#[derive(Debug, Error)]
pub enum PolicyEngineError {
    /// Failed to load or parse Cedar schema
    #[error("Failed to load schema: {0}")]
    SchemaLoadError(String),

    /// Failed to parse Cedar policy files
    #[error("Failed to parse policies: {0}")]
    PolicyParseError(String),

    /// Failed to create authorization request
    #[error("Failed to create request: {0}")]
    RequestError(String),

    /// Failed to parse entity UID
    #[error("Failed to parse entity UID: {0}")]
    EntityUidError(String),

    /// Failed to create context
    #[error("Failed to create context: {0}")]
    ContextError(String),

    /// I/O error reading schema or policy files
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Cedar policy engine wrapper
///
/// Manages Cedar schema, policies, and authorization evaluation.
pub struct PolicyEngine {
    authorizer: Authorizer,
    policies: PolicySet,
    schema: Schema,
}

/// Authorization request containing principal, action, resource, and context
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    /// Principal entity UID (e.g., `ConvergePersonas::User::"security-auditor"`)
    pub principal: String,

    /// Action entity UID (e.g., `ConvergePersonas::Action::"EvaluateGate"`)
    pub action: String,

    /// Resource entity UID (e.g., `ConvergePersonas::Gate::"release-approval"`)
    pub resource: String,

    /// Context key-value pairs for additional request data
    pub context: Vec<(String, String)>,
}

/// Result of an authorization evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationResult {
    /// Request was explicitly allowed by permit policy
    Allow,

    /// Request was denied (either explicitly forbidden or no permit matched)
    Deny {
        /// Reasons for denial (policy IDs that forbade or errors)
        reasons: Vec<String>,
        /// Structured authority violation details for actionable messages (AUTH-04)
        missing_authority: Option<AuthorityViolation>,
    },

    /// Error occurred during evaluation
    Error {
        /// Error message describing what went wrong
        message: String,
    },
}

impl PolicyEngine {
    /// Create a new policy engine from schema and policy files
    ///
    /// # Arguments
    /// * `schema_path` - Path to Cedar schema file (.cedarschema)
    /// * `policy_paths` - Paths to Cedar policy files (.cedar)
    ///
    /// # Errors
    /// Returns `PolicyEngineError` if schema or policies cannot be loaded or parsed
    pub fn new(schema_path: &Path, policy_paths: &[PathBuf]) -> Result<Self, PolicyEngineError> {
        // Load schema
        let schema_src = std::fs::read_to_string(schema_path).map_err(|e| {
            PolicyEngineError::SchemaLoadError(format!("Failed to read {}: {e}", schema_path.display()))
        })?;

        let schema = Schema::from_str(&schema_src).map_err(|e| {
            PolicyEngineError::SchemaLoadError(format!("Failed to parse schema: {e}"))
        })?;

        // Load and combine all policy files
        let mut combined_policies = String::new();
        for policy_path in policy_paths {
            let policy_src = std::fs::read_to_string(policy_path).map_err(|e| {
                PolicyEngineError::PolicyParseError(format!(
                    "Failed to read {}: {e}",
                    policy_path.display()
                ))
            })?;
            combined_policies.push_str(&policy_src);
            combined_policies.push('\n');
        }

        // Parse combined policy set
        let policies = combined_policies.parse::<PolicySet>().map_err(|e| {
            PolicyEngineError::PolicyParseError(format!("Failed to parse policies: {e}"))
        })?;

        let authorizer = Authorizer::new();

        Ok(Self {
            authorizer,
            policies,
            schema,
        })
    }

    /// Evaluate an authorization request
    ///
    /// # Arguments
    /// * `request` - Authorization request with principal, action, resource, context
    /// * `entities` - Entity data for hierarchy/attribute lookups
    ///
    /// # Returns
    /// `AuthorizationResult` indicating Allow, Deny, or Error
    pub fn is_authorized(
        &self,
        request: &AuthorizationRequest,
        entities: &Entities,
    ) -> AuthorizationResult {
        // Parse entity UIDs
        let principal = match EntityUid::from_str(&request.principal) {
            Ok(uid) => uid,
            Err(e) => {
                return AuthorizationResult::Error {
                    message: format!("Invalid principal UID: {e}"),
                }
            }
        };

        let action = match EntityUid::from_str(&request.action) {
            Ok(uid) => uid,
            Err(e) => {
                return AuthorizationResult::Error {
                    message: format!("Invalid action UID: {e}"),
                }
            }
        };

        let resource = match EntityUid::from_str(&request.resource) {
            Ok(uid) => uid,
            Err(e) => {
                return AuthorizationResult::Error {
                    message: format!("Invalid resource UID: {e}"),
                }
            }
        };

        // Build context from string pairs
        let context_pairs: HashMap<String, cedar_policy::RestrictedExpression> = request
            .context
            .iter()
            .map(|(k, v)| {
                // Convert string values to Cedar restricted expressions
                let expr = cedar_policy::RestrictedExpression::from_str(&format!("\"{v}\""))
                    .unwrap_or_else(|_| {
                        // Fallback: try as unquoted literal
                        cedar_policy::RestrictedExpression::from_str(v)
                            .unwrap_or_else(|_| cedar_policy::RestrictedExpression::from_str("\"\"").unwrap())
                    });
                (k.clone(), expr)
            })
            .collect();

        let context = match Context::from_pairs(context_pairs) {
            Ok(ctx) => ctx,
            Err(e) => {
                return AuthorizationResult::Error {
                    message: format!("Invalid context: {e}"),
                }
            }
        };

        // Create Cedar request
        let cedar_request = match Request::new(principal, action, resource, context, None) {
            Ok(req) => req,
            Err(e) => {
                return AuthorizationResult::Error {
                    message: format!("Failed to create request: {e}"),
                }
            }
        };

        // Evaluate
        let response = self
            .authorizer
            .is_authorized(&cedar_request, &self.policies, entities);

        // Convert result
        match response.decision() {
            Decision::Allow => AuthorizationResult::Allow,
            Decision::Deny => {
                // Collect diagnostic errors
                let reasons: Vec<String> = response
                    .diagnostics()
                    .errors()
                    .map(|e| format!("{e}"))
                    .collect();

                let reasons = if reasons.is_empty() {
                    vec!["No permit policy matched".to_string()]
                } else {
                    reasons
                };

                // Extract policy IDs that contributed to deny decision
                let policy_ids: Vec<String> = response
                    .diagnostics()
                    .reason()
                    .map(|pid| pid.to_string())
                    .collect();

                // Analyze denial to build structured violation
                let missing_authority = analyze_denial(request, &policy_ids);

                AuthorizationResult::Deny { reasons, missing_authority }
            }
        }
    }

    /// Validate policies against the schema
    ///
    /// Checks that all policies reference valid entity types, attributes,
    /// and actions according to the schema.
    ///
    /// # Errors
    /// Returns error with validation failure messages if validation fails
    pub fn validate_policies(&self) -> Result<(), Vec<String>> {
        let validator = Validator::new(self.schema.clone());
        let result = validator.validate(&self.policies, ValidationMode::default());

        if result.validation_passed() {
            Ok(())
        } else {
            let errors: Vec<String> = result
                .validation_errors()
                .map(|e| format!("{e}"))
                .collect();
            Err(errors)
        }
    }
}

/// Analyze a denied request to determine violation type
fn analyze_denial(
    request: &AuthorizationRequest,
    policy_ids: &[String],
) -> Option<AuthorityViolation> {
    // Extract IDs from entity UIDs
    let principal_id = extract_entity_id(&request.principal)?;
    let gate_id = extract_entity_id(&request.resource)?;

    // Check if any policy ID indicates high-risk forbid
    // Policy IDs from authority-rules.cedar will contain identifiable patterns
    let is_high_risk_forbid = policy_ids.iter().any(|pid| {
        pid.contains("high") || pid.contains("forbid")
    });

    // Determine team from principal UID
    // If principal contains "Extended" or matches Extended patterns
    let principal_team = if request.principal.contains("Extended") {
        "Extended".to_string()
    } else {
        "Core".to_string()  // Default assumption
    };

    // Build violation based on available info
    // Note: Full entity attribute lookup would require entities parameter
    // For now, infer from policy IDs and request structure
    let violation_type = if is_high_risk_forbid && principal_team == "Extended" {
        ViolationType::ExtendedOnHighRiskGate
    } else if principal_team == "Extended" {
        ViolationType::ExtendedNotElevated
    } else {
        ViolationType::InsufficientAuthority
    };

    Some(AuthorityViolation {
        principal_id,
        principal_team,
        principal_tier: "Unknown".to_string(), // Would need entity lookup for exact tier
        gate_id,
        gate_risk_class: "Unknown".to_string(), // Would need entity lookup
        violation_type,
    })
}

/// Extract entity ID from Cedar entity UID string
/// e.g., `ConvergePersonas::User::"sre-operations"` -> Some("sre-operations")
pub fn extract_entity_id(uid: &str) -> Option<String> {
    // Find the quoted ID at the end: Type::"id"
    if let Some(start) = uid.rfind("::\"") {
        let id_start = start + 3;  // Skip ::"
        if let Some(end) = uid[id_start..].find('"') {
            return Some(uid[id_start..id_start + end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_result_equality() {
        assert_eq!(AuthorizationResult::Allow, AuthorizationResult::Allow);

        assert_eq!(
            AuthorizationResult::Deny {
                reasons: vec!["test".to_string()],
                missing_authority: None,
            },
            AuthorizationResult::Deny {
                reasons: vec!["test".to_string()],
                missing_authority: None,
            }
        );

        assert_eq!(
            AuthorizationResult::Error {
                message: "test".to_string()
            },
            AuthorizationResult::Error {
                message: "test".to_string()
            }
        );
    }

    #[test]
    fn test_authorization_request_clone() {
        let request = AuthorizationRequest {
            principal: "User::\"alice\"".to_string(),
            action: "Action::\"read\"".to_string(),
            resource: "Resource::\"file1\"".to_string(),
            context: vec![("key".to_string(), "value".to_string())],
        };

        let cloned = request.clone();
        assert_eq!(request.principal, cloned.principal);
        assert_eq!(request.action, cloned.action);
        assert_eq!(request.resource, cloned.resource);
        assert_eq!(request.context.len(), cloned.context.len());
    }

    #[test]
    fn test_authority_violation_high_risk_message() {
        let violation = AuthorityViolation {
            principal_id: "sre-operations".to_string(),
            principal_team: "Extended".to_string(),
            principal_tier: "Escalating".to_string(),
            gate_id: "production-deploy".to_string(),
            gate_risk_class: "high".to_string(),
            violation_type: ViolationType::ExtendedOnHighRiskGate,
        };

        let message = violation.to_message();

        assert!(message.contains("sre-operations"));
        assert!(message.contains("production-deploy"));
        assert!(message.contains("high-risk"));
        assert!(message.contains("Core team authority"));
        assert!(message.contains("founder"));
    }

    #[test]
    fn test_authority_violation_not_elevated_message() {
        let violation = AuthorityViolation {
            principal_id: "marketing-lead".to_string(),
            principal_team: "Extended".to_string(),
            principal_tier: "Advisory".to_string(),
            gate_id: "pr-merge".to_string(),
            gate_risk_class: "medium".to_string(),
            violation_type: ViolationType::ExtendedNotElevated,
        };

        let message = violation.to_message();

        assert!(message.contains("marketing-lead"));
        assert!(message.contains("pr-merge"));
        assert!(message.contains("elevation"));
        assert!(message.contains("GATES.md"));
    }

    #[test]
    fn test_authority_violation_insufficient_authority_message() {
        let violation = AuthorityViolation {
            principal_id: "unknown-user".to_string(),
            principal_team: "Unknown".to_string(),
            principal_tier: "None".to_string(),
            gate_id: "release-approval".to_string(),
            gate_risk_class: "high".to_string(),
            violation_type: ViolationType::InsufficientAuthority,
        };

        let message = violation.to_message();

        assert!(message.contains("unknown-user"));
        assert!(message.contains("release-approval"));
        assert!(message.contains("Blocking-by-Policy"));
    }

    #[test]
    fn test_extract_entity_id() {
        assert_eq!(
            extract_entity_id("ConvergePersonas::User::\"sre-operations\""),
            Some("sre-operations".to_string())
        );
        assert_eq!(
            extract_entity_id("ConvergePersonas::Gate::\"production-deploy\""),
            Some("production-deploy".to_string())
        );
        assert_eq!(
            extract_entity_id("invalid-uid"),
            None
        );
    }

    #[test]
    fn test_violation_type_equality() {
        assert_eq!(
            ViolationType::ExtendedOnHighRiskGate,
            ViolationType::ExtendedOnHighRiskGate
        );
        assert_ne!(
            ViolationType::ExtendedOnHighRiskGate,
            ViolationType::ExtendedNotElevated
        );
    }
}
