// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Pack-based agent registration for Converge Runtime.
//!
//! This module provides the bridge between domain packs (converge-domain)
//! and the runtime execution engine. It mirrors the approach in converge-application
//! but is designed for server-side execution.

use converge_core::Engine;
use converge_provider::ProviderRegistry;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::RuntimeError;

/// Definition of a domain pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackDefinition {
    /// Pack identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Version.
    pub version: String,
    /// Available templates.
    pub templates: Vec<String>,
    /// Invariants.
    pub invariants: Vec<String>,
    /// Whether LLM agents are available.
    pub has_llm_agents: bool,
}

/// Registry of available packs.
pub struct PackRegistry {
    packs: Vec<PackDefinition>,
}

impl Default for PackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PackRegistry {
    /// Create a new pack registry with default packs.
    pub fn new() -> Self {
        let packs = vec![
            PackDefinition {
                id: "growth-strategy".to_string(),
                name: "Growth Strategy".to_string(),
                description: "Multi-agent growth strategy analysis with market signals, \
                             competitor analysis, strategy synthesis, and evaluation."
                    .to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["growth-strategy".to_string()],
                invariants: vec![
                    "BrandSafetyInvariant".to_string(),
                    "RequireMultipleStrategies".to_string(),
                    "RequireStrategyEvaluations".to_string(),
                    "RequireEvaluationRationale".to_string(),
                ],
                has_llm_agents: true,
            },
            PackDefinition {
                id: "ask-converge".to_string(),
                name: "Ask Converge".to_string(),
                description: "Grounded Q&A with recall-only sources.".to_string(),
                version: "0.1.0".to_string(),
                templates: vec!["ask-converge".to_string()],
                invariants: vec![
                    "GroundedAnswerInvariant".to_string(),
                    "RecallNotEvidenceInvariant".to_string(),
                ],
                has_llm_agents: false,
            },
            PackDefinition {
                id: "meeting-scheduler".to_string(),
                name: "Meeting Scheduler".to_string(),
                description:
                    "Meeting scheduling with constraint satisfaction and time zone handling."
                        .to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["meeting-scheduler".to_string()],
                invariants: vec![
                    "RequireParticipantAvailability".to_string(),
                    "RequirePositiveDuration".to_string(),
                    "RequireValidSlot".to_string(),
                ],
                has_llm_agents: false,
            },
            PackDefinition {
                id: "resource-routing".to_string(),
                name: "Resource Routing".to_string(),
                description: "Resource allocation and routing optimization.".to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["resource-routing".to_string()],
                invariants: vec![
                    "RequireAllTasksAssigned".to_string(),
                    "RequireCapacityRespected".to_string(),
                    "RequireValidDefinitions".to_string(),
                ],
                has_llm_agents: false,
            },
            PackDefinition {
                id: "release-readiness".to_string(),
                name: "Release Readiness".to_string(),
                description: "Engineering dependency and release quality gates.".to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["release-readiness".to_string()],
                invariants: vec![
                    "RequireAllChecksComplete".to_string(),
                    "RequireMinimumCoverage".to_string(),
                    "RequireNoCriticalVulnerabilities".to_string(),
                ],
                has_llm_agents: false,
            },
            PackDefinition {
                id: "sdr-sales".to_string(),
                name: "SDR Sales Pipeline".to_string(),
                description: "SDR sales qualification and outreach automation.".to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["sdr-qualify".to_string(), "sdr-outreach".to_string()],
                invariants: vec![
                    "RequireExplicitQualification".to_string(),
                    "RequireMessageStrategy".to_string(),
                    "RequireQualificationEvidence".to_string(),
                ],
                has_llm_agents: true,
            },
            PackDefinition {
                id: "linkedin-research".to_string(),
                name: "LinkedIn Research".to_string(),
                description: "Governed LinkedIn research with evidence and approvals.".to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["linkedin-research".to_string()],
                invariants: vec![
                    "EvidenceRequiresProvenanceInvariant".to_string(),
                    "NetworkPathRequiresVerificationInvariant".to_string(),
                    "ApprovalRequiredForExternalActionInvariant".to_string(),
                ],
                has_llm_agents: false,
            },
            PackDefinition {
                id: "drafting-short".to_string(),
                name: "Drafting Short".to_string(),
                description: "Short drafting flow with Perplexity research and Anthropic drafting."
                    .to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["drafting-short".to_string()],
                invariants: vec![],
                has_llm_agents: true,
            },
            PackDefinition {
                id: "novelty-search".to_string(),
                name: "Novelty Search".to_string(),
                description: "Short novelty search flow for patent prior art.".to_string(),
                version: "1.0.0".to_string(),
                templates: vec!["novelty-search".to_string()],
                invariants: vec![
                    "PatentEvidenceHasProvenanceInvariant".to_string(),
                    "EvidenceCitationInvariant".to_string(),
                ],
                has_llm_agents: false,
            },
        ];

        Self { packs }
    }

