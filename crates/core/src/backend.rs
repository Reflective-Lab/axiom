// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! LLM Backend Interface — The unification boundary for local and remote LLMs.
//!
//! # The Unification Rule
//!
//! All model invocations—local or remote—must produce the same top-level artifact:
//! - `BackendResponse` containing `ProposedContent`(s)
//! - Plus a `TraceLink` that makes the invocation auditable, budgeted, and comparable
//!
//! "Interchangeable" means:
//! - Same request type
//! - Same output type
//! - Same contract evaluation surface
//! - Different execution backend
//!
//! # Determinism Guarantees
//!
//! | Backend | Determinism | TraceLink |
//! |---------|-------------|-----------|
//! | Local (converge-llm) | Strong (replay-eligible) | `LocalTraceLink` |
//! | Remote (providers) | Bounded stochasticity (audit-eligible) | `RemoteTraceLink` |
//!
//! Remote runs are:
//! - **Auditable**: Full request/response + metadata
//! - **Repeatable-ish**: Best effort (temp=0 helps)
//! - **Non-replayable**: Strictly (model versions, safety layers can shift)
//!
//! # Example
//!
//! ```
//! use converge_core::backend::{
//!     LlmBackend, BackendCapability, BackendRequest, BackendResponse,
//!     BackendPrompt, BackendBudgets, BackendResult,
//! };
//!
//! // Both local and remote backends implement the same trait
//! fn process_with_any_backend<B: LlmBackend>(
//!     backend: &B,
//!     request: &BackendRequest,
//! ) -> BackendResult<BackendResponse> {
//!     // Check capabilities first
//!     if backend.supports_capability(BackendCapability::Replay) {
//!         println!("Using replay-eligible backend: {}", backend.name());
//!     }
//!     backend.execute(request)
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::kernel_boundary::{ProposedContent, TraceLink};

// ============================================================================
// Backend Error
// ============================================================================

/// Error type for backend operations.
///
/// This is capability-agnostic - implementations can wrap their specific errors.
///
/// # Retryable Errors
///
/// Some errors are transient and can be retried:
/// - `Timeout` - operation exceeded deadline but might succeed on retry
/// - `Unavailable` - backend temporarily unavailable
/// - `ExecutionFailed` - if caused by transient infrastructure issues
///
/// Use `is_retryable()` to check if an error should trigger retry logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    /// Request validation failed (NOT retryable - fix the request)
    InvalidRequest { message: String },
    /// Backend execution failed (may be retryable depending on cause)
    ExecutionFailed { message: String },
    /// Backend is unavailable (retryable - try again later)
    Unavailable { message: String },
    /// Budget exceeded (NOT retryable - increase budget or reduce request)
    BudgetExceeded { resource: String, limit: String },
    /// Contract validation failed (NOT retryable - output doesn't match contract)
    ContractFailed { contract: String, message: String },
    /// Capability not supported (NOT retryable - use different backend)
    UnsupportedCapability { capability: BackendCapability },
    /// Adapter not found or incompatible (NOT retryable - fix configuration)
    AdapterError { message: String },
    /// Recall operation failed (may be retryable)
    RecallError { message: String },
    /// Operation timed out (retryable - might succeed with more time)
    Timeout {
        /// Configured deadline in milliseconds
        deadline_ms: u64,
        /// Actual elapsed time in milliseconds
        elapsed_ms: u64,
    },
    /// Circuit breaker is open (NOT retryable until circuit closes)
    CircuitOpen {
        /// Name of the backend with open circuit
        backend: String,
        /// When the circuit will transition to half-open (Unix timestamp ms)
        retry_after_ms: Option<u64>,
    },
    /// Retryable wrapper - indicates retry was attempted
    Retried {
        /// The final error after all retries exhausted
        message: String,
        /// Number of attempts made
        attempts: usize,
        /// Whether the underlying error was transient
        was_transient: bool,
    },
    /// Generic error with context
    Other { message: String },
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest { message } => write!(f, "Invalid request: {}", message),
            Self::ExecutionFailed { message } => write!(f, "Execution failed: {}", message),
            Self::Unavailable { message } => write!(f, "Backend unavailable: {}", message),
            Self::BudgetExceeded { resource, limit } => {
                write!(f, "Budget exceeded: {} (limit: {})", resource, limit)
            }
            Self::ContractFailed { contract, message } => {
                write!(f, "Contract '{}' failed: {}", contract, message)
            }
            Self::UnsupportedCapability { capability } => {
                write!(f, "Unsupported capability: {:?}", capability)
            }
            Self::AdapterError { message } => write!(f, "Adapter error: {}", message),
            Self::RecallError { message } => write!(f, "Recall error: {}", message),
            Self::Timeout {
                deadline_ms,
                elapsed_ms,
            } => {
                write!(
                    f,
                    "Operation timed out: elapsed {}ms, deadline {}ms",
                    elapsed_ms, deadline_ms
                )
            }
            Self::CircuitOpen {
                backend,
                retry_after_ms,
            } => {
                if let Some(retry_after) = retry_after_ms {
                    write!(
                        f,
                        "Circuit breaker open for '{}', retry after {}ms",
                        backend, retry_after
                    )
                } else {
                    write!(f, "Circuit breaker open for '{}'", backend)
                }
            }
            Self::Retried {
                message,
                attempts,
                was_transient,
            } => {
                write!(
                    f,
                    "Failed after {} attempts (transient: {}): {}",
                    attempts, was_transient, message
                )
            }
            Self::Other { message } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for BackendError {}

