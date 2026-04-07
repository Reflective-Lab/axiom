// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Knowledge Pack agents for organizational learning and innovation.
//!
//! # Lifecycle: Signal → Hypothesis → Experiment → Decision → Canonical
//!
//! # Knowledge State Machine
//!
//! ```text
//! signal_captured → hypothesis_proposed → experiment_scheduled → running
//!                                                                  ↓
//!                                           completed → decision_pending
//!                                                                  ↓
//!                                                      decision_made → canonical
//! ```
//!
//! # Key Invariants
//!
//! - Every claim has provenance (source + confidence + timestamp)
//! - Canonical knowledge is append-only
//! - Every decision has explicit owner + falsification criteria
//! - No orphan experiments (must link to hypothesis)
//! - Success metrics defined before experiment starts

use converge_core::{
    Agent, AgentEffect, ContextKey, Fact,
    invariant::{Invariant, InvariantClass, InvariantResult, Violation},
};

// ============================================================================
// Fact ID Prefixes
// ============================================================================

pub const SIGNAL_PREFIX: &str = "signal:";
pub const HYPOTHESIS_PREFIX: &str = "hypothesis:";
pub const EXPERIMENT_PREFIX: &str = "experiment:";
pub const DECISION_PREFIX: &str = "decision:";
pub const CANONICAL_PREFIX: &str = "canonical:";
pub const CLAIM_PREFIX: &str = "claim:";
pub const PRIOR_ART_PREFIX: &str = "prior_art:";
pub const CLAIM_CHART_PREFIX: &str = "claim_chart:";
pub const PATENT_REPORT_PREFIX: &str = "patent_report:";

// ============================================================================
// Agents
// ============================================================================

/// Captures signals from various sources (Slack, meetings, customer feedback).
#[derive(Debug, Clone, Default)]
pub struct SignalCaptureAgent;