    /// Get a pack by ID.
    pub fn get(&self, id: &str) -> Option<&PackDefinition> {
        self.packs.iter().find(|p| p.id == id)
    }

    /// List all available packs.
    pub fn list(&self) -> &[PackDefinition] {
        &self.packs
    }

    /// Check if a pack exists.
    pub fn contains(&self, id: &str) -> bool {
        self.packs.iter().any(|p| p.id == id)
    }
}

/// Configuration for LLM provider selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Use mock LLM instead of real providers.
    pub use_mock: bool,
    /// Preferred model for Anthropic.
    pub anthropic_model: String,
    /// Preferred model for OpenAI.
    pub openai_model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            use_mock: false,
            anthropic_model: "claude-sonnet-4-20250514".to_string(),
            openai_model: "gpt-4o".to_string(),
        }
    }
}

impl LlmConfig {
    /// Creates a ProviderRegistry based on this configuration.
    ///
    /// If `use_mock` is true, returns None (callers should use mock agents instead).
    /// Otherwise, creates a registry from environment variables.
    #[must_use]
    pub fn create_registry(&self) -> Option<ProviderRegistry> {
        if self.use_mock {
            info!("Mock LLM mode enabled - LLM agents will use mock providers");
            return None;
        }

        let registry = ProviderRegistry::from_env();
        let available = registry.available_providers();

        if available.is_empty() {
            warn!(
                "No LLM API keys found. Set ANTHROPIC_API_KEY, OPENAI_API_KEY, or other provider keys. \
                 Using deterministic agents only."
            );
            return None;
        }

        info!(
            providers = ?available,
            "LLM provider registry initialized"
        );
        Some(registry)
    }
}

/// Register agents and invariants for a specific domain pack.
///
/// This is the bridge between the runtime and the domain packs defined
/// in converge-domain.
///
/// # Arguments
/// * `engine` - The convergence engine to register agents with
/// * `pack_id` - ID of the domain pack (e.g., "growth-strategy")
/// * `llm_config` - Configuration for LLM provider selection
pub fn register_pack_agents(
    engine: &mut Engine,
    pack_id: &str,
    llm_config: &LlmConfig,
) -> Result<(), RuntimeError> {
    info!(pack = %pack_id, "Registering pack agents and invariants");

    match pack_id {
        "growth-strategy" => {
            register_growth_strategy_pack(engine, llm_config)?;
        }
        "ask-converge" => {
            register_ask_converge_pack(engine)?;
        }
        "meeting-scheduler" => {
            register_meeting_scheduler_pack(engine)?;
        }
        "resource-routing" => {
            register_resource_routing_pack(engine)?;
        }
        "release-readiness" => {
            register_release_readiness_pack(engine)?;
        }
        "sdr-sales" => {
            register_sdr_sales_pack(engine, llm_config)?;
        }
        "linkedin-research" => {
            register_linkedin_research_pack(engine)?;
        }
        "drafting-short" => {
            register_drafting_short_pack(engine, llm_config)?;
        }
        "novelty-search" => {
            register_novelty_search_pack(engine)?;
        }
        _ => {
            warn!(pack = %pack_id, "Unknown pack requested, no agents registered");
        }
    }

    Ok(())
}

