//! Audit event types and serialization.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors related to audit events.
#[derive(Debug, Error)]
pub enum AuditEventError {
    /// Event is missing required fields.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid event type.
    #[error("invalid event type: {0}")]
    InvalidType(String),
}

/// Type of audit event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Successful authentication.
    AuthenticationSuccess,
    /// Failed authentication attempt.
    AuthenticationFailure,
    /// Authorization granted.
    AuthorizationGranted,
    /// Authorization denied.
    AuthorizationDenied,
    /// gRPC method invoked.
    MethodInvoked,
    /// Job submitted.
    JobSubmitted,
    /// Job completed successfully.
    JobCompleted,
    /// Job failed.
    JobFailed,
    /// Configuration changed.
    ConfigChanged,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::AuthenticationSuccess => write!(f, "authentication_success"),
            AuditEventType::AuthenticationFailure => write!(f, "authentication_failure"),
            AuditEventType::AuthorizationGranted => write!(f, "authorization_granted"),
            AuditEventType::AuthorizationDenied => write!(f, "authorization_denied"),
            AuditEventType::MethodInvoked => write!(f, "method_invoked"),
            AuditEventType::JobSubmitted => write!(f, "job_submitted"),
            AuditEventType::JobCompleted => write!(f, "job_completed"),
            AuditEventType::JobFailed => write!(f, "job_failed"),
            AuditEventType::ConfigChanged => write!(f, "config_changed"),
        }
    }
}

/// Outcome of an audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Action succeeded.
    #[default]
    Success,
    /// Action failed.
    Failure,
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Success => write!(f, "success"),
            Outcome::Failure => write!(f, "failure"),
        }
    }
}

/// An audit event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier.
    pub event_id: String,

    /// Type of event.
    pub event_type: AuditEventType,

    /// Timestamp in nanoseconds since Unix epoch.
    pub timestamp_ns: u64,

    /// Principal who performed the action (user ID or service ID).
    pub principal: String,

    /// Service that generated this event.
    pub service_id: String,

    /// gRPC method if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Resource identifier if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,

    /// Outcome of the action.
    pub outcome: Outcome,

    /// Additional context/details.
    #[serde(default)]
    pub details: serde_json::Value,

    /// Distributed trace ID if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Client IP address if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,

    /// User agent if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event.
    pub fn new(event_type: AuditEventType, principal: impl Into<String>) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp_ns: Self::now_ns(),
            principal: principal.into(),
            service_id: String::new(),
            method: None,
            resource: None,
            outcome: Outcome::Success,
            details: serde_json::Value::Null,
            trace_id: None,
            client_ip: None,
            user_agent: None,
        }
    }

    /// Set the service ID.
    #[must_use]
    pub fn with_service_id(mut self, service_id: impl Into<String>) -> Self {
        self.service_id = service_id.into();
        self
    }

    /// Set the method.
    #[must_use]
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Set the resource.
    #[must_use]
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set the outcome.
    #[must_use]
    pub fn with_outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set additional details.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    /// Set the trace ID.
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Set the client IP.
    #[must_use]
    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    /// Validate the event has required fields.
    pub fn validate(&self) -> Result<(), AuditEventError> {
        if self.principal.is_empty() {
            return Err(AuditEventError::MissingField("principal".to_string()));
        }
        if self.service_id.is_empty() {
            return Err(AuditEventError::MissingField("service_id".to_string()));
        }
        Ok(())
    }

    /// Get the NATS subject suffix for this event type.
    pub fn subject_suffix(&self) -> &'static str {
        match self.event_type {
            AuditEventType::AuthenticationSuccess | AuditEventType::AuthenticationFailure => "auth",
            AuditEventType::AuthorizationGranted | AuditEventType::AuthorizationDenied => "authz",
            AuditEventType::MethodInvoked => "method",
            AuditEventType::JobSubmitted
            | AuditEventType::JobCompleted
            | AuditEventType::JobFailed => "job",
            AuditEventType::ConfigChanged => "config",
        }
    }

    /// Serialize to JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, AuditEventError> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, AuditEventError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    fn now_ns() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

/// Builder for authentication events.
pub struct AuthEventBuilder {
    event: AuditEvent,
}

impl AuthEventBuilder {
    /// Create a successful authentication event.
    pub fn success(principal: impl Into<String>) -> Self {
        Self {
            event: AuditEvent::new(AuditEventType::AuthenticationSuccess, principal),
        }
    }

