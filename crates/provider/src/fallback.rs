// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Fallback-aware provider wrappers.
//!
//! When a provider fails with a retryable error (rate limit, timeout, 5xx),
//! the fallback wrapper transparently tries the next candidate. Non-retryable
//! errors (auth, invalid request) fail immediately.
//!
//! # Design
//!
//! The core utility [`try_with_fallback`] is provider-agnostic — it works with
//! any list of candidates and any error type that exposes `is_retryable()`.
//! Thin wrappers like [`FallbackLlmProvider`] adapt this to specific traits.
//!
//! # Example
//!
//! ```ignore
//! use converge_provider::fallback::FallbackLlmProvider;
//!
//! let provider = FallbackLlmProvider::new(vec![
//!     create_provider("gemini", "gemini-3-flash-preview")?,
//!     create_provider("anthropic", "claude-haiku-4-5-20251001")?,
//!     create_provider("openai", "gpt-4o-mini")?,
//! ]);
//!
//! // Uses gemini first; if rate-limited, falls back to haiku, then gpt-4o-mini
//! let response = provider.complete(&request)?;
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};

/// Try an operation across a list of candidates, falling back on retryable errors.
///
/// Starts from `start_index` and tries each candidate in order. On retryable
/// errors, advances to the next candidate. On success or non-retryable error,
/// returns immediately.
///
/// Returns the result and the index of the candidate that produced it.
///
/// This function is provider-agnostic. It works with LLMs, search providers,
/// or any backend — the only requirement is that the error type has a way
/// to indicate retryability.
pub fn try_with_fallback<T, R, E>(
    candidates: &[T],
    start_index: usize,
    operation: impl Fn(&T) -> Result<R, E>,
    is_retryable: impl Fn(&E) -> bool,
) -> (Result<R, E>, usize) {
    let n = candidates.len();
    let mut last_error = None;
    let mut idx = start_index;

    for _ in 0..n {
        let candidate = &candidates[idx % n];
        match operation(candidate) {
            Ok(result) => return (Ok(result), idx % n),
            Err(e) => {
                if is_retryable(&e) {
                    last_error = Some(e);
                    idx += 1;
                } else {
                    // Non-retryable: fail immediately
                    return (Err(e), idx % n);
                }
            }
        }
    }

    // All candidates exhausted
    (
        Err(last_error.expect("at least one candidate must exist")),
        idx % n,
    )
}

// ── FallbackLlmProvider ──────────────────────────────────────────────

/// An `LlmProvider` that tries multiple providers in order, falling back
/// on retryable errors (rate limits, timeouts, network issues).
///
/// On success, remembers which provider worked and starts there next time
/// (sticky routing), avoiding repeated failures on the same dead provider.
pub struct FallbackLlmProvider {
    candidates: Vec<Arc<dyn LlmProvider>>,
    /// Index of the last successful provider — start here next time.
    current: AtomicUsize,
}

impl FallbackLlmProvider {
    /// Creates a fallback provider from an ordered list of candidates.
    ///
    /// The first candidate is the primary (best-scoring). Subsequent candidates
    /// are tried in order if the primary fails with a retryable error.
    ///
    /// # Panics
    ///
    /// Panics if `candidates` is empty.
    pub fn new(candidates: Vec<Arc<dyn LlmProvider>>) -> Self {
        assert!(
            !candidates.is_empty(),
            "FallbackLlmProvider requires at least one candidate"
        );
        Self {
            candidates,
            current: AtomicUsize::new(0),
        }
    }

    /// Returns the number of fallback candidates.
    #[must_use]
    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    /// Returns the currently active provider index.
    #[must_use]
    pub fn active_index(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }

    /// Returns a description of all candidates for logging.
    #[must_use]
    pub fn describe_candidates(&self) -> Vec<String> {
        self.candidates
            .iter()
            .map(|p| format!("{}/{}", p.name(), p.model()))
            .collect()
    }
}

impl LlmProvider for FallbackLlmProvider {
    fn name(&self) -> &'static str {
        // Return the primary's name — the wrapper is transparent
        self.candidates[self.current.load(Ordering::Relaxed)].name()
    }

    fn model(&self) -> &str {
        self.candidates[self.current.load(Ordering::Relaxed)].model()
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let start = self.current.load(Ordering::Relaxed);
        let (result, used_index) = try_with_fallback(
            &self.candidates,
            start,
            |provider| provider.complete(request),
            |e| e.retryable,
        );

        // Sticky routing: remember which provider worked
        if result.is_ok() && used_index != start {
            self.current.store(used_index, Ordering::Relaxed);
        }

        result
    }

    fn health_check(&self) -> Result<(), LlmError> {
        let start = self.current.load(Ordering::Relaxed);
        let (result, used_index) = try_with_fallback(
            &self.candidates,
            start,
            |provider| provider.health_check(),
            |e| e.retryable,
        );

        if result.is_ok() && used_index != start {
            self.current.store(used_index, Ordering::Relaxed);
        }

        result
    }
}

impl std::fmt::Debug for FallbackLlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let active = self.current.load(Ordering::Relaxed);
        f.debug_struct("FallbackLlmProvider")
            .field(
                "active",
                &format_args!(
                    "{}/{}",
                    self.candidates[active].name(),
                    self.candidates[active].model()
                ),
            )
            .field("candidates", &self.describe_candidates())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_with_fallback_returns_first_success() {
        let items = vec![1, 2, 3];
        let (result, idx) = try_with_fallback(
            &items,
            0,
            |&n| if n >= 1 { Ok(n * 10) } else { Err("fail") },
            |_| true,
        );
        assert_eq!(result, Ok(10));
        assert_eq!(idx, 0);
    }

    #[test]
    fn try_with_fallback_skips_retryable_errors() {
        let items = vec![1, 2, 3];
        let (result, idx) = try_with_fallback(
            &items,
            0,
            |&n| if n >= 3 { Ok(n * 10) } else { Err("retryable") },
            |_| true, // all errors are retryable
        );
        assert_eq!(result, Ok(30));
        assert_eq!(idx, 2);
    }

    #[test]
    fn try_with_fallback_stops_on_non_retryable() {
        let items = vec![1, 2, 3];
        let (result, idx) = try_with_fallback(
            &items,
            0,
            |&n| if n == 1 { Err("fatal") } else { Ok(n) },
            |_| false, // no errors are retryable
        );
        assert_eq!(result, Err("fatal"));
        assert_eq!(idx, 0);
    }

    #[test]
    fn try_with_fallback_wraps_around_from_start_index() {
        let items = vec![10, 20, 30];
        let (result, idx) = try_with_fallback(
            &items,
            2, // start at index 2
            |&n| if n == 10 { Ok(n) } else { Err("retry") },
            |_| true,
        );
        // Should try: 30 (fail), 10 (success)
        assert_eq!(result, Ok(10));
        assert_eq!(idx, 0);
    }

    #[test]
    fn try_with_fallback_all_fail() {
        let items = vec![1, 2, 3];
        let (result, _) = try_with_fallback(
            &items,
            0,
            |_: &i32| -> Result<i32, &str> { Err("all bad") },
            |_| true,
        );
        assert!(result.is_err());
    }
}
