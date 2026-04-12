//! Audit logging via NATS JetStream.
//!
//! Provides a durable audit trail for security and operational events.

mod events;
mod logger;

pub use events::{AuditEvent, AuditEventError, AuditEventType, Outcome};
pub use logger::{AuditError, AuditLogger, AuditLoggerConfig};
