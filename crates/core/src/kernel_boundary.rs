// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! # Kernel Boundary Types
//!
//! These types define the **constitutional boundary** between reasoning kernels
//! (converge-llm) and the Converge platform. They encode core axioms:
//!
//! - **Proposed vs Fact**: Kernels emit `KernelProposal`, not `Fact`
//! - **Replayable vs Audit-only**: `LocalTraceLink` vs `RemoteTraceLink`
//! - **Explicit Authority**: All proposals have provenance via `TraceLink`
//!
//! ## Axiom Compliance
//!
//! | Axiom | Enforcement |
//! |-------|-------------|
//! | Agents Suggest, Engines Decide | `KernelProposal` cannot become `Fact` without validation |
//! | Transparent Determinism | `TraceLink` in every proposal |
//! | Human Authority First-Class | `requires_human` flag on proposals |
//!
//! ## Usage
//!
//! These types are re-exported by capability kernels (e.g., converge-llm)
//! but defined here to ensure a single source of truth across all kernels.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Kernel Input Types: The Platform-to-Kernel Contract
// ============================================================================

/// What the kernel should reason about.
///
/// This is the **intent contract** between the platform and any reasoning kernel.
/// It defines the task, success criteria, and resource budgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelIntent {
    /// The task to perform (e.g., "analyze_metrics", "generate_plan")
    pub task: String,
    /// Success criteria for the task
    pub criteria: Vec<String>,
    /// Maximum tokens budget for the entire kernel run
    pub max_tokens: usize,
}

impl KernelIntent {
    /// Create a new kernel intent with a task description.
    #[must_use]
    pub fn new(task: impl Into<String>) -> Self {
        Self {
            task: task.into(),
            criteria: Vec::new(),
            max_tokens: 1024,
        }
    }

    /// Add a success criterion.
    #[must_use]
    pub fn with_criteria(mut self, criteria: impl Into<String>) -> Self {
        self.criteria.push(criteria.into());
        self
    }

    /// Set maximum tokens budget.
    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

/// The context provided to the kernel (from converge-core's Context).
///
/// This is a **read-only view** of the platform's context, projected
/// for kernel consumption. Kernels cannot mutate this directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelContext {
    /// Structured state data (from Seeds, Signals, etc.)
    pub state: HashMap<String, serde_json::Value>,
    /// Relevant facts from context (read-only view)
    pub facts: Vec<ContextFact>,
    /// Tenant/session identifier for recall scoping
    pub tenant_id: Option<String>,
}

impl KernelContext {
    /// Create an empty kernel context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            facts: Vec::new(),
            tenant_id: None,
        }
    }

    /// Add state data.
    #[must_use]
    pub fn with_state(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.state.insert(key.into(), value);
        self
    }

    /// Add a fact from the platform context.
    #[must_use]
    pub fn with_fact(
        mut self,
        key: impl Into<String>,
        id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        self.facts.push(ContextFact {
            key: key.into(),
            id: id.into(),
            content: content.into(),
        });
        self
    }

    /// Set tenant identifier for recall scoping.
    #[must_use]
    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }
}

impl Default for KernelContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A fact from converge-core's context (read-only).
///
/// This is a projection of platform facts for kernel consumption.
/// The kernel cannot create or modify facts directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFact {
    /// The context key this fact belongs to
    pub key: String,
    /// Unique identifier for this fact
    pub id: String,
    /// The fact content
    pub content: String,
}

/// Policy controlling kernel behavior.
///
/// This is the **policy contract** from the platform/runtime to the kernel.
/// It controls adapter selection, recall behavior, determinism, and human gates.
///
/// # Axiom: Explicit Authority
///
/// Adapter selection comes from `KernelPolicy`, not emergent kernel behavior.
/// This ensures the platform maintains control over which capabilities are used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelPolicy {
    /// Which adapter to use (explicit authority from outside)
    pub adapter_id: Option<String>,
    /// Whether recall is enabled for this run
    pub recall_enabled: bool,
    /// Maximum recall candidates to consider
    pub recall_max_candidates: usize,
    /// Minimum relevance score for recall results
    pub recall_min_score: f32,
    /// Seed for deterministic execution (None = random)
    pub seed: Option<u64>,
    /// Whether proposals from this run require human approval
    pub requires_human: bool,
    /// Truth targets that must pass for auto-promotion
    pub required_truths: Vec<String>,
}