    /// Create a failed authentication event.
    pub fn failure(principal: impl Into<String>, reason: impl Into<String>) -> Self {
        let mut event = AuditEvent::new(AuditEventType::AuthenticationFailure, principal)
            .with_outcome(Outcome::Failure);
        event.details = serde_json::json!({ "reason": reason.into() });
        Self { event }
    }

    /// Set the service ID.
    #[must_use]
    pub fn service(mut self, service_id: impl Into<String>) -> Self {
        self.event = self.event.with_service_id(service_id);
        self
    }

    /// Set the client IP.
    #[must_use]
    pub fn client_ip(mut self, ip: impl Into<String>) -> Self {
        self.event = self.event.with_client_ip(ip);
        self
    }

    /// Build the event.
    pub fn build(self) -> AuditEvent {
        self.event
    }
}

/// Builder for authorization events.
pub struct AuthzEventBuilder {
    event: AuditEvent,
}

impl AuthzEventBuilder {
    /// Create an authorization granted event.
    pub fn granted(principal: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            event: AuditEvent::new(AuditEventType::AuthorizationGranted, principal)
                .with_method(method),
        }
    }

    /// Create an authorization denied event.
    pub fn denied(
        principal: impl Into<String>,
        method: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let mut event = AuditEvent::new(AuditEventType::AuthorizationDenied, principal)
            .with_method(method)
            .with_outcome(Outcome::Failure);
        event.details = serde_json::json!({ "reason": reason.into() });
        Self { event }
    }

    /// Set the service ID.
    #[must_use]
    pub fn service(mut self, service_id: impl Into<String>) -> Self {
        self.event = self.event.with_service_id(service_id);
        self
    }

    /// Build the event.
    pub fn build(self) -> AuditEvent {
        self.event
    }
}

/// Builder for job events.
pub struct JobEventBuilder {
    event: AuditEvent,
}

impl JobEventBuilder {
    /// Create a job submitted event.
    pub fn submitted(principal: impl Into<String>, job_id: impl Into<String>) -> Self {
        Self {
            event: AuditEvent::new(AuditEventType::JobSubmitted, principal).with_resource(job_id),
        }
    }

    /// Create a job completed event.
    pub fn completed(principal: impl Into<String>, job_id: impl Into<String>) -> Self {
        Self {
            event: AuditEvent::new(AuditEventType::JobCompleted, principal).with_resource(job_id),
        }
    }

    /// Create a job failed event.
    pub fn failed(
        principal: impl Into<String>,
        job_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        let mut event = AuditEvent::new(AuditEventType::JobFailed, principal)
            .with_resource(job_id)
            .with_outcome(Outcome::Failure);
        event.details = serde_json::json!({ "error": error.into() });
        Self { event }
    }

    /// Set the service ID.
    #[must_use]
    pub fn service(mut self, service_id: impl Into<String>) -> Self {
        self.event = self.event.with_service_id(service_id);
        self
    }

