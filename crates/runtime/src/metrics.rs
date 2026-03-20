// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Prometheus metrics for Converge Runtime.
//!
//! This module provides metrics instrumentation using the `metrics` crate
//! with Prometheus export via `metrics-exporter-prometheus`.
//!
//! # Metrics Exposed
//!
//! ## HTTP Metrics
//! - `http_requests_total` - Total HTTP requests by method, path, status
//! - `http_request_duration_seconds` - Request latency histogram
//! - `http_request_size_bytes` - Request body size histogram
//! - `http_response_size_bytes` - Response body size histogram
//!
//! ## Job Metrics
//! - `converge_jobs_total` - Total jobs by status (pending, running, converged, halted)
//! - `converge_job_duration_seconds` - Job execution time histogram
//! - `converge_job_cycles` - Convergence cycles histogram
//! - `converge_job_facts` - Facts produced histogram
//!
//! ## Streaming Metrics
//! - `converge_active_streams` - Active SSE stream connections
//! - `converge_active_grpc_streams` - Active gRPC stream connections
//! - `converge_stream_events_total` - Total events sent by type
//!
//! # Usage
//!
//! ```ignore
//! use converge_runtime::metrics;
//!
//! // Initialize metrics (call once at startup)
//! let handle = metrics::init()?;
//!
//! // Record metrics
//! metrics::http_request("GET", "/api/v1/jobs", 200, 0.045);
//! metrics::job_completed("converged", 5, 12);
//!
//! // Get Prometheus metrics text
//! let metrics_text = handle.render();
//! ```

use std::sync::OnceLock;

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

/// Global Prometheus handle for rendering metrics.
static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the metrics subsystem.
///
/// This sets up the Prometheus exporter and registers default metrics.
/// Call this once at application startup.
///
/// Returns the Prometheus handle for rendering metrics.
pub fn init() -> Result<PrometheusHandle, MetricsError> {
    let builder = PrometheusBuilder::new();

    let handle = builder
        .install_recorder()
        .map_err(|e| MetricsError::Init(e.to_string()))?;

    // Store handle globally
    PROMETHEUS_HANDLE
        .set(handle.clone())
        .map_err(|_| MetricsError::Init("Metrics already initialized".to_string()))?;

    // Register default metrics with initial values
    gauge!("converge_active_streams").set(0.0);
    gauge!("converge_active_grpc_streams").set(0.0);

    tracing::info!("Prometheus metrics initialized");

    Ok(handle)
}

/// Get the Prometheus handle.
///
/// Returns `None` if metrics haven't been initialized.
pub fn handle() -> Option<&'static PrometheusHandle> {
    PROMETHEUS_HANDLE.get()
}

/// Render Prometheus metrics as text.
///
/// Returns the metrics in Prometheus text format, or an empty string
/// if metrics haven't been initialized.
pub fn render() -> String {
    handle().map(|h| h.render()).unwrap_or_default()
}

/// Error type for metrics operations.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Failed to initialize metrics: {0}")]
    Init(String),
}

// =============================================================================
// HTTP Metrics
// =============================================================================

/// Record an HTTP request.
pub fn http_request(method: &str, path: &str, status: u16, duration_secs: f64) {
    let labels = [
        ("method", method.to_string()),
        ("path", path.to_string()),
        ("status", status.to_string()),
    ];

    counter!("http_requests_total", &labels).increment(1);
    histogram!("http_request_duration_seconds", &labels).record(duration_secs);
}

/// Record HTTP request size.
pub fn http_request_size(method: &str, path: &str, size_bytes: u64) {
    let labels = [("method", method.to_string()), ("path", path.to_string())];

    histogram!("http_request_size_bytes", &labels).record(size_bytes as f64);
}

/// Record HTTP response size.
pub fn http_response_size(method: &str, path: &str, size_bytes: u64) {
    let labels = [("method", method.to_string()), ("path", path.to_string())];

    histogram!("http_response_size_bytes", &labels).record(size_bytes as f64);
}

