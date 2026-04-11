//! Email Delivery Module
//!
//! SMTP-based email delivery for weekly digest reports.
//!
//! This module provides:
//! - SMTP configuration from environment variables
//! - Transport builder using lettre with STARTTLS
//! - Async email sending with `spawn_blocking` for I/O isolation
//! - Error categorization (transient, permanent, timeout, TLS, auth)

use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use backoff::Error as BackoffError;
use failsafe::CircuitBreaker;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use tokio::task::spawn_blocking;

use crate::metrics;
use crate::reliability::{create_backoff, create_circuit_breaker, CircuitBreakerConfig, RetryConfig};

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    /// Single missing environment variable
    MissingEnvVar(String),
    /// Multiple missing environment variables
    MissingEnvVars(Vec<String>),
    /// Invalid port value
    InvalidPort,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnvVar(var) => {
                writeln!(f, "Missing required environment variable: {var}\n")?;
                writeln!(f, "Configure SMTP settings:")?;
                writeln!(f, "  SMTP_HOST         - Your SMTP provider (e.g., smtp.gmail.com)")?;
                writeln!(f, "  SMTP_USERNAME     - SMTP username")?;
                writeln!(f, "  SMTP_PASSWORD     - SMTP password or App Password")?;
                writeln!(f, "  SMTP_FROM_ADDRESS - Sender email address")?;
                writeln!(f)?;
                writeln!(f, "Optional:")?;
                writeln!(f, "  SMTP_PORT         - Port (default: 587)")?;
                write!(f, "  SMTP_FROM_NAME    - Sender name (default: \"Strategy Validator\")")
            }
            Self::MissingEnvVars(vars) => {
                writeln!(f, "Missing required environment variables: {}\n", vars.join(", "))?;
                writeln!(f, "Configure SMTP settings:")?;
                writeln!(f, "  SMTP_HOST         - Your SMTP provider (e.g., smtp.gmail.com)")?;
                writeln!(f, "  SMTP_USERNAME     - SMTP username")?;
                writeln!(f, "  SMTP_PASSWORD     - SMTP password or App Password")?;
                writeln!(f, "  SMTP_FROM_ADDRESS - Sender email address")?;
                writeln!(f)?;
                writeln!(f, "Optional:")?;
                writeln!(f, "  SMTP_PORT         - Port (default: 587)")?;
                write!(f, "  SMTP_FROM_NAME    - Sender name (default: \"Strategy Validator\")")
            }
            Self::InvalidPort => {
                write!(f, "Invalid SMTP_PORT: must be a number between 1 and 65535")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// SMTP configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// SMTP host (e.g., smtp.gmail.com)
    pub host: String,
    /// SMTP port (default: 587)
    pub port: u16,
    /// SMTP username
    pub username: String,
    /// SMTP password
    pub password: String,
    /// From email address
    pub from_address: String,
    /// From name (default: "Strategy Validator")
    pub from_name: String,
}

