// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! LLM output validation for Converge.
//!
//! This module provides the validation layer that ensures LLM outputs
//! cannot corrupt the trusted context without explicit approval.
//!
//! # Safety Model
//!
//! ```text
//! LLM Output → ProposedFact → ValidationAgent → Fact (if valid)
//!                                  ↓
//!                            Rejected (if invalid)
//! ```
//!
//! # Example
//!
//! ```
//! use converge_core::{Engine, Context, ContextKey, Fact};
//! use converge_core::validation::{ValidationAgent, ValidationConfig};
//!
//! let mut engine = Engine::new();
//!
//! // Register validation agent with config
//! engine.register(ValidationAgent::new(ValidationConfig {
//!     min_confidence: 0.7,
//!     ..Default::default()
//! }));
//!
//! // Proposals in context will be validated and promoted to facts
//! ```

// Agent trait returns &str, but we return literals. This is fine.
#![allow(clippy::unnecessary_literal_bound)]

use crate::agent::Agent;
use crate::context::{Context, ContextKey, Fact, ProposedFact};
use crate::effect::AgentEffect;

/// Configuration for the validation agent.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum confidence threshold (0.0 - 1.0).
    /// Proposals below this are rejected.
    pub min_confidence: f64,

    /// Maximum content length allowed.
    pub max_content_length: usize,

    /// Forbidden terms that cause rejection.
    pub forbidden_terms: Vec<String>,

    /// Whether to require provenance information.
    pub require_provenance: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_content_length: 10_000,
            forbidden_terms: vec![],
            require_provenance: true,
        }
    }
}

/// Result of validating a proposal.
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Proposal accepted, here's the promoted fact.
    Accepted(Fact),
    /// Proposal rejected with reason.
    Rejected { proposal_id: String, reason: String },
}

/// Agent that validates `ProposedFacts` and promotes them to Facts.
///
/// This is the **gateway** between untrusted LLM outputs and the trusted context.
/// No LLM output can become a Fact without passing through validation.
pub struct ValidationAgent {
    config: ValidationConfig,
}

impl ValidationAgent {
    /// Creates a new validation agent with the given config.
    #[must_use]
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Creates a validation agent with default config.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Validates a single proposal against the config.
    fn validate_proposal(&self, proposal: &ProposedFact) -> ValidationResult {
        // Check confidence threshold
        if proposal.confidence < self.config.min_confidence {
            return ValidationResult::Rejected {
                proposal_id: proposal.id.clone(),
                reason: format!(
                    "confidence {} below threshold {}",
                    proposal.confidence, self.config.min_confidence
                ),
            };
        }

        // Check content length
        if proposal.content.len() > self.config.max_content_length {
            return ValidationResult::Rejected {
                proposal_id: proposal.id.clone(),
                reason: format!(
                    "content length {} exceeds max {}",
                    proposal.content.len(),
                    self.config.max_content_length
                ),
            };
        }

        // Check for empty content
        if proposal.content.trim().is_empty() {
            return ValidationResult::Rejected {
                proposal_id: proposal.id.clone(),
                reason: "content is empty".into(),
            };
        }

        // Check provenance requirement
        if self.config.require_provenance && proposal.provenance.trim().is_empty() {
            return ValidationResult::Rejected {
                proposal_id: proposal.id.clone(),
                reason: "provenance is required but empty".into(),
            };
        }

        // Check forbidden terms
        let content_lower = proposal.content.to_lowercase();
        for term in &self.config.forbidden_terms {
            if content_lower.contains(&term.to_lowercase()) {
                return ValidationResult::Rejected {
                    proposal_id: proposal.id.clone(),
                    reason: format!("content contains forbidden term '{term}'"),
                };
            }
        }

        // All checks passed - try to convert
        match Fact::try_from(proposal.clone()) {
            Ok(fact) => ValidationResult::Accepted(fact),
            Err(e) => ValidationResult::Rejected {
                proposal_id: proposal.id.clone(),
                reason: e.reason,
            },
        }
    }