/// Register growth-strategy pack agents and invariants.
fn register_growth_strategy_pack(
    engine: &mut Engine,
    llm_config: &LlmConfig,
) -> Result<(), RuntimeError> {
    use converge_domain::{
        // Invariants
        BrandSafetyInvariant,
        RequireEvaluationRationale,
        RequireMultipleStrategies,
        RequireStrategyEvaluations,
    };

    // Try to create a provider registry for LLM agents
    let registry = llm_config.create_registry();

    if let Some(ref reg) = registry {
        // Use LLM-enabled agents when providers are available
        info!("Registering growth-strategy LLM agents");

        match converge_domain::growth_strategy_llm::setup_llm_growth_strategy(engine, reg) {
            Ok(()) => {
                info!("Successfully registered LLM-enabled growth-strategy agents");
            }
            Err(e) => {
                warn!(error = %e, "Failed to set up LLM agents, falling back to deterministic");
                register_deterministic_growth_agents(engine);
            }
        }
    } else {
        // Use deterministic agents when no LLM providers available
        info!("Registering growth-strategy deterministic agents (no LLM providers)");
        register_deterministic_growth_agents(engine);
    }

    // Register invariants (same for both modes)
    info!("Registering growth-strategy invariants");
    engine.register_invariant(BrandSafetyInvariant::default());
    engine.register_invariant(RequireMultipleStrategies);
    engine.register_invariant(RequireStrategyEvaluations);
    engine.register_invariant(RequireEvaluationRationale);

    Ok(())
}

/// Register deterministic growth strategy agents (fallback when no LLM).
fn register_deterministic_growth_agents(engine: &mut Engine) {
    use converge_domain::{CompetitorAgent, EvaluationAgent, MarketSignalAgent, StrategyAgent};

    engine.register(MarketSignalAgent);
    engine.register(CompetitorAgent);
    engine.register(StrategyAgent);
    engine.register(EvaluationAgent);
}

/// Register ask-converge pack agents and invariants.
fn register_ask_converge_pack(engine: &mut Engine) -> Result<(), RuntimeError> {
    use converge_domain::{AskConvergeAgent, GroundedAnswerInvariant, RecallNotEvidenceInvariant};

    info!("Registering ask-converge agent");
    engine.register(AskConvergeAgent::default());

    info!("Registering ask-converge invariants");
    engine.register_invariant(GroundedAnswerInvariant);
    engine.register_invariant(RecallNotEvidenceInvariant);

    Ok(())
}

/// Register meeting-scheduler pack agents and invariants.
fn register_meeting_scheduler_pack(engine: &mut Engine) -> Result<(), RuntimeError> {
    use converge_domain::{
        // Agents
        AvailabilityRetrievalAgent,
        ConflictDetectionAgent,
        // Invariants
        RequireParticipantAvailability,
        RequirePositiveDuration,
        RequireValidSlot,
        SlotOptimizationAgent,
        TimeZoneNormalizationAgent,
        WorkingHoursConstraintAgent,
    };

    info!("Registering meeting-scheduler agents");
    engine.register(AvailabilityRetrievalAgent);
    engine.register(ConflictDetectionAgent);
    engine.register(SlotOptimizationAgent);
    engine.register(TimeZoneNormalizationAgent);
    engine.register(WorkingHoursConstraintAgent);

    info!("Registering meeting-scheduler invariants");
    engine.register_invariant(RequireParticipantAvailability);
    engine.register_invariant(RequirePositiveDuration);
    engine.register_invariant(RequireValidSlot);

    Ok(())
}

/// Register resource-routing pack agents and invariants.
fn register_resource_routing_pack(engine: &mut Engine) -> Result<(), RuntimeError> {
    use converge_domain::{
        // Agents
        ConstraintValidationAgent,
        FeasibilityAgent,
        // Invariants
        RequireAllTasksAssigned,
        RequireCapacityRespected,
        RequireValidDefinitions,
        ResourceRetrievalAgent,
        SolverAgent,
        TaskRetrievalAgent,
    };

    info!("Registering resource-routing agents");
    engine.register(ConstraintValidationAgent);
    engine.register(FeasibilityAgent);
    engine.register(ResourceRetrievalAgent);
    engine.register(SolverAgent);
    engine.register(TaskRetrievalAgent);

    info!("Registering resource-routing invariants");
    engine.register_invariant(RequireAllTasksAssigned);
    engine.register_invariant(RequireCapacityRespected);
    engine.register_invariant(RequireValidDefinitions);

    Ok(())
}