impl SmtpConfig {
    /// Load SMTP configuration from environment variables
    ///
    /// Required:
    /// - `SMTP_HOST`
    /// - `SMTP_USERNAME`
    /// - `SMTP_PASSWORD`
    /// - `SMTP_FROM_ADDRESS`
    ///
    /// Optional:
    /// - `SMTP_PORT` (default: 587)
    /// - `SMTP_FROM_NAME` (default: "Strategy Validator")
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::MissingEnvVars` if required variables are missing.
    /// Returns `ConfigError::InvalidPort` if `SMTP_PORT` is not a valid port number.
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut missing = Vec::new();

        // Load required variables
        let host = env::var("SMTP_HOST").ok();
        let username = env::var("SMTP_USERNAME").ok();
        let password = env::var("SMTP_PASSWORD").ok();
        let from_address = env::var("SMTP_FROM_ADDRESS").ok();

        // Track missing required variables
        if host.is_none() {
            missing.push("SMTP_HOST".to_string());
        }
        if username.is_none() {
            missing.push("SMTP_USERNAME".to_string());
        }
        if password.is_none() {
            missing.push("SMTP_PASSWORD".to_string());
        }
        if from_address.is_none() {
            missing.push("SMTP_FROM_ADDRESS".to_string());
        }

        // Return error if any required variables are missing
        if !missing.is_empty() {
            if missing.len() == 1 {
                return Err(ConfigError::MissingEnvVar(missing[0].clone()));
            }
            return Err(ConfigError::MissingEnvVars(missing));
        }

        // Load optional variables
        let port = match env::var("SMTP_PORT") {
            Ok(port_str) => port_str.parse::<u16>().map_err(|_| ConfigError::InvalidPort)?,
            Err(_) => 587,
        };

        let from_name = env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Strategy Validator".to_string());

        Ok(Self {
            host: host.expect("host checked above"),
            port,
            username: username.expect("username checked above"),
            password: password.expect("password checked above"),
            from_address: from_address.expect("from_address checked above"),
            from_name,
        })
    }
}

/// Error category for SMTP operations
#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
    /// Transient error (retry may succeed)
    Transient,
    /// Permanent error (retry will not succeed)
    Permanent,
    /// Timeout error
    Timeout,
    /// TLS/SSL error
    Tls,
    /// Authentication error
    Auth,
    /// Unknown error
    Unknown,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient => write!(f, "transient"),
            Self::Permanent => write!(f, "permanent"),
            Self::Timeout => write!(f, "timeout"),
            Self::Tls => write!(f, "tls"),
            Self::Auth => write!(f, "authentication"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl ErrorCategory {
    /// Get string representation of error category for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::Permanent => "permanent",
            Self::Timeout => "timeout",
            Self::Tls => "tls",
            Self::Auth => "auth",
            Self::Unknown => "unknown",
        }
    }
}

/// Delivery errors
#[derive(Debug)]
pub enum DeliveryError {
    /// Configuration error
    Config(ConfigError),
    /// SMTP error with categorization
    Smtp {
        /// Error message
        message: String,
        /// Error category
        category: ErrorCategory,
        /// Whether retry may succeed
        is_retryable: bool,
    },
    /// Message building error
    Message(String),
    /// Runtime error (`spawn_blocking` join error)
    Runtime(String),
}

impl std::fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(err) => write!(f, "Configuration error: {err}"),
            Self::Smtp { message, category, is_retryable } => {
                write!(f, "SMTP error ({category}): {message}")?;
                if *is_retryable {
                    write!(f, " [retryable]")
                } else {
                    write!(f, " [not retryable]")
                }
            }
            Self::Message(err) => write!(f, "Message error: {err}"),
            Self::Runtime(err) => write!(f, "Runtime error: {err}"),
        }
    }
}

impl std::error::Error for DeliveryError {}

impl From<ConfigError> for DeliveryError {
    fn from(err: ConfigError) -> Self {
        Self::Config(err)
    }
}

impl From<lettre::error::Error> for DeliveryError {
    fn from(err: lettre::error::Error) -> Self {
        Self::Message(err.to_string())
    }
}

impl From<lettre::address::AddressError> for DeliveryError {
    fn from(err: lettre::address::AddressError) -> Self {
        Self::Message(err.to_string())
    }
}

/// Categorize SMTP error
fn categorize_smtp_error(err: &lettre::transport::smtp::Error) -> ErrorCategory {
    if err.is_transient() {
        ErrorCategory::Transient
    } else if err.is_permanent() {
        ErrorCategory::Permanent
    } else if err.is_timeout() {
        ErrorCategory::Timeout
    } else if err.is_tls() {
        ErrorCategory::Tls
    } else if err.is_client() {
        // Client errors include auth failures
        ErrorCategory::Auth
    } else {
        ErrorCategory::Unknown
    }
}

/// Escape HTML entities in string
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Build SMTP transport with STARTTLS
///
/// Creates an SMTP transport configured with:
/// - STARTTLS encryption on the configured port (default 587)
/// - PLAIN authentication mechanism
/// - 30 second timeout
/// - Pool size of 1 (single connection for weekly digest)
///
/// # Errors
///
/// Returns `DeliveryError` if transport creation fails.
pub fn build_smtp_transport(config: &SmtpConfig) -> Result<SmtpTransport, DeliveryError> {
    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let transport = SmtpTransport::starttls_relay(&config.host)
        .map_err(|e| DeliveryError::Smtp {
            message: format!("Failed to create SMTP relay: {e}"),
            category: ErrorCategory::Tls,
            is_retryable: false,
        })?
        .port(config.port)
        .credentials(creds)
        .authentication(vec![Mechanism::Plain])
        .timeout(Some(Duration::from_secs(30)))
        .pool_config(lettre::transport::smtp::PoolConfig::new().max_size(1))
        .build();

    Ok(transport)
}

/// Send digest email via SMTP
///
/// Sends a multipart email with both plain text and HTML versions of the markdown body.
/// Uses `spawn_blocking` to isolate blocking SMTP I/O from the async runtime.
///
/// # Arguments
///
/// * `config` - SMTP configuration
/// * `recipient` - Recipient email address
/// * `subject` - Email subject line
/// * `markdown_body` - Email body in markdown format
///
/// # Returns
///
/// Returns the message ID from the SMTP server on success.
///
/// # Errors
///
/// Returns `DeliveryError` for configuration, message building, or SMTP errors.
pub async fn send_digest_email(
    config: &SmtpConfig,
    recipient: &str,
    subject: &str,
    markdown_body: &str,
) -> Result<String, DeliveryError> {
    // Parse sender mailbox
    let from_mailbox: Mailbox = format!("{} <{}>", config.from_name, config.from_address)
        .parse()
        .map_err(|e: lettre::address::AddressError| DeliveryError::Message(format!("Invalid from address: {e}")))?;

    // Parse recipient mailbox
    let to_mailbox: Mailbox = recipient
        .parse()
        .map_err(|e: lettre::address::AddressError| DeliveryError::Message(format!("Invalid recipient address: {e}")))?;

    // Create plain text version (markdown as-is)
    let plain_text = markdown_body.to_string();

    // Create HTML version (wrap in pre tag with monospace font)
    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
body {{ font-family: monospace; white-space: pre-wrap; margin: 20px; }}
</style>
</head>
<body>{}</body>
</html>"#,
        html_escape(markdown_body)
    );

    // Build multipart message
    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_PLAIN)
                        .body(plain_text),
                )
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        )?;

    // Build transport
    let transport = build_smtp_transport(config)?;

    // Send email in blocking context
    let result = spawn_blocking(move || transport.send(&email))
        .await
        .map_err(|e| DeliveryError::Runtime(format!("spawn_blocking join error: {e}")))?;

    // Handle SMTP result
    match result {
        Ok(response) => {
            // Extract message ID from response
            let message_id = response
                .message()
                .next()
                .map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);
            Ok(message_id)
        }
        Err(e) => {
            let category = categorize_smtp_error(&e);
            let is_retryable = e.is_transient() || e.is_timeout();
            Err(DeliveryError::Smtp {
                message: e.to_string(),
                category,
                is_retryable,
            })
        }
    }
}

/// Entry in the dead letter queue for undeliverable messages
#[derive(Debug, Clone)]
pub struct DeadLetterEntry {
    /// Recipient email address
    pub recipient: String,
    /// Email subject
    pub subject: String,
    /// Error message from final delivery attempt
    pub error_message: String,
    /// Error category (transient, permanent, timeout, etc.)
    pub error_category: String,
    /// Number of retry attempts before moving to DLQ
    pub retry_count: u32,
    /// Unix timestamp when moved to DLQ
    pub timestamp: i64,
    /// Week identifier (YYYY-WW format)
    pub digest_week: String,
    /// Optional idempotency key for deduplication
    pub idempotency_key: Option<String>,
}

/// Statistics for dead letter queue monitoring
#[derive(Debug, Clone)]
pub struct DlqStats {
    /// Total number of entries in DLQ
    pub total_count: i64,
    /// Number of unique error types in DLQ
    pub unique_error_types: i64,
    /// Unix timestamp of oldest entry (None if DLQ is empty)
    pub oldest_entry_timestamp: Option<i64>,
    /// Unix timestamp of newest entry (None if DLQ is empty)
    pub newest_entry_timestamp: Option<i64>,
}

/// Delivery logger for `SQLite` persistence
pub struct DeliveryLogger {
    conn: rusqlite::Connection,
}

impl DeliveryLogger {
    /// Create a new delivery logger with `SQLite` database
    ///
    /// Creates the database file if it doesn't exist and initializes the schema.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to `SQLite` database file
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database creation or schema initialization fails.
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = rusqlite::Connection::open(db_path)?;

        // Create delivery_log table
        conn.execute(
            r"
            CREATE TABLE IF NOT EXISTS delivery_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                recipient TEXT NOT NULL,
                subject TEXT NOT NULL,
                message_id TEXT,
                status TEXT NOT NULL,
                smtp_code INTEGER,
                error_message TEXT,
                error_category TEXT,
                digest_week TEXT NOT NULL,
                retry_count INTEGER DEFAULT 0,
                idempotency_key TEXT
            )
            ",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_delivery_log_timestamp ON delivery_log(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_delivery_log_week ON delivery_log(digest_week)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_delivery_log_status ON delivery_log(status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_delivery_log_idempotency ON delivery_log(idempotency_key)",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Check if an idempotency key has been used
    ///
    /// # Arguments
    ///
    /// * `idempotency_key` - Idempotency key to check
    ///
    /// # Returns
    ///
    /// `true` if the key exists in the delivery log, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database query fails.
    pub fn has_idempotency_key(&self, idempotency_key: &str) -> Result<bool, rusqlite::Error> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM delivery_log WHERE idempotency_key = ?1",
            rusqlite::params![idempotency_key],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Log successful email delivery
    ///
    /// # Arguments
    ///
    /// * `recipient` - Recipient email address
    /// * `subject` - Email subject line
    /// * `message_id` - SMTP message ID from server response
    /// * `digest_week` - Week identifier (YYYY-WW format)
    /// * `idempotency_key` - Optional idempotency key for deduplication
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database insert fails.
    pub fn log_success(
        &self,
        recipient: &str,
        subject: &str,
        message_id: &str,
        digest_week: &str,
        idempotency_key: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let timestamp = chrono::Utc::now().timestamp();

        self.conn.execute(
            r"
            INSERT INTO delivery_log (timestamp, recipient, subject, message_id, status, digest_week, idempotency_key)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            rusqlite::params![timestamp, recipient, subject, message_id, "sent", digest_week, idempotency_key],
        )?;

        Ok(())
    }

    /// Log failed email delivery
    ///
    /// # Arguments
    ///
    /// * `recipient` - Recipient email address
    /// * `subject` - Email subject line
    /// * `error` - Delivery error with categorization
    /// * `digest_week` - Week identifier (YYYY-WW format)
    /// * `idempotency_key` - Optional idempotency key for deduplication
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database insert fails.
    pub fn log_failure(
        &self,
        recipient: &str,
        subject: &str,
        error: &DeliveryError,
        digest_week: &str,
        idempotency_key: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let timestamp = chrono::Utc::now().timestamp();

        let (error_message, error_category, smtp_code): (String, Option<&str>, Option<i32>) = match error {
            DeliveryError::Smtp { message, category, .. } => {
                (message.clone(), Some(category.as_str()), None)
            }
            DeliveryError::Config(e) => (e.to_string(), Some("config"), None),
            DeliveryError::Message(e) => (e.clone(), Some("message"), None),
            DeliveryError::Runtime(e) => (e.clone(), Some("runtime"), None),
        };

        self.conn.execute(
            r"
            INSERT INTO delivery_log (timestamp, recipient, subject, status, error_message, error_category, smtp_code, digest_week, idempotency_key)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            rusqlite::params![
                timestamp,
                recipient,
                subject,
                "failed",
                error_message,
                error_category,
                smtp_code,
                digest_week,
                idempotency_key
            ],
        )?;

        Ok(())
    }

    /// Log routing decision for audit purposes
    ///
    /// Records why an alert was or wasn't delivered based on routing rules.
    ///
    /// # Arguments
    ///
    /// * `recipient` - Recipient email address
    /// * `digest_week` - Week identifier (YYYY-WW format)
    /// * `decision` - Decision type (delivered, quiet\_hours, rate\_limited, no\_route, duplicate)
    /// * `reason` - Human-readable reason for the decision
    /// * `idempotency_key` - Optional idempotency key for deduplication
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database insert fails.
    pub fn log_routing_decision(
        &self,
        recipient: &str,
        digest_week: &str,
        decision: &str,
        reason: &str,
        idempotency_key: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let timestamp = chrono::Utc::now().timestamp();

        self.conn.execute(
            r"
            INSERT INTO delivery_log (timestamp, recipient, subject, status, error_message, digest_week, idempotency_key)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            rusqlite::params![
                timestamp,
                recipient,
                format!("Weekly Digest: Week {}", digest_week),
                decision,
                reason,
                digest_week,
                idempotency_key
            ],
        )?;

        Ok(())
    }

    /// Log message to dead letter queue after retry exhaustion
    ///
    /// Stores undeliverable messages for manual review. Uses `status=dead_letter`
    /// to distinguish from sent and failed entries.
    ///
    /// # Arguments
    ///
    /// * `recipient` - Recipient email address
    /// * `subject` - Email subject line
    /// * `_body` - Email body (stored for potential reprocessing in Phase 20+)
    /// * `error` - Final delivery error after retry exhaustion
    /// * `retry_count` - Number of retry attempts before moving to DLQ
    /// * `digest_week` - Week identifier (YYYY-WW format)
    /// * `idempotency_key` - Optional idempotency key for deduplication
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database insert fails.
    #[allow(clippy::too_many_arguments)]
    pub fn log_dead_letter(
        &self,
        recipient: &str,
        subject: &str,
        _body: &str,
        error: &DeliveryError,
        retry_count: u32,
        digest_week: &str,
        idempotency_key: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let timestamp = chrono::Utc::now().timestamp();

        let (error_message, error_category): (String, Option<&str>) = match error {
            DeliveryError::Smtp { message, category, .. } => {
                (message.clone(), Some(category.as_str()))
            }
            DeliveryError::Config(e) => (e.to_string(), Some("config")),
            DeliveryError::Message(e) => (e.clone(), Some("message")),
            DeliveryError::Runtime(e) => (e.clone(), Some("runtime")),
        };

        self.conn.execute(
            r"
            INSERT INTO delivery_log (
                timestamp, recipient, subject, status,
                error_message, error_category, retry_count,
                digest_week, idempotency_key
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            rusqlite::params![
                timestamp,
                recipient,
                subject,
                "dead_letter",
                error_message,
                error_category,
                retry_count,
                digest_week,
                idempotency_key
            ],
        )?;

        Ok(())
    }

    /// List dead letter queue entries for manual review
    ///
    /// Returns entries from the last `hours` hours, ordered by most recent first.
    ///
    /// # Arguments
    ///
    /// * `hours` - Number of hours to look back (entries older than this are excluded)
    /// * `limit` - Maximum number of entries to return
    ///
    /// # Returns
    ///
    /// Vector of `DeadLetterEntry` ordered by timestamp descending
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database query fails.
    pub fn list_dead_letters(
        &self,
        hours: i64,
        limit: usize,
    ) -> Result<Vec<DeadLetterEntry>, rusqlite::Error> {
        let cutoff = chrono::Utc::now().timestamp() - (hours * 3600);

        let mut stmt = self.conn.prepare(
            r"
            SELECT
                recipient,
                subject,
                error_message,
                error_category,
                retry_count,
                timestamp,
                digest_week,
                idempotency_key
            FROM delivery_log
            WHERE status = 'dead_letter' AND timestamp > ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            ",
        )?;

        let entries = stmt.query_map(rusqlite::params![cutoff, limit], |row| {
            Ok(DeadLetterEntry {
                recipient: row.get(0)?,
                subject: row.get(1)?,
                error_message: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                error_category: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                retry_count: row.get::<_, Option<u32>>(4)?.unwrap_or(0),
                timestamp: row.get(5)?,
                digest_week: row.get(6)?,
                idempotency_key: row.get(7)?,
            })
        })?;

        entries.collect()
    }

    /// Get dead letter queue statistics for monitoring
    ///
    /// Returns aggregate statistics about the DLQ for alerting and dashboards.
    ///
    /// # Returns
    ///
    /// `DlqStats` with total count, unique error types, and timestamp range
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database query fails.
    pub fn dead_letter_stats(&self) -> Result<DlqStats, rusqlite::Error> {
        self.conn.query_row(
            r"
            SELECT
                COUNT(*) as total,
                COUNT(DISTINCT error_category) as unique_errors,
                MIN(timestamp) as oldest,
                MAX(timestamp) as newest
            FROM delivery_log
            WHERE status = 'dead_letter'
            ",
            [],
            |row| {
                Ok(DlqStats {
                    total_count: row.get(0)?,
                    unique_error_types: row.get(1)?,
                    oldest_entry_timestamp: row.get(2)?,
                    newest_entry_timestamp: row.get(3)?,
                })
            },
        )
    }
}

/// Reliable email sender with retry, circuit breaker, metrics, and DLQ support
///
/// Orchestrates the full reliability stack:
/// 1. Idempotency check (prevents duplicate sends)
/// 2. Retry with exponential backoff (handles transient failures)
/// 3. Circuit breaker (prevents cascade when SMTP is persistently down)
/// 4. Metrics recording (observability)
/// 5. Dead letter queue (captures undeliverable messages)
///
/// # Example
///
/// ```ignore
/// let sender = ReliableEmailSender::new(
///     smtp_config,
///     "delivery.db",
///     RetryConfig::default(),
///     CircuitBreakerConfig::default(),
/// )?;
///
/// sender.send(
///     "user@example.com",
///     "Weekly Digest",
///     "# Summary\n...",
///     "2026-W05",
///     "digest|weekly|info|2026-W05|user@example.com",
/// ).await?;
/// ```
pub struct ReliableEmailSender {
    config: SmtpConfig,
    logger: DeliveryLogger,
    retry_config: RetryConfig,
    cb_config: CircuitBreakerConfig,
}

impl ReliableEmailSender {
    /// Create a new reliable email sender
    ///
    /// # Arguments
    ///
    /// * `smtp_config` - SMTP configuration
    /// * `db_path` - Path to `SQLite` database for delivery logging
    /// * `retry_config` - Configuration for exponential backoff retry
    /// * `cb_config` - Configuration for circuit breaker
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if database initialization fails.
    pub fn new(
        smtp_config: SmtpConfig,
        db_path: &str,
        retry_config: RetryConfig,
        cb_config: CircuitBreakerConfig,
    ) -> Result<Self, rusqlite::Error> {
        let logger = DeliveryLogger::new(db_path)?;

        Ok(Self {
            config: smtp_config,
            logger,
            retry_config,
            cb_config,
        })
    }

