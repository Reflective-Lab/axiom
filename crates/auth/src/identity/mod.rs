//! Service identity management for mTLS.
//!
//! Provides certificate-based identity for service-to-service authentication.
//! Supports file-based certificates (dev/test) with SPIFFE as future enhancement.

mod file;

pub use file::{FileIdentity, FileIdentityConfig};

use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "security")]
use rustls::{ClientConfig, ServerConfig};

/// Errors that can occur during identity operations.
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("failed to load certificate: {0}")]
    CertificateLoad(String),

    #[error("failed to load private key: {0}")]
    PrivateKeyLoad(String),

    #[error("failed to load CA certificates: {0}")]
    CaLoad(String),

    #[error("TLS configuration error: {0}")]
    TlsConfig(String),

    #[error("no valid certificates found")]
    NoCertificates,

    #[error("no valid private key found")]
    NoPrivateKey,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Trait for service identity providers.
///
/// Implementations provide TLS configurations for both client and server roles.
#[cfg(feature = "security")]
pub trait Identity: Send + Sync {
    /// Get the service identifier (e.g., SPIFFE ID or CN from certificate).
    fn service_id(&self) -> &str;

    /// Build a TLS client config for outgoing connections.
    fn client_config(&self) -> Result<Arc<ClientConfig>, IdentityError>;

    /// Build a TLS server config for incoming connections.
    fn server_config(&self) -> Result<Arc<ServerConfig>, IdentityError>;
}

/// Identity provider that can be used when security feature is disabled.
#[cfg(not(feature = "security"))]
pub trait Identity: Send + Sync {
    fn service_id(&self) -> &str;
}
