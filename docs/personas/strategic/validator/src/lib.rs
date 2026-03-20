//! Strategy Validator Library
//!
//! Core library providing validation and drift detection functionality.

pub mod ack;
pub mod audit;
pub mod delivery;
pub mod detector;
pub mod digest;
pub mod fingerprint;
pub mod idempotency;
pub mod metrics;
pub mod policy_engine;
pub mod reliability;
pub mod routing;
pub mod routing_config;
pub mod sla;
pub mod slack;
pub mod thresholds;

// Re-export key types
pub use detector::{DetectorError, DetectorExitCode};
pub use fingerprint::{ArtifactFingerprint, fingerprint_repo};