impl BackendError {
    /// Check if this error is retryable.
    ///
    /// Retryable errors are transient and might succeed on retry:
    /// - Timeout
    /// - Unavailable
    /// - Some ExecutionFailed (network issues, rate limits)
    ///
    /// Non-retryable errors require intervention:
    /// - InvalidRequest (fix the request)
    /// - BudgetExceeded (increase budget)
    /// - ContractFailed (output doesn't match)
    /// - UnsupportedCapability (use different backend)
    /// - CircuitOpen (wait for circuit to close)
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } => true,
            Self::Unavailable { .. } => true,
            Self::ExecutionFailed { message } => {
                // Heuristic: network/rate limit errors are retryable
                let msg_lower = message.to_lowercase();
                msg_lower.contains("timeout")
                    || msg_lower.contains("rate limit")
                    || msg_lower.contains("429")
                    || msg_lower.contains("503")
                    || msg_lower.contains("502")
                    || msg_lower.contains("504")
                    || msg_lower.contains("connection")
                    || msg_lower.contains("network")
            }
            Self::RecallError { message } => {
                // Recall errors might be transient (embedding service down)
                let msg_lower = message.to_lowercase();
                msg_lower.contains("timeout") || msg_lower.contains("unavailable")
            }
            // Not retryable
            Self::InvalidRequest { .. } => false,
            Self::BudgetExceeded { .. } => false,
            Self::ContractFailed { .. } => false,
            Self::UnsupportedCapability { .. } => false,
            Self::AdapterError { .. } => false,
            Self::CircuitOpen { .. } => false, // Must wait for circuit to close
            Self::Retried { .. } => false,     // Already retried
            Self::Other { .. } => false,
        }
    }

    /// Check if this error indicates the backend is overloaded.
    ///
    /// Used by circuit breakers to track failure patterns.
    #[must_use]
    pub fn is_overload(&self) -> bool {
        match self {
            Self::Unavailable { .. } => true,
            Self::Timeout { .. } => true,
            Self::ExecutionFailed { message } => {
                let msg_lower = message.to_lowercase();
                msg_lower.contains("rate limit")
                    || msg_lower.contains("429")
                    || msg_lower.contains("503")
                    || msg_lower.contains("overloaded")
            }
            _ => false,
        }
    }
}

/// Result type for backend operations.
pub type BackendResult<T> = Result<T, BackendError>;

// ============================================================================
// Backend Capability
// ============================================================================

