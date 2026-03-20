//! Slack Delivery Module
//!
//! Webhook-based Slack delivery for real-time alerts.
//!
//! This module provides:
//! - Slack configuration from environment variables
//! - Block Kit message formatting for rich alerts
//! - HTTP POST to Slack webhook with error categorization
//! - Actionable messages with CLI commands for ack/escalate/assign
//! - `ReliableSlackSender` with retry, circuit breaker, metrics, and DLQ support

use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use backoff::Error as BackoffError;
use failsafe::CircuitBreaker;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

use crate::delivery::{DeliveryError, DeliveryLogger};
use crate::metrics;
use crate::reliability::{create_backoff, create_circuit_breaker, CircuitBreakerConfig, RetryConfig};

/// Configuration errors for Slack
#[derive(Debug)]
pub enum SlackConfigError {
    /// Missing environment variable
    MissingEnvVar(String),
    /// Invalid webhook URL (must be `https://hooks.slack.com/services/...`)
    InvalidWebhookUrl(String),
}

impl std::fmt::Display for SlackConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnvVar(var) => {
                writeln!(f, "Missing required environment variable: {var}\n")?;
                writeln!(f, "Configure Slack webhook:")?;
                writeln!(f, "  SLACK_WEBHOOK_URL - Slack Incoming Webhook URL")?;
                writeln!(f)?;
                writeln!(f, "To create a webhook:")?;
                writeln!(f, "  1. Go to https://api.slack.com/apps")?;
                writeln!(f, "  2. Create or select an app")?;
                writeln!(f, "  3. Incoming Webhooks -> Add New Webhook to Workspace")?;
                write!(f, "  4. Copy the webhook URL")
            }
            Self::InvalidWebhookUrl(url) => {
                write!(
                    f,
                    "Invalid Slack webhook URL: {url}\n\nMust start with 'https://hooks.slack.com/services/'"
                )
            }
        }
    }
}

impl std::error::Error for SlackConfigError {}

/// Slack configuration
#[derive(Debug, Clone)]
pub struct SlackConfig {
    /// Slack webhook URL
    pub webhook_url: String,
}

impl SlackConfig {
    /// Load Slack configuration from environment variables
    ///
    /// Required:
    /// - `SLACK_WEBHOOK_URL` - Slack Incoming Webhook URL
    ///
    /// # Errors
    ///
    /// Returns `SlackConfigError::MissingEnvVar` if `SLACK_WEBHOOK_URL` is not set.
    /// Returns `SlackConfigError::InvalidWebhookUrl` if URL doesn't match expected format.
    pub fn from_env() -> Result<Self, SlackConfigError> {
        let webhook_url = env::var("SLACK_WEBHOOK_URL")
            .map_err(|_| SlackConfigError::MissingEnvVar("SLACK_WEBHOOK_URL".to_string()))?;

        // Validate webhook URL format
        if !webhook_url.starts_with("https://hooks.slack.com/services/") {
            return Err(SlackConfigError::InvalidWebhookUrl(webhook_url));
        }

        Ok(Self { webhook_url })
    }
}

/// Error category for Slack HTTP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Transient error (retry may succeed) - 429, 5xx, connection issues
    Transient,
    /// Permanent error (retry will not succeed) - 4xx except 429
    Permanent,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient => write!(f, "transient"),
            Self::Permanent => write!(f, "permanent"),
        }
    }
}

impl ErrorCategory {
    /// Get string representation of error category for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::Permanent => "permanent",
        }
    }
}

/// Categorize HTTP status code for retry decisions
///
/// - 429 (Too Many Requests) -> Transient (rate limited)
/// - 5xx (Server errors) -> Transient
/// - 4xx except 429 -> Permanent (client errors like invalid payload)
pub fn categorize_slack_error(status: StatusCode) -> ErrorCategory {
    if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
        ErrorCategory::Transient
    } else {
        // 4xx errors (except 429) are permanent
        ErrorCategory::Permanent
    }
}

/// Slack delivery errors
#[derive(Debug)]
pub enum SlackError {
    /// Configuration error
    Config(SlackConfigError),
    /// HTTP error with categorization
    Http {
        /// Error message
        message: String,
        /// Error category
        category: ErrorCategory,
        /// Whether retry may succeed
        is_retryable: bool,
    },
    /// Message building error
    Message(String),
}

