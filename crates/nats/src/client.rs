//! NATS client with mTLS support.

use super::NatsError;
use crate::identity::Identity;
use async_nats::{Client, ConnectOptions};
use tracing::{debug, info};

/// Configuration for NATS client.
#[derive(Debug, Clone)]
pub struct NatsClientConfig {
    /// NATS server addresses.
    pub servers: Vec<String>,

    /// Connection name (for monitoring).
    pub name: Option<String>,

    /// Whether to require TLS (should always be true in production).
    pub require_tls: bool,
}

impl Default for NatsClientConfig {
    fn default() -> Self {
        Self {
            servers: vec!["nats://localhost:4222".to_string()],
            name: None,
            require_tls: true,
        }
    }
}

impl NatsClientConfig {
    /// Create a new NATS client config with the given servers.
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            ..Default::default()
        }
    }

    /// Set the connection name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set whether TLS is required.
    #[must_use]
    pub fn with_require_tls(mut self, require: bool) -> Self {
        self.require_tls = require;
        self
    }
}

/// NATS client wrapper with mTLS support.
pub struct NatsClient {
    client: Client,
    service_id: String,
}

impl NatsClient {
    /// Connect to NATS with mTLS using the provided identity.
    pub async fn connect(
        config: NatsClientConfig,
        identity: &dyn Identity,
    ) -> Result<Self, NatsError> {
        info!(
            servers = ?config.servers,
            service_id = %identity.service_id(),
            require_tls = config.require_tls,
            "Connecting to NATS"
        );

        let tls_config = identity.client_config()?;

        let mut options = ConnectOptions::new();

        if let Some(name) = &config.name {
            options = options.name(name);
        }

        if config.require_tls {
            // Convert Arc<rustls::ClientConfig> to the format async-nats expects
            options = options
                .require_tls(true)
                .tls_client_config((*tls_config).clone());
        }

        let servers: Vec<async_nats::ServerAddr> = config
            .servers
            .iter()
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| NatsError::Connection(format!("invalid server address: {e}")))?;

        let client = options
            .connect(&servers[..])
            .await
            .map_err(|e| NatsError::Connection(e.to_string()))?;

        info!(
            service_id = %identity.service_id(),
            "Connected to NATS successfully"
        );

        Ok(Self {
            client,
            service_id: identity.service_id().to_string(),
        })
    }

    /// Get the underlying NATS client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the service ID used for this connection.
    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    /// Publish a message to a subject.
    pub async fn publish(
        &self,
        subject: impl Into<String>,
        payload: impl AsRef<[u8]>,
    ) -> Result<(), NatsError> {
        let subject = subject.into();
        debug!(subject = %subject, "Publishing message");

        self.client
            .publish(subject, payload.as_ref().to_vec().into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))
    }

    /// Get a JetStream context for durable messaging.
    pub fn jetstream(&self) -> async_nats::jetstream::Context {
        async_nats::jetstream::new(self.client.clone())
    }

    /// Flush pending messages.
    pub async fn flush(&self) -> Result<(), NatsError> {
        self.client
            .flush()
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))
    }
}

impl Clone for NatsClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            service_id: self.service_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_config_default() {
        let config = NatsClientConfig::default();
        assert!(config.require_tls);
        assert_eq!(config.servers.len(), 1);
    }

    #[test]
    fn test_nats_config_builder() {
        let config = NatsClientConfig::new(vec!["nats://server1:4222".into()])
            .with_name("test-client")
            .with_require_tls(false);

        assert_eq!(config.name, Some("test-client".to_string()));
        assert!(!config.require_tls);
    }
}
