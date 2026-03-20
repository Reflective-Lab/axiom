//! Secret reference parsing and resolution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
#[cfg(feature = "gcp")]
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors from secret resolution.
#[derive(Debug, Error)]
pub enum SecretResolveError {
    /// Secret not found.
    #[error("secret not found: {0}")]
    NotFound(String),

    /// Invalid secret reference format.
    #[error("invalid secret reference: {0}")]
    InvalidRef(String),

    /// Failed to read secret from backend.
    #[error("backend error: {0}")]
    Backend(String),

    /// File read error.
    #[error("file error: {0}")]
    File(#[from] std::io::Error),

    /// Environment variable not set.
    #[error("environment variable not set: {0}")]
    EnvNotSet(String),
}

/// A resolved secret value.
///
/// Implements secure handling:
/// - Debug shows "[REDACTED]"
/// - Clone is explicit (secrets shouldn't be copied carelessly)
#[derive(Clone)]
pub struct SecretValue {
    value: String,
}

impl SecretValue {
    /// Create a new secret value.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the secret value.
    ///
    /// Use sparingly - prefer passing SecretValue around.
    pub fn expose(&self) -> &str {
        &self.value
    }

    /// Consume and return the inner value.
    pub fn into_inner(self) -> String {
        self.value
    }

    /// Get length without exposing value.
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED {} bytes]", self.value.len())
    }
}

impl fmt::Display for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// A named secret with metadata.
#[derive(Debug)]
pub struct Secret {
    /// Secret name/identifier.
    pub name: String,

    /// The secret value.
    pub value: SecretValue,

    /// Source backend.
    pub source: SecretBackend,
}

/// Secret backend type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretBackend {
    /// Environment variable.
    Env,
    /// GCP Secret Manager.
    Gcp,
    /// Local file.
    File,
    /// Direct value (testing only).
    Direct,
}

/// Parsed secret reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretRef {
    /// Environment variable: `${VAR}` or `env://VAR`
    Env(String),
    /// GCP Secret Manager: `gsm://project/secret` or `gsm://project/secret/version`
    Gcp {
        project: String,
        secret: String,
        version: Option<String>,
    },
    /// File path: `file:///path/to/secret`
    File(String),
    /// Direct value (for testing): `direct://value`
    Direct(String),
}

impl SecretRef {
    /// Parse a secret reference string.
    ///
    /// Supported formats:
    /// - `${VAR_NAME}` - environment variable
    /// - `env://VAR_NAME` - environment variable (explicit)
    /// - `gsm://project/secret` - GCP Secret Manager (latest version)
    /// - `gsm://project/secret/version` - GCP Secret Manager (specific version)
    /// - `file:///path/to/file` - file contents
    /// - `direct://value` - direct value (testing only)
    pub fn parse(s: &str) -> Result<Self, SecretResolveError> {
        let s = s.trim();

        // ${VAR} format
        if s.starts_with("${") && s.ends_with('}') {
            let var_name = &s[2..s.len() - 1];
            if var_name.is_empty() {
                return Err(SecretResolveError::InvalidRef(
                    "empty variable name".to_string(),
                ));
            }
            return Ok(SecretRef::Env(var_name.to_string()));
        }

        // URL-style formats
        if let Some(rest) = s.strip_prefix("env://") {
            if rest.is_empty() {
                return Err(SecretResolveError::InvalidRef(
                    "empty variable name".to_string(),
                ));
            }
            return Ok(SecretRef::Env(rest.to_string()));
        }

        if let Some(rest) = s.strip_prefix("gsm://") {
            let parts: Vec<&str> = rest.splitn(3, '/').collect();
            if parts.len() < 2 {
                return Err(SecretResolveError::InvalidRef(
                    "gsm:// requires project/secret".to_string(),
                ));
            }
            return Ok(SecretRef::Gcp {
                project: parts[0].to_string(),
                secret: parts[1].to_string(),
                version: parts.get(2).map(|s| s.to_string()),
            });
        }

        if let Some(rest) = s.strip_prefix("file://") {
            if rest.is_empty() {
                return Err(SecretResolveError::InvalidRef(
                    "empty file path".to_string(),
                ));
            }
            return Ok(SecretRef::File(rest.to_string()));
        }

        if let Some(rest) = s.strip_prefix("direct://") {
            return Ok(SecretRef::Direct(rest.to_string()));
        }

        // No prefix - assume it's a direct value or error
        Err(SecretResolveError::InvalidRef(format!(
            "unknown secret reference format: {}",
            s
        )))
    }

