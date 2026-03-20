//! Reliability Module
//!
//! Provides retry logic with exponential backoff and circuit breaker patterns
//! for resilient email delivery.
//!
//! This module provides:
//! - `RetryConfig` for configurable exponential backoff parameters
//! - `create_backoff()` to build `ExponentialBackoff` instances
//! - `CircuitBreakerConfig` for circuit breaker settings
//! - `create_circuit_breaker()` to build failsafe circuit breakers
//! - `ReliabilityError` for retry/circuit breaker error handling

use std::time::Duration;

use backoff::ExponentialBackoff;
use backoff::ExponentialBackoffBuilder;
use failsafe::backoff as cb_backoff;
use failsafe::failure_policy;
use failsafe::Config;
use failsafe::StateMachine;

/// Configuration for exponential backoff retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Initial retry interval (default: 1 second)
    pub initial_interval: Duration,
    /// Maximum retry interval (default: 60 seconds)
    pub max_interval: Duration,
    /// Randomization factor for jitter (default: 0.5 = 50%)
    pub randomization_factor: f64,
    /// Multiplier for interval growth (default: 2.0)
    pub multiplier: f64,
    /// Maximum total elapsed time for retries (default: 300 seconds = 5 minutes)
    pub max_elapsed_time: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            randomization_factor: 0.5,
            multiplier: 2.0,
            max_elapsed_time: Duration::from_secs(300),
        }
    }
}

/// Create an exponential backoff instance from configuration
///
/// Uses `ExponentialBackoffBuilder` to create a backoff with jitter,
/// interval growth, and time budget constraints.
///
/// # Arguments
///
/// * `config` - Retry configuration parameters
///
/// # Returns
///
/// An `ExponentialBackoff` instance ready for use with `backoff::retry`
pub fn create_backoff(config: &RetryConfig) -> ExponentialBackoff {
    ExponentialBackoffBuilder::new()
        .with_initial_interval(config.initial_interval)
        .with_max_interval(config.max_interval)
        .with_randomization_factor(config.randomization_factor)
        .with_multiplier(config.multiplier)
        .with_max_elapsed_time(Some(config.max_elapsed_time))
        .build()
}

/// Configuration for circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit (default: 3)
    pub consecutive_failures: u32,
    /// Initial backoff duration when circuit opens (default: 10 seconds)
    pub initial_backoff: Duration,
    /// Maximum backoff duration for half-open probes (default: 60 seconds)
    pub max_backoff: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            consecutive_failures: 3,
            initial_backoff: Duration::from_secs(10),
            max_backoff: Duration::from_secs(60),
        }
    }
}

/// Create a circuit breaker from configuration
///
/// Uses `failsafe` crate with consecutive failures policy and
/// exponential backoff for recovery timing.
///
/// # Arguments
///
/// * `config` - Circuit breaker configuration parameters
///
/// # Returns
///
/// A `StateMachine` implementing `CircuitBreaker` trait
pub fn create_circuit_breaker(
    config: &CircuitBreakerConfig,
) -> StateMachine<failure_policy::ConsecutiveFailures<cb_backoff::Exponential>, ()> {
    let backoff_strategy = cb_backoff::exponential(config.initial_backoff, config.max_backoff);
    let policy = failure_policy::consecutive_failures(config.consecutive_failures, backoff_strategy);

    Config::new().failure_policy(policy).build()
}

/// Errors related to reliability operations
#[derive(Debug, Clone)]
pub enum ReliabilityError {
    /// Transient error that may succeed on retry
    Transient(String),
    /// Permanent error that will not succeed on retry
    Permanent(String),
    /// Circuit breaker is open, rejecting calls
    CircuitOpen,
}

impl std::fmt::Display for ReliabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient(msg) => write!(f, "Transient error: {msg}"),
            Self::Permanent(msg) => write!(f, "Permanent error: {msg}"),
            Self::CircuitOpen => write!(f, "Circuit breaker is open"),
        }
    }
}

impl std::error::Error for ReliabilityError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_backoff_defaults() {
        let config = RetryConfig::default();
        let backoff = create_backoff(&config);

        // ExponentialBackoff doesn't expose all fields, but we can verify it was created
        // by checking it has a current_interval (starts at initial_interval)
        assert!(backoff.current_interval >= Duration::from_millis(500)); // With jitter, should be >= 500ms
        assert!(backoff.current_interval <= Duration::from_millis(1500)); // With jitter, should be <= 1.5s
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            initial_interval: Duration::from_secs(2),
            max_interval: Duration::from_secs(120),
            randomization_factor: 0.3,
            multiplier: 1.5,
            max_elapsed_time: Duration::from_secs(600),
        };

        let backoff = create_backoff(&config);

        // Verify backoff was created with custom config
        // current_interval with 0.3 jitter on 2s = 1.4s to 2.6s
        assert!(backoff.current_interval >= Duration::from_millis(1400));
        assert!(backoff.current_interval <= Duration::from_millis(2600));
    }

    #[test]
    fn test_circuit_breaker_config_defaults() {
        let config = CircuitBreakerConfig::default();
        let _cb = create_circuit_breaker(&config);

        // Circuit breaker created successfully (would panic if config invalid)
        // Can't easily inspect internal state, but creation success is verification
    }

    #[test]
    fn test_reliability_error_display() {
        let transient = ReliabilityError::Transient("network timeout".to_string());
        assert_eq!(format!("{transient}"), "Transient error: network timeout");

        let permanent = ReliabilityError::Permanent("invalid address".to_string());
        assert_eq!(format!("{permanent}"), "Permanent error: invalid address");

        let circuit_open = ReliabilityError::CircuitOpen;
        assert_eq!(format!("{circuit_open}"), "Circuit breaker is open");
    }
}
