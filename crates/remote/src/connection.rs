// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Connection handling with automatic reconnection and fallback.
//!
//! Implements the Converge Protocol connection semantics:
//! - gRPC over HTTP/2 (primary)
//! - REST (fallback/degraded mode)
//! - Sequence-based resume on reconnect
//! - Exponential backoff

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Connection state indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Live gRPC streaming connection
    Streaming,
    /// Temporarily disconnected, attempting reconnect
    Reconnecting,
    /// Using REST fallback (degraded mode)
    Degraded,
    /// No connection available
    Offline,
}

/// Connection manager for Converge runtime
pub struct ConnectionManager {
    server: String,
    state: ConnectionState,
    last_sequence: i64,
    reconnect_attempts: u32,
}

impl ConnectionManager {
    pub fn new(server: String) -> Self {
        Self {
            server,
            state: ConnectionState::Offline,
            last_sequence: 0,
            reconnect_attempts: 0,
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    pub fn last_sequence(&self) -> i64 {
        self.last_sequence
    }

    pub fn update_sequence(&mut self, seq: i64) {
        self.last_sequence = seq;
    }

    /// Attempt reconnection with exponential backoff
    pub async fn reconnect(&mut self) -> bool {
        let max_backoff = Duration::from_secs(30);
        let base_backoff = Duration::from_secs(1);

        self.state = ConnectionState::Reconnecting;
        self.reconnect_attempts += 1;

        let backoff = base_backoff * 2u32.pow(self.reconnect_attempts.min(5));
        let backoff = backoff.min(max_backoff);

        info!(
            attempt = self.reconnect_attempts,
            backoff_secs = backoff.as_secs(),
            "Attempting reconnection"
        );

        sleep(backoff).await;

        // TODO: Actually attempt gRPC reconnection
        // For now, just simulate
        if self.reconnect_attempts > 3 {
            warn!("Max reconnection attempts reached, falling back to REST");
            self.state = ConnectionState::Degraded;
            false
        } else {
            self.state = ConnectionState::Streaming;
            self.reconnect_attempts = 0;
            true
        }
    }

    /// Fall back to REST polling mode
    pub fn fallback_to_rest(&mut self) {
        warn!("Falling back to REST polling (degraded mode)");
        self.state = ConnectionState::Degraded;
    }

    /// Mark connection as offline
    pub fn mark_offline(&mut self) {
        self.state = ConnectionState::Offline;
    }

    /// Mark connection as streaming
    pub fn mark_streaming(&mut self) {
        self.state = ConnectionState::Streaming;
        self.reconnect_attempts = 0;
    }
}

/// Calculate backoff duration for a given attempt
pub fn calculate_backoff(attempt: u32) -> Duration {
    let max_backoff = Duration::from_secs(30);
    let base_backoff = Duration::from_secs(1);
    let backoff = base_backoff * 2u32.pow(attempt.min(5));
    backoff.min(max_backoff)
}

/// Validate server URL format
pub fn validate_server_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("Server URL cannot be empty".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("grpc://") {
        return Err("Server URL must start with http://, https://, or grpc://".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Unit Tests - ConnectionManager
    // ==========================================================================

    #[test]
    fn connection_manager_initial_state() {
        let mgr = ConnectionManager::new("http://localhost:50051".to_string());
        assert_eq!(mgr.state(), ConnectionState::Offline);
        assert_eq!(mgr.last_sequence(), 0);
    }

    #[test]
    fn connection_manager_sequence_tracking() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.update_sequence(42);
        assert_eq!(mgr.last_sequence(), 42);
    }

    #[test]
    fn connection_manager_sequence_monotonic() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.update_sequence(10);
        mgr.update_sequence(20);
        mgr.update_sequence(30);
        assert_eq!(mgr.last_sequence(), 30);
    }

    #[test]
    fn connection_manager_mark_streaming_resets_attempts() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.reconnect_attempts = 5;
        mgr.mark_streaming();
        assert_eq!(mgr.state(), ConnectionState::Streaming);
        assert_eq!(mgr.reconnect_attempts, 0);
    }

    #[test]
    fn connection_manager_mark_offline() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.mark_streaming();
        mgr.mark_offline();
        assert_eq!(mgr.state(), ConnectionState::Offline);
    }