    /// Get the backend type for this reference.
    pub fn backend(&self) -> SecretBackend {
        match self {
            SecretRef::Env(_) => SecretBackend::Env,
            SecretRef::Gcp { .. } => SecretBackend::Gcp,
            SecretRef::File(_) => SecretBackend::File,
            SecretRef::Direct(_) => SecretBackend::Direct,
        }
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretRef::Env(var) => write!(f, "${{{}}}", var),
            SecretRef::Gcp {
                project,
                secret,
                version,
            } => {
                if let Some(v) = version {
                    write!(f, "gsm://{}/{}/{}", project, secret, v)
                } else {
                    write!(f, "gsm://{}/{}", project, secret)
                }
            }
            SecretRef::File(path) => write!(f, "file://{}", path),
            SecretRef::Direct(_) => write!(f, "direct://[REDACTED]"),
        }
    }
}

/// Secret resolver with caching.
pub struct SecretResolver {
    /// Cached secrets.
    cache: HashMap<String, SecretValue>,

    /// Values loaded from .env file (checked before actual env vars).
    dotenv_values: HashMap<String, String>,

    /// GCP project for gsm:// refs without explicit project.
    default_gcp_project: Option<String>,

    /// GCP Secret Manager client (when gcp feature enabled).
    #[cfg(feature = "gcp")]
    gcp_client: Option<Arc<super::gcp::GcpSecretManager>>,
}

impl SecretResolver {
    /// Create a new resolver for development (loads .env file).
    pub fn from_env() -> Result<Self, SecretResolveError> {
        Self::from_env_file(".env")
    }

    /// Create a resolver loading a specific .env file.
    pub fn from_env_file(path: impl AsRef<Path>) -> Result<Self, SecretResolveError> {
        let path = path.as_ref();

        let dotenv_values = if path.exists() {
            info!(path = %path.display(), "Loading secrets from .env file");
            load_dotenv(path)?
        } else {
            debug!(path = %path.display(), ".env file not found, using environment only");
            HashMap::new()
        };

        Ok(Self {
            cache: HashMap::new(),
            dotenv_values,
            default_gcp_project: None,
            #[cfg(feature = "gcp")]
            gcp_client: None,
        })
    }

    /// Create a resolver for production with GCP Secret Manager.
    #[cfg(feature = "gcp")]
    pub async fn gcp(project: impl Into<String>) -> Result<Self, SecretResolveError> {
        let project = project.into();
        info!(project = %project, "Initializing GCP Secret Manager");

        let client = super::gcp::GcpSecretManager::new(&project)
            .await
            .map_err(|e| SecretResolveError::Backend(e.to_string()))?;

        Ok(Self {
            cache: HashMap::new(),
            dotenv_values: HashMap::new(),
            default_gcp_project: Some(project),
            gcp_client: Some(Arc::new(client)),
        })
    }

    /// Create an empty resolver (for testing).
    pub fn empty() -> Self {
        Self {
            cache: HashMap::new(),
            dotenv_values: HashMap::new(),
            default_gcp_project: None,
            #[cfg(feature = "gcp")]
            gcp_client: None,
        }
    }

    /// Pre-load a secret into the cache.
    pub fn preload(&mut self, key: &str, value: SecretValue) {
        self.cache.insert(key.to_string(), value);
    }

    /// Resolve a secret reference.
    pub async fn resolve(&self, reference: &str) -> Result<SecretValue, SecretResolveError> {
        // Check cache first
        if let Some(cached) = self.cache.get(reference) {
            debug!(reference = %reference, "Secret found in cache");
            return Ok(cached.clone());
        }

        let secret_ref = SecretRef::parse(reference)?;
        self.resolve_ref(&secret_ref).await
    }

