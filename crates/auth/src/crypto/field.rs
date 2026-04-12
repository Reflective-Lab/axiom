//! Field-level encryption using ChaCha20-Poly1305.
//!
//! Provides encryption/decryption for individual data fields with:
//! - Authenticated encryption (AEAD)
//! - Key versioning for rotation
//! - Associated data (AAD) support for context binding

use super::keystore::{KeyId, KeyMaterial, KeyStore, KeyStoreError};
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during field encryption/decryption.
#[derive(Debug, Error)]
pub enum FieldCryptoError {
    /// Key store error.
    #[error("key store error: {0}")]
    KeyStore(#[from] KeyStoreError),

    /// Encryption failed.
    #[error("encryption failed: {0}")]
    Encryption(String),

    /// Decryption failed.
    #[error("decryption failed: {0}")]
    Decryption(String),

    /// Invalid ciphertext format.
    #[error("invalid ciphertext: {0}")]
    InvalidCiphertext(String),

    /// Key cannot be used for this operation.
    #[error("key not allowed for encryption")]
    KeyNotAllowed,

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Encrypted field with metadata for decryption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    /// Key identifier used for encryption.
    pub key_id: KeyId,

    /// Nonce (12 bytes for ChaCha20-Poly1305).
    #[serde(with = "base64_bytes")]
    pub nonce: Vec<u8>,

    /// Ciphertext with authentication tag.
    #[serde(with = "base64_bytes")]
    pub ciphertext: Vec<u8>,

    /// Optional associated data that was authenticated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aad: Option<String>,
}

impl EncryptedField {
    /// Create a new encrypted field.
    pub fn new(key_id: KeyId, nonce: Vec<u8>, ciphertext: Vec<u8>) -> Self {
        Self {
            key_id,
            nonce,
            ciphertext,
            aad: None,
        }
    }