impl KernelPolicy {
    /// Create a new default policy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter_id: None,
            recall_enabled: false,
            recall_max_candidates: 5,
            recall_min_score: 0.7,
            seed: None,
            requires_human: false,
            required_truths: Vec::new(),
        }
    }

    /// Create a deterministic policy with a fixed seed.
    #[must_use]
    pub fn deterministic(seed: u64) -> Self {
        Self {
            seed: Some(seed),
            ..Self::new()
        }
    }

    /// Set the adapter to use.
    #[must_use]
    pub fn with_adapter(mut self, adapter_id: impl Into<String>) -> Self {
        self.adapter_id = Some(adapter_id.into());
        self
    }

    /// Enable or disable recall.
    #[must_use]
    pub fn with_recall(mut self, enabled: bool) -> Self {
        self.recall_enabled = enabled;
        self
    }

    /// Mark proposals as requiring human approval.
    #[must_use]
    pub fn with_human_required(mut self) -> Self {
        self.requires_human = true;
        self
    }

    /// Add a required truth for auto-promotion.
    #[must_use]
    pub fn with_required_truth(mut self, truth: impl Into<String>) -> Self {
        self.required_truths.push(truth.into());
        self
    }
}

impl Default for KernelPolicy {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Routing Policy: Backend Selection Vocabulary
// ============================================================================

/// Risk tier for routing decisions.
///
/// This enum is part of the platform's vocabulary for backend selection.
/// Policies can restrict which backends are allowed for each risk tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskTier {
    Low,
    Medium,
    High,
    Critical,
}

/// Data classification for routing decisions.
///
/// This enum controls which backends can handle data based on sensitivity.
/// Policies can restrict remote backends for confidential/restricted data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Policy for routing requests to backends.
///
/// Routing should be **policy-based**, not ad-hoc. This type encodes
/// the rules for selecting backends based on:
/// - Truth preferences (which truths prefer which backends)
/// - Risk tier (critical/high-risk operations may require local)
/// - Data classification (sensitive data may require local)
///
/// # Axiom: Explicit Authority
///
/// Backend selection is never implicit. Policies must explicitly allow
/// remote backends, and default-deny is the recommended stance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPolicy {
    /// Truth target → preferred backend
    pub truth_preferences: HashMap<String, String>,
    /// Risk tier → allowed backends
    pub risk_tier_backends: HashMap<RiskTier, Vec<String>>,
    /// Data classification → allowed backends
    pub data_classification_backends: HashMap<DataClassification, Vec<String>>,
    /// Default backend if no rule matches
    pub default_backend: String,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self {
            truth_preferences: HashMap::new(),
            risk_tier_backends: HashMap::new(),
            data_classification_backends: HashMap::new(),
            default_backend: "local".to_string(),
        }
    }
}

impl RoutingPolicy {
    /// Create a policy that denies remote backends by default.
    ///
    /// Remote backends must be explicitly allowed via risk tier or data classification.
    /// This is the **recommended default** for security-conscious deployments.
    #[must_use]
    pub fn default_deny_remote() -> Self {
        let mut policy = Self::default();
        // Only allow local for high-risk and restricted data by default
        policy
            .risk_tier_backends
            .insert(RiskTier::Critical, vec!["local".to_string()]);
        policy
            .risk_tier_backends
            .insert(RiskTier::High, vec!["local".to_string()]);
        policy
            .data_classification_backends
            .insert(DataClassification::Restricted, vec!["local".to_string()]);
        policy
            .data_classification_backends
            .insert(DataClassification::Confidential, vec!["local".to_string()]);
        policy
    }

    /// Check if a backend is allowed for the given context.
    #[must_use]
    pub fn is_backend_allowed(
        &self,
        backend_name: &str,
        risk_tier: RiskTier,
        data_classification: DataClassification,
    ) -> bool {
        // Check if explicitly denied by risk tier
        if let Some(allowed) = self.risk_tier_backends.get(&risk_tier) {
            if !allowed.contains(&backend_name.to_string()) && !allowed.is_empty() {
                return false;
            }
        }

        // Check if explicitly denied by data classification
        if let Some(allowed) = self.data_classification_backends.get(&data_classification) {
            if !allowed.contains(&backend_name.to_string()) && !allowed.is_empty() {
                return false;
            }
        }

        true
    }

