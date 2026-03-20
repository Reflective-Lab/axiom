//! Idempotency Key Generation
//!
//! Deterministic SHA-256 based key generation for alert deduplication.

use sha2::{Digest, Sha256};

/// Generate deterministic idempotency key from alert components
///
/// Uses SHA-256 hash of pipe-separated components:
/// `gate_id|check_id|severity|week|recipient`
///
/// Returns lowercase hex string (64 chars)
pub fn generate_idempotency_key(
    gate_id: &str,
    check_id: &str,
    severity: &str,
    week: &str,
    recipient: &str,
) -> String {
    let components = format!("{gate_id}|{check_id}|{severity}|{week}|{recipient}");

    let mut hasher = Sha256::new();
    hasher.update(components.as_bytes());
    let result = hasher.finalize();

    format!("{result:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_keys() {
        let key1 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        let key2 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        assert_eq!(key1, key2, "Same inputs must produce same key");
    }

    #[test]
    fn test_unique_keys_different_gate() {
        let key1 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        let key2 = generate_idempotency_key(
            "G_PROMOTE_02",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        assert_ne!(key1, key2, "Different gate_id must produce different key");
    }

    #[test]
    fn test_unique_keys_different_check() {
        let key1 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        let key2 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_002",
            "high",
            "2026-W04",
            "team@example.com",
        );
        assert_ne!(key1, key2, "Different check_id must produce different key");
    }

    #[test]
    fn test_unique_keys_different_severity() {
        let key1 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        let key2 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "critical",
            "2026-W04",
            "team@example.com",
        );
        assert_ne!(key1, key2, "Different severity must produce different key");
    }

    #[test]
    fn test_unique_keys_different_week() {
        let key1 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        let key2 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W05",
            "team@example.com",
        );
        assert_ne!(key1, key2, "Different week must produce different key");
    }

    #[test]
    fn test_unique_keys_different_recipient() {
        let key1 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );
        let key2 = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "oncall@example.com",
        );
        assert_ne!(key1, key2, "Different recipient must produce different key");
    }

    #[test]
    fn test_key_format() {
        let key = generate_idempotency_key(
            "G_PROMOTE_01",
            "check_001",
            "high",
            "2026-W04",
            "team@example.com",
        );

        // SHA-256 produces 64 hex characters
        assert_eq!(key.len(), 64, "Key must be 64 characters (SHA-256 hex)");

        // All characters should be lowercase hex
        assert!(
            key.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Key must be lowercase hex"
        );
    }
}