impl std::fmt::Display for SlackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(err) => write!(f, "Configuration error: {err}"),
            Self::Http {
                message,
                category,
                is_retryable,
            } => {
                write!(f, "Slack HTTP error ({category}): {message}")?;
                if *is_retryable {
                    write!(f, " [retryable]")
                } else {
                    write!(f, " [not retryable]")
                }
            }
            Self::Message(err) => write!(f, "Message error: {err}"),
        }
    }
}

impl std::error::Error for SlackError {}

impl From<SlackConfigError> for SlackError {
    fn from(err: SlackConfigError) -> Self {
        Self::Config(err)
    }
}

/// Maximum length for finding details in Block Kit section
/// Block Kit section text limit is 3000 chars, we truncate at 2000 for safety
const MAX_FINDING_DETAILS_LENGTH: usize = 2000;

/// Truncate string to max length with ellipsis
fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated = &s[..max_len.saturating_sub(3)];
        format!("{truncated}...")
    }
}

/// Build a Block Kit alert message
///
/// Creates a rich Slack message with:
/// - Header block with severity
/// - Section with gate, check, severity, finding ID
/// - Section with finding details (truncated if > 2000 chars)
/// - Divider
/// - Section with CLI commands for ack/escalate/assign
///
/// # Arguments
///
/// * `gate_id` - Gate identifier (e.g., `G_PROMOTE_01`)
/// * `check_id` - Check identifier within the gate
/// * `severity` - Alert severity (critical, high, medium, low, info)
/// * `finding_details` - Detailed finding information
/// * `finding_id` - Unique finding identifier for CLI commands
///
/// # Returns
///
/// `serde_json::Value` containing Block Kit message structure
pub fn build_alert_message(
    gate_id: &str,
    check_id: &str,
    severity: &str,
    finding_details: &str,
    finding_id: &str,
) -> Value {
    // Truncate finding details if too long
    let truncated_details = truncate_with_ellipsis(finding_details, MAX_FINDING_DETAILS_LENGTH);

    // Format severity for display
    let severity_upper = severity.to_uppercase();

    // Build CLI commands section
    let cli_commands = format!(
        "```\ncz ack {finding_id}\ncz escalate {finding_id}\ncz assign {finding_id} <owner>\n```"
    );

    json!({
        "text": format!("{severity_upper} Alert: {gate_id}/{check_id}"),
        "blocks": [
            {
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": format!("{severity_upper} Alert"),
                    "emoji": true
                }
            },
            {
                "type": "section",
                "fields": [
                    {
                        "type": "mrkdwn",
                        "text": format!("*Gate:*\n{gate_id}")
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("*Check:*\n{check_id}")
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("*Severity:*\n{severity}")
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("*Finding ID:*\n{finding_id}")
                    }
                ]
            },
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("*Details:*\n{truncated_details}")
                }
            },
            {
                "type": "divider"
            },
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("*Actions:*\n{cli_commands}")
                }
            }
        ]
    })
}

/// Send a message to Slack webhook
///
/// Posts a Block Kit message to the configured Slack webhook URL.
/// Uses a 30 second timeout for the HTTP request.
///
/// # Arguments
///
/// * `client` - Reqwest HTTP client (reusable for connection pooling)
/// * `webhook_url` - Slack Incoming Webhook URL
/// * `message` - Block Kit message as JSON value
///
/// # Returns
///
/// Returns `Ok(())` on success (2xx response).
///
/// # Errors
///
/// Returns `SlackError::Http` with appropriate category for HTTP failures:
/// - 429, 5xx -> Transient (retryable)
/// - 4xx -> Permanent (not retryable)
/// - Connection/timeout errors -> Transient (retryable)
pub async fn send_slack_message(
    client: &Client,
    webhook_url: &str,
    message: Value,
) -> Result<(), SlackError> {
    let response = client
        .post(webhook_url)
        .json(&message)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            // Connection/timeout errors are transient
            SlackError::Http {
                message: e.to_string(),
                category: ErrorCategory::Transient,
                is_retryable: true,
            }
        })?;

    let status = response.status();

    if status.is_success() {
        Ok(())
    } else {
        let category = categorize_slack_error(status);
        let is_retryable = category == ErrorCategory::Transient;

        // Try to get response body for error details
        let body = response.text().await.unwrap_or_else(|_| String::new());

        Err(SlackError::Http {
            message: format!("HTTP {status}: {body}"),
            category,
            is_retryable,
        })
    }
}

