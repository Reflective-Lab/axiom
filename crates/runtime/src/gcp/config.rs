//! GCP configuration
//!
//! Supports local development with emulators:
//! - Set `LOCAL_DEV=true` to enable local development mode
//! - Firestore: Set `FIRESTORE_EMULATOR_HOST=localhost:8080`
//! - Service Directory: Automatically mocked in local mode

use serde::Deserialize;

/// GCP configuration loaded from environment or config file
#[derive(Debug, Clone, Deserialize)]
pub struct GcpConfig {
    /// GCP project ID
    #[serde(default = "default_project")]
    pub project_id: String,

    /// Firestore database ID (usually "(default)")
    #[serde(default = "default_database")]
    pub firestore_database: String,

    /// Service Directory namespace
    #[serde(default = "default_namespace")]
    pub service_directory_namespace: String,

    /// Service Directory region
    #[serde(default = "default_region")]
    pub service_directory_region: String,

    /// Current service name for registration
    #[serde(default = "default_service_name")]
    pub service_name: String,

    /// Current service version
    #[serde(default = "default_version")]
    pub service_version: String,

    /// Local development mode (skips real GCP calls where no emulator exists)
    #[serde(default)]
    pub local_dev: bool,

    /// Firestore emulator host (e.g., "localhost:8080")
    #[serde(default)]
    pub firestore_emulator_host: Option<String>,
}

impl Default for GcpConfig {
    fn default() -> Self {
        Self {
            project_id: default_project(),
            firestore_database: default_database(),
            service_directory_namespace: default_namespace(),
            service_directory_region: default_region(),
            service_name: default_service_name(),
            service_version: default_version(),
            local_dev: false,
            firestore_emulator_host: None,
        }
    }
}

impl GcpConfig {
    /// Load configuration from environment variables
    ///
    /// Environment variables:
    /// - `LOCAL_DEV=true` - Enable local development mode
    /// - `FIRESTORE_EMULATOR_HOST=localhost:8080` - Use Firestore emulator
    /// - `GCP_PROJECT_ID` or `GOOGLE_CLOUD_PROJECT` - GCP project ID
    /// - `K_SERVICE` or `SERVICE_NAME` - Service name (auto-detected on Cloud Run)
    pub fn from_env() -> Self {
        let firestore_emulator_host = std::env::var("FIRESTORE_EMULATOR_HOST").ok();
        let local_dev = std::env::var("LOCAL_DEV")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
            || firestore_emulator_host.is_some();

        Self {
            project_id: std::env::var("GCP_PROJECT_ID")
                .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT"))
                .unwrap_or_else(|_| {
                    if local_dev {
                        "local-project".to_string()
                    } else {
                        default_project()
                    }
                }),
            firestore_database: std::env::var("FIRESTORE_DATABASE")
                .unwrap_or_else(|_| default_database()),
            service_directory_namespace: std::env::var("SERVICE_DIRECTORY_NAMESPACE")
                .unwrap_or_else(|_| default_namespace()),
            service_directory_region: std::env::var("SERVICE_DIRECTORY_REGION")
                .or_else(|_| std::env::var("CLOUD_RUN_REGION"))
                .unwrap_or_else(|_| default_region()),
            service_name: std::env::var("K_SERVICE")
                .or_else(|_| std::env::var("SERVICE_NAME"))
                .unwrap_or_else(|_| default_service_name()),
            service_version: std::env::var("K_REVISION")
                .or_else(|_| std::env::var("SERVICE_VERSION"))
                .unwrap_or_else(|_| default_version()),
            local_dev,
            firestore_emulator_host,
        }
    }

    /// Check if running in local development mode
    pub fn is_local(&self) -> bool {
        self.local_dev
    }

    /// Check if Firestore emulator is configured
    pub fn has_firestore_emulator(&self) -> bool {
        self.firestore_emulator_host.is_some()
    }
}

fn default_project() -> String {
    "hey-sh-production".to_string()
}

fn default_database() -> String {
    "(default)".to_string()
}

fn default_namespace() -> String {
    "converge".to_string()
}

fn default_region() -> String {
    "europe-west1".to_string()
}

fn default_service_name() -> String {
    "converge-runtime".to_string()
}

fn default_version() -> String {
    "v1".to_string()
}
