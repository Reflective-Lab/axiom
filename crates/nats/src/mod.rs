//! NATS client with mTLS support.
//!
//! Provides secure NATS connections using the service identity for authentication.

#[cfg(feature = "nats")]
mod client;

#[cfg(feature = "nats")]
pub use client::{NatsClient, NatsClientConfig};

use thiserror::Error;

/// Errors that can occur during NATS operations.
#[derive(Debug, Error)]
pub enum NatsError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("TLS configuration error: {0}")]
    TlsConfig(String),

    #[error("publish failed: {0}")]
    Publish(String),

    #[error("subscribe failed: {0}")]
    Subscribe(String),

    #[error("JetStream error: {0}")]
    JetStream(String),

    #[error("identity error: {0}")]
    Identity(#[from] crate::identity::IdentityError),
}