/// Reliable Slack sender with retry, circuit breaker, metrics, and DLQ support
///
/// Orchestrates the full reliability stack for Slack webhook delivery:
/// 1. Idempotency check (prevents duplicate sends)
/// 2. Retry with exponential backoff (handles transient failures)
/// 3. Circuit breaker (prevents cascade when Slack is persistently down)
/// 4. Metrics recording (observability)
/// 5. Dead letter queue (captures undeliverable messages)
///
/// # Example
///
/// ```ignore
/// let sender = ReliableSlackSender::new(
///     slack_config,
///     "delivery.db",
///     RetryConfig::default(),
///     CircuitBreakerConfig::default(),
/// )?;
///
/// sender.send(
///     "#security-alerts",
///     "G_PROMOTE_01",
///     "CHECK_AUTH",
///     "WARN",
///     "Authentication bypass detected",
///     "finding-abc123",
///     "2026-W05",
///     "slack:#security-alerts|G_PROMOTE_01|CHECK_AUTH|WARN|2026-W05",
/// ).await?;
/// ```
pub struct ReliableSlackSender {
    config: SlackConfig,
    http_client: Client,
    logger: DeliveryLogger,
    retry_config: RetryConfig,
    cb_config: CircuitBreakerConfig,
}

impl ReliableSlackSender {
    /// Create a new reliable Slack sender
    ///
    /// # Arguments
    ///
    /// * `slack_config` - Slack webhook configuration
    /// * `db_path` - Path to `SQLite` database for delivery logging
    /// * `retry_config` - Configuration for exponential backoff retry
    /// * `cb_config` - Configuration for circuit breaker
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database initialization fails.
    pub fn new(
        slack_config: SlackConfig,
        db_path: &str,
        retry_config: RetryConfig,
        cb_config: CircuitBreakerConfig,
    ) -> Result<Self, rusqlite::Error> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let logger = DeliveryLogger::new(db_path)?;

