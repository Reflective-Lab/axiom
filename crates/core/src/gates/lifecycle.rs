// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Generic proposal lifecycle trait.
//!
//! The `ProposalLifecycle<I, P, V, F>` trait defines the contract for
//! transforming intents into facts through validation and promotion.
//!
//! # Type Parameters
//!
//! - `I`: Intent type (what we're trying to achieve)
//! - `P`: Proposal type (what agents suggest)
//! - `V`: Validation proof type (evidence validation occurred)
//! - `F`: Fact type (promoted result)
//!
//! # Design Axiom
//!
//! **"Agents suggest, engine decides"** - proposals become facts only
//! through explicit validation and promotion via this trait.
//!
//! # The Gate Pattern
//!
//! The lifecycle enforces a two-step process:
//!
//! 1. **Validation**: `validate(intent, proposal) -> Result<V, ValidationError>`
//!    - Checks proposal against policy and intent
//!    - Returns a proof object (typically wrapping a `ValidationReport`)
//!    - This proof is required for promotion
//!
//! 2. **Promotion**: `promote(validated) -> Result<F, PromotionError>`
//!    - Consumes the validation proof
//!    - Creates the final fact with complete audit trail
//!    - No bypass path - must have proof
//!
//! # Example Implementation
//!
//! ```ignore
//! impl ProposalLifecycle<Intent, Proposal<Draft>, ValidatedProposal, Fact>
//!     for MyGate
//! {
//!     fn validate(&self, intent: &Intent, proposal: Proposal<Draft>)
//!         -> Result<ValidatedProposal, ValidationError>
//!     {
//!         // Run validation checks
//!         // Return proof bundled with validated proposal
//!     }
//!
//!     fn promote(&self, validated: ValidatedProposal)
//!         -> Result<Fact, PromotionError>
//!     {
//!         // Extract report and proposal from proof
//!         // Create Fact with PromotionRecord
//!     }
//! }
//! ```
//!
//! # Relationship to PromotionGate
//!
//! `PromotionGate` is the concrete implementation of this trait for
//! Converge's standard proposal-to-fact flow. Custom gates can implement
//! this trait with different type parameters for specialized domains.

use super::validation::ValidationError;
use crate::types::PromotionError;

/// Generic lifecycle for proposal-to-fact promotion.
///
/// This trait defines the contract for transforming proposals into facts
/// through explicit validation and promotion. The key invariant is that
/// promotion requires validation proof - there is no bypass path.
///
/// # Type Parameters
///
/// - `I`: Intent type - defines what we're trying to achieve. The intent
///   provides context for validation decisions.
///
/// - `P`: Proposal type - what agents suggest. This is the input that
///   needs to be validated before it can become a fact.
///
/// - `V`: Validation proof type - evidence that validation occurred.
///   Typically bundles a `ValidationReport` with the validated proposal.
///   This type ensures validation cannot be skipped.
///
/// - `F`: Fact type - the promoted result. Facts are immutable, governed
///   truths that can only be created through this lifecycle.
///
/// # Invariants
///
/// - **No bypass path**: `promote()` requires validation proof `V`
/// - **Proof consumption**: The proof is consumed by `promote()`
/// - **Complete audit**: The resulting fact has a `PromotionRecord`
/// - **Determinism**: Same input should produce same validation result
pub trait ProposalLifecycle<I, P, V, F> {
    /// Validate a proposal against intent and policy.
    ///
    /// This method runs validation checks on the proposal, considering
    /// the intent context and current policy. If validation passes,
    /// it returns a proof object that must be consumed to promote.
    ///
    /// The proof type `V` typically contains or wraps a `ValidationReport`
    /// along with the validated proposal, ensuring they cannot be separated.
    ///
    /// # Arguments
    ///
    /// - `intent`: The intent providing context for validation decisions
    /// - `proposal`: The proposal to validate (consumed)
    ///
    /// # Returns
    ///
    /// - `Ok(V)`: Validation proof if validation passes
    /// - `Err(ValidationError)`: If validation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let validated = gate.validate(&intent, proposal)?;
    /// // validated now contains both the report and validated proposal
    /// ```
    fn validate(&self, intent: &I, proposal: P) -> Result<V, ValidationError>;

    /// Promote a validated proposal to a fact.
    ///
    /// This method consumes the validation proof and creates a fact.
    /// The proof contains everything needed for promotion, including
    /// the validated proposal and validation report.
    ///
    /// # Arguments
    ///
    /// - `validated`: The validation proof (consumed)
    ///
    /// # Returns
    ///
    /// - `Ok(F)`: The promoted fact with complete audit trail
    /// - `Err(PromotionError)`: If promotion fails despite valid proof
    ///
    /// # Errors
    ///
    /// Promotion can fail even with valid proof due to:
    /// - Policy version mismatch during promotion
    /// - Evidence verification failure
    /// - Gate rejection based on runtime conditions
    ///
    /// # Example
    ///
    /// ```ignore
    /// let validated = gate.validate(&intent, proposal)?;
    /// let fact = gate.promote(validated)?;
    /// // fact now has PromotionRecord with complete audit trail
    /// ```
    fn promote(&self, validated: V) -> Result<F, PromotionError>;
}

#[cfg(test)]
mod tests {
    // Note: The trait itself has no tests - it defines a contract.
    // Tests for concrete implementations are in promotion.rs.
    //
    // This test verifies the trait is object-safe and can be used
    // as a trait object if needed.

    use super::*;

    // Marker types for testing object safety
    struct TestIntent;
    struct TestProposal;
    struct TestValidated;
    struct TestFact;

    // The trait should be object-safe for these simple types
    // (though in practice we use static dispatch)
    fn _assert_object_safe(
        _gate: &dyn ProposalLifecycle<TestIntent, TestProposal, TestValidated, TestFact>,
    ) {
        // This function just needs to compile
    }
}
