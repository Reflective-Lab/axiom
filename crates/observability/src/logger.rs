//! Audit logger that publishes events to NATS JetStream.

use super::events::{AuditEvent, AuditEventError};
use crate::config::AuditConfig;
use crate::nats::NatsClient;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

/// Configuration for the audit logger.
#[derive(Debug, Clone)]
pub struct AuditLoggerConfig {
    /// JetStream stream name.
    pub stream: String,

    /// Subject prefix for audit events.
    pub subject_prefix: String,

    /// Buffer size for async logging.
    pub buffer_size: usize,

    /// Whether to block when buffer is full.
    pub block_on_full: bool,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            stream: "AUDIT".to_string(),
            subject_prefix: "audit.runtime".to_string(),
            buffer_size: 1000,
            block_on_full: false,
        }
    }
}

impl From<&AuditConfig> for AuditLoggerConfig {
    fn from(config: &AuditConfig) -> Self {
        Self {
            stream: config.stream.clone(),
            subject_prefix: config.subject_prefix.clone(),
            buffer_size: 1000,
            block_on_full: false,
        }
    }
}

/// Errors that can occur during audit logging.
#[derive(Debug, Error)]
pub enum AuditError {
    /// NATS connection unavailable.
    #[error("NATS connection unavailable")]
    NatsUnavailable,

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Publish failed.
    #[error("publish failed: {0}")]
    Publish(String),

    /// Event validation failed.
    #[error("event validation failed: {0}")]
    Validation(#[from] AuditEventError),

    /// Channel send error.
    #[error("channel send error: buffer full")]
    ChannelFull,

    /// Channel closed.
    #[error("channel closed")]
    ChannelClosed,
}

/// Audit logger that publishes events to NATS JetStream.
///
/// The logger uses a background task for non-blocking event publishing.
/// When NATS is unavailable, events are logged locally via tracing.
pub struct AuditLogger {
    config: AuditLoggerConfig,
    service_id: String,
    sender: mpsc::Sender<AuditEvent>,
    nats_available: bool,
}

impl AuditLogger {
    /// Create a new audit logger.
    ///
    /// If `nats` is `None`, the logger will operate in fallback mode,
    /// logging events via tracing instead of NATS.
    pub fn new(
        nats: Option<Arc<NatsClient>>,
        config: AuditLoggerConfig,
        service_id: impl Into<String>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<AuditEvent>(config.buffer_size);
        let service_id = service_id.into();
        let nats_available = nats.is_some();
        let config_clone = config.clone();

        // Spawn background task
        tokio::spawn(Self::background_publisher(receiver, nats, config_clone));

        Self {
            config,
            service_id,
            sender,
            nats_available,
        }
    }

    /// Create an audit logger without NATS (local logging only).
    pub fn local_only(service_id: impl Into<String>) -> Self {
        Self::new(None, AuditLoggerConfig::default(), service_id)
    }