/// Register release-readiness pack agents and invariants.
fn register_release_readiness_pack(engine: &mut Engine) -> Result<(), RuntimeError> {
    use converge_domain::{
        // Agents
        DependencyGraphAgent,
        DocumentationAgent,
        PerformanceRegressionAgent,
        ReleaseReadyAgent,
        // Invariants
        RequireAllChecksComplete,
        RequireMinimumCoverage,
        RequireNoCriticalVulnerabilities,
        RiskSummaryAgent,
        SecurityScanAgent,
        TestCoverageAgent,
    };

    info!("Registering release-readiness agents");
    engine.register(DependencyGraphAgent);
    engine.register(DocumentationAgent);
    engine.register(PerformanceRegressionAgent);
    engine.register(ReleaseReadyAgent);
    engine.register(RiskSummaryAgent);
    engine.register(SecurityScanAgent);
    engine.register(TestCoverageAgent);

    info!("Registering release-readiness invariants");
    engine.register_invariant(RequireAllChecksComplete);
    engine.register_invariant(RequireMinimumCoverage);
    engine.register_invariant(RequireNoCriticalVulnerabilities);

    Ok(())
}

/// Register sdr-sales pack agents and invariants.
fn register_sdr_sales_pack(
    engine: &mut Engine,
    _llm_config: &LlmConfig,
) -> Result<(), RuntimeError> {
    use converge_domain::{
        // Agents - these require names to be passed
        ChannelDecisionAgent,
        FitEvidenceAgent,
        MarketScanAgent,
        MessageHypothesisAgent,
        NeedEvidenceAgent,
        // Invariants
        RequireExplicitQualification,
        RequireMessageStrategy,
        RequireQualificationEvidence,
        RequireValidICP,
        RiskEvidenceAgent,
        SignalExtractionAgent,
        TimingEvidenceAgent,
    };

    info!("Registering sdr-sales agents");
    // SDR agents require names - use default names based on the pack
    engine.register(MarketScanAgent::new("sdr-market-scan"));
    engine.register(SignalExtractionAgent::new("sdr-signal-extraction"));
    engine.register(FitEvidenceAgent::new("sdr-fit-evidence"));
    engine.register(TimingEvidenceAgent::new("sdr-timing-evidence"));
    engine.register(NeedEvidenceAgent::new("sdr-need-evidence"));
    engine.register(RiskEvidenceAgent::new("sdr-risk-evidence"));
    engine.register(MessageHypothesisAgent::new("sdr-message-hypothesis"));
    engine.register(ChannelDecisionAgent::new("sdr-channel-decision"));

    info!("Registering sdr-sales invariants");
    engine.register_invariant(RequireExplicitQualification);
    engine.register_invariant(RequireMessageStrategy::new(0.7));
    engine.register_invariant(RequireQualificationEvidence::new(3));
    engine.register_invariant(RequireValidICP);

    Ok(())
}

fn register_linkedin_research_pack(engine: &mut Engine) -> Result<(), RuntimeError> {
    use converge_domain::{
        ApprovalRecorderAgent, ApprovalRequiredForExternalActionInvariant, DossierBuilderAgent,
        EvidenceRequiresProvenanceInvariant, EvidenceValidatorAgent, LinkedInTargetDiscoveryAgent,
        NetworkPathRequiresVerificationInvariant, PathVerifierAgent, SignalIngestAgent,
    };
    use converge_provider::{LinkedInApiProvider, LinkedInProvider, StubLinkedInProvider};
    use std::sync::Arc;

    info!("Registering linkedin-research agents");
    engine.register(SignalIngestAgent);
    engine.register(EvidenceValidatorAgent);
    engine.register(DossierBuilderAgent);
    engine.register(PathVerifierAgent);
    engine.register(ApprovalRecorderAgent);

    let provider = LinkedInApiProvider::from_env()
        .map(|provider| Arc::new(provider) as Arc<dyn LinkedInProvider>)
        .unwrap_or_else(|_| Arc::new(StubLinkedInProvider::default()));
    engine.register(LinkedInTargetDiscoveryAgent::new(provider));

    info!("Registering linkedin-research invariants");
    engine.register_invariant(EvidenceRequiresProvenanceInvariant::default());
    engine.register_invariant(NetworkPathRequiresVerificationInvariant::default());
    engine.register_invariant(ApprovalRequiredForExternalActionInvariant::default());

    Ok(())
}

