//! In-memory key store for testing and development.

use super::keystore::{KeyId, KeyMaterial, KeyStore, KeyStoreError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory key store for testing.
///
/// Stores keys in memory with version support. Not suitable for production.
pub struct MemoryKeyStore {
    /// Keys indexed by name, then version.
    keys: RwLock<HashMap<String, HashMap<u32, Vec<u8>>>>,

    /// Current version for each key name.
    current_versions: RwLock<HashMap<String, u32>>,
}

impl MemoryKeyStore {
    /// Create a new empty key store.
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            current_versions: RwLock::new(HashMap::new()),
        }
    }

    /// Create a key store with a default encryption key.
    pub fn with_default_key() -> Self {
        let store = Self::new();
        // Generate a deterministic key for testing
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        store.add_key("default", 1, key.to_vec()).unwrap();
        store
    }

    /// Add a key to the store.
    pub fn add_key(&self, name: &str, version: u32, key: Vec<u8>) -> Result<(), KeyStoreError> {
        if key.len() != 32 {
            return Err(KeyStoreError::Invalid(format!(
                "key must be 32 bytes, got {}",
                key.len()
            )));
        }

        let mut keys = self.keys.write().unwrap();
        let mut versions = self.current_versions.write().unwrap();

        let key_versions = keys.entry(name.to_string()).or_default();
        key_versions.insert(version, key);

        // Update current version if this is newer
        let current = versions.entry(name.to_string()).or_insert(0);
        if version > *current {
            *current = version;
        }

        Ok(())
    }

    /// Generate and add a random key.
    pub fn generate_key(&self, name: &str) -> Result<u32, KeyStoreError> {
        use chacha20poly1305::ChaCha20Poly1305;
        use chacha20poly1305::KeyInit;
        use chacha20poly1305::aead::OsRng;

        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let version = {
            let versions = self.current_versions.read().unwrap();
            versions.get(name).copied().unwrap_or(0) + 1
        };

        self.add_key(name, version, key.to_vec())?;
        Ok(version)
    }

    /// Remove a specific key version.
    pub fn remove_key(&self, name: &str, version: u32) -> Result<(), KeyStoreError> {
        let mut keys = self.keys.write().unwrap();

        if let Some(versions) = keys.get_mut(name) {
            if versions.remove(&version).is_none() {
                return Err(KeyStoreError::NotFound(format!("{}:{}", name, version)));
            }
        } else {
            return Err(KeyStoreError::NotFound(name.to_string()));
        }

        Ok(())
    }
}

impl Default for MemoryKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyStore for MemoryKeyStore {
    async fn get_current_key(&self, name: &str) -> Result<KeyMaterial, KeyStoreError> {
        let versions = self.current_versions.read().unwrap();
        let current_version = versions
            .get(name)
            .ok_or_else(|| KeyStoreError::NotFound(name.to_string()))?;

        let keys = self.keys.read().unwrap();
        let key_versions = keys
            .get(name)
            .ok_or_else(|| KeyStoreError::NotFound(name.to_string()))?;

        let key = key_versions
            .get(current_version)
            .ok_or_else(|| KeyStoreError::NotFound(format!("{}:{}", name, current_version)))?;

        KeyMaterial::new(KeyId::new(name, *current_version), key.clone())
    }

    async fn get_key(&self, id: &KeyId) -> Result<KeyMaterial, KeyStoreError> {
        // If version is 0, get current
        if id.version == 0 {
            return self.get_current_key(&id.name).await;
        }

        let keys = self.keys.read().unwrap();
        let key_versions = keys
            .get(&id.name)
            .ok_or_else(|| KeyStoreError::NotFound(id.name.clone()))?;

        let key = key_versions
            .get(&id.version)
            .ok_or_else(|| KeyStoreError::NotFound(id.to_string()))?;

        // Check if this is the current version
        let versions = self.current_versions.read().unwrap();
        let is_current = versions.get(&id.name) == Some(&id.version);

        if is_current {
            KeyMaterial::new(id.clone(), key.clone())
        } else {
            // Old version - decrypt only
            KeyMaterial::decrypt_only(id.clone(), key.clone())
        }
    }