    /// Background task that publishes events to NATS.
    async fn background_publisher(
        mut receiver: mpsc::Receiver<AuditEvent>,
        nats: Option<Arc<NatsClient>>,
        config: AuditLoggerConfig,
    ) {
        while let Some(event) = receiver.recv().await {
            let subject = format!("{}.{}", config.subject_prefix, event.subject_suffix());

            if let Some(ref nats_client) = nats {
                match event.to_json_bytes() {
                    Ok(bytes) => {
                        if let Err(e) = nats_client.publish(&subject, &bytes).await {
                            error!(
                                event_id = %event.event_id,
                                error = %e,
                                "Failed to publish audit event to NATS"
                            );
                            // Log locally as fallback
                            Self::log_locally(&event);
                        } else {
                            debug!(
                                event_id = %event.event_id,
                                subject = %subject,
                                "Published audit event"
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            event_id = %event.event_id,
                            error = %e,
                            "Failed to serialize audit event"
                        );
                    }
                }
            } else {
                // No NATS, log locally
                Self::log_locally(&event);
            }
        }

        debug!("Audit logger background task shutting down");
    }

    /// Log an event locally via tracing.
    fn log_locally(event: &AuditEvent) {
        warn!(
            event_id = %event.event_id,
            event_type = %event.event_type,
            principal = %event.principal,
            service_id = %event.service_id,
            method = ?event.method,
            resource = ?event.resource,
            outcome = %event.outcome,
            "Audit event (NATS unavailable)"
        );
    }

    /// Log an audit event asynchronously.
    ///
    /// The event is sent to a background task for publishing.
    /// This method validates the event and returns an error if invalid.
    pub async fn log(&self, mut event: AuditEvent) -> Result<(), AuditError> {
        // Ensure service_id is set
        if event.service_id.is_empty() {
            event.service_id = self.service_id.clone();
        }

        // Validate event
        event.validate()?;

        // Send to background task
        self.sender
            .send(event)
            .await
            .map_err(|_| AuditError::ChannelClosed)
    }

    /// Log an audit event synchronously (fire and forget).
    ///
    /// This method does not wait for the event to be published.
    /// If the buffer is full, the event is dropped with a warning.
    pub fn log_sync(&self, mut event: AuditEvent) {
        // Ensure service_id is set
        if event.service_id.is_empty() {
            event.service_id = self.service_id.clone();
        }

        // Best-effort validation
        if let Err(e) = event.validate() {
            warn!(error = %e, "Invalid audit event, dropping");
            return;
        }

        // Try to send, drop if channel is full
        if let Err(e) = self.sender.try_send(event) {
            match e {
                mpsc::error::TrySendError::Full(_) => {
                    warn!("Audit event buffer full, dropping event");
                }
                mpsc::error::TrySendError::Closed(_) => {
                    warn!("Audit logger channel closed");
                }
            }
        }
    }

    /// Check if NATS is available for audit logging.
    pub fn is_nats_available(&self) -> bool {
        self.nats_available
    }

    /// Get the service ID.
    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    /// Get the subject prefix.
    pub fn subject_prefix(&self) -> &str {
        &self.config.subject_prefix
    }
}

impl Clone for AuditLogger {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            service_id: self.service_id.clone(),
            sender: self.sender.clone(),
            nats_available: self.nats_available,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::events::AuditEventType;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let logger = AuditLogger::local_only("test-service");
        assert_eq!(logger.service_id(), "test-service");
        assert!(!logger.is_nats_available());
    }

    #[tokio::test]
    async fn test_audit_logger_log_without_nats() {
        let logger = AuditLogger::local_only("test-service");

        let event = AuditEvent::new(AuditEventType::MethodInvoked, "user-1")
            .with_service_id("test-service");

        // Should succeed even without NATS
        let result = logger.log(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_logger_sets_service_id() {
        let logger = AuditLogger::local_only("auto-service");

        // Event without service_id - logger should set it
        let event = AuditEvent::new(AuditEventType::AuthenticationSuccess, "user-1");

        // This would fail validation if service_id wasn't set
        let result = logger.log(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_logger_validation_failure() {
        let logger = AuditLogger::local_only("test-service");

        // Event with empty principal
        let mut event = AuditEvent::new(AuditEventType::MethodInvoked, "user");
        event.principal = String::new();

        let result = logger.log(event).await;
        assert!(matches!(result, Err(AuditError::Validation(_))));
    }

    #[test]
    fn test_audit_logger_config_default() {
        let config = AuditLoggerConfig::default();
        assert_eq!(config.stream, "AUDIT");
        assert_eq!(config.subject_prefix, "audit.runtime");
        assert_eq!(config.buffer_size, 1000);
    }

    #[test]
    fn test_audit_error_display() {
        let err = AuditError::NatsUnavailable;
        assert_eq!(err.to_string(), "NATS connection unavailable");

        let err = AuditError::ChannelFull;
        assert_eq!(err.to_string(), "channel send error: buffer full");
    }

    #[tokio::test]
    async fn test_log_sync_drops_on_invalid() {
        let logger = AuditLogger::local_only("test");

        // Invalid event - empty principal
        let mut event = AuditEvent::new(AuditEventType::MethodInvoked, "user");
        event.principal = String::new();

        // Should not panic, just warn and drop
        logger.log_sync(event);
    }

    #[test]
    fn test_config_from_audit_config() {
        let audit_config = AuditConfig {
            stream: "CUSTOM".to_string(),
            subject_prefix: "custom.prefix".to_string(),
            retention_days: 90,
        };

        let logger_config = AuditLoggerConfig::from(&audit_config);
        assert_eq!(logger_config.stream, "CUSTOM");
        assert_eq!(logger_config.subject_prefix, "custom.prefix");
    }

    #[tokio::test]
    async fn test_multiple_events() {
        let logger = AuditLogger::local_only("test-service");

        for i in 0..10 {
            let event = AuditEvent::new(AuditEventType::MethodInvoked, format!("user-{}", i))
                .with_method("/test/Method");

            logger.log(event).await.unwrap();
        }

        // Give background task time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}