    /// Add job details.
    #[must_use]
    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.event = self.event.with_details(details);
        self
    }

    /// Build the event.
    pub fn build(self) -> AuditEvent {
        self.event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(
            AuditEventType::AuthenticationSuccess.to_string(),
            "authentication_success"
        );
        assert_eq!(AuditEventType::JobFailed.to_string(), "job_failed");
    }

    #[test]
    fn test_outcome_display() {
        assert_eq!(Outcome::Success.to_string(), "success");
        assert_eq!(Outcome::Failure.to_string(), "failure");
    }

    #[test]
    fn test_event_creation() {
        let event = AuditEvent::new(AuditEventType::MethodInvoked, "user-123")
            .with_service_id("runtime")
            .with_method("/test/Method");

        assert_eq!(event.principal, "user-123");
        assert_eq!(event.service_id, "runtime");
        assert_eq!(event.method, Some("/test/Method".to_string()));
        assert!(!event.event_id.is_empty());
        assert!(event.timestamp_ns > 0);
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let event = AuditEvent::new(AuditEventType::JobSubmitted, "user-456")
            .with_service_id("runtime")
            .with_resource("job-789")
            .with_details(serde_json::json!({"blueprint": "test"}));

        let bytes = event.to_json_bytes().unwrap();
        let restored = AuditEvent::from_json_bytes(&bytes).unwrap();

        assert_eq!(restored.principal, event.principal);
        assert_eq!(restored.service_id, event.service_id);
        assert_eq!(restored.resource, event.resource);
        assert_eq!(restored.event_type, event.event_type);
    }

    #[test]
    fn test_event_validation_missing_principal() {
        let mut event = AuditEvent::new(AuditEventType::MethodInvoked, "user");
        event.principal = String::new();
        event.service_id = "svc".to_string();

        let result = event.validate();
        assert!(matches!(result, Err(AuditEventError::MissingField(_))));
    }

    #[test]
    fn test_event_validation_missing_service() {
        let event = AuditEvent::new(AuditEventType::MethodInvoked, "user");
        // service_id is empty by default

        let result = event.validate();
        assert!(matches!(result, Err(AuditEventError::MissingField(_))));
    }

    #[test]
    fn test_subject_suffix() {
        assert_eq!(
            AuditEvent::new(AuditEventType::AuthenticationSuccess, "u").subject_suffix(),
            "auth"
        );
        assert_eq!(
            AuditEvent::new(AuditEventType::AuthorizationDenied, "u").subject_suffix(),
            "authz"
        );
        assert_eq!(
            AuditEvent::new(AuditEventType::JobCompleted, "u").subject_suffix(),
            "job"
        );
    }

    #[test]
    fn test_auth_event_builder_success() {
        let event = AuthEventBuilder::success("user-123")
            .service("runtime")
            .client_ip("192.168.1.1")
            .build();

        assert_eq!(event.event_type, AuditEventType::AuthenticationSuccess);
        assert_eq!(event.principal, "user-123");
        assert_eq!(event.outcome, Outcome::Success);
    }

    #[test]
    fn test_auth_event_builder_failure() {
        let event = AuthEventBuilder::failure("user-123", "invalid token")
            .service("runtime")
            .build();

        assert_eq!(event.event_type, AuditEventType::AuthenticationFailure);
        assert_eq!(event.outcome, Outcome::Failure);
        assert!(
            event.details["reason"]
                .as_str()
                .unwrap()
                .contains("invalid")
        );
    }

    #[test]
    fn test_authz_event_builder() {
        let granted = AuthzEventBuilder::granted("user-123", "/test/Method")
            .service("runtime")
            .build();
        assert_eq!(granted.event_type, AuditEventType::AuthorizationGranted);

        let denied = AuthzEventBuilder::denied("user-123", "/admin/Method", "no permission")
            .service("runtime")
            .build();
        assert_eq!(denied.event_type, AuditEventType::AuthorizationDenied);
        assert_eq!(denied.outcome, Outcome::Failure);
    }

    #[test]
    fn test_job_event_builder() {
        let submitted = JobEventBuilder::submitted("user-123", "job-456")
            .service("runtime")
            .details(serde_json::json!({"blueprint": "growth"}))
            .build();

        assert_eq!(submitted.event_type, AuditEventType::JobSubmitted);
        assert_eq!(submitted.resource, Some("job-456".to_string()));

        let failed = JobEventBuilder::failed("user-123", "job-456", "timeout")
            .service("runtime")
            .build();

        assert_eq!(failed.event_type, AuditEventType::JobFailed);
        assert_eq!(failed.outcome, Outcome::Failure);
    }

    // Property-like tests
    #[test]
    fn test_all_event_types_have_subject_suffix() {
        let types = [
            AuditEventType::AuthenticationSuccess,
            AuditEventType::AuthenticationFailure,
            AuditEventType::AuthorizationGranted,
            AuditEventType::AuthorizationDenied,
            AuditEventType::MethodInvoked,
            AuditEventType::JobSubmitted,
            AuditEventType::JobCompleted,
            AuditEventType::JobFailed,
            AuditEventType::ConfigChanged,
        ];

        for event_type in types {
            let event = AuditEvent::new(event_type, "test");
            let suffix = event.subject_suffix();
            assert!(
                !suffix.is_empty(),
                "Event type {:?} has empty suffix",
                event_type
            );
        }
    }

    #[test]
    fn test_event_id_uniqueness() {
        let e1 = AuditEvent::new(AuditEventType::MethodInvoked, "u");
        let e2 = AuditEvent::new(AuditEventType::MethodInvoked, "u");
        assert_ne!(e1.event_id, e2.event_id);
    }
}