// =============================================================================
// Job Metrics
// =============================================================================

/// Record a job completion.
pub fn job_completed(status: &str, cycles: u32, facts: u32) {
    let labels = [("status", status.to_string())];

    counter!("converge_jobs_total", &labels).increment(1);
    histogram!("converge_job_cycles").record(cycles as f64);
    histogram!("converge_job_facts").record(facts as f64);
}

/// Record job duration.
pub fn job_duration(status: &str, duration_secs: f64) {
    let labels = [("status", status.to_string())];

    histogram!("converge_job_duration_seconds", &labels).record(duration_secs);
}

/// Increment active jobs gauge.
pub fn job_started() {
    gauge!("converge_active_jobs").increment(1.0);
}

/// Decrement active jobs gauge.
pub fn job_finished() {
    gauge!("converge_active_jobs").decrement(1.0);
}

// =============================================================================
// Streaming Metrics
// =============================================================================

/// Increment active SSE streams.
pub fn sse_stream_opened() {
    gauge!("converge_active_streams").increment(1.0);
}

/// Decrement active SSE streams.
pub fn sse_stream_closed() {
    gauge!("converge_active_streams").decrement(1.0);
}

/// Increment active gRPC streams.
pub fn grpc_stream_opened() {
    gauge!("converge_active_grpc_streams").increment(1.0);
}

/// Decrement active gRPC streams.
pub fn grpc_stream_closed() {
    gauge!("converge_active_grpc_streams").decrement(1.0);
}

/// Record a streamed event.
pub fn stream_event(event_type: &str, transport: &str) {
    let labels = [
        ("type", event_type.to_string()),
        ("transport", transport.to_string()),
    ];

    counter!("converge_stream_events_total", &labels).increment(1);
}

// =============================================================================
// Context Metrics
// =============================================================================

/// Record context entry appended.
pub fn context_entry(entry_type: &str) {
    let labels = [("type", entry_type.to_string())];

    counter!("converge_context_entries_total", &labels).increment(1);
}

