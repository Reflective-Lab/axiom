//! Authentication module for JWT validation, token issuance, and identity extraction.
//!
//! # Security Architecture
//!
//! This module implements the authentication layer of the zero-trust security model:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Zero Trust Authentication Flow                              │
//! ├─────────────────────────────────────────────────────────────┤
//! │ 1. Passkey/Password/API Key → AuthProvider validates        │
//! │ 2. TokenIssuer creates JWT with user claims                 │
//! │ 3. Every request carries JWT (Authorization: Bearer ...)    │
//! │ 4. JwtValidator/FirebaseValidator verifies identity         │
//! │ 5. Policy engine checks permissions                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Components
//!
//! - **Validators**: Verify incoming JWTs (HS256 internal, RS256 Firebase)
//! - **Issuer**: Create JWTs after successful authentication
//! - **Providers**: Pluggable authentication backends (password, passkey, API key)
//! - **Identity**: Verified user/service identity for authorization
//!
//! # Usage
//!
//! ```ignore
//! // Set up authentication
//! let issuer = TokenIssuer::new(issuer_config);
//! let validator = JwtValidator::new(validator_config);
//!
//! // Authenticate user (via any provider)
//! let user = provider.authenticate(&credentials).await?;
//!
//! // Issue tokens
//! let tokens = issuer.issue(&user)?;
//!
//! // Later: validate incoming request
//! let identity = validator.validate(&token)?;
//! ```

mod identity;
mod issuer;
mod jwt;
mod provider;

#[cfg(feature = "firebase")]
mod firebase;

// Core identity types
pub use identity::{ServiceIdentity, UserIdentity, VerifiedIdentity};

// JWT validation (HS256)
pub use jwt::{Audience, Claims, JwtError, JwtValidator, JwtValidatorConfig};

// Token issuance
pub use issuer::{
    AuthMethod, AuthenticatedUser, IssuerError, TokenIssuer, TokenIssuerConfig, TokenPair,
};

// Auth providers
pub use provider::{
    ApiKeyProvider, AuthProvider, AuthProviderError, AuthProviderRegistry, Credentials,
    FirebaseAuthProvider, MemoryAuthProvider, ServiceAccountProvider, UserInfo,
};

// Firebase validation (RS256)
#[cfg(feature = "firebase")]
pub use firebase::{FirebaseClaims, FirebaseConfig, FirebaseValidator};
