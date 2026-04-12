//! GCP Secret Manager client.
//!
//! Provides authenticated access to secrets stored in GCP Secret Manager.

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, info};

/// GCP Secret Manager API base URL.
const SECRET_MANAGER_API: &str = "https://secretmanager.googleapis.com/v1";

/// Errors from GCP Secret Manager.
#[derive(Debug, Error)]
pub enum GcpSecretError {
    /// Secret not found.
    #[error("secret not found: {0}")]
    NotFound(String),

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Authentication error.
    #[error("authentication error: {0}")]
    Auth(String),

    /// API error.
    #[error("API error: {0}")]
    Api(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
}

/// GCP Secret Manager client.
pub struct GcpSecretManager {
    client: Client,
    default_project: String,
    access_token: Option<String>,
}

impl GcpSecretManager {
    /// Create a new client using Application Default Credentials.
    pub async fn new(default_project: &str) -> Result<Self, GcpSecretError> {
        info!(project = %default_project, "Initializing GCP Secret Manager client");

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Get access token from metadata service or ADC
        let access_token = Self::get_access_token(&client).await?;

        Ok(Self {
            client,
            default_project: default_project.to_string(),
            access_token: Some(access_token),
        })
    }

    /// Create a client with an explicit access token (for testing).
    pub fn with_token(default_project: &str, token: &str) -> Self {
        Self {
            client: Client::new(),
            default_project: default_project.to_string(),
            access_token: Some(token.to_string()),
        }
    }

    /// Get a secret value.
    ///
    /// # Arguments
    /// * `project` - GCP project ID
    /// * `secret` - Secret name
    /// * `version` - Version (default: "latest")
    pub async fn get_secret(
        &self,
        project: &str,
        secret: &str,
        version: &str,
    ) -> Result<String, GcpSecretError> {
        let url = format!(
            "{}/projects/{}/secrets/{}/versions/{}:access",
            SECRET_MANAGER_API, project, secret, version
        );

        debug!(project = %project, secret = %secret, version = %version, "Fetching secret");

        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| GcpSecretError::Auth("no access token available".to_string()))?;

        let response = self.client.get(&url).bearer_auth(token).send().await?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GcpSecretError::NotFound(format!(
                "{}/{}/{}",
                project, secret, version
            )));
        }

        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
            let body = response.text().await.unwrap_or_default();
            return Err(GcpSecretError::PermissionDenied(body));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GcpSecretError::Api(format!("HTTP {status}: {body}")));
        }

        let body: SecretAccessResponse = response.json().await?;

        // Decode base64 payload
        let decoded = base64_decode(&body.payload.data)?;

        debug!(
            project = %project,
            secret = %secret,
            bytes = decoded.len(),
            "Secret retrieved successfully"
        );

        Ok(decoded)
    }

    /// Get a secret from the default project.
    pub async fn get(&self, secret: &str) -> Result<String, GcpSecretError> {
        self.get_secret(&self.default_project, secret, "latest")
            .await
    }

    /// Get access token from GCP metadata service or ADC.
    async fn get_access_token(client: &Client) -> Result<String, GcpSecretError> {
        // Try metadata service first (works on GCE, Cloud Run, GKE)
        if let Ok(token) = Self::get_token_from_metadata(client).await {
            debug!("Got access token from metadata service");
            return Ok(token);
        }

        // Try Application Default Credentials file
        if let Ok(token) = Self::get_token_from_adc().await {
            debug!("Got access token from ADC");
            return Ok(token);
        }

        Err(GcpSecretError::Auth(
            "Could not obtain access token. Set GOOGLE_APPLICATION_CREDENTIALS or run on GCP."
                .to_string(),
        ))
    }

    /// Get token from GCP metadata service.
    async fn get_token_from_metadata(client: &Client) -> Result<String, GcpSecretError> {
        let url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

        let response = client
            .get(url)
            .header("Metadata-Flavor", "Google")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .map_err(|e| GcpSecretError::Auth(e.to_string()))?;

        if !response.status().is_success() {
            return Err(GcpSecretError::Auth(
                "metadata service unavailable".to_string(),
            ));
        }

        let body: MetadataTokenResponse = response
            .json()
            .await
            .map_err(|e| GcpSecretError::Auth(e.to_string()))?;

        Ok(body.access_token)
    }

    /// Get token from Application Default Credentials.
    async fn get_token_from_adc() -> Result<String, GcpSecretError> {
        // Check for GOOGLE_APPLICATION_CREDENTIALS
        let creds_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").map_err(|_| {
            GcpSecretError::Auth("GOOGLE_APPLICATION_CREDENTIALS not set".to_string())
        })?;

        // Read and parse credentials file
        let creds_content = std::fs::read_to_string(&creds_path)
            .map_err(|e| GcpSecretError::Auth(format!("Failed to read credentials: {e}")))?;

        let creds: ServiceAccountCredentials = serde_json::from_str(&creds_content)
            .map_err(|e| GcpSecretError::Auth(format!("Invalid credentials format: {e}")))?;

        // For service account credentials, we need to exchange for access token
        // This is a simplified version - production should use proper JWT signing
        if creds.creds_type == "service_account" {
            // In a real implementation, we'd sign a JWT and exchange it
            // For now, return an error indicating we need the metadata service
            return Err(GcpSecretError::Auth(
                "Service account key file found but JWT signing not implemented. \
                 Use workload identity or run on GCP."
                    .to_string(),
            ));
        }

        Err(GcpSecretError::Auth(
            "Unsupported credential type".to_string(),
        ))
    }
}

/// Response from Secret Manager access API.
#[derive(Debug, Deserialize)]
struct SecretAccessResponse {
    payload: SecretPayload,
}

#[derive(Debug, Deserialize)]
struct SecretPayload {
    data: String, // base64 encoded
}

/// Response from metadata service.
#[derive(Debug, Deserialize)]
struct MetadataTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: u64,
    #[allow(dead_code)]
    token_type: String,
}

/// Service account credentials file format.
#[derive(Debug, Deserialize)]
struct ServiceAccountCredentials {
    #[serde(rename = "type")]
    creds_type: String,
    #[allow(dead_code)]
    project_id: Option<String>,
    #[allow(dead_code)]
    private_key_id: Option<String>,
    #[allow(dead_code)]
    private_key: Option<String>,
    #[allow(dead_code)]
    client_email: Option<String>,
}

/// Decode base64 string to UTF-8.
fn base64_decode(encoded: &str) -> Result<String, GcpSecretError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| GcpSecretError::Api(format!("Invalid base64: {e}")))?;

    String::from_utf8(bytes).map_err(|e| GcpSecretError::Api(format!("Invalid UTF-8: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode() {
        // "hello" in base64 is "aGVsbG8="
        let result = base64_decode("aGVsbG8=").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_base64_decode_invalid() {
        let result = base64_decode("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_client_with_token() {
        let client = GcpSecretManager::with_token("my-project", "fake-token");
        assert_eq!(client.default_project, "my-project");
        assert!(client.access_token.is_some());
    }
}
