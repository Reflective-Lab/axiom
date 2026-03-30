// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Newtype ID wrappers for type safety.
//!
//! All ID types use `#[serde(transparent)]` for clean serialization
//! and provide `new()`, accessor methods, and `Display` implementations.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// FactId - Unique identifier for a promoted Fact
// ============================================================================

/// Unique identifier for a Fact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FactId(String);

impl FactId {
    /// Create a new FactId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for FactId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for FactId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// ObservationId - Unique identifier for raw observations
// ============================================================================

/// Unique identifier for an Observation (raw provider output).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObservationId(String);

impl ObservationId {
    /// Create a new ObservationId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObservationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ObservationId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ObservationId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// ProposalId - Unique identifier for proposals in any state
// ============================================================================

/// Unique identifier for a Proposal (in any lifecycle state).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProposalId(String);

impl ProposalId {
    /// Create a new ProposalId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProposalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ProposalId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ProposalId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// GateId - Unique identifier for promotion gates
// ============================================================================

/// Unique identifier for a promotion gate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GateId(String);

impl GateId {
    /// Create a new GateId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for GateId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for GateId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// ApprovalId - Unique identifier for human approvals
// ============================================================================

/// Unique identifier for a human approval.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ApprovalId(String);

impl ApprovalId {
    /// Create a new ApprovalId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ApprovalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ApprovalId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ApprovalId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// ArtifactId - Unique identifier for derived artifacts
// ============================================================================

/// Unique identifier for a derived artifact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArtifactId(String);

impl ArtifactId {
    /// Create a new ArtifactId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ArtifactId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ArtifactId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// ContentHash - Wraps [u8; 32] with hex Display
// ============================================================================

/// Content-addressable hash (32 bytes, displayed as hex).
///
/// Used for referencing raw payloads, policy versions, etc.
/// Follows the existing Fingerprint trait pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentHash(#[serde(with = "hex_bytes")] [u8; 32]);

impl ContentHash {
    /// Create a new ContentHash from raw bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create from a hex string.
    ///
    /// # Panics
    /// Panics if the hex string is not exactly 64 characters or contains invalid hex.
    pub fn from_hex(hex: &str) -> Self {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex, &mut bytes).expect("invalid hex string");
        Self(bytes)
    }

    /// Get the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get as hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create a zero hash (useful for stubs/tests).
    pub fn zero() -> Self {
        Self([0u8; 32])
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for ContentHash {
    fn default() -> Self {
        Self::zero()
    }
}

/// Serde helper for hex encoding [u8; 32].
mod hex_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(&s, &mut bytes).map_err(serde::de::Error::custom)?;
        Ok(bytes)
    }
}

// ============================================================================
// Timestamp - ISO-8601 timestamp string (per RESEARCH.md recommendation)
// ============================================================================

/// ISO-8601 timestamp string.
///
/// Per RESEARCH.md: Use String for timestamps to avoid external time dependencies.
/// Format: "2024-01-15T10:30:00Z"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(String);

impl Timestamp {
    /// Create a new Timestamp from an ISO-8601 string.
    pub fn new(iso: impl Into<String>) -> Self {
        Self(iso.into())
    }

    /// Get the timestamp as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a timestamp for "now" using std::time.
    ///
    /// Note: This uses basic formatting without external crates.
    /// For production, consider chrono or time crate.
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();

        // Basic UTC formatting (approximation without full calendar math)
        // For accurate formatting, use chrono in production
        Self(format!("{}Z", secs))
    }

    /// Create an epoch timestamp (Unix epoch start).
    pub fn epoch() -> Self {
        Self::new("1970-01-01T00:00:00Z")
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Timestamp {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Timestamp {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fact_id_display() {
        let id = FactId::new("fact-123");
        assert_eq!(id.to_string(), "fact-123");
        assert_eq!(id.as_str(), "fact-123");
    }

    #[test]
    fn observation_id_from_string() {
        let id: ObservationId = "obs-456".into();
        assert_eq!(id.as_str(), "obs-456");
    }

    #[test]
    fn content_hash_hex_roundtrip() {
        let hash = ContentHash::from_hex(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        assert_eq!(
            hash.to_hex(),
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    fn content_hash_display() {
        let hash = ContentHash::zero();
        assert_eq!(
            hash.to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn timestamp_epoch() {
        let ts = Timestamp::epoch();
        assert_eq!(ts.as_str(), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn id_serde_transparent() {
        let id = FactId::new("test-id");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""test-id""#); // Not {"0":"test-id"}
    }
}