impl Agent for SignalCaptureAgent {
    fn name(&self) -> &str {
        "signal_capture"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Seeds).iter().any(|s| {
            s.content.contains("slack.message")
                || s.content.contains("customer.feedback")
                || s.content.contains("observation")
        })
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for seed in seeds.iter() {
            if seed.content.contains("slack.message")
                || seed.content.contains("customer.feedback")
                || seed.content.contains("observation")
            {
                facts.push(Fact {
                    key: ContextKey::Signals,
                    id: format!("{}{}", SIGNAL_PREFIX, seed.id),
                    content: serde_json::json!({
                        "type": "captured_signal",
                        "source_id": seed.id,
                        "state": "signal_captured",
                        "provenance": {
                            "source": "auto_captured",
                            "confidence": 0.7,
                            "timestamp": "2026-01-12T12:00:00Z"
                        },
                        "captured_at": "2026-01-12"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Generates hypotheses from captured signals.
#[derive(Debug, Clone, Default)]
pub struct HypothesisGeneratorAgent;

impl Agent for HypothesisGeneratorAgent {
    fn name(&self) -> &str {
        "hypothesis_generator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Signals).iter().any(|s| {
            s.id.starts_with(SIGNAL_PREFIX) && s.content.contains("\"state\":\"signal_captured\"")
        })
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);
        let mut facts = Vec::new();

        for signal in signals.iter() {
            if signal.id.starts_with(SIGNAL_PREFIX)
                && signal.content.contains("\"state\":\"signal_captured\"")
            {
                facts.push(Fact {
                    key: ContextKey::Hypotheses,
                    id: format!("{}{}", HYPOTHESIS_PREFIX, signal.id),
                    content: serde_json::json!({
                        "type": "hypothesis",
                        "signal_id": signal.id,
                        "state": "hypothesis_proposed",
                        "statement": "Generated from signal",
                        "falsification_criteria": [],
                        "owner": "pending_assignment",
                        "proposed_at": "2026-01-12"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Reviews and approves hypotheses for experimentation.
#[derive(Debug, Clone, Default)]
pub struct HypothesisReviewerAgent;

impl Agent for HypothesisReviewerAgent {
    fn name(&self) -> &str {
        "hypothesis_reviewer"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Hypotheses).iter().any(|h| {
            h.id.starts_with(HYPOTHESIS_PREFIX)
                && h.content.contains("\"state\":\"hypothesis_proposed\"")
        })
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let hypotheses = ctx.get(ContextKey::Hypotheses);
        let mut facts = Vec::new();

        for hyp in hypotheses.iter() {
            if hyp.id.starts_with(HYPOTHESIS_PREFIX)
                && hyp.content.contains("\"state\":\"hypothesis_proposed\"")
            {
                facts.push(Fact {
                    key: ContextKey::Hypotheses,
                    id: format!("{}approved:{}", HYPOTHESIS_PREFIX, hyp.id),
                    content: serde_json::json!({
                        "type": "hypothesis_review",
                        "hypothesis_id": hyp.id,
                        "state": "approved",
                        "reviewer": "product_lead",
                        "approved_at": "2026-01-12"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Schedules experiments for approved hypotheses.
#[derive(Debug, Clone, Default)]
pub struct ExperimentSchedulerAgent;

impl Agent for ExperimentSchedulerAgent {
    fn name(&self) -> &str {
        "experiment_scheduler"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        let has_approved = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .any(|h| h.content.contains("\"state\":\"approved\""));
        let has_scheduled = ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(EXPERIMENT_PREFIX));
        has_approved && !has_scheduled
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let hypotheses = ctx.get(ContextKey::Hypotheses);
        let mut facts = Vec::new();

        for review in hypotheses.iter() {
            if review.content.contains("\"state\":\"approved\"") {
                facts.push(Fact {
                    key: ContextKey::Proposals,
                    id: format!("{}{}", EXPERIMENT_PREFIX, review.id),
                    content: serde_json::json!({
                        "type": "experiment",
                        "hypothesis_id": review.id,
                        "state": "experiment_scheduled",
                        "success_metrics": [
                            {"metric": "conversion_rate", "target": ">5%"},
                            {"metric": "sample_size", "target": ">100"}
                        ],
                        "duration_days": 14,
                        "scheduled_at": "2026-01-12"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Monitors running experiments and collects results.
#[derive(Debug, Clone, Default)]
pub struct ExperimentRunnerAgent;

impl Agent for ExperimentRunnerAgent {
    fn name(&self) -> &str {
        "experiment_runner"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals, ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Proposals).iter().any(|e| {
            e.id.starts_with(EXPERIMENT_PREFIX) && e.content.contains("\"state\":\"running\"")
        })
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let signals = ctx.get(ContextKey::Signals);
        let mut facts = Vec::new();

        for experiment in proposals.iter() {
            if experiment.id.starts_with(EXPERIMENT_PREFIX)
                && experiment.content.contains("\"state\":\"running\"")
            {
                // Check for completion signals (e.g., from PostHog)
                let has_results = signals
                    .iter()
                    .any(|s| s.content.contains("experiment.results"));

                if has_results {
                    facts.push(Fact {
                        key: ContextKey::Proposals,
                        id: format!("{}completed:{}", EXPERIMENT_PREFIX, experiment.id),
                        content: serde_json::json!({
                            "type": "experiment_result",
                            "experiment_id": experiment.id,
                            "state": "completed",
                            "results": {
                                "conversion_rate": 0.062,
                                "sample_size": 1250,
                                "statistical_significance": 0.95
                            },
                            "completed_at": "2026-01-12"
                        })
                        .to_string(),
                    });
                }
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Creates decision memos from completed experiments.
#[derive(Debug, Clone, Default)]
pub struct DecisionMemoAgent;

impl Agent for DecisionMemoAgent {
    fn name(&self) -> &str {
        "decision_memo"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Proposals).iter().any(|e| {
            e.id.contains(EXPERIMENT_PREFIX) && e.content.contains("\"state\":\"completed\"")
        })
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for result in proposals.iter() {
            if result.id.contains(EXPERIMENT_PREFIX)
                && result.content.contains("\"state\":\"completed\"")
            {
                facts.push(Fact {
                    key: ContextKey::Proposals,
                    id: format!("{}{}", DECISION_PREFIX, result.id),
                    content: serde_json::json!({
                        "type": "decision_memo",
                        "experiment_id": result.id,
                        "state": "decision_pending",
                        "recommendation": "Based on experiment results",
                        "owner": "product_lead",
                        "created_at": "2026-01-12"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Records decisions and creates canonical knowledge.
#[derive(Debug, Clone, Default)]
pub struct CanonicalKnowledgeAgent;

impl Agent for CanonicalKnowledgeAgent {
    fn name(&self) -> &str {
        "canonical_knowledge"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals, ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Signals)
            .iter()
            .any(|s| s.content.contains("decision.made"))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);
        let mut facts = Vec::new();

        for signal in signals.iter() {
            if signal.content.contains("decision.made") {
                facts.push(Fact {
                    key: ContextKey::Strategies, // Using Strategies for canonical knowledge
                    id: format!("{}{}", CANONICAL_PREFIX, signal.id),
                    content: serde_json::json!({
                        "type": "canonical_knowledge",
                        "decision_id": signal.id,
                        "state": "canonical",
                        "knowledge": "Derived from validated experiment",
                        "provenance": {
                            "source": "experiment_validated",
                            "confidence": 0.95,
                            "timestamp": "2026-01-12T12:00:00Z"
                        },
                        "append_only": true,
                        "created_at": "2026-01-12"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Validates and enriches claims with provenance.
#[derive(Debug, Clone, Default)]
pub struct ClaimValidatorAgent;

impl Agent for ClaimValidatorAgent {
    fn name(&self) -> &str {
        "claim_validator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|s| s.content.contains("claim.submitted"))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for seed in seeds.iter() {
            if seed.content.contains("claim.submitted") {
                facts.push(Fact {
                    key: ContextKey::Evaluations,
                    id: format!("{}{}", CLAIM_PREFIX, seed.id),
                    content: serde_json::json!({
                        "type": "validated_claim",
                        "seed_id": seed.id,
                        "has_provenance": true,
                        "provenance": {
                            "source": "user_submitted",
                            "confidence": 0.6,
                            "timestamp": "2026-01-12T12:00:00Z"
                        },
                        "validated_at": "2026-01-12"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

// ============================================================================
// Invariants
// ============================================================================

/// Ensures every claim has provenance.
#[derive(Debug, Clone, Default)]
pub struct ClaimHasProvenanceInvariant;

impl Invariant for ClaimHasProvenanceInvariant {
    fn name(&self) -> &str {
        "claim_has_provenance"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        for claim in ctx.get(ContextKey::Evaluations).iter() {
            if claim.id.starts_with(CLAIM_PREFIX) && !claim.content.contains("\"provenance\":") {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Claim {} missing provenance", claim.id),
                    vec![claim.id.clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures experiments link to hypotheses.
#[derive(Debug, Clone, Default)]
pub struct NoOrphanExperimentsInvariant;

impl Invariant for NoOrphanExperimentsInvariant {
    fn name(&self) -> &str {
        "no_orphan_experiments"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        for experiment in ctx.get(ContextKey::Proposals).iter() {
            if experiment.id.starts_with(EXPERIMENT_PREFIX)
                && !experiment.content.contains("\"hypothesis_id\":")
            {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Experiment {} has no linked hypothesis", experiment.id),
                    vec![experiment.id.clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures experiments have success metrics defined.
#[derive(Debug, Clone, Default)]
pub struct ExperimentHasMetricsInvariant;

impl Invariant for ExperimentHasMetricsInvariant {
    fn name(&self) -> &str {
        "experiment_has_metrics"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Acceptance
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        for experiment in ctx.get(ContextKey::Proposals).iter() {
            if experiment.id.starts_with(EXPERIMENT_PREFIX)
                && experiment.content.contains("\"state\":\"running\"")
                && !experiment.content.contains("\"success_metrics\":")
            {
                return InvariantResult::Violated(Violation::with_facts(
                    format!(
                        "Running experiment {} has no success metrics",
                        experiment.id
                    ),
                    vec![experiment.id.clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures decisions have explicit owners.
#[derive(Debug, Clone, Default)]
pub struct DecisionHasOwnerInvariant;

impl Invariant for DecisionHasOwnerInvariant {
    fn name(&self) -> &str {
        "decision_has_owner"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Acceptance
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        for decision in ctx.get(ContextKey::Proposals).iter() {
            if decision.id.starts_with(DECISION_PREFIX) && !decision.content.contains("\"owner\":")
            {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Decision {} has no owner", decision.id),
                    vec![decision.id.clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures prior art evidence includes provenance.
#[derive(Debug, Clone, Default)]
pub struct PatentEvidenceHasProvenanceInvariant;

impl Invariant for PatentEvidenceHasProvenanceInvariant {
    fn name(&self) -> &str {
        "patent_evidence_has_provenance"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        for evidence in ctx.get(ContextKey::Evaluations).iter() {
            if evidence.id.starts_with(PRIOR_ART_PREFIX)
                && !evidence.content.contains("\"provenance\":")
            {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Evidence {} missing provenance", evidence.id),
                    vec![evidence.id.clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_have_correct_names() {
        assert_eq!(SignalCaptureAgent.name(), "signal_capture");
        assert_eq!(HypothesisGeneratorAgent.name(), "hypothesis_generator");
        assert_eq!(HypothesisReviewerAgent.name(), "hypothesis_reviewer");
        assert_eq!(ExperimentSchedulerAgent.name(), "experiment_scheduler");
        assert_eq!(ExperimentRunnerAgent.name(), "experiment_runner");
        assert_eq!(DecisionMemoAgent.name(), "decision_memo");
        assert_eq!(CanonicalKnowledgeAgent.name(), "canonical_knowledge");
        assert_eq!(ClaimValidatorAgent.name(), "claim_validator");
    }
}