    /// Resolve a parsed secret reference.
    pub async fn resolve_ref(
        &self,
        secret_ref: &SecretRef,
    ) -> Result<SecretValue, SecretResolveError> {
        match secret_ref {
            SecretRef::Env(var) => {
                debug!(var = %var, "Resolving environment variable");
                // Check .env values first, then actual environment
                if let Some(value) = self.dotenv_values.get(var) {
                    return Ok(SecretValue::new(value));
                }
                std::env::var(var)
                    .map(SecretValue::new)
                    .map_err(|_| SecretResolveError::EnvNotSet(var.clone()))
            }

            SecretRef::File(path) => {
                debug!(path = %path, "Resolving file secret");
                let contents = fs::read_to_string(path)?;
                Ok(SecretValue::new(contents.trim()))
            }

            SecretRef::Direct(value) => {
                warn!("Using direct secret value - not recommended for production");
                Ok(SecretValue::new(value))
            }

            #[cfg(feature = "gcp")]
            SecretRef::Gcp {
                project,
                secret,
                version,
            } => {
                debug!(project = %project, secret = %secret, "Resolving GCP secret");

                let client = self.gcp_client.as_ref().ok_or_else(|| {
                    SecretResolveError::Backend("GCP client not initialized".to_string())
                })?;

                let ver = version.as_deref().unwrap_or("latest");
                let value = client.get_secret(project, secret, ver).await.map_err(
                    |e: super::gcp::GcpSecretError| SecretResolveError::Backend(e.to_string()),
                )?;
                Ok(SecretValue::new(value))
            }

            #[cfg(not(feature = "gcp"))]
            SecretRef::Gcp { .. } => Err(SecretResolveError::Backend(
                "GCP support not enabled (compile with 'gcp' feature)".to_string(),
            )),
        }
    }

    /// Resolve a comma-separated list of secrets (for API keys).
    pub async fn resolve_list(
        &self,
        reference: &str,
    ) -> Result<Vec<SecretValue>, SecretResolveError> {
        let value = self.resolve(reference).await?;
        let keys: Vec<SecretValue> = value
            .expose()
            .split(',')
            .map(|s| SecretValue::new(s.trim()))
            .filter(|s| !s.is_empty())
            .collect();

        if keys.is_empty() {
            return Err(SecretResolveError::NotFound(
                "empty secret list".to_string(),
            ));
        }

        Ok(keys)
    }

    /// Check if a secret reference is valid (without resolving).
    pub fn validate_ref(reference: &str) -> Result<SecretRef, SecretResolveError> {
        SecretRef::parse(reference)
    }
}

impl fmt::Debug for SecretResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretResolver")
            .field("cached_secrets", &self.cache.len())
            .field("default_gcp_project", &self.default_gcp_project)
            .finish()
    }
}