/// Register drafting-short pack agents.
fn register_drafting_short_pack(
    engine: &mut Engine,
    llm_config: &LlmConfig,
) -> Result<(), RuntimeError> {
    let registry = llm_config.create_registry();

    if registry.is_some() {
        info!("Registering drafting-short LLM agents");
        match converge_domain::setup_llm_drafting(engine) {
            Ok(()) => {
                info!("Successfully registered drafting-short LLM agents");
            }
            Err(e) => {
                warn!(error = %e, "Failed to set up drafting LLM agents, falling back to deterministic");
                register_deterministic_drafting_agents(engine);
            }
        }
    } else {
        info!("Registering drafting-short deterministic agents (no LLM providers)");
        register_deterministic_drafting_agents(engine);
    }

    Ok(())
}

fn register_deterministic_drafting_agents(engine: &mut Engine) {
    use converge_domain::{DraftingComposerAgent, DraftingResearchAgent};

    engine.register(DraftingResearchAgent);
    engine.register(DraftingComposerAgent);
}

fn register_novelty_search_pack(engine: &mut Engine) -> Result<(), RuntimeError> {
    use converge_core::validation::ValidationAgent;
    use converge_domain::{
        EvidenceCitationInvariant, PatentEvidenceCollectorAgent,
        PatentEvidenceHasProvenanceInvariant, PatentOperatorPlannerAgent, PatentQueryBuilderAgent,
        PatentSearchExecutorAgent, PriorArtShortlistAgent,
    };
    use converge_provider::{CompositePatentProvider, PatentOperator, StubPatentProvider};
    use std::sync::Arc;

    let provider = Arc::new(CompositePatentProvider::from_env().unwrap_or_else(|_| {
        CompositePatentProvider::new()
            .with_provider(PatentOperator::Uspto, Arc::new(StubPatentProvider::new()))
    }));

    engine.register(PatentQueryBuilderAgent);
    engine.register(PatentOperatorPlannerAgent);
    engine.register(PatentSearchExecutorAgent::new(provider));
    engine.register(PatentEvidenceCollectorAgent);
    engine.register(PriorArtShortlistAgent);

    engine.register(ValidationAgent::with_defaults());
    engine.register_invariant(PatentEvidenceHasProvenanceInvariant);
    engine.register_invariant(EvidenceCitationInvariant);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_registry_has_growth_strategy() {
        let registry = PackRegistry::new();
        assert!(registry.contains("growth-strategy"));
    }

    #[test]
    fn test_pack_registry_has_linkedin_research() {
        let registry = PackRegistry::new();
        assert!(registry.contains("linkedin-research"));
    }

    #[test]
    fn test_pack_registry_get() {
        let registry = PackRegistry::new();
        let pack = registry.get("growth-strategy").unwrap();
        assert_eq!(pack.name, "Growth Strategy");
        assert!(pack.has_llm_agents);
    }

    #[test]
    fn test_pack_registry_get_linkedin_research() {
        let registry = PackRegistry::new();
        let pack = registry.get("linkedin-research").unwrap();
        assert_eq!(pack.name, "LinkedIn Research");
        assert!(!pack.has_llm_agents);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert!(!config.use_mock);
        assert!(config.anthropic_model.contains("claude"));
    }

    #[test]
    fn test_llm_config_mock_returns_none() {
        let mut config = LlmConfig::default();
        config.use_mock = true;

        // Mock mode should return None (no registry needed)
        let registry = config.create_registry();
        assert!(registry.is_none());
    }

    #[test]
    fn test_llm_config_no_keys_returns_none() {
        // Without API keys, registry creation should return None
        // (this test assumes no API keys are set in the test environment)
        let config = LlmConfig::default();
        let registry = config.create_registry();

        // Registry is None if no API keys are available
        // (or Some if API keys happen to be set)
        // We can't assert the exact value without knowing the environment
        // but we can verify it doesn't panic
        let _ = registry;
    }

    #[test]
    fn test_register_growth_strategy_with_mock() {
        let mut engine = converge_core::Engine::new();
        let mut config = LlmConfig::default();
        config.use_mock = true;

        // Should succeed and register deterministic agents
        let result = register_growth_strategy_pack(&mut engine, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_pack_agents_unknown_pack() {
        let mut engine = converge_core::Engine::new();
        let config = LlmConfig::default();

        // Unknown pack should succeed (just logs a warning)
        let result = register_pack_agents(&mut engine, "unknown-pack", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_linkedin_research_pack() {
        let mut engine = converge_core::Engine::new();
        let config = LlmConfig::default();
        let result = register_pack_agents(&mut engine, "linkedin-research", &config);
        assert!(result.is_ok());
    }
}