    /// Select a backend for the given request context.
    #[must_use]
    pub fn select_backend(
        &self,
        truth_ids: &[String],
        risk_tier: RiskTier,
        data_classification: DataClassification,
    ) -> &str {
        // Check truth preferences first
        for truth_id in truth_ids {
            if let Some(backend) = self.truth_preferences.get(truth_id) {
                return backend;
            }
        }

        // Check risk tier
        if let Some(backends) = self.risk_tier_backends.get(&risk_tier) {
            if let Some(backend) = backends.first() {
                return backend;
            }
        }

        // Check data classification
        if let Some(backends) = self.data_classification_backends.get(&data_classification) {
            if let Some(backend) = backends.first() {
                return backend;
            }
        }

        // Default
        &self.default_backend
    }
}

// ============================================================================
// Decision Step: Kernel Reasoning Phases
// ============================================================================

/// A step in the multi-phase reasoning process.
///
/// Kernels (like converge-llm) execute reasoning in distinct phases.
/// This enum represents those phases and is part of the kernel boundary
/// vocabulary for tracing, recall scoping, and contract validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionStep {
    /// First step: derive conclusions from state
    Reasoning,
    /// Second step: score/evaluate options
    Evaluation,
    /// Third step: produce action plan
    Planning,
}

impl DecisionStep {
    /// Get the expected contract type for this step.
    #[must_use]
    pub fn expected_contract(&self) -> &'static str {
        match self {
            Self::Reasoning => "Reasoning",
            Self::Evaluation => "Evaluation",
            Self::Planning => "Planning",
        }
    }

    /// Get the step name as a string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reasoning => "reasoning",
            Self::Evaluation => "evaluation",
            Self::Planning => "planning",
        }
    }
}

impl Default for DecisionStep {
    fn default() -> Self {
        Self::Reasoning
    }
}

// ============================================================================
// TraceLink: Two Concrete Shapes
// ============================================================================

/// Trace link for audit and (possibly) replay.
///
/// The shape depends on the backend type. This is the **constitutional type**
/// that prevents "TraceLink + hope" semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TraceLink {
    /// Local backend: replay-eligible (deterministic)
    Local(LocalTraceLink),
    /// Remote backend: audit-eligible only (bounded stochasticity)
    Remote(RemoteTraceLink),
}

impl TraceLink {
    /// Check if this trace is replay-eligible (only local).
    #[must_use]
    pub fn is_replay_eligible(&self) -> bool {
        matches!(self, TraceLink::Local(_))
    }

    /// Get the replayability level.
    #[must_use]
    pub fn replayability(&self) -> Replayability {
        match self {
            TraceLink::Local(_) => Replayability::Deterministic,
            TraceLink::Remote(r) => r.replayability,
        }
    }
}

/// Replayability level of the trace.
///
/// This enum enforces explicit acknowledgment of replay guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Replayability {
    /// Can be replayed with identical output (local inference)
    Deterministic,
    /// Best effort, may vary slightly (temp=0 remote)
    BestEffort,
    /// Cannot be replayed (external factors, safety layers)
    None,
}

impl Default for Replayability {
    fn default() -> Self {
        Self::None
    }
}

/// Reason why proposal replayability was downgraded.
///
/// This is a **stable contract surface** - serialization shape must not change
/// without careful migration planning. Used for audit trails showing which
/// component caused a replayability downgrade.
///
/// # Axiom: System Tells the Truth About Itself
///
/// If a kernel proposal includes stochastic components, the system must
/// explicitly document why replayability was downgraded, not silently degrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplayabilityDowngradeReason {
    /// Recall embedder is not bit-exact deterministic
    RecallEmbedderNotDeterministic,
    /// Corpus content hash missing (cannot verify exact corpus state)
    RecallCorpusNotContentAddressed,
    /// Remote backend was used (cannot guarantee exact replay)
    RemoteBackendUsed,
    /// No seed was provided (inference is stochastic)
    NoSeedProvided,
    /// Multiple components caused downgrade
    MultipleReasons,
}

