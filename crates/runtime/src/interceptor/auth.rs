//! Authentication interceptor for gRPC requests.
//!
//! Extracts and validates identity from:
//! - mTLS client certificates (service identity)
//! - Authorization header JWT (user identity)

use crate::auth::{
    JwtError, JwtValidator, JwtValidatorConfig, ServiceIdentity, UserIdentity, VerifiedIdentity,
};
use std::sync::Arc;
use tonic::{Request, Status};
use tracing::{debug, warn};

/// Extension key for verified identity in request extensions.
#[derive(Debug, Clone)]
pub struct VerifiedIdentityExt(pub VerifiedIdentity);

/// Authentication interceptor configuration.
#[derive(Debug, Clone)]
pub struct AuthInterceptorConfig {
    /// JWT validator configuration.
    pub jwt: Option<JwtValidatorConfig>,

    /// Whether to require user authentication (JWT).
    /// If false, service-only authentication is allowed.
    pub require_user_auth: bool,

    /// List of methods that don't require authentication.
    pub unauthenticated_methods: Vec<String>,
}

impl Default for AuthInterceptorConfig {
    fn default() -> Self {
        Self {
            jwt: None,
            require_user_auth: false,
            unauthenticated_methods: vec![
                // Capability negotiation doesn't require auth
                "/converge.ConvergeService/GetCapabilities".to_string(),
            ],
        }
    }
}

impl AuthInterceptorConfig {
    /// Create config with JWT validation.
    pub fn with_jwt(jwt_config: JwtValidatorConfig) -> Self {
        Self {
            jwt: Some(jwt_config),
            ..Default::default()
        }
    }

    /// Require user authentication for all methods.
    #[must_use]
    pub fn require_user_auth(mut self) -> Self {
        self.require_user_auth = true;
        self
    }

    /// Add a method that doesn't require authentication.
    #[must_use]
    pub fn allow_unauthenticated(mut self, method: impl Into<String>) -> Self {
        self.unauthenticated_methods.push(method.into());
        self
    }
}

/// Authentication interceptor for gRPC.
#[derive(Clone)]
pub struct AuthInterceptor {
    config: AuthInterceptorConfig,
    jwt_validator: Option<Arc<JwtValidator>>,
}

impl AuthInterceptor {
    /// Create a new authentication interceptor.
    pub fn new(config: AuthInterceptorConfig) -> Self {
        let jwt_validator = config.jwt.clone().map(|c| Arc::new(JwtValidator::new(c)));

        Self {
            config,
            jwt_validator,
        }
    }

    /// Check if a method requires authentication.
    ///
    /// Call this from the service layer to determine if a specific method
    /// needs authentication before processing.
    pub fn requires_auth(&self, method: &str) -> bool {
        !self
            .config
            .unauthenticated_methods
            .iter()
            .any(|m| m == method)
    }

    /// Check method authentication requirement and return error if needed.
    ///
    /// Use this in service methods:
    /// ```ignore
    /// if let Err(status) = interceptor.check_method("/converge.ConvergeService/SubmitJob", &request) {
    ///     return Err(status);
    /// }
    /// ```
    pub fn check_method<T>(&self, method: &str, request: &Request<T>) -> Result<(), Status> {
        if !self.requires_auth(method) {
            return Ok(());
        }

        // If method requires auth, verify identity is present
        if request.extensions().get::<VerifiedIdentityExt>().is_none() {
            return Err(Status::unauthenticated(
                "Authentication required for this method",
            ));
        }

        Ok(())
    }

    /// Extract service identity from request.
    ///
    /// In a real mTLS setup, this would extract from the TLS connection.
    /// For now, we check for a custom header as a fallback.
    fn extract_service_identity<T>(&self, request: &Request<T>) -> Option<ServiceIdentity> {
        // Try to get from TLS peer certificate (would be set by tonic-tls)
        // This is a placeholder - actual implementation depends on how tonic
        // exposes peer certificate info.

        // Fallback: check for X-Service-Id header (for testing/development)
        if let Some(service_id) = request.metadata().get("x-service-id") {
            if let Ok(id) = service_id.to_str() {
                debug!(service_id = %id, "Service identity from header");
                return Some(ServiceIdentity::new(id));
            }
        }

        // In production without mTLS, we might reject requests
        // For now, allow with a default service identity
        None
    }

    /// Extract user identity from JWT in Authorization header.
    fn extract_user_identity<T>(
        &self,
        request: &Request<T>,
    ) -> Result<Option<UserIdentity>, Status> {
        let Some(ref validator) = self.jwt_validator else {
            return Ok(None);
        };

        // Get Authorization header
        let auth_header = match request.metadata().get("authorization") {
            Some(h) => h,
            None => return Ok(None),
        };

        let auth_str = auth_header
            .to_str()
            .map_err(|_| Status::unauthenticated("Invalid authorization header encoding"))?;

        // Extract Bearer token
        let token = auth_str
            .strip_prefix("Bearer ")
            .or_else(|| auth_str.strip_prefix("bearer "))
            .ok_or_else(|| Status::unauthenticated("Authorization header must be Bearer token"))?;

        // Validate JWT
        let identity = validator.validate(token).map_err(|e| {
            warn!(error = %e, "JWT validation failed");
            match e {
                JwtError::Expired => Status::unauthenticated("Token expired"),
                JwtError::InvalidSignature => Status::unauthenticated("Invalid token signature"),
                JwtError::InvalidIssuer { .. } => Status::unauthenticated("Invalid token issuer"),
                JwtError::InvalidAudience => Status::unauthenticated("Invalid token audience"),
                _ => Status::unauthenticated("Invalid token"),
            }
        })?;

        Ok(Some(identity))
    }