/// Backend capabilities for routing decisions.
///
/// These capabilities determine what a backend can do and influence
/// which backend is selected for a given request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackendCapability {
    /// Deterministic replay - same inputs produce identical outputs
    Replay,
    /// LoRA adapters for task-specific tuning
    Adapters,
    /// Recall injection from corpus
    Recall,
    /// Step-level contract validation
    StepContracts,
    /// Frontier reasoning capabilities (Claude Opus, GPT-4, etc.)
    FrontierReasoning,
    /// Fast iteration for interactive use
    FastIteration,
    /// Offline operation (no network required)
    Offline,
    /// Streaming output
    Streaming,
    /// Vision/multimodal input
    Vision,
    /// Tool use / function calling
    ToolUse,
}

// ============================================================================
// Retry Policy
// ============================================================================

/// Backoff strategy for retries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linear increase: delay * attempt
    Linear,
    /// Exponential increase: delay * 2^attempt
    Exponential,
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential
    }
}

/// Configuration for retry behavior.
///
/// # Example
///
/// ```
/// use converge_core::backend::{RetryPolicy, BackoffStrategy};
///
/// // Retry up to 3 times with exponential backoff starting at 100ms
/// let policy = RetryPolicy {
///     max_attempts: 3,
///     initial_delay_ms: 100,
///     max_delay_ms: 5000,
///     backoff: BackoffStrategy::Exponential,
///     jitter_percent: 20,
/// };
///
/// assert_eq!(policy.delay_for_attempt(1), 100); // First retry: 100ms
/// // Second retry: ~200ms, Third retry: ~400ms (plus jitter)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including initial attempt)
    pub max_attempts: usize,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay cap in milliseconds
    pub max_delay_ms: u64,
    /// Backoff strategy
    pub backoff: BackoffStrategy,
    /// Jitter percentage (0-100) to add randomness to delays
    pub jitter_percent: u8,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff: BackoffStrategy::Exponential,
            jitter_percent: 20,
        }
    }
}

impl RetryPolicy {
    /// Create a policy that never retries.
    #[must_use]
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            ..Default::default()
        }
    }

    /// Create an aggressive retry policy for critical operations.
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 50,
            max_delay_ms: 30_000,
            backoff: BackoffStrategy::Exponential,
            jitter_percent: 25,
        }
    }

    /// Calculate the delay for a given attempt number (1-indexed).
    ///
    /// Does not include jitter - caller should add jitter separately.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: usize) -> u64 {
        if attempt == 0 {
            return 0;
        }
        let attempt = attempt.saturating_sub(1); // Convert to 0-indexed for calculation

        let delay = match self.backoff {
            BackoffStrategy::Fixed => self.initial_delay_ms,
            BackoffStrategy::Linear => self.initial_delay_ms.saturating_mul(attempt as u64 + 1),
            BackoffStrategy::Exponential => self
                .initial_delay_ms
                .saturating_mul(1u64 << attempt.min(10)),
        };

        delay.min(self.max_delay_ms)
    }

    /// Check if another attempt should be made.
    #[must_use]
    pub fn should_retry(&self, attempt: usize) -> bool {
        attempt < self.max_attempts
    }
}

// ============================================================================
// Circuit Breaker
// ============================================================================

/// Configuration for circuit breaker behavior.
///
/// Circuit breakers protect against cascading failures by stopping requests
/// to a failing backend until it recovers.
///
/// # States
///
/// - **Closed**: Normal operation, requests pass through
/// - **Open**: Backend is failing, requests fast-fail immediately
/// - **Half-Open**: Testing if backend recovered, limited requests allowed
///
/// # Example
///
/// ```
/// use converge_core::backend::CircuitBreakerConfig;
///
/// let config = CircuitBreakerConfig {
///     failure_threshold: 5,
///     success_threshold: 2,
///     timeout_ms: 30_000,
///     half_open_max_requests: 3,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before circuit opens
    pub failure_threshold: usize,
    /// Number of successes in half-open state before closing
    pub success_threshold: usize,
    /// Time in milliseconds before transitioning from open to half-open
    pub timeout_ms: u64,
    /// Maximum requests allowed in half-open state
    pub half_open_max_requests: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_ms: 30_000,
            half_open_max_requests: 3,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a sensitive circuit breaker that opens quickly.
    #[must_use]
    pub fn sensitive() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 1,
            timeout_ms: 15_000,
            half_open_max_requests: 1,
        }
    }

    /// Create a tolerant circuit breaker that allows more failures.
    #[must_use]
    pub fn tolerant() -> Self {
        Self {
            failure_threshold: 10,
            success_threshold: 3,
            timeout_ms: 60_000,
            half_open_max_requests: 5,
        }
    }

    /// Disable circuit breaker (never opens).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            failure_threshold: usize::MAX,
            success_threshold: 1,
            timeout_ms: 0,
            half_open_max_requests: usize::MAX,
        }
    }
}