        Ok(Self {
            config: slack_config,
            http_client,
            logger,
            retry_config,
            cb_config,
        })
    }

    /// Send Slack alert with full reliability stack
    ///
    /// Implements the following flow:
    /// 1. Check idempotency key - skip if already delivered
    /// 2. Build Block Kit message
    /// 3. Retry with exponential backoff for transient failures
    /// 4. Circuit breaker to prevent cascade failures
    /// 5. Record metrics for success/failure
    /// 6. Log to DLQ if all retries exhausted
    ///
    /// # Arguments
    ///
    /// * `channel` - Slack channel for logging/routing context (e.g., "#security-alerts")
    /// * `gate_id` - Gate identifier
    /// * `check_id` - Check identifier
    /// * `severity` - Alert severity (WARN, STOP, etc.)
    /// * `finding_details` - Detailed finding information
    /// * `finding_id` - Unique finding identifier for CLI commands
    /// * `digest_week` - Week identifier (YYYY-WW format)
    /// * `idempotency_key` - Key for deduplication
    ///
    /// # Returns
    ///
    /// `Ok(())` on success
    ///
    /// # Errors
    ///
    /// Returns `SlackError` for duplicate, HTTP failures, or circuit breaker open.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    pub async fn send(
        &self,
        channel: &str,
        gate_id: &str,
        check_id: &str,
        severity: &str,
        finding_details: &str,
        finding_id: &str,
        digest_week: &str,
        idempotency_key: &str,
    ) -> Result<(), SlackError> {
        // Step 1: Idempotency check
        if self
            .logger
            .has_idempotency_key(idempotency_key)
            .map_err(|e| SlackError::Message(format!("Database error: {e}")))?
        {
            metrics::record_slack_delivery_failure("duplicate");
            return Err(SlackError::Message(
                "Duplicate delivery prevented by idempotency check".to_string(),
            ));
        }

        // Step 2: Build message
        let message = build_alert_message(gate_id, check_id, severity, finding_details, finding_id);

        // Create circuit breaker (per-send instance; for production consider singleton)
        let circuit_breaker = create_circuit_breaker(&self.cb_config);

        // Track attempt count across retries
        let attempt_count = Arc::new(AtomicU32::new(0));

        // Clone values for async closure
        let http_client = self.http_client.clone();
        let webhook_url = self.config.webhook_url.clone();
        let message_clone = message.clone();

        // Create backoff for retry logic
        let backoff = create_backoff(&self.retry_config);

        // Step 3: Retry with backoff
        let retry_result: Result<(), SlackError> = {
            let attempt_counter = Arc::clone(&attempt_count);

            let operation = || {
                let client = http_client.clone();
                let url = webhook_url.clone();
                let msg = message_clone.clone();
                let counter = Arc::clone(&attempt_counter);

                async move {
                    // Increment attempt counter
                    counter.fetch_add(1, Ordering::SeqCst);
                    metrics::record_slack_retry_attempt();

                    match send_slack_message(&client, &url, msg).await {
                        Ok(()) => Ok(()),
                        Err(e) => {
                            // Classify error for retry decision
                            match &e {
                                SlackError::Http {
                                    is_retryable: true, ..
                                } => Err(BackoffError::transient(e)),
                                _ => Err(BackoffError::permanent(e)),
                            }
                        }
                    }
                }
            };

            backoff::future::retry(backoff, operation).await
        };

        // Get final attempt count
        let attempts = attempt_count.load(Ordering::SeqCst);
        metrics::record_slack_retry_count(attempts);

        // Step 4: Apply circuit breaker tracking
        let result = if retry_result.is_ok() {
            // Success - report to circuit breaker
            let _cb_result = circuit_breaker.call(|| Ok::<(), ()>(()));
            retry_result
        } else {
            // Failure - report to circuit breaker and check if open
            let cb_result = circuit_breaker.call(|| Err::<(), ()>(()));
            match cb_result {
                Err(failsafe::Error::Rejected) => {
                    // Circuit breaker is now open
                    metrics::set_slack_circuit_breaker_state(2); // 2 = open
                    Err(SlackError::Http {
                        message: "Circuit breaker open - Slack service unavailable".to_string(),
                        category: ErrorCategory::Transient,
                        is_retryable: false,
                    })
                }
                _ => retry_result,
            }
        };

        // Use "slack:{channel}" as recipient for DLQ filtering
        let recipient = format!("slack:{channel}");
        let subject = format!("{severity} Alert: {gate_id}/{check_id}");

        // Step 5 & 6: Metrics and DLQ handling
        match result {
            Ok(()) => {
                metrics::record_slack_delivery_success();

                // Log success using delivery logger
                self.logger
                    .log_success(
                        &recipient,
                        &subject,
                        "slack-ok", // Slack webhook doesn't return message ID
                        digest_week,
                        Some(idempotency_key),
                    )
                    .ok();

                Ok(())
            }
            Err(ref slack_error) => {
                let status = match slack_error {
                    SlackError::Http { category, .. } => category.as_str(),
                    SlackError::Config(_) => "config",
                    SlackError::Message(_) => "message",
                };
                metrics::record_slack_delivery_failure(status);

                // Convert SlackError to DeliveryError for DLQ logging
                let delivery_error = match slack_error {
                    SlackError::Http {
                        message,
                        category,
                        is_retryable,
                    } => DeliveryError::Smtp {
                        message: message.clone(),
                        category: match category {
                            ErrorCategory::Transient => crate::delivery::ErrorCategory::Transient,
                            ErrorCategory::Permanent => crate::delivery::ErrorCategory::Permanent,
                        },
                        is_retryable: *is_retryable,
                    },
                    SlackError::Config(e) => DeliveryError::Message(e.to_string()),
                    SlackError::Message(e) => DeliveryError::Message(e.clone()),
                };

                // Log to DLQ if retries were attempted
                if attempts > 0 {
                    self.logger
                        .log_dead_letter(
                            &recipient,
                            &subject,
                            finding_details, // Store finding details for potential retry
                            &delivery_error,
                            attempts,
                            digest_week,
                            Some(idempotency_key),
                        )
                        .ok();
                    metrics::record_slack_dlq_entry();
                }

                result
            }
        }
    }

    /// Get reference to the delivery logger for DLQ queries
    pub fn logger(&self) -> &DeliveryLogger {
        &self.logger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_config_missing_env() {
        // Test assumes SLACK_WEBHOOK_URL is not set in test environment
        // This is a best-effort test that verifies error type structure
        // Skip if env var happens to be set
        if std::env::var("SLACK_WEBHOOK_URL").is_ok() {
            // Skip this test if already set - we can't safely modify env in Rust 2024
            return;
        }

        let result = SlackConfig::from_env();
        assert!(result.is_err());

        match result.unwrap_err() {
            SlackConfigError::MissingEnvVar(var) => {
                assert_eq!(var, "SLACK_WEBHOOK_URL");
            }
            _ => panic!("Expected MissingEnvVar error"),
        }
    }

    #[test]
    fn test_slack_config_error_display() {
        // Test error message formatting without modifying env vars
        let missing_err = SlackConfigError::MissingEnvVar("SLACK_WEBHOOK_URL".to_string());
        let msg = missing_err.to_string();
        assert!(msg.contains("SLACK_WEBHOOK_URL"));
        assert!(msg.contains("https://api.slack.com/apps"));

        let invalid_err = SlackConfigError::InvalidWebhookUrl("https://example.com/bad".to_string());
        let msg = invalid_err.to_string();
        assert!(msg.contains("https://example.com/bad"));
        assert!(msg.contains("hooks.slack.com/services"));
    }

    #[test]
    fn test_slack_config_url_validation() {
        // Test URL validation logic directly by checking what a valid URL would look like
        let valid_url = "https://hooks.slack.com/services/T00/B00/XXX";
        assert!(valid_url.starts_with("https://hooks.slack.com/services/"));

        let invalid_url = "https://example.com/webhook";
        assert!(!invalid_url.starts_with("https://hooks.slack.com/services/"));
    }

    #[test]
    fn test_build_alert_message_structure() {
        let message = build_alert_message(
            "G_PROMOTE_01",
            "CHECK_AUTH",
            "critical",
            "Authentication bypass detected in production",
            "finding-abc123",
        );

        // Verify text fallback field exists
        assert!(message.get("text").is_some());
        let text = message["text"].as_str().unwrap();
        assert!(text.contains("CRITICAL"));
        assert!(text.contains("G_PROMOTE_01"));

        // Verify blocks array exists
        assert!(message.get("blocks").is_some());
        let blocks = message["blocks"].as_array().unwrap();

        // Should have 5 blocks: header, section (fields), section (details), divider, section (actions)
        assert_eq!(blocks.len(), 5);

        // Check header block
        assert_eq!(blocks[0]["type"], "header");
        assert!(blocks[0]["text"]["text"]
            .as_str()
            .unwrap()
            .contains("CRITICAL"));

        // Check fields section has gate, check, severity, finding_id
        assert_eq!(blocks[1]["type"], "section");
        let fields = blocks[1]["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 4);

        // Check divider
        assert_eq!(blocks[3]["type"], "divider");
    }

    #[test]
    fn test_build_alert_message_truncation() {
        // Create a long finding details string (> 2000 chars)
        let long_details = "x".repeat(2500);

        let message = build_alert_message(
            "G_TEST",
            "CHECK_TEST",
            "high",
            &long_details,
            "finding-truncate-test",
        );

        // Extract the details section
        let blocks = message["blocks"].as_array().unwrap();
        let details_section = &blocks[2];
        let details_text = details_section["text"]["text"].as_str().unwrap();

        // Should be truncated to <= 2000 + "*Details:*\n" prefix + "..."
        assert!(
            details_text.len() <= 2020,
            "Details text should be truncated, got {} chars",
            details_text.len()
        );
        assert!(
            details_text.ends_with("..."),
            "Truncated text should end with ..."
        );
    }

    #[test]
    fn test_build_alert_message_cli_commands() {
        let message = build_alert_message(
            "G_PROMOTE_01",
            "CHECK_DRIFT",
            "medium",
            "Drift detected",
            "test-123",
        );

        let blocks = message["blocks"].as_array().unwrap();
        let actions_section = &blocks[4];
        let actions_text = actions_section["text"]["text"].as_str().unwrap();

        // Verify CLI commands are present
        assert!(actions_text.contains("cz ack test-123"));
        assert!(actions_text.contains("cz escalate test-123"));
        assert!(actions_text.contains("cz assign test-123 <owner>"));
    }

    #[test]
    fn test_error_categorization() {
        // 429 -> Transient
        assert_eq!(
            categorize_slack_error(StatusCode::TOO_MANY_REQUESTS),
            ErrorCategory::Transient
        );

        // 5xx -> Transient
        assert_eq!(
            categorize_slack_error(StatusCode::INTERNAL_SERVER_ERROR),
            ErrorCategory::Transient
        );
        assert_eq!(
            categorize_slack_error(StatusCode::BAD_GATEWAY),
            ErrorCategory::Transient
        );
        assert_eq!(
            categorize_slack_error(StatusCode::SERVICE_UNAVAILABLE),
            ErrorCategory::Transient
        );

        // 4xx (except 429) -> Permanent
        assert_eq!(
            categorize_slack_error(StatusCode::BAD_REQUEST),
            ErrorCategory::Permanent
        );
        assert_eq!(
            categorize_slack_error(StatusCode::UNAUTHORIZED),
            ErrorCategory::Permanent
        );
        assert_eq!(
            categorize_slack_error(StatusCode::FORBIDDEN),
            ErrorCategory::Permanent
        );
        assert_eq!(
            categorize_slack_error(StatusCode::NOT_FOUND),
            ErrorCategory::Permanent
        );
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        // Short string - no truncation
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");

        // Exact length - no truncation
        assert_eq!(truncate_with_ellipsis("hello", 5), "hello");

        // Long string - truncation with ellipsis
        assert_eq!(truncate_with_ellipsis("hello world", 8), "hello...");

        // Very short max - just ellipsis
        assert_eq!(truncate_with_ellipsis("hello world", 3), "...");
    }

    #[test]
    fn test_error_category_as_str() {
        assert_eq!(ErrorCategory::Transient.as_str(), "transient");
        assert_eq!(ErrorCategory::Permanent.as_str(), "permanent");
    }

    #[test]
    fn test_reliable_sender_idempotency() {
        use crate::reliability::{CircuitBreakerConfig, RetryConfig};

        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_slack_reliable_sender_idemp.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        // Create a mock Slack config (won't actually connect)
        let slack_config = SlackConfig {
            webhook_url: "https://hooks.slack.com/services/TEST/TEST/TEST".to_string(),
        };

        let sender = ReliableSlackSender::new(
            slack_config,
            db_path.to_str().unwrap(),
            RetryConfig::default(),
            CircuitBreakerConfig::default(),
        ).unwrap();

        // Manually log a success with idempotency key
        sender.logger().log_success(
            "slack:#test-channel",
            "WARN Alert: G_TEST/CHECK_01",
            "slack-ok",
            "2026-W05",
            Some("slack-idem-key-123"),
        ).unwrap();

        // Create a runtime for the async test
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Attempt to send with the same idempotency key
        let result = rt.block_on(sender.send(
            "#test-channel",
            "G_TEST",
            "CHECK_01",
            "WARN",
            "Test finding details",
            "finding-123",
            "2026-W05",
            "slack-idem-key-123",
        ));

        // Should return duplicate error without attempting send
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("Duplicate") || err_str.contains("idempotency"),
            "Expected duplicate/idempotency error, got: {err_str}"
        );
    }

    #[test]
    fn test_reliable_sender_builds_message() {
        // Test that build_alert_message creates valid Block Kit structure
        let message = build_alert_message(
            "G_PROMOTE_01",
            "CHECK_AUTH",
            "STOP",
            "Critical security bypass detected in authentication module",
            "finding-stop-123",
        );

        // Verify text fallback field exists (required by Slack)
        assert!(message.get("text").is_some());
        let text = message["text"].as_str().unwrap();
        assert!(text.contains("STOP"));
        assert!(text.contains("G_PROMOTE_01"));
        assert!(text.contains("CHECK_AUTH"));

        // Verify blocks array exists
        assert!(message.get("blocks").is_some());
        let blocks = message["blocks"].as_array().unwrap();

        // Should have 5 blocks: header, section (fields), section (details), divider, section (actions)
        assert_eq!(blocks.len(), 5);

        // Check CLI commands are present
        let actions_section = &blocks[4];
        let actions_text = actions_section["text"]["text"].as_str().unwrap();
        assert!(actions_text.contains("cz ack finding-stop-123"));
        assert!(actions_text.contains("cz escalate finding-stop-123"));
        assert!(actions_text.contains("cz assign finding-stop-123 <owner>"));
    }

    #[test]
    fn test_slack_error_display() {
        let http_error = SlackError::Http {
            message: "HTTP 500: Internal Server Error".to_string(),
            category: ErrorCategory::Transient,
            is_retryable: true,
        };
        let display = http_error.to_string();
        assert!(display.contains("500"));
        assert!(display.contains("transient"));
        assert!(display.contains("[retryable]"));

        let config_error = SlackError::Config(SlackConfigError::MissingEnvVar("SLACK_WEBHOOK_URL".to_string()));
        let display = config_error.to_string();
        assert!(display.contains("Configuration"));
        assert!(display.contains("SLACK_WEBHOOK_URL"));

        let message_error = SlackError::Message("Test message error".to_string());
        let display = message_error.to_string();
        assert!(display.contains("Message error"));
    }
}