    /// Authenticate a request and attach verified identity.
    ///
    /// Note: Method-level authentication checks should be done at the service
    /// layer using the `check_method` function, as tonic::Request doesn't
    /// expose the URI path in the interceptor context.
    pub fn authenticate<T>(&self, mut request: Request<T>) -> Result<Request<T>, Status> {
        debug!("Authenticating request");

        // Extract service identity (from mTLS or fallback)
        let service_identity = self.extract_service_identity(&request);

        // Extract user identity (from JWT)
        let user_identity = self.extract_user_identity(&request)?;

        // Build verified identity
        let verified = match (service_identity, user_identity) {
            (Some(svc), Some(user)) => VerifiedIdentity::with_user(svc, user),
            (Some(svc), None) => {
                if self.config.require_user_auth {
                    return Err(Status::unauthenticated("User authentication required"));
                }
                VerifiedIdentity::service_only(svc)
            }
            (None, Some(user)) => {
                // User auth present but no service auth - create anonymous service
                let svc = ServiceIdentity::new("anonymous");
                VerifiedIdentity::with_user(svc, user)
            }
            (None, None) => {
                if self.config.require_user_auth {
                    return Err(Status::unauthenticated("Authentication required"));
                }
                // Allow with anonymous identity for development
                let svc = ServiceIdentity::new("anonymous");
                VerifiedIdentity::service_only(svc)
            }
        };

        debug!(
            principal = %verified.principal(),
            has_user = verified.has_user(),
            "Request authenticated"
        );

        // Attach identity to request extensions
        request
            .extensions_mut()
            .insert(VerifiedIdentityExt(verified));

        Ok(request)
    }
}

/// Tower layer for the auth interceptor.
#[derive(Clone)]
pub struct AuthInterceptorLayer {
    interceptor: AuthInterceptor,
}

impl AuthInterceptorLayer {
    /// Create a new auth interceptor layer.
    pub fn new(config: AuthInterceptorConfig) -> Self {
        Self {
            interceptor: AuthInterceptor::new(config),
        }
    }

    /// Get a reference to the interceptor.
    pub fn interceptor(&self) -> &AuthInterceptor {
        &self.interceptor
    }
}

/// Helper trait to extract verified identity from request extensions.
pub trait RequestIdentityExt {
    /// Get the verified identity if present.
    fn identity(&self) -> Option<&VerifiedIdentity>;

    /// Get the verified identity or return Unauthenticated error.
    fn require_identity(&self) -> Result<&VerifiedIdentity, Status>;
}

impl<T> RequestIdentityExt for Request<T> {
    fn identity(&self) -> Option<&VerifiedIdentity> {
        self.extensions().get::<VerifiedIdentityExt>().map(|e| &e.0)
    }

    fn require_identity(&self) -> Result<&VerifiedIdentity, Status> {
        self.identity()
            .ok_or_else(|| Status::internal("Identity not found in request extensions"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

    fn test_jwt_config() -> JwtValidatorConfig {
        JwtValidatorConfig::new("test-secret-key-32-chars-long!!", "https://auth.test.com")
            .with_audience("converge-runtime")
    }

    #[test]
    fn test_default_config() {
        let config = AuthInterceptorConfig::default();
        assert!(!config.require_user_auth);
        assert!(
            config
                .unauthenticated_methods
                .contains(&"/converge.ConvergeService/GetCapabilities".to_string())
        );
    }

    #[test]
    fn test_requires_auth() {
        let config = AuthInterceptorConfig::default();
        let interceptor = AuthInterceptor::new(config);

        assert!(interceptor.requires_auth("/converge.ConvergeService/SubmitJob"));
        assert!(!interceptor.requires_auth("/converge.ConvergeService/GetCapabilities"));
    }

    #[test]
    fn test_authenticate_with_service_header() {
        let config = AuthInterceptorConfig::default();
        let interceptor = AuthInterceptor::new(config);

        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("x-service-id", MetadataValue::from_static("test-service"));

        let result = interceptor.authenticate(request);
        assert!(result.is_ok());

        let request = result.unwrap();
        let identity = request.identity().unwrap();
        assert_eq!(identity.service.service_id, "test-service");
    }

    #[test]
    fn test_authenticate_unauthenticated_method() {
        let config = AuthInterceptorConfig::default();
        let interceptor = AuthInterceptor::new(config);

        let mut request = Request::new(());
        // Simulate the method path
        // Note: In real usage, the URI is set by tonic

        // This test would need more setup to properly test URI-based method checking
        let result = interceptor.authenticate(request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_user_auth_fails_without_jwt() {
        let config = AuthInterceptorConfig::default().require_user_auth();
        let interceptor = AuthInterceptor::new(config);

        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("x-service-id", MetadataValue::from_static("test-service"));

        let result = interceptor.authenticate(request);
        assert!(result.is_err());
    }
}