    /// Set associated authenticated data.
    #[must_use]
    pub fn with_aad(mut self, aad: impl Into<String>) -> Self {
        self.aad = Some(aad.into());
        self
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, FieldCryptoError> {
        serde_json::to_string(self).map_err(|e| FieldCryptoError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, FieldCryptoError> {
        serde_json::from_str(json).map_err(|e| FieldCryptoError::Serialization(e.to_string()))
    }

    /// Serialize to bytes (for Protobuf fields).
    pub fn to_bytes(&self) -> Result<Vec<u8>, FieldCryptoError> {
        serde_json::to_vec(self).map_err(|e| FieldCryptoError::Serialization(e.to_string()))
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FieldCryptoError> {
        serde_json::from_slice(bytes).map_err(|e| FieldCryptoError::Serialization(e.to_string()))
    }
}

/// Field encryptor with key store integration.
pub struct FieldEncryptor<K: KeyStore> {
    key_store: Arc<K>,
    default_key_name: String,
}

impl<K: KeyStore> FieldEncryptor<K> {
    /// Create a new field encryptor.
    pub fn new(key_store: Arc<K>, default_key_name: impl Into<String>) -> Self {
        Self {
            key_store,
            default_key_name: default_key_name.into(),
        }
    }

    /// Encrypt data using the default key.
    pub async fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedField, FieldCryptoError> {
        self.encrypt_with_key(&self.default_key_name, plaintext, None)
            .await
    }

    /// Encrypt data with associated data.
    pub async fn encrypt_with_aad(
        &self,
        plaintext: &[u8],
        aad: &str,
    ) -> Result<EncryptedField, FieldCryptoError> {
        self.encrypt_with_key(&self.default_key_name, plaintext, Some(aad))
            .await
    }

    /// Encrypt data using a specific key.
    pub async fn encrypt_with_key(
        &self,
        key_name: &str,
        plaintext: &[u8],
        aad: Option<&str>,
    ) -> Result<EncryptedField, FieldCryptoError> {
        let key_material = self.key_store.get_current_key(key_name).await?;

        if !key_material.can_encrypt {
            return Err(FieldCryptoError::KeyNotAllowed);
        }

        encrypt_with_material(&key_material, plaintext, aad)
    }

    /// Decrypt an encrypted field.
    pub async fn decrypt(&self, encrypted: &EncryptedField) -> Result<Vec<u8>, FieldCryptoError> {
        let key_material = self.key_store.get_key(&encrypted.key_id).await?;
        decrypt_with_material(&key_material, encrypted)
    }

    /// Decrypt and deserialize JSON.
    pub async fn decrypt_json<T: for<'de> Deserialize<'de>>(
        &self,
        encrypted: &EncryptedField,
    ) -> Result<T, FieldCryptoError> {
        let plaintext = self.decrypt(encrypted).await?;
        serde_json::from_slice(&plaintext)
            .map_err(|e| FieldCryptoError::Serialization(e.to_string()))
    }

    /// Encrypt and serialize as JSON.
    pub async fn encrypt_json<T: Serialize>(
        &self,
        value: &T,
    ) -> Result<EncryptedField, FieldCryptoError> {
        let plaintext = serde_json::to_vec(value)
            .map_err(|e| FieldCryptoError::Serialization(e.to_string()))?;
        self.encrypt(&plaintext).await
    }
}

impl<K: KeyStore> Clone for FieldEncryptor<K> {
    fn clone(&self) -> Self {
        Self {
            key_store: self.key_store.clone(),
            default_key_name: self.default_key_name.clone(),
        }
    }
}

/// Encrypt data with the given key material.
pub fn encrypt_with_material(
    key: &KeyMaterial,
    plaintext: &[u8],
    aad: Option<&str>,
) -> Result<EncryptedField, FieldCryptoError> {
    let cipher = ChaCha20Poly1305::new_from_slice(key.key())
        .map_err(|e| FieldCryptoError::Encryption(e.to_string()))?;

    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = if let Some(aad_str) = aad {
        cipher
            .encrypt(
                &nonce,
                chacha20poly1305::aead::Payload {
                    msg: plaintext,
                    aad: aad_str.as_bytes(),
                },
            )
            .map_err(|e| FieldCryptoError::Encryption(e.to_string()))?
    } else {
        cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| FieldCryptoError::Encryption(e.to_string()))?
    };

    let mut field = EncryptedField::new(key.id.clone(), nonce.to_vec(), ciphertext);
    if let Some(aad_str) = aad {
        field = field.with_aad(aad_str);
    }

    Ok(field)
}

/// Decrypt an encrypted field with the given key material.
pub fn decrypt_with_material(
    key: &KeyMaterial,
    encrypted: &EncryptedField,
) -> Result<Vec<u8>, FieldCryptoError> {
    if encrypted.nonce.len() != 12 {
        return Err(FieldCryptoError::InvalidCiphertext(format!(
            "invalid nonce length: expected 12, got {}",
            encrypted.nonce.len()
        )));
    }

    let cipher = ChaCha20Poly1305::new_from_slice(key.key())
        .map_err(|e| FieldCryptoError::Decryption(e.to_string()))?;

    let nonce = Nonce::from_slice(&encrypted.nonce);

    let plaintext = if let Some(ref aad) = encrypted.aad {
        cipher
            .decrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: &encrypted.ciphertext,
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| FieldCryptoError::Decryption("authentication failed".to_string()))?
    } else {
        cipher
            .decrypt(nonce, encrypted.ciphertext.as_slice())
            .map_err(|_| FieldCryptoError::Decryption("authentication failed".to_string()))?
    };

    Ok(plaintext)
}

/// Convenience function to encrypt a field.
pub async fn encrypt_field<K: KeyStore>(
    key_store: &K,
    key_name: &str,
    plaintext: &[u8],
) -> Result<EncryptedField, FieldCryptoError> {
    let key_material = key_store.get_current_key(key_name).await?;
    encrypt_with_material(&key_material, plaintext, None)
}

/// Convenience function to decrypt a field.
pub async fn decrypt_field<K: KeyStore>(
    key_store: &K,
    encrypted: &EncryptedField,
) -> Result<Vec<u8>, FieldCryptoError> {
    let key_material = key_store.get_key(&encrypted.key_id).await?;
    decrypt_with_material(&key_material, encrypted)
}

/// Base64 serialization for bytes.
mod base64_bytes {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        STANDARD.decode(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::MemoryKeyStore;

    fn setup_keystore() -> Arc<MemoryKeyStore> {
        Arc::new(MemoryKeyStore::with_default_key())
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        let plaintext = b"Hello, World!";
        let encrypted = encryptor.encrypt(plaintext).await.unwrap();
        let decrypted = encryptor.decrypt(&encrypted).await.unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_with_aad() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        let plaintext = b"Sensitive data";
        let aad = "user:123";

        let encrypted = encryptor.encrypt_with_aad(plaintext, aad).await.unwrap();
        assert_eq!(encrypted.aad, Some(aad.to_string()));

        let decrypted = encryptor.decrypt(&encrypted).await.unwrap();
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_aad_mismatch_fails() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        let plaintext = b"Sensitive data";
        let mut encrypted = encryptor
            .encrypt_with_aad(plaintext, "correct-aad")
            .await
            .unwrap();

        // Tamper with AAD
        encrypted.aad = Some("wrong-aad".to_string());

        let result = encryptor.decrypt(&encrypted).await;
        assert!(matches!(result, Err(FieldCryptoError::Decryption(_))));
    }

    #[tokio::test]
    async fn test_encrypt_json() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Secret {
            api_key: String,
            count: u32,
        }

        let original = Secret {
            api_key: "sk_test_123".to_string(),
            count: 42,
        };

        let encrypted = encryptor.encrypt_json(&original).await.unwrap();
        let decrypted: Secret = encryptor.decrypt_json(&encrypted).await.unwrap();

        assert_eq!(original, decrypted);
    }

    #[tokio::test]
    async fn test_encrypted_field_serialization() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        let encrypted = encryptor.encrypt(b"test data").await.unwrap();

        // JSON roundtrip
        let json = encrypted.to_json().unwrap();
        let restored = EncryptedField::from_json(&json).unwrap();

        assert_eq!(encrypted.key_id, restored.key_id);
        assert_eq!(encrypted.nonce, restored.nonce);
        assert_eq!(encrypted.ciphertext, restored.ciphertext);
    }

    #[tokio::test]
    async fn test_encrypted_field_bytes() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        let encrypted = encryptor.encrypt(b"test data").await.unwrap();

        let bytes = encrypted.to_bytes().unwrap();
        let restored = EncryptedField::from_bytes(&bytes).unwrap();

        assert_eq!(encrypted.key_id, restored.key_id);
    }

