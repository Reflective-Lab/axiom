//! Key storage abstraction for encryption keys.
//!
//! Provides a trait for retrieving encryption keys from various sources:
//! - In-memory (testing)
//! - Cloud KMS (production)
//! - File-based (development)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors that can occur with key storage.
#[derive(Debug, Error)]
pub enum KeyStoreError {
    /// Key not found.
    #[error("key not found: {0}")]
    NotFound(String),

    /// Key has been revoked.
    #[error("key revoked: {0}")]
    Revoked(String),

    /// Key has expired.
    #[error("key expired: {0}")]
    Expired(String),

    /// Failed to fetch key from remote.
    #[error("fetch error: {0}")]
    Fetch(String),

    /// Invalid key format.
    #[error("invalid key: {0}")]
    Invalid(String),

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Key identifier with version support.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId {
    /// Key name/alias.
    pub name: String,

    /// Key version (for rotation support).
    pub version: u32,
}

impl KeyId {
    /// Create a new key ID.
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }

    /// Create a key ID with version 1 (latest).
    pub fn latest(name: impl Into<String>) -> Self {
        Self::new(name, 0) // 0 means "latest"
    }

    /// Parse from string format "name:version" or "name" (latest).
    pub fn parse(s: &str) -> Result<Self, KeyStoreError> {
        if let Some((name, ver)) = s.rsplit_once(':') {
            let version = ver
                .parse()
                .map_err(|_| KeyStoreError::Invalid(format!("invalid version: {}", ver)))?;
            Ok(Self::new(name, version))
        } else {
            Ok(Self::latest(s))
        }
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.version == 0 {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}:{}", self.name, self.version)
        }
    }
}

/// Key material for encryption/decryption.
#[derive(Clone)]
pub struct KeyMaterial {
    /// The actual key bytes (32 bytes for ChaCha20-Poly1305).
    key: Vec<u8>,

    /// Key identifier.
    pub id: KeyId,

    /// Whether this key can be used for encryption (vs decrypt-only).
    pub can_encrypt: bool,
}

impl KeyMaterial {
    /// Create new key material.
    pub fn new(id: KeyId, key: Vec<u8>) -> Result<Self, KeyStoreError> {
        if key.len() != 32 {
            return Err(KeyStoreError::Invalid(format!(
                "key must be 32 bytes, got {}",
                key.len()
            )));
        }
        Ok(Self {
            key,
            id,
            can_encrypt: true,
        })
    }

    /// Create key material for decryption only (rotated key).
    pub fn decrypt_only(id: KeyId, key: Vec<u8>) -> Result<Self, KeyStoreError> {
        let mut km = Self::new(id, key)?;
        km.can_encrypt = false;
        Ok(km)
    }

    /// Get the key bytes.
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    /// Get key as fixed-size array.
    pub fn key_array(&self) -> [u8; 32] {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&self.key);
        arr
    }
}

impl fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyMaterial")
            .field("id", &self.id)
            .field("can_encrypt", &self.can_encrypt)
            .field("key", &"[REDACTED]")
            .finish()
    }
}

/// Trait for key storage backends.
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Get the current (latest) encryption key for a key name.
    async fn get_current_key(&self, name: &str) -> Result<KeyMaterial, KeyStoreError>;

    /// Get a specific key version (for decryption).
    async fn get_key(&self, id: &KeyId) -> Result<KeyMaterial, KeyStoreError>;

    /// List available key names.
    async fn list_keys(&self) -> Result<Vec<String>, KeyStoreError>;

    /// Check if a key exists.
    async fn key_exists(&self, name: &str) -> Result<bool, KeyStoreError> {
        match self.get_current_key(name).await {
            Ok(_) => Ok(true),
            Err(KeyStoreError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_id_new() {
        let id = KeyId::new("my-key", 5);
        assert_eq!(id.name, "my-key");
        assert_eq!(id.version, 5);
    }

    #[test]
    fn test_key_id_latest() {
        let id = KeyId::latest("my-key");
        assert_eq!(id.name, "my-key");
        assert_eq!(id.version, 0);
    }

    #[test]
    fn test_key_id_parse_with_version() {
        let id = KeyId::parse("encryption-key:3").unwrap();
        assert_eq!(id.name, "encryption-key");
        assert_eq!(id.version, 3);
    }

    #[test]
    fn test_key_id_parse_latest() {
        let id = KeyId::parse("encryption-key").unwrap();
        assert_eq!(id.name, "encryption-key");
        assert_eq!(id.version, 0);
    }

    #[test]
    fn test_key_id_parse_invalid_version() {
        let result = KeyId::parse("key:abc");
        assert!(matches!(result, Err(KeyStoreError::Invalid(_))));
    }

    #[test]
    fn test_key_id_display() {
        assert_eq!(KeyId::new("key", 5).to_string(), "key:5");
        assert_eq!(KeyId::latest("key").to_string(), "key");
    }

    #[test]
    fn test_key_material_valid() {
        let key = vec![0u8; 32];
        let id = KeyId::new("test", 1);
        let km = KeyMaterial::new(id, key).unwrap();
        assert!(km.can_encrypt);
        assert_eq!(km.key().len(), 32);
    }

    #[test]
    fn test_key_material_invalid_length() {
        let key = vec![0u8; 16]; // Wrong size
        let id = KeyId::new("test", 1);
        let result = KeyMaterial::new(id, key);
        assert!(matches!(result, Err(KeyStoreError::Invalid(_))));
    }

    #[test]
    fn test_key_material_decrypt_only() {
        let key = vec![0u8; 32];
        let id = KeyId::new("test", 1);
        let km = KeyMaterial::decrypt_only(id, key).unwrap();
        assert!(!km.can_encrypt);
    }

    #[test]
    fn test_key_material_debug_redacts_key() {
        let key = vec![42u8; 32];
        let id = KeyId::new("test", 1);
        let km = KeyMaterial::new(id, key).unwrap();
        let debug = format!("{:?}", km);
        assert!(debug.contains("REDACTED"));
        assert!(!debug.contains("42"));
    }

    #[test]
    fn test_key_material_array() {
        let key = (0..32).collect::<Vec<u8>>();
        let id = KeyId::new("test", 1);
        let km = KeyMaterial::new(id, key.clone()).unwrap();
        let arr = km.key_array();
        assert_eq!(arr.len(), 32);
        assert_eq!(&arr[..], &key[..]);
    }
}
