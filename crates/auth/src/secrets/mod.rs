//! Secret management with multiple backends.
//!
//! Supports loading secrets from:
//! - Environment variables (dev, via `.env` file)
//! - GCP Secret Manager (production)
//!
//! # Usage
//!
//! ```ignore
//! // At startup, initialize secrets
//! let resolver = SecretResolver::from_env()?;  // Dev: loads .env
//! // or
//! let resolver = SecretResolver::gcp("project-id").await?;  // Prod: GCP
//!
//! // Resolve secret references
//! let jwt_secret = resolver.resolve("${JWT_SECRET}").await?;
//! let api_keys = resolver.resolve_list("${API_KEYS}").await?;
//! ```
//!
//! # Secret Reference Formats
//!
//! | Format | Example | Backend |
//! |--------|---------|---------|
//! | `${VAR}` | `${JWT_SECRET}` | Environment variable |
//! | `env://VAR` | `env://JWT_SECRET` | Environment variable (explicit) |
//! | `gsm://project/secret` | `gsm://converge/jwt-key` | GCP Secret Manager |
//! | `file:///path` | `file:///run/secrets/jwt` | File contents |
//!
//! # Security
//!
//! - Secrets are never logged
//! - Secret values implement `Zeroize` for secure memory cleanup
//! - Debug output shows "[REDACTED]" instead of values

mod resolver;

#[cfg(feature = "gcp")]
mod gcp;

pub use resolver::{
    Secret, SecretBackend, SecretRef, SecretResolveError, SecretResolver, SecretValue,
};

#[cfg(feature = "gcp")]
pub use gcp::GcpSecretManager;
