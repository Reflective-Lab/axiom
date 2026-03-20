//! Authentication provider abstraction.
//!
//! Defines traits for different authentication methods that can be
//! plugged into the auth system.

use super::issuer::{AuthMethod, AuthenticatedUser};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors from authentication providers.
#[derive(Debug, Error)]
pub enum AuthProviderError {
    /// Invalid credentials.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// User not found.
    #[error("user not found: {0}")]
    UserNotFound(String),

    /// Account locked or disabled.
    #[error("account disabled: {0}")]
    AccountDisabled(String),

    /// Rate limited.
    #[error("rate limited")]
    RateLimited,

    /// Provider not available.
    #[error("provider unavailable: {0}")]
    Unavailable(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Credentials for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Credentials {
    /// Username/password authentication.
    Password { username: String, password: String },

    /// API key authentication.
    ApiKey { key: String },

    /// Passkey/WebAuthn assertion.
    Passkey {
        credential_id: String,
        authenticator_data: String,
        client_data_json: String,
        signature: String,
    },

    /// OAuth authorization code.
    OAuth {
        provider: String,
        code: String,
        redirect_uri: String,
    },

    /// Pre-validated Firebase token (already verified by FirebaseValidator).
    FirebaseToken {
        user_id: String,
        email: Option<String>,
    },

    /// Service account credentials.
    ServiceAccount { service_id: String, secret: String },
}

/// Trait for authentication providers.
///
/// Use `#[async_trait]` on implementations.
#[async_trait]
pub trait AuthProvider: Send + Sync + 'static {
    /// Provider name for logging/metrics.
    fn name(&self) -> &str;

    /// Authenticate with the given credentials.
    async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<AuthenticatedUser, AuthProviderError>;

    /// Check if this provider can handle the given credentials.
    fn supports(&self, credentials: &Credentials) -> bool;
}

/// Registry of authentication providers.
pub struct AuthProviderRegistry {
    providers: Vec<Arc<dyn AuthProvider>>,
}

impl AuthProviderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a provider.
    pub fn register(&mut self, provider: Arc<dyn AuthProvider>) {
        self.providers.push(provider);
    }

    /// Authenticate using the first provider that supports the credentials.
    pub async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<AuthenticatedUser, AuthProviderError> {
        for provider in &self.providers {
            if provider.supports(credentials) {
                return provider.authenticate(credentials).await;
            }
        }

        Err(AuthProviderError::Unavailable(
            "no provider for credential type".to_string(),
        ))
    }

    /// Get provider count.
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for AuthProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Built-in Providers
// ============================================================================

/// Simple in-memory user store for testing/development.
pub struct MemoryAuthProvider {
    /// Users: username -> (password_hash, user_info)
    users: HashMap<String, (String, UserInfo)>,
}

/// User information stored in memory provider.
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub org_id: Option<String>,
    pub disabled: bool,
}

impl MemoryAuthProvider {
    /// Create a new memory provider.
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Add a user (password stored as-is, use hashing in production!).
    pub fn add_user(&mut self, username: &str, password: &str, info: UserInfo) {
        self.users
            .insert(username.to_string(), (password.to_string(), info));
    }

    /// Create with a default test user.
    pub fn with_test_user() -> Self {
        let mut provider = Self::new();
        provider.add_user(
            "test",
            "test123",
            UserInfo {
                user_id: "user-test".to_string(),
                email: Some("test@example.com".to_string()),
                roles: vec!["user".to_string()],
                org_id: None,
                disabled: false,
            },
        );
        provider
    }
}

impl Default for MemoryAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for MemoryAuthProvider {
    fn name(&self) -> &str {
        "memory"
    }

    fn supports(&self, credentials: &Credentials) -> bool {
        matches!(credentials, Credentials::Password { .. })
    }