/// Current state of a circuit breaker.
///
/// This is runtime state, not configuration. Implementations track this
/// per-backend to manage circuit breaker behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    #[default]
    Closed,
    /// Backend failing - requests fast-fail
    Open,
    /// Testing recovery - limited requests allowed
    HalfOpen,
}

// ============================================================================
// Backend Request
// ============================================================================

/// A unified request to any LLM backend.
///
/// Both local kernel and remote providers accept this same request type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendRequest {
    /// Intent identifier for tracking
    pub intent_id: String,
    /// Truth targets this invocation aims to satisfy
    pub truth_ids: Vec<String>,
    /// Prompt version for reproducibility
    pub prompt_version: String,
    /// Hash of the state injection (for audit)
    pub state_injection_hash: String,
    /// The actual prompt/messages to send
    pub prompt: BackendPrompt,
    /// Contracts to validate against
    pub contracts: Vec<ContractSpec>,
    /// Resource budgets
    pub budgets: BackendBudgets,
    /// Recall policy (optional, local-only capability)
    pub recall_policy: Option<BackendRecallPolicy>,
    /// Adapter policy (optional, local-only capability)
    pub adapter_policy: Option<BackendAdapterPolicy>,
    /// Retry policy (optional, overrides backend default)
    #[serde(default)]
    pub retry_policy: Option<RetryPolicy>,
}

/// The prompt content for the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendPrompt {
    /// Simple text prompt
    Text(String),
    /// Chat-style messages
    Messages(Vec<Message>),
}

/// A chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Message role in chat format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Contract specification for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSpec {
    /// Contract/truth name
    pub name: String,
    /// Expected output schema (JSON Schema)
    pub schema: Option<serde_json::Value>,
    /// Whether this contract is required to pass
    pub required: bool,
}

/// Resource budgets for the invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendBudgets {
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Maximum iterations (for multi-step)
    pub max_iterations: usize,
    /// Latency ceiling in milliseconds (0 = no limit)
    pub latency_ceiling_ms: u64,
    /// Maximum cost in microdollars (0 = no limit)
    pub cost_ceiling_microdollars: u64,
}

impl Default for BackendBudgets {
    fn default() -> Self {
        Self {
            max_tokens: 1024,
            max_iterations: 1,
            latency_ceiling_ms: 0,
            cost_ceiling_microdollars: 0,
        }
    }
}

/// Recall policy for backend requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendRecallPolicy {
    pub enabled: bool,
    pub max_candidates: usize,
    pub min_score: f32,
    pub corpus_filter: Option<String>,
}

impl Default for BackendRecallPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            max_candidates: 5,
            min_score: 0.5,
            corpus_filter: None,
        }
    }
}

/// Adapter policy for backend requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendAdapterPolicy {
    /// Explicit adapter ID (authority from outside)
    pub adapter_id: Option<String>,
    /// Whether adapter is required (fail if not available)
    pub required: bool,
}

impl Default for BackendAdapterPolicy {
    fn default() -> Self {
        Self {
            adapter_id: None,
            required: false,
        }
    }
}

// ============================================================================
// Backend Response
// ============================================================================

/// A unified response from any LLM backend.
///
/// Both local kernel and remote providers return this same response type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendResponse {
    /// The generated proposals
    pub proposals: Vec<ProposedContent>,
    /// Contract validation report
    pub contract_report: ContractReport,
    /// Trace link (backend-specific but normalized interface)
    pub trace_link: TraceLink,
    /// Resource usage
    pub usage: BackendUsage,
}