    async fn list_keys(&self) -> Result<Vec<String>, KeyStoreError> {
        let keys = self.keys.read().unwrap();
        Ok(keys.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_keystore_new() {
        let store = MemoryKeyStore::new();
        let keys = store.list_keys().await.unwrap();
        assert!(keys.is_empty());
    }

    #[tokio::test]
    async fn test_memory_keystore_with_default() {
        let store = MemoryKeyStore::with_default_key();
        let key = store.get_current_key("default").await.unwrap();
        assert_eq!(key.id.name, "default");
        assert_eq!(key.id.version, 1);
        assert!(key.can_encrypt);
    }

    #[tokio::test]
    async fn test_memory_keystore_add_key() {
        let store = MemoryKeyStore::new();
        let key = vec![0u8; 32];

        store.add_key("test-key", 1, key).unwrap();

        let retrieved = store.get_current_key("test-key").await.unwrap();
        assert_eq!(retrieved.id.name, "test-key");
        assert_eq!(retrieved.id.version, 1);
    }

    #[tokio::test]
    async fn test_memory_keystore_multiple_versions() {
        let store = MemoryKeyStore::new();

        store.add_key("key", 1, vec![1u8; 32]).unwrap();
        store.add_key("key", 2, vec![2u8; 32]).unwrap();

        // Current should be version 2
        let current = store.get_current_key("key").await.unwrap();
        assert_eq!(current.id.version, 2);
        assert!(current.can_encrypt);

        // Can still get version 1 (decrypt only)
        let old = store.get_key(&KeyId::new("key", 1)).await.unwrap();
        assert_eq!(old.id.version, 1);
        assert!(!old.can_encrypt);
    }

    #[tokio::test]
    async fn test_memory_keystore_generate_key() {
        let store = MemoryKeyStore::new();

        let v1 = store.generate_key("generated").unwrap();
        assert_eq!(v1, 1);

        let v2 = store.generate_key("generated").unwrap();
        assert_eq!(v2, 2);

        let current = store.get_current_key("generated").await.unwrap();
        assert_eq!(current.id.version, 2);
    }

    #[tokio::test]
    async fn test_memory_keystore_not_found() {
        let store = MemoryKeyStore::new();
        let result = store.get_current_key("nonexistent").await;
        assert!(matches!(result, Err(KeyStoreError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_memory_keystore_list_keys() {
        let store = MemoryKeyStore::new();
        store.add_key("key1", 1, vec![0u8; 32]).unwrap();
        store.add_key("key2", 1, vec![0u8; 32]).unwrap();

        let mut keys = store.list_keys().await.unwrap();
        keys.sort();

        assert_eq!(keys, vec!["key1", "key2"]);
    }

    #[tokio::test]
    async fn test_memory_keystore_key_exists() {
        let store = MemoryKeyStore::new();
        store.add_key("exists", 1, vec![0u8; 32]).unwrap();

        assert!(store.key_exists("exists").await.unwrap());
        assert!(!store.key_exists("not-exists").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_keystore_remove_key() {
        let store = MemoryKeyStore::new();
        store.add_key("key", 1, vec![0u8; 32]).unwrap();
        store.add_key("key", 2, vec![0u8; 32]).unwrap();

        // Remove version 1
        store.remove_key("key", 1).unwrap();

        // Version 1 should be gone
        let result = store.get_key(&KeyId::new("key", 1)).await;
        assert!(matches!(result, Err(KeyStoreError::NotFound(_))));

        // Version 2 still exists
        let key = store.get_key(&KeyId::new("key", 2)).await.unwrap();
        assert_eq!(key.id.version, 2);
    }

    #[test]
    fn test_memory_keystore_invalid_key_size() {
        let store = MemoryKeyStore::new();
        let result = store.add_key("bad", 1, vec![0u8; 16]);
        assert!(matches!(result, Err(KeyStoreError::Invalid(_))));
    }

    #[tokio::test]
    async fn test_memory_keystore_get_key_version_zero() {
        let store = MemoryKeyStore::new();
        store.add_key("key", 5, vec![0u8; 32]).unwrap();

        // Version 0 means "latest"
        let key = store.get_key(&KeyId::new("key", 0)).await.unwrap();
        assert_eq!(key.id.version, 5);
    }
}