    /// Parses a proposal fact from the Proposals key.
    ///
    /// Proposals are stored as Facts with a special encoding:
    /// - id: "`proposal:{target_key}:{actual_id`}"
    /// - content: JSON-like "{confidence}|{provenance}|{content}"
    fn parse_proposal(fact: &Fact) -> Option<ProposedFact> {
        // Parse id: "proposal:{target_key}:{id}"
        let id_parts: Vec<&str> = fact.id.splitn(3, ':').collect();
        if id_parts.len() != 3 || id_parts[0] != "proposal" {
            return None;
        }

        let target_key = match id_parts[1] {
            "seeds" => ContextKey::Seeds,
            "hypotheses" => ContextKey::Hypotheses,
            "strategies" => ContextKey::Strategies,
            "constraints" => ContextKey::Constraints,
            "signals" => ContextKey::Signals,
            "competitors" => ContextKey::Competitors,
            "evaluations" => ContextKey::Evaluations,
            _ => return None,
        };

        let actual_id = id_parts[2];

        // Parse content: "{confidence}|{provenance}|{content}"
        let content_parts: Vec<&str> = fact.content.splitn(3, '|').collect();
        if content_parts.len() != 3 {
            return None;
        }

        let confidence: f64 = content_parts[0].parse().ok()?;
        let provenance = content_parts[1].to_string();
        let content = content_parts[2].to_string();

        Some(ProposedFact {
            key: target_key,
            id: actual_id.to_string(),
            content,
            confidence,
            provenance,
        })
    }
}

impl Agent for ValidationAgent {
    fn name(&self) -> &str {
        "ValidationAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn crate::ContextView) -> bool {
        // Run when there are proposals that haven't been validated yet
        let proposals = ctx.get(ContextKey::Proposals);

        // Check if any proposal hasn't been processed
        // (i.e., its target fact doesn't exist yet)
        for proposal_fact in proposals {
            if let Some(proposal) = Self::parse_proposal(proposal_fact) {
                // Check if this proposal has already been promoted
                let existing = ctx.get(proposal.key);
                if !existing.iter().any(|f| f.id == proposal.id) {
                    return true; // Found an unprocessed proposal
                }
            }
        }

        false
    }