/// Load environment variables from a .env file.
///
/// Returns a map of key-value pairs. Does not modify actual environment.
/// Actual env vars take precedence over .env values during resolution.
fn load_dotenv(path: &Path) -> Result<HashMap<String, String>, SecretResolveError> {
    let contents = fs::read_to_string(path)?;
    let mut values = HashMap::new();

    for line in contents.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse KEY=value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Remove quotes if present
            let value = value
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                .unwrap_or(value);

            // Only use if not already set in actual env (env vars take precedence)
            if std::env::var(key).is_err() {
                debug!(key = %key, "Loaded from .env");
                values.insert(key.to_string(), value.to_string());
            } else {
                debug!(key = %key, "Skipped (env var already set)");
            }
        }
    }

    info!(count = values.len(), "Loaded secrets from .env");
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_ref_parse_env_dollar() {
        let r = SecretRef::parse("${JWT_SECRET}").unwrap();
        assert_eq!(r, SecretRef::Env("JWT_SECRET".to_string()));
    }

    #[test]
    fn test_secret_ref_parse_env_explicit() {
        let r = SecretRef::parse("env://API_KEY").unwrap();
        assert_eq!(r, SecretRef::Env("API_KEY".to_string()));
    }

    #[test]
    fn test_secret_ref_parse_gsm() {
        let r = SecretRef::parse("gsm://my-project/jwt-secret").unwrap();
        assert_eq!(
            r,
            SecretRef::Gcp {
                project: "my-project".to_string(),
                secret: "jwt-secret".to_string(),
                version: None,
            }
        );
    }

    #[test]
    fn test_secret_ref_parse_gsm_with_version() {
        let r = SecretRef::parse("gsm://my-project/jwt-secret/5").unwrap();
        assert_eq!(
            r,
            SecretRef::Gcp {
                project: "my-project".to_string(),
                secret: "jwt-secret".to_string(),
                version: Some("5".to_string()),
            }
        );
    }

    #[test]
    fn test_secret_ref_parse_file() {
        let r = SecretRef::parse("file:///run/secrets/jwt").unwrap();
        assert_eq!(r, SecretRef::File("/run/secrets/jwt".to_string()));
    }

    #[test]
    fn test_secret_ref_parse_direct() {
        let r = SecretRef::parse("direct://test-value").unwrap();
        assert_eq!(r, SecretRef::Direct("test-value".to_string()));
    }

    #[test]
    fn test_secret_ref_parse_invalid() {
        assert!(SecretRef::parse("invalid").is_err());
        assert!(SecretRef::parse("${}").is_err());
        assert!(SecretRef::parse("env://").is_err());
        assert!(SecretRef::parse("gsm://project").is_err());
    }

    #[test]
    fn test_secret_ref_display() {
        assert_eq!(SecretRef::Env("JWT".to_string()).to_string(), "${JWT}");
        assert_eq!(
            SecretRef::Gcp {
                project: "p".to_string(),
                secret: "s".to_string(),
                version: None,
            }
            .to_string(),
            "gsm://p/s"
        );
        assert_eq!(
            SecretRef::Direct("secret".to_string()).to_string(),
            "direct://[REDACTED]"
        );
    }

    #[test]
    fn test_secret_value_debug_redacted() {
        let secret = SecretValue::new("super-secret-key");
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("super-secret-key"));
        assert!(debug.contains("REDACTED"));
        assert!(debug.contains("16")); // length
    }

    #[test]
    fn test_secret_value_display_redacted() {
        let secret = SecretValue::new("super-secret-key");
        let display = format!("{}", secret);
        assert!(!display.contains("super-secret-key"));
        assert!(display.contains("REDACTED"));
    }

    #[test]
    fn test_secret_value_expose() {
        let secret = SecretValue::new("my-secret");
        assert_eq!(secret.expose(), "my-secret");
    }

    #[tokio::test]
    async fn test_resolver_env_from_dotenv() {
        // Simulate loading from .env by creating resolver with dotenv_values
        let mut resolver = SecretResolver::empty();
        resolver
            .dotenv_values
            .insert("TEST_SECRET_123".to_string(), "test-value".to_string());

        let secret = resolver.resolve("${TEST_SECRET_123}").await.unwrap();
        assert_eq!(secret.expose(), "test-value");
    }

    #[tokio::test]
    async fn test_resolver_env_not_set() {
        let resolver = SecretResolver::empty();
        let result = resolver.resolve("${NONEXISTENT_VAR_XYZ}").await;

        assert!(matches!(result, Err(SecretResolveError::EnvNotSet(_))));
    }

    #[tokio::test]
    async fn test_resolver_direct() {
        let resolver = SecretResolver::empty();
        let secret = resolver.resolve("direct://test-secret").await.unwrap();

        assert_eq!(secret.expose(), "test-secret");
    }

    #[tokio::test]
    async fn test_resolver_list() {
        let mut resolver = SecretResolver::empty();
        resolver
            .dotenv_values
            .insert("TEST_API_KEYS".to_string(), "key1, key2, key3".to_string());

        let keys = resolver.resolve_list("${TEST_API_KEYS}").await.unwrap();

        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0].expose(), "key1");
        assert_eq!(keys[1].expose(), "key2");
        assert_eq!(keys[2].expose(), "key3");
    }

    #[test]
    fn test_secret_backend() {
        assert_eq!(
            SecretRef::Env("x".to_string()).backend(),
            SecretBackend::Env
        );
        assert_eq!(
            SecretRef::Gcp {
                project: "p".to_string(),
                secret: "s".to_string(),
                version: None
            }
            .backend(),
            SecretBackend::Gcp
        );
    }
}