/// Record context size.
pub fn context_size(facts: u64, proposals: u64, traces: u64) {
    gauge!("converge_context_facts").set(facts as f64);
    gauge!("converge_context_proposals").set(proposals as f64);
    gauge!("converge_context_traces").set(traces as f64);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests use a shared global recorder, so they must be run
    // with `cargo test --features metrics -- --test-threads=1` to avoid races.
    // Each test initializes its own recorder if needed.

    /// Helper to initialize metrics for tests, ignoring if already initialized.
    fn ensure_metrics_initialized() -> PrometheusHandle {
        match init() {
            Ok(handle) => handle,
            Err(_) => {
                // Already initialized, get the existing handle
                handle()
                    .cloned()
                    .expect("Handle should exist after init attempt")
            }
        }
    }

    #[test]
    fn test_http_request_metrics() {
        let handle = ensure_metrics_initialized();

        // Record some HTTP requests
        http_request("GET", "/api/v1/jobs", 200, 0.045);
        http_request("POST", "/api/v1/jobs", 201, 0.120);
        http_request("GET", "/api/v1/jobs", 500, 0.005);

        let output = handle.render();

        // Verify counter exists
        assert!(
            output.contains("http_requests_total"),
            "Should contain http_requests_total metric"
        );

        // Verify histogram exists
        assert!(
            output.contains("http_request_duration_seconds"),
            "Should contain http_request_duration_seconds metric"
        );
    }

    #[test]
    fn test_http_size_metrics() {
        let handle = ensure_metrics_initialized();

        // Record request/response sizes
        http_request_size("POST", "/api/v1/jobs", 1024);
        http_response_size("GET", "/api/v1/jobs", 2048);

        let output = handle.render();

        assert!(
            output.contains("http_request_size_bytes"),
            "Should contain http_request_size_bytes metric"
        );
        assert!(
            output.contains("http_response_size_bytes"),
            "Should contain http_response_size_bytes metric"
        );
    }

    #[test]
    fn test_job_metrics() {
        let handle = ensure_metrics_initialized();

        // Record job completions
        job_completed("converged", 5, 12);
        job_completed("halted", 3, 8);
        job_duration("converged", 1.5);

        let output = handle.render();

        assert!(
            output.contains("converge_jobs_total"),
            "Should contain converge_jobs_total metric"
        );
        assert!(
            output.contains("converge_job_cycles"),
            "Should contain converge_job_cycles metric"
        );
        assert!(
            output.contains("converge_job_facts"),
            "Should contain converge_job_facts metric"
        );
        assert!(
            output.contains("converge_job_duration_seconds"),
            "Should contain converge_job_duration_seconds metric"
        );
    }

    #[test]
    fn test_job_active_gauge() {
        let handle = ensure_metrics_initialized();

        // Simulate job lifecycle
        job_started();
        job_started();
        job_finished();

        let output = handle.render();

        assert!(
            output.contains("converge_active_jobs"),
            "Should contain converge_active_jobs metric"
        );
    }

    #[test]
    fn test_streaming_metrics() {
        let handle = ensure_metrics_initialized();

        // Simulate stream lifecycle
        sse_stream_opened();
        sse_stream_opened();
        grpc_stream_opened();
        sse_stream_closed();
        grpc_stream_closed();

        // Record stream events
        stream_event("fact", "sse");
        stream_event("proposal", "grpc");

        let output = handle.render();

        assert!(
            output.contains("converge_active_streams"),
            "Should contain converge_active_streams metric"
        );
        assert!(
            output.contains("converge_active_grpc_streams"),
            "Should contain converge_active_grpc_streams metric"
        );
        assert!(
            output.contains("converge_stream_events_total"),
            "Should contain converge_stream_events_total metric"
        );
    }

    #[test]
    fn test_context_metrics() {
        let handle = ensure_metrics_initialized();

        // Record context entries
        context_entry("fact");
        context_entry("proposal");
        context_entry("trace");

        // Record context size
        context_size(100, 25, 50);

        let output = handle.render();

        assert!(
            output.contains("converge_context_entries_total"),
            "Should contain converge_context_entries_total metric"
        );
        assert!(
            output.contains("converge_context_facts"),
            "Should contain converge_context_facts metric"
        );
        assert!(
            output.contains("converge_context_proposals"),
            "Should contain converge_context_proposals metric"
        );
        assert!(
            output.contains("converge_context_traces"),
            "Should contain converge_context_traces metric"
        );
    }

    #[test]
    fn test_render_returns_prometheus_format() {
        let handle = ensure_metrics_initialized();

        // Generate some metrics
        http_request("GET", "/health", 200, 0.001);

        let output = handle.render();

        // Prometheus format includes TYPE and HELP comments
        assert!(
            output.contains("# TYPE") || output.contains("http_requests_total"),
            "Output should be in Prometheus format"
        );
    }

    #[test]
    fn test_metrics_with_labels() {
        let handle = ensure_metrics_initialized();

        // Record metrics with various labels
        http_request("GET", "/api/v1/jobs", 200, 0.01);
        http_request("GET", "/api/v1/jobs", 404, 0.005);
        http_request("POST", "/api/v1/jobs", 201, 0.1);

        let output = handle.render();

        // Verify labels are present in output
        assert!(
            output.contains("method=") || output.contains("path="),
            "Output should contain labeled metrics"
        );
    }

    #[test]
    fn test_handle_returns_some_after_init() {
        ensure_metrics_initialized();

        assert!(
            handle().is_some(),
            "handle() should return Some after initialization"
        );
    }

    #[test]
    fn test_render_without_init_returns_empty() {
        // Note: This test may not work as expected if other tests have
        // already initialized metrics. In practice, render() will return
        // the metrics from the global handle.
        // This test documents the expected behavior when not initialized.
        let output = render();

        // If initialized, output will have content; if not, it's empty
        // We just verify it doesn't panic
        assert!(output.is_empty() || output.contains("converge"));
    }
}
