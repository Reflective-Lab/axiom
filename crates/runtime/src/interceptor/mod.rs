//! gRPC interceptors for authentication and authorization.
//!
//! Provides request-level processing for:
//! - Extracting service identity from mTLS client certificates
//! - Validating JWT tokens and extracting user identity
//! - Attaching verified identity to request extensions

mod auth;

pub use auth::{AuthInterceptor, AuthInterceptorLayer};