    async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<AuthenticatedUser, AuthProviderError> {
        let (username, password) = match credentials {
            Credentials::Password { username, password } => (username, password),
            _ => return Err(AuthProviderError::InvalidCredentials),
        };

        let (stored_password, info) = self
            .users
            .get(username)
            .ok_or_else(|| AuthProviderError::UserNotFound(username.clone()))?;

        if info.disabled {
            return Err(AuthProviderError::AccountDisabled(username.clone()));
        }

        // In production, use constant-time comparison and proper hashing!
        if password != stored_password {
            return Err(AuthProviderError::InvalidCredentials);
        }

        Ok(AuthenticatedUser::new(&info.user_id)
            .with_email(info.email.clone().unwrap_or_default())
            .with_roles(info.roles.clone())
            .with_org_id(info.org_id.clone().unwrap_or_default())
            .with_auth_method(AuthMethod::Password))
    }
}

/// API key authentication provider.
pub struct ApiKeyProvider {
    /// API keys: key -> user_info
    keys: HashMap<String, UserInfo>,
}

impl ApiKeyProvider {
    /// Create a new API key provider.
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Add an API key.
    pub fn add_key(&mut self, key: &str, info: UserInfo) {
        self.keys.insert(key.to_string(), info);
    }
}

impl Default for ApiKeyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    fn name(&self) -> &str {
        "api_key"
    }

    fn supports(&self, credentials: &Credentials) -> bool {
        matches!(credentials, Credentials::ApiKey { .. })
    }

    async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<AuthenticatedUser, AuthProviderError> {
        let key = match credentials {
            Credentials::ApiKey { key } => key,
            _ => return Err(AuthProviderError::InvalidCredentials),
        };

        let info = self
            .keys
            .get(key)
            .ok_or(AuthProviderError::InvalidCredentials)?;

        if info.disabled {
            return Err(AuthProviderError::AccountDisabled(info.user_id.clone()));
        }

        Ok(AuthenticatedUser::new(&info.user_id)
            .with_email(info.email.clone().unwrap_or_default())
            .with_roles(info.roles.clone())
            .with_org_id(info.org_id.clone().unwrap_or_default())
            .with_auth_method(AuthMethod::ApiKey))
    }
}

/// Firebase token provider (for pre-validated tokens).
pub struct FirebaseAuthProvider;

impl FirebaseAuthProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FirebaseAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for FirebaseAuthProvider {
    fn name(&self) -> &str {
        "firebase"
    }

    fn supports(&self, credentials: &Credentials) -> bool {
        matches!(credentials, Credentials::FirebaseToken { .. })
    }

    async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<AuthenticatedUser, AuthProviderError> {
        // Firebase tokens are pre-validated by FirebaseValidator
        // This provider just converts the validated claims to AuthenticatedUser
        let (user_id, email) = match credentials {
            Credentials::FirebaseToken { user_id, email } => (user_id, email),
            _ => return Err(AuthProviderError::InvalidCredentials),
        };

        let mut user = AuthenticatedUser::new(user_id).with_auth_method(AuthMethod::Firebase);

        if let Some(email) = email {
            user = user.with_email(email);
        }

        Ok(user)
    }
}

/// Service account provider.
pub struct ServiceAccountProvider {
    /// Service accounts: service_id -> secret
    accounts: HashMap<String, String>,
}

impl ServiceAccountProvider {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Add a service account.
    pub fn add_account(&mut self, service_id: &str, secret: &str) {
        self.accounts
            .insert(service_id.to_string(), secret.to_string());
    }
}

impl Default for ServiceAccountProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for ServiceAccountProvider {
    fn name(&self) -> &str {
        "service_account"
    }

    fn supports(&self, credentials: &Credentials) -> bool {
        matches!(credentials, Credentials::ServiceAccount { .. })
    }