/// Local trace link — replay-eligible.
///
/// Contains all information needed to reproduce the exact output.
/// Only local inference can provide this level of determinism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTraceLink {
    /// Hash of base model weights
    pub base_model_hash: String,
    /// Adapter ID + hash (if used)
    pub adapter: Option<AdapterTrace>,
    /// Tokenizer hash
    pub tokenizer_hash: String,
    /// Random seed used
    pub seed: u64,
    /// Sampler parameters
    pub sampler: SamplerParams,
    /// Prompt version
    pub prompt_version: String,
    /// Recall trace (if used)
    pub recall: Option<RecallTrace>,
    /// Whether weights were mutated (merge)
    pub weights_mutated: bool,
    /// Execution environment
    pub execution_env: ExecutionEnv,
}

/// Adapter trace for local runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterTrace {
    pub adapter_id: String,
    pub adapter_hash: String,
    pub merged: bool,
}

/// Sampler parameters for reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplerParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: Option<usize>,
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            top_p: 1.0,
            top_k: None,
        }
    }
}

/// Recall trace for local runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallTrace {
    pub corpus_fingerprint: String,
    pub candidate_ids: Vec<String>,
    pub candidate_scores: Vec<f32>,
    pub injected_count: usize,
}

/// Execution environment info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEnv {
    pub device: String,
    pub backend: String,
    pub precision: String,
}

impl Default for ExecutionEnv {
    fn default() -> Self {
        Self {
            device: "cpu".to_string(),
            backend: "ndarray".to_string(),
            precision: "f32".to_string(),
        }
    }
}

/// Remote trace link — audit-eligible only.
///
/// Contains enough info to audit but NOT replay deterministically.
/// This explicitly acknowledges the bounded stochasticity of remote providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTraceLink {
    /// Provider name (e.g., "anthropic", "openai")
    pub provider_name: String,
    /// Model ID as returned by provider
    pub provider_model_id: String,
    /// Hash of canonicalized request
    pub request_fingerprint: String,
    /// Hash of response payload
    pub response_fingerprint: String,
    /// Temperature used
    pub temperature: f32,
    /// Top-p used
    pub top_p: f32,
    /// Max tokens requested
    pub max_tokens: usize,
    /// Provider-specific metadata (e.g., system_fingerprint)
    pub provider_metadata: HashMap<String, String>,
    /// Whether this was retried
    pub retried: bool,
    /// Retry reasons (if retried)
    pub retry_reasons: Vec<String>,
    /// Explicit replayability flag — prevents "TraceLink + hope" semantics
    pub replayability: Replayability,
}

// ============================================================================
// Proposal Types: The Kernel Output Boundary
// ============================================================================

/// The kind of proposal a kernel is making.
///
/// This taxonomy helps the engine understand what kind of validation
/// and promotion logic to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProposalKind {
    /// Claims or assertions derived from reasoning
    Claims,
    /// An action plan with ordered steps
    Plan,
    /// A classification or categorization
    Classification,
    /// An evaluation with scores and justification
    Evaluation,
    /// A draft document or text artifact
    DraftDocument,
    /// Raw reasoning output (when no specific kind applies)
    Reasoning,
}

impl Default for ProposalKind {
    fn default() -> Self {
        Self::Reasoning
    }
}

/// Kind of proposed content (backend-level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentKind {
    Claim,
    Plan,
    Classification,
    Evaluation,
    Draft,
    Reasoning,
}

impl Default for ContentKind {
    fn default() -> Self {
        Self::Reasoning
    }
}

impl From<ContentKind> for ProposalKind {
    fn from(kind: ContentKind) -> Self {
        match kind {
            ContentKind::Claim => ProposalKind::Claims,
            ContentKind::Plan => ProposalKind::Plan,
            ContentKind::Classification => ProposalKind::Classification,
            ContentKind::Evaluation => ProposalKind::Evaluation,
            ContentKind::Draft => ProposalKind::DraftDocument,
            ContentKind::Reasoning => ProposalKind::Reasoning,
        }
    }
}

/// A proposed piece of content (not yet a Fact).
///
/// This is the backend-level proposal type. It gets wrapped in
/// `KernelProposal` at the kernel boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedContent {
    /// Unique identifier
    pub id: String,
    /// The content type
    pub kind: ContentKind,
    /// The actual content
    pub content: String,
    /// Structured content (if applicable)
    pub structured: Option<serde_json::Value>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: Option<f32>,
    /// Whether this requires human approval
    pub requires_human: bool,
}