/// Contract validation report for backend responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractReport {
    /// Per-contract results
    pub results: Vec<BackendContractResult>,
    /// Overall pass/fail
    pub all_passed: bool,
}

impl ContractReport {
    /// Create an empty passing report.
    pub fn empty_pass() -> Self {
        Self {
            results: vec![],
            all_passed: true,
        }
    }

    /// Create a report from results.
    pub fn from_results(results: Vec<BackendContractResult>) -> Self {
        let all_passed = results.iter().all(|r| r.passed);
        Self {
            results,
            all_passed,
        }
    }
}

/// Result of a single contract check (backend-level).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendContractResult {
    pub name: String,
    pub passed: bool,
    pub diagnostics: Option<String>,
}

impl BackendContractResult {
    /// Create a passing result.
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            diagnostics: None,
        }
    }

    /// Create a failing result with diagnostics.
    pub fn fail(name: impl Into<String>, diagnostics: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            diagnostics: Some(diagnostics.into()),
        }
    }
}

/// Resource usage from the invocation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub latency_ms: u64,
    pub cost_microdollars: Option<u64>,
}

// ============================================================================
// LlmBackend Trait
// ============================================================================

/// The unified backend interface.
///
/// Both local kernel (converge-llm) and remote providers (converge-provider)
/// implement this trait, making "local vs remote" genuinely interchangeable.
///
/// # Implementors
///
/// - `LlamaEngine` (converge-llm) - Local inference with Burn/llama-burn
/// - `AnthropicBackend` (converge-provider) - Remote Claude API
/// - Future: OpenAIBackend, CohereReranker, etc.
///
/// # Thread Safety
///
/// Backends must be `Send + Sync` to support concurrent request handling.
///
/// # Deprecation Notice
///
/// This trait is deprecated in favor of the capability boundary traits in
/// `converge_core::traits`:
///
/// - [`ChatBackend`](crate::traits::ChatBackend) - For chat completion (GAT async)
/// - [`EmbedBackend`](crate::traits::EmbedBackend) - For embedding generation (GAT async)
/// - [`LlmBackend`](crate::traits::LlmBackend) - Umbrella combining both
///
/// The new traits use the GAT async pattern for zero-cost async without
/// `async_trait`. See `converge-core/BOUNDARY.md` for migration guide.
#[deprecated(
    since = "0.2.0",
    note = "Use converge_core::traits::LlmBackend (GAT async) instead. See BOUNDARY.md for migration."
)]
pub trait LlmBackend: Send + Sync {
    /// Backend name for identification and routing.
    fn name(&self) -> &str;

    /// Whether this backend supports deterministic replay.
    ///
    /// - Local backends with fixed seeds: `true`
    /// - Remote backends: `false` (model versions can change)
    fn supports_replay(&self) -> bool;

    /// Execute an LLM request.
    ///
    /// This is the core interface. Implementations handle:
    /// - Prompt formatting
    /// - Model invocation
    /// - Contract validation
    /// - Trace link generation
    fn execute(&self, request: &BackendRequest) -> BackendResult<BackendResponse>;

    /// Check if this backend supports a specific capability.
    ///
    /// Used by routing policies to select appropriate backends.
    fn supports_capability(&self, capability: BackendCapability) -> bool;

