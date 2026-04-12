//! Field-level encryption using ChaCha20-Poly1305.
//!
//! Provides E2E encryption for sensitive data fields with:
//! - ChaCha20-Poly1305 AEAD cipher
//! - Key versioning for rotation support
//! - KeyStore abstraction for different key sources

mod field;
mod keystore;
mod memory;

pub use field::{EncryptedField, FieldCryptoError, FieldEncryptor, decrypt_field, encrypt_field};
pub use keystore::{KeyId, KeyMaterial, KeyStore, KeyStoreError};
pub use memory::MemoryKeyStore;

// Re-export for convenience
pub use chacha20poly1305;
