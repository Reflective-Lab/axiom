//! Service Directory integration
//!
//! Provides service registration and discovery using GCP Service Directory.
//! This enables gRPC clients to discover services by name.
//!
//! In local development mode (`LOCAL_DEV=true`), Service Directory operations
//! are mocked and return success without making real API calls.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use super::GcpConfig;

/// Service Directory error types
#[derive(Error, Debug)]
pub enum ServiceDirectoryError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Service not found: {0}")]
    NotFound(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint name (e.g., "primary")
    pub name: String,

    /// IP address (optional for Cloud Run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Port number (optional for Cloud Run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Metadata key-value pairs
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,

    /// Metadata key-value pairs
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,

    /// Endpoints
    #[serde(default)]
    pub endpoints: Vec<ServiceEndpoint>,
}

/// Service Directory client
pub struct ServiceDirectory {
    config: GcpConfig,
    client: Client,
    base_url: String,
}

impl ServiceDirectory {
    /// Create a new Service Directory client
    pub fn new(config: GcpConfig) -> Self {
        let base_url = format!(
            "https://servicedirectory.googleapis.com/v1/projects/{}/locations/{}/namespaces/{}",
            config.project_id, config.service_directory_region, config.service_directory_namespace
        );

        Self {
            config,
            client: Client::new(),
            base_url,
        }
    }

    /// Get access token using Application Default Credentials
    async fn get_access_token(&self) -> Result<String, ServiceDirectoryError> {
        // On Cloud Run, use the metadata server
        if std::env::var("K_SERVICE").is_ok() {
            let response = self.client
                .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
                .header("Metadata-Flavor", "Google")
                .send()
                .await?;

            if response.status().is_success() {
                #[derive(Deserialize)]
                struct TokenResponse {
                    access_token: String,
                }
                let token: TokenResponse = response.json().await?;
                return Ok(token.access_token);
            }
        }

        // Locally, use gcloud auth
        let output = std::process::Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
            .map_err(|e| ServiceDirectoryError::Auth(e.to_string()))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(ServiceDirectoryError::Auth(
                "Failed to get access token. Run 'gcloud auth application-default login'"
                    .to_string(),
            ))
        }
    }

    /// Register this service with Service Directory
    ///
    /// Call this on startup to register the service endpoint.
    /// In local development mode, this is a no-op.
    pub async fn register(&self, endpoint_url: &str) -> Result<(), ServiceDirectoryError> {
        // Skip in local development mode
        if self.config.is_local() {
            debug!(
                service = %self.config.service_name,
                endpoint = %endpoint_url,
                "Skipping Service Directory registration (local dev mode)"
            );
            return Ok(());
        }

        let token = self.get_access_token().await?;

        // Create or update the endpoint
        let endpoint = ServiceEndpoint {
            name: "primary".to_string(),
            address: None, // Cloud Run doesn't use IP addresses
            port: None,
            metadata: [
                ("url".to_string(), endpoint_url.to_string()),
                ("version".to_string(), self.config.service_version.clone()),
                ("protocol".to_string(), "grpc".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        let url = format!(
            "{}/services/{}/endpoints/primary",
            self.base_url, self.config.service_name
        );

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&token)
            .json(&endpoint)
            .query(&[("updateMask", "metadata")])
            .send()
            .await?;

        if response.status().is_success() {
            info!(
                service = %self.config.service_name,
                endpoint = %endpoint_url,
                "Registered with Service Directory"
            );
            Ok(())
        } else if response.status().as_u16() == 404 {
            // Endpoint doesn't exist, create it
            self.create_endpoint(&token, &endpoint).await
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ServiceDirectoryError::Api { status, message })
        }
    }

    /// Create a new endpoint
    async fn create_endpoint(
        &self,
        token: &str,
        endpoint: &ServiceEndpoint,
    ) -> Result<(), ServiceDirectoryError> {
        let url = format!(
            "{}/services/{}/endpoints",
            self.base_url, self.config.service_name
        );

        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(endpoint)
            .query(&[("endpointId", &endpoint.name)])
            .send()
            .await?;

        if response.status().is_success() {
            info!(
                service = %self.config.service_name,
                endpoint = %endpoint.name,
                "Created endpoint in Service Directory"
            );
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ServiceDirectoryError::Api { status, message })
        }
    }

    /// Deregister this service from Service Directory
    ///
    /// Call this on shutdown (optional - endpoints can be left for health checks).
    /// In local development mode, this is a no-op.
    pub async fn deregister(&self) -> Result<(), ServiceDirectoryError> {
        // Skip in local development mode
        if self.config.is_local() {
            debug!(
                service = %self.config.service_name,
                "Skipping Service Directory deregistration (local dev mode)"
            );
            return Ok(());
        }

        let token = self.get_access_token().await?;

        let url = format!(
            "{}/services/{}/endpoints/primary",
            self.base_url, self.config.service_name
        );

        let response = self.client.delete(&url).bearer_auth(&token).send().await?;

        if response.status().is_success() || response.status().as_u16() == 404 {
            info!(service = %self.config.service_name, "Deregistered from Service Directory");
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ServiceDirectoryError::Api { status, message })
        }
    }

    /// Resolve a service by name
    ///
    /// Returns the service information including endpoints.
    /// In local development mode, returns localhost endpoints.
    pub async fn resolve(&self, service_name: &str) -> Result<ServiceInfo, ServiceDirectoryError> {
        // Return mock data in local development mode
        if self.config.is_local() {
            debug!(
                service = %service_name,
                "Returning mock service info (local dev mode)"
            );
            return Ok(ServiceInfo {
                name: service_name.to_string(),
                metadata: [
                    ("version".to_string(), "local".to_string()),
                    ("environment".to_string(), "development".to_string()),
                ]
                .into_iter()
                .collect(),
                endpoints: vec![ServiceEndpoint {
                    name: "primary".to_string(),
                    address: Some("127.0.0.1".to_string()),
                    port: Some(8080),
                    metadata: [("url".to_string(), "http://localhost:8080".to_string())]
                        .into_iter()
                        .collect(),
                }],
            });
        }

        let token = self.get_access_token().await?;

        let url = format!("{}/services/{}", self.base_url, service_name);

        let response = self.client.get(&url).bearer_auth(&token).send().await?;

        if response.status().is_success() {
            let info: ServiceInfo = response.json().await?;
            Ok(info)
        } else if response.status().as_u16() == 404 {
            Err(ServiceDirectoryError::NotFound(service_name.to_string()))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ServiceDirectoryError::Api { status, message })
        }
    }

    /// List all services in the namespace
    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>, ServiceDirectoryError> {
        let token = self.get_access_token().await?;

        let url = format!("{}/services", self.base_url);

        let response = self.client.get(&url).bearer_auth(&token).send().await?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct ListResponse {
                services: Option<Vec<ServiceInfo>>,
            }
            let list: ListResponse = response.json().await?;
            Ok(list.services.unwrap_or_default())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ServiceDirectoryError::Api { status, message })
        }
    }
}

/// Register service on startup and deregister on shutdown
pub async fn register_on_startup(config: &GcpConfig, service_url: &str) {
    if config.is_local() {
        info!("Running in local development mode - skipping Service Directory registration");
        return;
    }

    let sd = ServiceDirectory::new(config.clone());

    match sd.register(service_url).await {
        Ok(()) => info!("Service registered with Service Directory"),
        Err(e) => warn!("Failed to register with Service Directory: {}", e),
    }
}