    /// List all capabilities this backend supports.
    ///
    /// Default implementation checks each capability individually.
    fn capabilities(&self) -> Vec<BackendCapability> {
        let all_caps = [
            BackendCapability::Replay,
            BackendCapability::Adapters,
            BackendCapability::Recall,
            BackendCapability::StepContracts,
            BackendCapability::FrontierReasoning,
            BackendCapability::FastIteration,
            BackendCapability::Offline,
            BackendCapability::Streaming,
            BackendCapability::Vision,
            BackendCapability::ToolUse,
        ];
        all_caps
            .iter()
            .filter(|cap| self.supports_capability(**cap))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_budgets_default() {
        let budgets = BackendBudgets::default();
        assert_eq!(budgets.max_tokens, 1024);
        assert_eq!(budgets.max_iterations, 1);
        assert_eq!(budgets.latency_ceiling_ms, 0);
        assert_eq!(budgets.cost_ceiling_microdollars, 0);
    }

    #[test]
    fn test_message_constructors() {
        let system = Message::system("You are a helpful assistant");
        assert_eq!(system.role, MessageRole::System);
        assert_eq!(system.content, "You are a helpful assistant");

        let user = Message::user("Hello");
        assert_eq!(user.role, MessageRole::User);

        let assistant = Message::assistant("Hi there!");
        assert_eq!(assistant.role, MessageRole::Assistant);
    }

    #[test]
    fn test_contract_report_from_results() {
        let results = vec![
            BackendContractResult::pass("contract1"),
            BackendContractResult::pass("contract2"),
        ];
        let report = ContractReport::from_results(results);
        assert!(report.all_passed);

        let mixed = vec![
            BackendContractResult::pass("contract1"),
            BackendContractResult::fail("contract2", "missing field"),
        ];
        let report = ContractReport::from_results(mixed);
        assert!(!report.all_passed);
    }

    #[test]
    fn test_backend_error_display() {
        let err = BackendError::BudgetExceeded {
            resource: "tokens".to_string(),
            limit: "1024".to_string(),
        };
        assert!(err.to_string().contains("tokens"));
        assert!(err.to_string().contains("1024"));
    }

    #[test]
    fn test_capability_serialization_stable() {
        assert_eq!(
            serde_json::to_string(&BackendCapability::Replay).unwrap(),
            "\"Replay\""
        );
        assert_eq!(
            serde_json::to_string(&BackendCapability::FrontierReasoning).unwrap(),
            "\"FrontierReasoning\""
        );
    }

    #[test]
    fn test_message_role_serialization_stable() {
        assert_eq!(
            serde_json::to_string(&MessageRole::System).unwrap(),
            "\"System\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::User).unwrap(),
            "\"User\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::Assistant).unwrap(),
            "\"Assistant\""
        );
    }

    // =========================================================================
    // Retry Policy Tests
    // =========================================================================

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_delay_ms, 100);
        assert_eq!(policy.backoff, BackoffStrategy::Exponential);
    }

    #[test]
    fn test_retry_policy_no_retry() {
        let policy = RetryPolicy::no_retry();
        assert_eq!(policy.max_attempts, 1);
        assert!(!policy.should_retry(1));
    }

    #[test]
    fn test_retry_policy_delay_exponential() {
        let policy = RetryPolicy {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff: BackoffStrategy::Exponential,
            jitter_percent: 0,
        };

        assert_eq!(policy.delay_for_attempt(1), 100); // 100 * 2^0
        assert_eq!(policy.delay_for_attempt(2), 200); // 100 * 2^1
        assert_eq!(policy.delay_for_attempt(3), 400); // 100 * 2^2
        assert_eq!(policy.delay_for_attempt(4), 800); // 100 * 2^3
    }

    #[test]
    fn test_retry_policy_delay_linear() {
        let policy = RetryPolicy {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff: BackoffStrategy::Linear,
            jitter_percent: 0,
        };

        assert_eq!(policy.delay_for_attempt(1), 100); // 100 * 1
        assert_eq!(policy.delay_for_attempt(2), 200); // 100 * 2
        assert_eq!(policy.delay_for_attempt(3), 300); // 100 * 3
    }

    #[test]
    fn test_retry_policy_delay_fixed() {
        let policy = RetryPolicy {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff: BackoffStrategy::Fixed,
            jitter_percent: 0,
        };

        assert_eq!(policy.delay_for_attempt(1), 100);
        assert_eq!(policy.delay_for_attempt(2), 100);
        assert_eq!(policy.delay_for_attempt(3), 100);
    }

    #[test]
    fn test_retry_policy_max_delay_cap() {
        let policy = RetryPolicy {
            max_attempts: 20,
            initial_delay_ms: 1000,
            max_delay_ms: 5000,
            backoff: BackoffStrategy::Exponential,
            jitter_percent: 0,
        };

        // Exponential would be 1000 * 2^9 = 512000, but capped at 5000
        assert_eq!(policy.delay_for_attempt(10), 5000);
    }

    #[test]
    fn test_retry_policy_should_retry() {
        let policy = RetryPolicy {
            max_attempts: 3,
            ..Default::default()
        };

        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));
        assert!(!policy.should_retry(4));
    }

    // =========================================================================
    // Circuit Breaker Tests
    // =========================================================================

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 2);
        assert_eq!(config.timeout_ms, 30_000);
    }

    #[test]
    fn test_circuit_breaker_config_sensitive() {
        let config = CircuitBreakerConfig::sensitive();
        assert_eq!(config.failure_threshold, 3);
        assert!(config.failure_threshold < CircuitBreakerConfig::default().failure_threshold);
    }

    #[test]
    fn test_circuit_breaker_config_tolerant() {
        let config = CircuitBreakerConfig::tolerant();
        assert_eq!(config.failure_threshold, 10);
        assert!(config.failure_threshold > CircuitBreakerConfig::default().failure_threshold);
    }

    #[test]
    fn test_circuit_state_default() {
        let state = CircuitState::default();
        assert_eq!(state, CircuitState::Closed);
    }

    // =========================================================================
    // Backend Error Retryable Tests
    // =========================================================================

    #[test]
    fn test_timeout_is_retryable() {
        let err = BackendError::Timeout {
            deadline_ms: 5000,
            elapsed_ms: 5001,
        };
        assert!(err.is_retryable());
        assert!(err.is_overload());
    }

    #[test]
    fn test_unavailable_is_retryable() {
        let err = BackendError::Unavailable {
            message: "Service temporarily unavailable".to_string(),
        };
        assert!(err.is_retryable());
        assert!(err.is_overload());
    }

    #[test]
    fn test_rate_limit_is_retryable() {
        let err = BackendError::ExecutionFailed {
            message: "Rate limit exceeded (429)".to_string(),
        };
        assert!(err.is_retryable());
        assert!(err.is_overload());
    }

    #[test]
    fn test_invalid_request_not_retryable() {
        let err = BackendError::InvalidRequest {
            message: "Missing required field".to_string(),
        };
        assert!(!err.is_retryable());
        assert!(!err.is_overload());
    }

    #[test]
    fn test_budget_exceeded_not_retryable() {
        let err = BackendError::BudgetExceeded {
            resource: "tokens".to_string(),
            limit: "1024".to_string(),
        };
        assert!(!err.is_retryable());
        assert!(!err.is_overload());
    }

    #[test]
    fn test_circuit_open_not_retryable() {
        let err = BackendError::CircuitOpen {
            backend: "anthropic".to_string(),
            retry_after_ms: Some(30_000),
        };
        assert!(!err.is_retryable());
        assert!(!err.is_overload());
    }

    #[test]
    fn test_timeout_error_display() {
        let err = BackendError::Timeout {
            deadline_ms: 5000,
            elapsed_ms: 6000,
        };
        let msg = err.to_string();
        assert!(msg.contains("6000"));
        assert!(msg.contains("5000"));
    }

    #[test]
    fn test_circuit_open_error_display() {
        let err = BackendError::CircuitOpen {
            backend: "test-backend".to_string(),
            retry_after_ms: Some(30_000),
        };
        let msg = err.to_string();
        assert!(msg.contains("test-backend"));
        assert!(msg.contains("30000"));
    }

    #[test]
    fn test_retried_error_display() {
        let err = BackendError::Retried {
            message: "Final error".to_string(),
            attempts: 3,
            was_transient: true,
        };
        let msg = err.to_string();
        assert!(msg.contains("3 attempts"));
        assert!(msg.contains("transient: true"));
    }

    // =========================================================================
    // Serialization Stability Tests
    // =========================================================================

    #[test]
    fn test_retry_policy_serialization_stable() {
        let policy = RetryPolicy::default();
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("\"max_attempts\":3"));
        assert!(json.contains("\"Exponential\""));

        // Round-trip
        let parsed: RetryPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, policy);
    }

    #[test]
    fn test_circuit_breaker_config_serialization_stable() {
        let config = CircuitBreakerConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"failure_threshold\":5"));

        // Round-trip
        let parsed: CircuitBreakerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }
}
