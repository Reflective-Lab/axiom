//! Metrics Module
//!
//! Provides metrics instrumentation facade for delivery reliability monitoring.
//!
//! This module uses the `metrics` crate facade pattern, which allows metrics
//! registration and recording without committing to a specific exporter.
//! Metrics have zero overhead when no exporter is configured.
//!
//! Email Metrics:
//! - `email_delivery_total` (counter): Total delivery attempts by status
//! - `email_delivery_retries_total` (counter): Total retry attempts
//! - `email_delivery_retry_count` (histogram): Distribution of retries per delivery
//! - `email_delivery_dlq_total` (counter): Messages moved to dead letter queue
//! - `email_delivery_dlq_depth` (gauge): Current DLQ depth
//! - `email_delivery_circuit_breaker_state` (gauge): Circuit breaker state
//!
//! Slack Metrics:
//! - `slack_delivery_total` (counter): Total Slack delivery attempts by status
//! - `slack_delivery_retries_total` (counter): Total Slack retry attempts
//! - `slack_delivery_retry_count` (histogram): Distribution of retries per Slack delivery
//! - `slack_delivery_dlq_total` (counter): Slack messages moved to dead letter queue
//! - `slack_delivery_circuit_breaker_state` (gauge): Slack circuit breaker state

use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};

/// Register all delivery metrics with descriptions
///
/// Call this once during application startup to register metric descriptions.
/// This makes metrics visible in exporters even before they're recorded.
pub fn register_delivery_metrics() {
    describe_counter!(
        "email_delivery_total",
        "Total email delivery attempts with status label (success, transient, permanent, etc)"
    );

    describe_counter!(
        "email_delivery_retries_total",
        "Total number of retry attempts across all deliveries"
    );

    describe_histogram!(
        "email_delivery_retry_count",
        "Distribution of retry attempts per delivery (0 = success on first try)"
    );

    describe_counter!(
        "email_delivery_dlq_total",
        "Total messages moved to dead letter queue"
    );

    describe_gauge!(
        "email_delivery_dlq_depth",
        "Current number of unprocessed messages in dead letter queue"
    );

    describe_gauge!(
        "email_delivery_circuit_breaker_state",
        "Circuit breaker state (0=closed, 1=half-open, 2=open)"
    );

    // Slack delivery metrics
    describe_counter!(
        "slack_delivery_total",
        "Total Slack delivery attempts with status label (success, transient, permanent, rate_limited)"
    );

    describe_counter!(
        "slack_delivery_retries_total",
        "Total number of retry attempts for Slack deliveries"
    );

    describe_histogram!(
        "slack_delivery_retry_count",
        "Distribution of retry attempts per Slack delivery"
    );

    describe_counter!(
        "slack_delivery_dlq_total",
        "Total Slack messages moved to dead letter queue"
    );

    describe_gauge!(
        "slack_delivery_circuit_breaker_state",
        "Slack circuit breaker state (0=closed, 1=half-open, 2=open)"
    );
}

/// Record successful delivery
pub fn record_delivery_success() {
    counter!("email_delivery_total", "status" => "success").increment(1);
}

/// Record failed delivery with status label
///
/// # Arguments
///
/// * `status` - Error category (e.g., "transient", "permanent", "timeout", "auth", "tls")
pub fn record_delivery_failure(status: &str) {
    counter!("email_delivery_total", "status" => status.to_string()).increment(1);
}

/// Record a retry attempt
pub fn record_retry_attempt() {
    counter!("email_delivery_retries_total").increment(1);
}

/// Record the total number of retries for a delivery attempt
///
/// # Arguments
///
/// * `count` - Number of retry attempts (0 = success on first try)
pub fn record_retry_count(count: u32) {
    histogram!("email_delivery_retry_count").record(f64::from(count));
}

/// Record a message moved to dead letter queue
pub fn record_dlq_entry() {
    counter!("email_delivery_dlq_total").increment(1);
}

/// Set the current dead letter queue depth
///
/// # Arguments
///
/// * `count` - Current number of messages in DLQ
#[allow(clippy::cast_precision_loss)]
pub fn set_dlq_depth(count: u64) {
    gauge!("email_delivery_dlq_depth").set(count as f64);
}

/// Set the circuit breaker state
///
/// # Arguments
///
/// * `state` - Circuit breaker state: 0=closed, 1=half-open, 2=open
pub fn set_circuit_breaker_state(state: u8) {
    gauge!("email_delivery_circuit_breaker_state").set(f64::from(state));
}

// =============================================================================
// Slack Delivery Metrics
// =============================================================================

/// Record successful Slack delivery
pub fn record_slack_delivery_success() {
    counter!("slack_delivery_total", "status" => "success").increment(1);
}

/// Record failed Slack delivery with status label
///
/// # Arguments
///
/// * `status` - Error category (e.g., transient, permanent, rate\_limited)
pub fn record_slack_delivery_failure(status: &str) {
    counter!("slack_delivery_total", "status" => status.to_string()).increment(1);
}

/// Record a Slack retry attempt
pub fn record_slack_retry_attempt() {
    counter!("slack_delivery_retries_total").increment(1);
}

/// Record the total number of retries for a Slack delivery attempt
///
/// # Arguments
///
/// * `count` - Number of retry attempts (0 = success on first try)
pub fn record_slack_retry_count(count: u32) {
    histogram!("slack_delivery_retry_count").record(f64::from(count));
}

/// Record a Slack message moved to dead letter queue
pub fn record_slack_dlq_entry() {
    counter!("slack_delivery_dlq_total").increment(1);
}

/// Set the Slack circuit breaker state
///
/// # Arguments
///
/// * `state` - Circuit breaker state: 0=closed, 1=half-open, 2=open
pub fn set_slack_circuit_breaker_state(state: u8) {
    gauge!("slack_delivery_circuit_breaker_state").set(f64::from(state));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_metrics() {
        // Should not panic - metrics facade accepts registration even without exporter
        register_delivery_metrics();
    }

    #[test]
    fn test_record_functions() {
        // All record functions should not panic without exporter configured
        // Metrics facade provides zero-overhead no-op when no exporter

        record_delivery_success();
        record_delivery_failure("transient");
        record_delivery_failure("permanent");
        record_delivery_failure("timeout");
        record_retry_attempt();
        record_retry_count(0);
        record_retry_count(3);
        record_retry_count(10);
        record_dlq_entry();
        set_dlq_depth(0);
        set_dlq_depth(100);
        set_circuit_breaker_state(0); // closed
        set_circuit_breaker_state(1); // half-open
        set_circuit_breaker_state(2); // open
    }

    #[test]
    fn test_slack_record_functions() {
        // All Slack record functions should not panic without exporter configured
        // Metrics facade provides zero-overhead no-op when no exporter

        record_slack_delivery_success();
        record_slack_delivery_failure("transient");
        record_slack_delivery_failure("permanent");
        record_slack_delivery_failure("rate_limited");
        record_slack_retry_attempt();
        record_slack_retry_count(0);
        record_slack_retry_count(3);
        record_slack_retry_count(10);
        record_slack_dlq_entry();
        set_slack_circuit_breaker_state(0); // closed
        set_slack_circuit_breaker_state(1); // half-open
        set_slack_circuit_breaker_state(2); // open
    }
}