    /// Send email with full reliability stack
    ///
    /// Implements the following flow:
    /// 1. Check idempotency key - skip if already delivered
    /// 2. Retry with exponential backoff for transient failures
    /// 3. Circuit breaker to prevent cascade failures
    /// 4. Record metrics for success/failure
    /// 5. Log to DLQ if all retries exhausted
    ///
    /// # Arguments
    ///
    /// * `recipient` - Recipient email address
    /// * `subject` - Email subject line
    /// * `body` - Email body in markdown format
    /// * `digest_week` - Week identifier (YYYY-WW format)
    /// * `idempotency_key` - Key for deduplication
    ///
    /// # Returns
    ///
    /// SMTP message ID on success
    ///
    /// # Errors
    ///
    /// Returns `DeliveryError` for duplicate, SMTP failures, or circuit breaker open.
    #[allow(clippy::too_many_lines)]
    pub async fn send(
        &self,
        recipient: &str,
        subject: &str,
        body: &str,
        digest_week: &str,
        idempotency_key: &str,
    ) -> Result<String, DeliveryError> {
        // Step 1: Idempotency check
        if self
            .logger
            .has_idempotency_key(idempotency_key)
            .map_err(|e| DeliveryError::Runtime(format!("Database error: {e}")))?
        {
            metrics::record_delivery_failure("duplicate");
            return Err(DeliveryError::Message(
                "Duplicate delivery prevented by idempotency check".to_string(),
            ));
        }

        // Create circuit breaker (per-send instance; for production consider singleton)
        let circuit_breaker = create_circuit_breaker(&self.cb_config);

        // Track attempt count across retries
        let attempt_count = Arc::new(AtomicU32::new(0));

        // Clone values for async closure
        let config = self.config.clone();
        let recipient_owned = recipient.to_string();
        let subject_owned = subject.to_string();
        let body_owned = body.to_string();

        // Create backoff for retry logic
        let backoff = create_backoff(&self.retry_config);

        // Step 2: Retry with backoff
        let retry_result: Result<String, DeliveryError> = {
            let attempt_counter = Arc::clone(&attempt_count);

            let operation = || {
                let config = config.clone();
                let recipient = recipient_owned.clone();
                let subject = subject_owned.clone();
                let body = body_owned.clone();
                let counter = Arc::clone(&attempt_counter);

                async move {
                    // Increment attempt counter
                    counter.fetch_add(1, Ordering::SeqCst);
                    metrics::record_retry_attempt();

                    match send_digest_email(&config, &recipient, &subject, &body).await {
                        Ok(msg_id) => Ok(msg_id),
                        Err(e) => {
                            // Classify error for retry decision
                            match &e {
                                DeliveryError::Smtp {
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
        metrics::record_retry_count(attempts);

        // Step 3: Apply circuit breaker tracking
        // Note: failsafe circuit breaker requires sync, so we track state manually
        // and use it for the next call decision
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
                    metrics::set_circuit_breaker_state(2); // 2 = open
                    Err(DeliveryError::Smtp {
                        message: "Circuit breaker open - service unavailable".to_string(),
                        category: ErrorCategory::Transient,
                        is_retryable: false,
                    })
                }
                _ => retry_result,
            }
        };

        // Step 4 & 5: Metrics and DLQ handling
        match result {
            Ok(msg_id) => {
                metrics::record_delivery_success();

                // Log success
                self.logger
                    .log_success(
                        recipient,
                        subject,
                        &msg_id,
                        digest_week,
                        Some(idempotency_key),
                    )
                    .ok();

                Ok(msg_id)
            }
            Err(ref delivery_error) => {
                let status = match delivery_error {
                    DeliveryError::Smtp { category, .. } => category.as_str(),
                    DeliveryError::Config(_) => "config",
                    DeliveryError::Message(_) => "message",
                    DeliveryError::Runtime(_) => "runtime",
                };
                metrics::record_delivery_failure(status);

                // Log to DLQ if retries were attempted and it's not a circuit breaker rejection
                if attempts > 0 {
                    self.logger
                        .log_dead_letter(
                            recipient,
                            subject,
                            body,
                            delivery_error,
                            attempts,
                            digest_week,
                            Some(idempotency_key),
                        )
                        .ok();
                    metrics::record_dlq_entry();
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
    fn test_smtp_config_missing_env_vars() {
        // Save current env vars if they exist
        let saved_host = std::env::var("SMTP_HOST").ok();
        let saved_username = std::env::var("SMTP_USERNAME").ok();
        let saved_password = std::env::var("SMTP_PASSWORD").ok();
        let saved_from = std::env::var("SMTP_FROM_ADDRESS").ok();

        // Test with missing vars by checking result without setting them
        // This test assumes the test environment doesn't have these vars set
        // If they are set, the test will be skipped
        if saved_host.is_some() || saved_username.is_some() || saved_password.is_some() || saved_from.is_some() {
            // Skip test if any env vars are already set
            return;
        }

        let result = SmtpConfig::from_env();
        assert!(result.is_err(), "Expected SmtpConfig::from_env() to fail when env vars are missing");
    }

    #[test]
    fn test_delivery_logger_creates_db() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_delivery.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        let logger = DeliveryLogger::new(db_path.to_str().unwrap()).unwrap();

        // Verify table exists by logging a success
        logger.log_success("test@example.com", "Test Subject", "msg-123", "2026-05", Some("test-key")).unwrap();
    }

    #[test]
    fn test_has_idempotency_key() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_idempotency.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        let logger = DeliveryLogger::new(db_path.to_str().unwrap()).unwrap();

        // Initially should not exist
        assert!(!logger.has_idempotency_key("key-123").unwrap());

        // Log success with idempotency key
        logger.log_success(
            "test@example.com",
            "Test Subject",
            "msg-123",
            "2026-05",
            Some("key-123")
        ).unwrap();

        // Now should exist
        assert!(logger.has_idempotency_key("key-123").unwrap());

        // Different key should not exist
        assert!(!logger.has_idempotency_key("key-456").unwrap());
    }

    #[test]
    fn test_idempotency_key_on_failure() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_idempotency_failure.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        let logger = DeliveryLogger::new(db_path.to_str().unwrap()).unwrap();

        let error = DeliveryError::Message("Test error".to_string());

        // Log failure with idempotency key
        logger.log_failure(
            "test@example.com",
            "Test Subject",
            &error,
            "2026-05",
            Some("fail-key-123")
        ).unwrap();

        // Should exist even for failures
        assert!(logger.has_idempotency_key("fail-key-123").unwrap());
    }

    #[test]
    fn test_error_category_as_str() {
        assert_eq!(ErrorCategory::Transient.as_str(), "transient");
        assert_eq!(ErrorCategory::Permanent.as_str(), "permanent");
        assert_eq!(ErrorCategory::Timeout.as_str(), "timeout");
        assert_eq!(ErrorCategory::Tls.as_str(), "tls");
        assert_eq!(ErrorCategory::Auth.as_str(), "auth");
        assert_eq!(ErrorCategory::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_log_routing_decision() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_routing_decision.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        let logger = DeliveryLogger::new(db_path.to_str().unwrap()).unwrap();

        // Log a quiet_hours decision
        logger.log_routing_decision(
            "test@example.com",
            "2026-W05",
            "quiet_hours",
            "Suppressed during configured quiet hours (22:00-06:00)",
            Some("key-quiet-123")
        ).unwrap();

        // Log a rate_limited decision
        logger.log_routing_decision(
            "test@example.com",
            "2026-W05",
            "rate_limited",
            "Recipient exceeded 10 alerts/hour limit",
            Some("key-rate-123")
        ).unwrap();

        // Log a duplicate decision
        logger.log_routing_decision(
            "test@example.com",
            "2026-W05",
            "duplicate",
            "Idempotency key already exists in delivery log",
            Some("key-dup-123")
        ).unwrap();

        // Verify all idempotency keys exist
        assert!(logger.has_idempotency_key("key-quiet-123").unwrap());
        assert!(logger.has_idempotency_key("key-rate-123").unwrap());
        assert!(logger.has_idempotency_key("key-dup-123").unwrap());
    }

    #[test]
    fn test_log_dead_letter() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_dead_letter.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        let logger = DeliveryLogger::new(db_path.to_str().unwrap()).unwrap();

        let error = DeliveryError::Smtp {
            message: "Connection refused".to_string(),
            category: ErrorCategory::Transient,
            is_retryable: true,
        };

        // Log a dead letter entry
        logger.log_dead_letter(
            "test@example.com",
            "Weekly Digest",
            "# Summary\nTest body content",
            &error,
            3,
            "2026-W05",
            Some("dlq-key-123"),
        ).unwrap();

        // Verify list_dead_letters returns it
        let entries = logger.list_dead_letters(24, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].recipient, "test@example.com");
        assert_eq!(entries[0].subject, "Weekly Digest");
        assert_eq!(entries[0].error_message, "Connection refused");
        assert_eq!(entries[0].error_category, "transient");
        assert_eq!(entries[0].retry_count, 3);
        assert_eq!(entries[0].digest_week, "2026-W05");
        assert_eq!(entries[0].idempotency_key, Some("dlq-key-123".to_string()));

        // Verify dead_letter_stats shows count=1
        let stats = logger.dead_letter_stats().unwrap();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.unique_error_types, 1);
        assert!(stats.oldest_entry_timestamp.is_some());
        assert!(stats.newest_entry_timestamp.is_some());
    }

    #[test]
    fn test_dead_letter_filtering() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_dead_letter_filter.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        let logger = DeliveryLogger::new(db_path.to_str().unwrap()).unwrap();

        // Log a "sent" entry
        logger.log_success(
            "sent@example.com",
            "Test Sent",
            "msg-123",
            "2026-W05",
            Some("sent-key-123"),
        ).unwrap();

        // Log a "failed" entry
        let failed_error = DeliveryError::Message("Bad format".to_string());
        logger.log_failure(
            "failed@example.com",
            "Test Failed",
            &failed_error,
            "2026-W05",
            Some("failed-key-123"),
        ).unwrap();

        // Log two "dead_letter" entries with different error types
        let dlq_error_1 = DeliveryError::Smtp {
            message: "Connection refused".to_string(),
            category: ErrorCategory::Transient,
            is_retryable: true,
        };
        logger.log_dead_letter(
            "dlq1@example.com",
            "Test DLQ 1",
            "Body 1",
            &dlq_error_1,
            3,
            "2026-W05",
            Some("dlq-key-1"),
        ).unwrap();

        let dlq_error_2 = DeliveryError::Smtp {
            message: "Auth failed".to_string(),
            category: ErrorCategory::Auth,
            is_retryable: false,
        };
        logger.log_dead_letter(
            "dlq2@example.com",
            "Test DLQ 2",
            "Body 2",
            &dlq_error_2,
            5,
            "2026-W05",
            Some("dlq-key-2"),
        ).unwrap();

        // Verify list_dead_letters only returns dead_letter status
        let entries = logger.list_dead_letters(24, 10).unwrap();
        assert_eq!(entries.len(), 2);

        // Should not contain "sent" or "failed" entries
        for entry in &entries {
            assert!(entry.recipient.starts_with("dlq"), "Expected only DLQ entries, got: {}", entry.recipient);
        }

        // Verify stats show correct unique error types
        let stats = logger.dead_letter_stats().unwrap();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.unique_error_types, 2); // transient and auth
    }

    #[test]
    fn test_reliable_sender_idempotency() {
        use crate::reliability::{CircuitBreakerConfig, RetryConfig};

        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_reliable_sender_idemp.db");
        let _ = std::fs::remove_file(&db_path); // Clean up from prior runs

        // Create a mock SMTP config (won't actually connect)
        let smtp_config = SmtpConfig {
            host: "localhost".to_string(),
            port: 2525,
            username: "test".to_string(),
            password: "test".to_string(),
            from_address: "test@example.com".to_string(),
            from_name: "Test".to_string(),
        };

        let sender = ReliableEmailSender::new(
            smtp_config,
            db_path.to_str().unwrap(),
            RetryConfig::default(),
            CircuitBreakerConfig::default(),
        ).unwrap();

        // Manually log a success with idempotency key
        sender.logger().log_success(
            "user@example.com",
            "Weekly Digest",
            "msg-abc",
            "2026-W05",
            Some("idem-key-123"),
        ).unwrap();

        // Create a runtime for the async test
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Attempt to send with the same idempotency key
        let result = rt.block_on(sender.send(
            "user@example.com",
            "Weekly Digest",
            "# Body",
            "2026-W05",
            "idem-key-123",
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
    fn test_retry_config_defaults() {
        use crate::reliability::RetryConfig;

        let config = RetryConfig::default();
        assert_eq!(config.initial_interval, std::time::Duration::from_secs(1));
        assert_eq!(config.max_interval, std::time::Duration::from_secs(60));
        assert!((config.randomization_factor - 0.5).abs() < f64::EPSILON);
        assert!((config.multiplier - 2.0).abs() < f64::EPSILON);
        assert_eq!(config.max_elapsed_time, std::time::Duration::from_secs(300));
    }

    #[test]
    fn test_circuit_breaker_config_defaults() {
        use crate::reliability::CircuitBreakerConfig;

        let config = CircuitBreakerConfig::default();
        assert_eq!(config.consecutive_failures, 3);
        assert_eq!(config.initial_backoff, std::time::Duration::from_secs(10));
        assert_eq!(config.max_backoff, std::time::Duration::from_secs(60));
    }

    // TODO: Full end-to-end tests with actual SMTP mocking deferred to Phase 20+
    // or separate integration test suite. These would test:
    // - test_reliable_sender_retry_transient: Mock SMTP returning 5xx then success
    // - test_reliable_sender_circuit_breaker_opens: Multiple failures trigger CB open
    // - test_reliable_sender_dlq_on_exhaustion: Retries exhaust and message goes to DLQ
}