impl ProposedContent {
    /// Create a new proposed content with minimal fields.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: ContentKind, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            content: content.into(),
            structured: None,
            confidence: None,
            requires_human: false,
        }
    }

    /// Mark this proposal as requiring human approval.
    #[must_use]
    pub fn with_human_required(mut self) -> Self {
        self.requires_human = true;
        self
    }

    /// Add confidence score.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }
}

/// Contract validation result for a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    /// Name of the contract or truth
    pub name: String,
    /// Whether it passed
    pub passed: bool,
    /// Failure reason if not passed
    pub failure_reason: Option<String>,
}

impl ContractResult {
    /// Create a passing result.
    #[must_use]
    pub fn passed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            failure_reason: None,
        }
    }

    /// Create a failing result.
    #[must_use]
    pub fn failed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            failure_reason: Some(reason.into()),
        }
    }
}

/// A proposal from the reasoning kernel.
///
/// This is the **only** output type that crosses the kernel boundary.
/// It must be validated and promoted by converge-core before becoming a Fact.
///
/// # Axiom: "Agents Suggest, Engines Decide"
///
/// Kernels (including LLM kernels) emit `KernelProposal`, not `Fact`.
/// The engine validates proposals against contracts and truth requirements
/// before promoting them to facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelProposal {
    /// Unique identifier for this proposal
    pub id: String,
    /// What kind of proposal this is
    pub kind: ProposalKind,
    /// The actual content/payload
    pub payload: String,
    /// Structured payload (if applicable)
    pub structured_payload: Option<serde_json::Value>,
    /// Link to the generation trace (for audit/replay)
    pub trace_link: TraceLink,
    /// Contract/truth validation results
    pub contract_results: Vec<ContractResult>,
    /// Whether this proposal requires human approval
    pub requires_human: bool,
    /// Confidence score (0.0 - 1.0) if available
    pub confidence: Option<f32>,
}

impl KernelProposal {
    /// Check if all contracts passed.
    #[must_use]
    pub fn all_contracts_passed(&self) -> bool {
        self.contract_results.iter().all(|r| r.passed)
    }