    async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<AuthenticatedUser, AuthProviderError> {
        let (service_id, secret) = match credentials {
            Credentials::ServiceAccount { service_id, secret } => (service_id, secret),
            _ => return Err(AuthProviderError::InvalidCredentials),
        };

        let stored_secret = self
            .accounts
            .get(service_id)
            .ok_or_else(|| AuthProviderError::UserNotFound(service_id.clone()))?;

        // In production, use constant-time comparison!
        if secret != stored_secret {
            return Err(AuthProviderError::InvalidCredentials);
        }

        Ok(AuthenticatedUser::new(service_id)
            .with_role("service")
            .with_auth_method(AuthMethod::Service))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_provider_valid_credentials() {
        let provider = MemoryAuthProvider::with_test_user();

        let creds = Credentials::Password {
            username: "test".to_string(),
            password: "test123".to_string(),
        };

        let user = provider.authenticate(&creds).await.unwrap();
        assert_eq!(user.user_id, "user-test");
        assert_eq!(user.email, Some("test@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_memory_provider_invalid_password() {
        let provider = MemoryAuthProvider::with_test_user();

        let creds = Credentials::Password {
            username: "test".to_string(),
            password: "wrong".to_string(),
        };

        let result = provider.authenticate(&creds).await;
        assert!(matches!(result, Err(AuthProviderError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_memory_provider_user_not_found() {
        let provider = MemoryAuthProvider::with_test_user();

        let creds = Credentials::Password {
            username: "unknown".to_string(),
            password: "test123".to_string(),
        };

        let result = provider.authenticate(&creds).await;
        assert!(matches!(result, Err(AuthProviderError::UserNotFound(_))));
    }

    #[tokio::test]
    async fn test_api_key_provider() {
        let mut provider = ApiKeyProvider::new();
        provider.add_key(
            "sk-test-123",
            UserInfo {
                user_id: "api-user".to_string(),
                email: Some("api@example.com".to_string()),
                roles: vec!["api".to_string()],
                org_id: None,
                disabled: false,
            },
        );

        let creds = Credentials::ApiKey {
            key: "sk-test-123".to_string(),
        };

        let user = provider.authenticate(&creds).await.unwrap();
        assert_eq!(user.user_id, "api-user");
        assert_eq!(user.auth_method, AuthMethod::ApiKey);
    }

    #[tokio::test]
    async fn test_registry() {
        let mut registry = AuthProviderRegistry::new();
        registry.register(Arc::new(MemoryAuthProvider::with_test_user()));

        let creds = Credentials::Password {
            username: "test".to_string(),
            password: "test123".to_string(),
        };

        let user = registry.authenticate(&creds).await.unwrap();
        assert_eq!(user.user_id, "user-test");
    }

    #[tokio::test]
    async fn test_registry_no_provider() {
        let registry = AuthProviderRegistry::new();

        let creds = Credentials::Password {
            username: "test".to_string(),
            password: "test123".to_string(),
        };

        let result = registry.authenticate(&creds).await;
        assert!(matches!(result, Err(AuthProviderError::Unavailable(_))));
    }

    #[tokio::test]
    async fn test_service_account_provider() {
        let mut provider = ServiceAccountProvider::new();
        provider.add_account("converge-worker", "secret-123");

        let creds = Credentials::ServiceAccount {
            service_id: "converge-worker".to_string(),
            secret: "secret-123".to_string(),
        };

        let user = provider.authenticate(&creds).await.unwrap();
        assert_eq!(user.user_id, "converge-worker");
        assert_eq!(user.auth_method, AuthMethod::Service);
        assert!(user.roles.contains(&"service".to_string()));
    }

    #[tokio::test]
    async fn test_firebase_provider() {
        let provider = FirebaseAuthProvider::new();

        let creds = Credentials::FirebaseToken {
            user_id: "firebase-user-123".to_string(),
            email: Some("user@example.com".to_string()),
        };

        let user = provider.authenticate(&creds).await.unwrap();
        assert_eq!(user.user_id, "firebase-user-123");
        assert_eq!(user.auth_method, AuthMethod::Firebase);
    }

    #[test]
    fn test_credentials_serialization() {
        let creds = Credentials::Password {
            username: "test".to_string(),
            password: "pass".to_string(),
        };

        let json = serde_json::to_string(&creds).unwrap();
        assert!(json.contains("password"));
        assert!(json.contains("username"));
    }
}