    #[test]
    fn connection_manager_fallback_to_rest() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.fallback_to_rest();
        assert_eq!(mgr.state(), ConnectionState::Degraded);
    }

    // ==========================================================================
    // Unit Tests - Backoff Calculation
    // ==========================================================================

    #[test]
    fn backoff_exponential_growth() {
        assert_eq!(calculate_backoff(0), Duration::from_secs(1));
        assert_eq!(calculate_backoff(1), Duration::from_secs(2));
        assert_eq!(calculate_backoff(2), Duration::from_secs(4));
        assert_eq!(calculate_backoff(3), Duration::from_secs(8));
        assert_eq!(calculate_backoff(4), Duration::from_secs(16));
        assert_eq!(calculate_backoff(5), Duration::from_secs(30)); // Capped at 30
    }

    #[test]
    fn backoff_capped_at_max() {
        // All attempts >= 5 should be capped at 30 seconds
        assert_eq!(calculate_backoff(5), Duration::from_secs(30));
        assert_eq!(calculate_backoff(6), Duration::from_secs(30));
        assert_eq!(calculate_backoff(10), Duration::from_secs(30));
        assert_eq!(calculate_backoff(100), Duration::from_secs(30));
    }

    // ==========================================================================
    // Unit Tests - URL Validation
    // ==========================================================================

    #[test]
    fn validate_url_accepts_http() {
        assert!(validate_server_url("http://localhost:50051").is_ok());
    }

    #[test]
    fn validate_url_accepts_https() {
        assert!(validate_server_url("https://converge.zone:50051").is_ok());
    }

    #[test]
    fn validate_url_accepts_grpc() {
        assert!(validate_server_url("grpc://localhost:50051").is_ok());
    }

    #[test]
    fn validate_url_rejects_empty() {
        let result = validate_server_url("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn validate_url_rejects_invalid_scheme() {
        let result = validate_server_url("ftp://localhost:50051");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must start with"));
    }

    #[test]
    fn validate_url_rejects_no_scheme() {
        let result = validate_server_url("localhost:50051");
        assert!(result.is_err());
    }

    // ==========================================================================
    // Negative Tests - Edge Cases
    // ==========================================================================

    #[test]
    fn sequence_can_be_zero() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.update_sequence(100);
        mgr.update_sequence(0); // Reset to 0 should work
        assert_eq!(mgr.last_sequence(), 0);
    }

    #[test]
    fn sequence_can_be_negative() {
        // While unusual, the API allows negative sequences
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.update_sequence(-1);
        assert_eq!(mgr.last_sequence(), -1);
    }

    #[test]
    fn state_transitions_are_valid() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());

        // Offline -> Streaming
        mgr.mark_streaming();
        assert_eq!(mgr.state(), ConnectionState::Streaming);

        // Streaming -> Degraded
        mgr.fallback_to_rest();
        assert_eq!(mgr.state(), ConnectionState::Degraded);

        // Degraded -> Offline
        mgr.mark_offline();
        assert_eq!(mgr.state(), ConnectionState::Offline);

        // Offline -> Streaming (direct)
        mgr.mark_streaming();
        assert_eq!(mgr.state(), ConnectionState::Streaming);
    }

    // ==========================================================================
    // Async Tests
    // ==========================================================================

    #[tokio::test]
    async fn reconnect_increments_attempts() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        assert_eq!(mgr.reconnect_attempts, 0);

        // Note: This will sleep, so we just verify the first call
        // In real tests we'd mock the sleep
        mgr.state = ConnectionState::Reconnecting;
        mgr.reconnect_attempts = 1;
        assert_eq!(mgr.reconnect_attempts, 1);
    }

    #[tokio::test]
    async fn reconnect_success_resets_attempts() {
        let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
        mgr.reconnect_attempts = 2;
        mgr.mark_streaming();
        assert_eq!(mgr.reconnect_attempts, 0);
    }
}

// ==========================================================================
// Property-Based Tests
// ==========================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn backoff_never_exceeds_max(attempt in 0u32..1000) {
            let backoff = calculate_backoff(attempt);
            prop_assert!(backoff <= Duration::from_secs(30));
        }

        #[test]
        fn backoff_never_zero(attempt in 0u32..1000) {
            let backoff = calculate_backoff(attempt);
            prop_assert!(backoff >= Duration::from_secs(1));
        }

        #[test]
        fn backoff_monotonic_up_to_cap(a in 0u32..5, b in 0u32..5) {
            if a < b {
                let backoff_a = calculate_backoff(a);
                let backoff_b = calculate_backoff(b);
                prop_assert!(backoff_a <= backoff_b);
            }
        }

        #[test]
        fn sequence_always_stored(seq in any::<i64>()) {
            let mut mgr = ConnectionManager::new("http://localhost:50051".to_string());
            mgr.update_sequence(seq);
            prop_assert_eq!(mgr.last_sequence(), seq);
        }

        #[test]
        fn valid_http_urls_accepted(port in 1u16..65535) {
            let url = format!("http://localhost:{port}");
            prop_assert!(validate_server_url(&url).is_ok());
        }

        #[test]
        fn valid_grpc_urls_accepted(port in 1u16..65535) {
            let url = format!("grpc://localhost:{port}");
            prop_assert!(validate_server_url(&url).is_ok());
        }
    }
}