    /// Get failed contract names.
    #[must_use]
    pub fn failed_contracts(&self) -> Vec<&str> {
        self.contract_results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.name.as_str())
            .collect()
    }

    /// Check if this proposal is eligible for automatic promotion.
    ///
    /// A proposal can be auto-promoted if:
    /// - All contracts passed
    /// - Human approval is not required
    #[must_use]
    pub fn is_auto_promotable(&self) -> bool {
        self.all_contracts_passed() && !self.requires_human
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_link_replayability() {
        let local = TraceLink::Local(LocalTraceLink {
            base_model_hash: "abc123".to_string(),
            adapter: None,
            tokenizer_hash: "tok123".to_string(),
            seed: 42,
            sampler: SamplerParams::default(),
            prompt_version: "v1".to_string(),
            recall: None,
            weights_mutated: false,
            execution_env: ExecutionEnv::default(),
        });

        let remote = TraceLink::Remote(RemoteTraceLink {
            provider_name: "anthropic".to_string(),
            provider_model_id: "claude-3-opus".to_string(),
            request_fingerprint: "req123".to_string(),
            response_fingerprint: "resp456".to_string(),
            temperature: 0.0,
            top_p: 1.0,
            max_tokens: 1024,
            provider_metadata: HashMap::new(),
            retried: false,
            retry_reasons: vec![],
            replayability: Replayability::BestEffort,
        });

        assert!(local.is_replay_eligible());
        assert!(!remote.is_replay_eligible());

        assert_eq!(local.replayability(), Replayability::Deterministic);
        assert_eq!(remote.replayability(), Replayability::BestEffort);
    }

    #[test]
    fn proposal_kind_conversion() {
        assert_eq!(ProposalKind::from(ContentKind::Claim), ProposalKind::Claims);
        assert_eq!(ProposalKind::from(ContentKind::Plan), ProposalKind::Plan);
        assert_eq!(
            ProposalKind::from(ContentKind::Reasoning),
            ProposalKind::Reasoning
        );
    }

    #[test]
    fn contract_result_helpers() {
        let passed = ContractResult::passed("grounded-answering");
        assert!(passed.passed);
        assert!(passed.failure_reason.is_none());

        let failed = ContractResult::failed("reasoning", "missing CONCLUSION");
        assert!(!failed.passed);
        assert_eq!(failed.failure_reason.as_deref(), Some("missing CONCLUSION"));
    }

    #[test]
    fn kernel_proposal_auto_promotable() {
        let local_trace = TraceLink::Local(LocalTraceLink {
            base_model_hash: "hash".to_string(),
            adapter: None,
            tokenizer_hash: "tok".to_string(),
            seed: 1,
            sampler: SamplerParams::default(),
            prompt_version: "v1".to_string(),
            recall: None,
            weights_mutated: false,
            execution_env: ExecutionEnv::default(),
        });

        // All passed, no human required
        let promotable = KernelProposal {
            id: "p1".to_string(),
            kind: ProposalKind::Claims,
            payload: "claim".to_string(),
            structured_payload: None,
            trace_link: local_trace.clone(),
            contract_results: vec![ContractResult::passed("c1")],
            requires_human: false,
            confidence: Some(0.9),
        };
        assert!(promotable.is_auto_promotable());

        // Human required
        let needs_human = KernelProposal {
            id: "p2".to_string(),
            kind: ProposalKind::Claims,
            payload: "claim".to_string(),
            structured_payload: None,
            trace_link: local_trace.clone(),
            contract_results: vec![ContractResult::passed("c1")],
            requires_human: true,
            confidence: Some(0.9),
        };
        assert!(!needs_human.is_auto_promotable());

        // Contract failed
        let failed_contract = KernelProposal {
            id: "p3".to_string(),
            kind: ProposalKind::Claims,
            payload: "claim".to_string(),
            structured_payload: None,
            trace_link: local_trace,
            contract_results: vec![ContractResult::failed("c1", "reason")],
            requires_human: false,
            confidence: Some(0.9),
        };
        assert!(!failed_contract.is_auto_promotable());
    }

    // ========================================================================
    // Kernel Input Types Tests
    // ========================================================================

    #[test]
    fn kernel_intent_builder() {
        let intent = KernelIntent::new("analyze_metrics")
            .with_criteria("find anomalies")
            .with_criteria("suggest fixes")
            .with_max_tokens(512);

        assert_eq!(intent.task, "analyze_metrics");
        assert_eq!(intent.criteria.len(), 2);
        assert_eq!(intent.criteria[0], "find anomalies");
        assert_eq!(intent.criteria[1], "suggest fixes");
        assert_eq!(intent.max_tokens, 512);
    }

    #[test]
    fn kernel_context_builder() {
        let context = KernelContext::new()
            .with_state("metric", serde_json::json!(0.5))
            .with_fact("Seeds", "seed-1", "Some seed fact")
            .with_tenant("tenant-123");

        assert!(context.state.contains_key("metric"));
        assert_eq!(context.facts.len(), 1);
        assert_eq!(context.facts[0].key, "Seeds");
        assert_eq!(context.facts[0].id, "seed-1");
        assert_eq!(context.tenant_id, Some("tenant-123".to_string()));
    }

    #[test]
    fn kernel_context_default() {
        let context = KernelContext::default();
        assert!(context.state.is_empty());
        assert!(context.facts.is_empty());
        assert!(context.tenant_id.is_none());
    }

    #[test]
    fn kernel_policy_default() {
        let policy = KernelPolicy::default();
        assert!(policy.adapter_id.is_none());
        assert!(!policy.recall_enabled);
        assert_eq!(policy.recall_max_candidates, 5);
        assert!((policy.recall_min_score - 0.7).abs() < f32::EPSILON);
        assert!(policy.seed.is_none());
        assert!(!policy.requires_human);
        assert!(policy.required_truths.is_empty());
    }

    #[test]
    fn kernel_policy_deterministic() {
        let policy = KernelPolicy::deterministic(42)
            .with_adapter("llm/grounded@1.0.0")
            .with_recall(true)
            .with_human_required()
            .with_required_truth("grounded-answering");

        assert_eq!(policy.seed, Some(42));
        assert_eq!(policy.adapter_id, Some("llm/grounded@1.0.0".to_string()));
        assert!(policy.recall_enabled);
        assert!(policy.requires_human);
        assert_eq!(policy.required_truths, vec!["grounded-answering"]);
    }
}