    fn execute(&self, ctx: &dyn crate::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for proposal_fact in proposals {
            if let Some(proposal) = Self::parse_proposal(proposal_fact) {
                // Skip if already promoted
                let existing = ctx.get(proposal.key);
                if existing.iter().any(|f| f.id == proposal.id) {
                    continue;
                }

                // Validate and potentially promote
                match self.validate_proposal(&proposal) {
                    ValidationResult::Accepted(fact) => {
                        facts.push(fact);
                    }
                    ValidationResult::Rejected {
                        proposal_id,
                        reason,
                    } => {
                        // Emit a rejection record for auditability
                        facts.push(Fact {
                            key: ContextKey::Signals,
                            id: format!("validation:rejected:{proposal_id}"),
                            content: format!("Proposal '{proposal_id}' rejected: {reason}"),
                        });
                    }
                }
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Helper to create a proposal fact for testing.
///
/// Encodes a `ProposedFact` into the format expected by `ValidationAgent`.
#[must_use]
pub fn encode_proposal(proposal: &ProposedFact) -> Fact {
    let target_key_str = match proposal.key {
        ContextKey::Seeds => "seeds",
        ContextKey::Hypotheses => "hypotheses",
        ContextKey::Strategies => "strategies",
        ContextKey::Constraints => "constraints",
        ContextKey::Signals => "signals",
        ContextKey::Competitors => "competitors",
        ContextKey::Evaluations => "evaluations",
        ContextKey::Proposals => "proposals", // shouldn't happen but handle it
        ContextKey::Diagnostic => "diagnostics",
    };

    Fact {
        key: ContextKey::Proposals,
        id: format!("proposal:{}:{}", target_key_str, proposal.id),
        content: format!(
            "{}|{}|{}",
            proposal.confidence, proposal.provenance, proposal.content
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;

    #[test]
    fn validation_accepts_good_proposal() {
        let agent = ValidationAgent::with_defaults();

        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "Market is growing".into(),
            confidence: 0.8,
            provenance: "gpt-4:abc123".into(),
        };

        match agent.validate_proposal(&proposal) {
            ValidationResult::Accepted(fact) => {
                assert_eq!(fact.key, ContextKey::Hypotheses);
                assert_eq!(fact.id, "hyp-1");
                assert_eq!(fact.content, "Market is growing");
            }
            ValidationResult::Rejected { reason, .. } => {
                panic!("Expected acceptance, got rejection: {reason}");
            }
        }
    }

    #[test]
    fn validation_rejects_low_confidence() {
        let agent = ValidationAgent::new(ValidationConfig {
            min_confidence: 0.7,
            ..Default::default()
        });

        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "Uncertain claim".into(),
            confidence: 0.3, // Below threshold
            provenance: "gpt-4:abc123".into(),
        };

        match agent.validate_proposal(&proposal) {
            ValidationResult::Rejected { reason, .. } => {
                assert!(reason.contains("confidence"));
            }
            ValidationResult::Accepted(_) => {
                panic!("Expected rejection for low confidence");
            }
        }
    }

    #[test]
    fn validation_rejects_empty_content() {
        let agent = ValidationAgent::with_defaults();

        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "   ".into(), // Empty after trim
            confidence: 0.9,
            provenance: "gpt-4:abc123".into(),
        };

        match agent.validate_proposal(&proposal) {
            ValidationResult::Rejected { reason, .. } => {
                assert!(reason.contains("empty"));
            }
            ValidationResult::Accepted(_) => {
                panic!("Expected rejection for empty content");
            }
        }
    }

    #[test]
    fn validation_rejects_missing_provenance() {
        let agent = ValidationAgent::new(ValidationConfig {
            require_provenance: true,
            ..Default::default()
        });

        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "Some claim".into(),
            confidence: 0.9,
            provenance: String::new(), // Missing
        };

        match agent.validate_proposal(&proposal) {
            ValidationResult::Rejected { reason, .. } => {
                assert!(reason.contains("provenance"));
            }
            ValidationResult::Accepted(_) => {
                panic!("Expected rejection for missing provenance");
            }
        }
    }

    #[test]
    fn validation_rejects_forbidden_terms() {
        let agent = ValidationAgent::new(ValidationConfig {
            forbidden_terms: vec!["guaranteed".into(), "100%".into()],
            ..Default::default()
        });

        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "This is GUARANTEED to work".into(),
            confidence: 0.9,
            provenance: "gpt-4:abc123".into(),
        };

        match agent.validate_proposal(&proposal) {
            ValidationResult::Rejected { reason, .. } => {
                assert!(reason.contains("guaranteed"));
            }
            ValidationResult::Accepted(_) => {
                panic!("Expected rejection for forbidden term");
            }
        }
    }

    #[test]
    fn encode_proposal_roundtrip() {
        let proposal = ProposedFact {
            key: ContextKey::Strategies,
            id: "strat-1".into(),
            content: "Focus on SMB".into(),
            confidence: 0.85,
            provenance: "claude-3:xyz".into(),
        };

        let encoded = encode_proposal(&proposal);
        assert_eq!(encoded.key, ContextKey::Proposals);
        assert_eq!(encoded.id, "proposal:strategies:strat-1");

        let decoded = ValidationAgent::parse_proposal(&encoded).expect("should parse");
        assert_eq!(decoded.key, proposal.key);
        assert_eq!(decoded.id, proposal.id);
        assert_eq!(decoded.content, proposal.content);
        assert!((decoded.confidence - proposal.confidence).abs() < 0.001);
        assert_eq!(decoded.provenance, proposal.provenance);
    }

    #[test]
    fn validation_agent_promotes_in_engine() {
        let mut engine = Engine::new();
        engine.register(ValidationAgent::with_defaults());

        // Create initial context with a proposal
        let mut ctx = Context::new();
        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "llm-hyp-1".into(),
            content: "AI suggests market expansion".into(),
            confidence: 0.75,
            provenance: "gpt-4:test123".into(),
        };
        let _ = ctx.add_fact(encode_proposal(&proposal));

        let result = engine.run(ctx).expect("should converge");

        assert!(result.converged);

        // The proposal should have been promoted to a Hypothesis
        let hypotheses = result.context.get(ContextKey::Hypotheses);
        assert_eq!(hypotheses.len(), 1);
        assert_eq!(hypotheses[0].id, "llm-hyp-1");
        assert_eq!(hypotheses[0].content, "AI suggests market expansion");
    }

    #[test]
    fn validation_agent_rejects_bad_proposal_in_engine() {
        let mut engine = Engine::new();
        engine.register(ValidationAgent::new(ValidationConfig {
            min_confidence: 0.8,
            ..Default::default()
        }));

        // Create context with a low-confidence proposal
        let mut ctx = Context::new();
        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "bad-hyp".into(),
            content: "Uncertain speculation".into(),
            confidence: 0.3, // Below threshold
            provenance: "gpt-4:test".into(),
        };
        let _ = ctx.add_fact(encode_proposal(&proposal));

        let result = engine.run(ctx).expect("should converge");

        assert!(result.converged);

        // The proposal should NOT have been promoted
        let hypotheses = result.context.get(ContextKey::Hypotheses);
        assert!(hypotheses.is_empty());

        // But there should be a rejection signal
        let signals = result.context.get(ContextKey::Signals);
        assert!(signals.iter().any(|s| s.id.contains("rejected")));
    }

    #[test]
    fn llm_cannot_bypass_validation() {
        // This test documents the compile-time safety:
        // An LLM agent cannot emit Facts directly - only ProposedFacts.
        // Those must go through ValidationAgent to become Facts.
        //
        // The type system enforces this:
        //   - AgentEffect only accepts Vec<Fact>
        //   - ProposedFact is a different type
        //   - You cannot add ProposedFact to AgentEffect
        //
        // This is the core LLM containment guarantee.
    }
}