    #[tokio::test]
    async fn test_key_version_in_ciphertext() {
        let store = Arc::new(MemoryKeyStore::new());
        store.add_key("rotate", 1, vec![1u8; 32]).unwrap();

        let encryptor = FieldEncryptor::new(store.clone(), "rotate");
        let encrypted_v1 = encryptor.encrypt(b"data").await.unwrap();

        assert_eq!(encrypted_v1.key_id.version, 1);

        // Add new version
        store.add_key("rotate", 2, vec![2u8; 32]).unwrap();

        let encrypted_v2 = encryptor.encrypt(b"data").await.unwrap();
        assert_eq!(encrypted_v2.key_id.version, 2);

        // Can still decrypt v1
        let decrypted = encryptor.decrypt(&encrypted_v1).await.unwrap();
        assert_eq!(decrypted, b"data");
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        let store = MemoryKeyStore::with_default_key();

        let plaintext = b"Quick encrypt/decrypt";
        let encrypted = encrypt_field(&store, "default", plaintext).await.unwrap();
        let decrypted = decrypt_field(&store, &encrypted).await.unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_tampered_ciphertext_fails() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        let mut encrypted = encryptor.encrypt(b"sensitive").await.unwrap();

        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        let result = encryptor.decrypt(&encrypted).await;
        assert!(matches!(result, Err(FieldCryptoError::Decryption(_))));
    }

    #[tokio::test]
    async fn test_key_not_found() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "nonexistent");

        let result = encryptor.encrypt(b"data").await;
        assert!(matches!(result, Err(FieldCryptoError::KeyStore(_))));
    }

    #[test]
    fn test_invalid_nonce_length() {
        let encrypted = EncryptedField {
            key_id: KeyId::new("test", 1),
            nonce: vec![0u8; 5], // Wrong length
            ciphertext: vec![0u8; 32],
            aad: None,
        };

        let key = KeyMaterial::new(KeyId::new("test", 1), vec![0u8; 32]).unwrap();
        let result = decrypt_with_material(&key, &encrypted);

        assert!(matches!(
            result,
            Err(FieldCryptoError::InvalidCiphertext(_))
        ));
    }

    #[tokio::test]
    async fn test_different_plaintexts_different_ciphertexts() {
        let store = setup_keystore();
        let encryptor = FieldEncryptor::new(store, "default");

        // Same plaintext encrypted twice should have different ciphertext (due to random nonce)
        let e1 = encryptor.encrypt(b"same data").await.unwrap();
        let e2 = encryptor.encrypt(b"same data").await.unwrap();

        assert_ne!(e1.nonce, e2.nonce);
        assert_ne!(e1.ciphertext, e2.ciphertext);

        // But both decrypt to the same thing
        let d1 = encryptor.decrypt(&e1).await.unwrap();
        let d2 = encryptor.decrypt(&e2).await.unwrap();
        assert_eq!(d1, d2);
    }
}
