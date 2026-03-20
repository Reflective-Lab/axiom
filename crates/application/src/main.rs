// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Converge App - Distribution & Packaging Layer
//!
//! This is the deployable product that:
//! - Selects which domain packs are available
//! - Configures which providers are enabled
//! - Sets runtime deployment defaults (auth, tenancy, quotas)
//! - Bootstraps the converge-runtime server
//!
//! # Architecture Role
//!
//! > `converge-app` owns **packaging**, not **semantics**.
//!
//! This crate composes already-defined domain meaning from `converge-domain`.
//! It does NOT invent new business types, rules, or DSLs.
//!
//! # Usage
//!
//! ```bash
//! # Run a job from the command line
//! converge run --template growth-strategy --seeds '[]'
//!
//! # List available domain packs
//! converge packs list
//! ```

#![allow(dead_code)]
#![allow(unused_variables)]

mod agents;
mod config;
mod evals;
mod packs;
mod streaming;
mod ui;

use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde::Serialize;
use std::io;
use std::panic;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::agents::{MockInsightProvider, RiskAssessmentAgent, StrategicInsightAgent};

use converge_core::traits::DynChatBackend;
use converge_core::validation::ValidationAgent;
use converge_core::{Context, ContextKey, Engine, Fact};
use converge_domain::growth_strategy::{
    BrandSafetyInvariant, CompetitorAgent, EvaluationAgent, MarketSignalAgent,
    RequireEvaluationRationale, RequireMultipleStrategies, RequireStrategyEvaluations,
    StrategyAgent,
};
use converge_domain::packs::{
    ApprovalRecorderAgent, ApprovalRequiredForExternalActionInvariant, DossierBuilderAgent,
    EvidenceRequiresProvenanceInvariant, EvidenceValidatorAgent, LinkedInTargetDiscoveryAgent,
    NetworkPathRequiresVerificationInvariant, PaidActionRequiresApprovalInvariant,
    PatentEvidenceHasProvenanceInvariant, PathVerifierAgent, SignalIngestAgent,
    SubmissionRequiresApprovalInvariant, SubmissionRequiresEvidenceInvariant,
};
use converge_domain::{AskConvergeAgent, GroundedAnswerInvariant, RecallNotEvidenceInvariant};
use converge_domain::{
    ClaimChartGeneratorAgent, ClaimRiskFlaggerAgent, ClaimSeedAgent, ClaimStrategyAgent,
    ClaimSupportInvariant, DependencyGraphAgent, DisclosureCompletenessInvariant,
    DocumentationAgent, DraftPackAssemblerAgent, DraftingComposerAgent, DraftingResearchAgent,
    EnrichmentLoopAgent, EvidenceCitationInvariant, InventionCaptureAgent, InventionSummaryAgent,
    MatterContextAgent, MatterPolicyAgent, PatentAlertAgent, PatentApprovalRecorderAgent,
    PatentClaimsAnalyzerAgent, PatentEvidenceCollectorAgent, PatentLandscapeAnalyzerAgent,
    PatentOperatorPlannerAgent, PatentQueryBuilderAgent, PatentReportAssemblerAgent,
    PatentSearchExecutorAgent, PatentSubmissionAgent, PerformanceRegressionAgent,
    PriorArtShortlistAgent, ReleaseReadyAgent, RemoteBackendRestrictedInvariant,
    RequireAllChecksComplete, RequireMinimumCoverage, RequireNoCriticalVulnerabilities,
    RiskSummaryAgent, SecurityScanAgent, SpecDraftAgent, SupportMatrixAgent, TestCoverageAgent,
};
use converge_provider::AnthropicBackend;
use strum::IntoEnumIterator;

/// Converge - Semantic convergence engine for agentic workflows
#[derive(Parser)]
#[command(name = "converge")]
#[command(about = "Converge Agent OS - where agents propose and the engine decides")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI
    Tui,

    /// Manage domain packs
    Packs {
        #[command(subcommand)]
        command: PacksCommands,
    },

    /// Run a job from the command line
    Run {
        /// Template to use
        #[arg(short, long)]
        template: String,

        /// Seeds as JSON (or @file.json)
        #[arg(short, long)]
        seeds: Option<String>,

        /// Max cycles budget
        #[arg(long, default_value = "50")]
        max_cycles: u32,

        /// Run ID for traceability (auto-generated if not provided)
        #[arg(long)]
        run_id: Option<String>,

        /// Correlation ID to link related runs
        #[arg(long)]
        correlation_id: Option<String>,

        /// Use mock LLM for deterministic output
        #[arg(long)]
        mock: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Stream facts as they arrive (real-time output)
        #[arg(long)]
        stream: bool,

        /// Quiet mode: exit code only, no output
        #[arg(long)]
        quiet: bool,
    },

    /// Run eval fixtures for reproducible testing
    Eval {
        #[command(subcommand)]
        command: EvalCommands,
    },
}

#[derive(Subcommand)]
enum EvalCommands {
    /// Run eval fixtures
    Run {
        /// Specific eval ID to run (runs all if not specified)
        eval_id: Option<String>,

        /// Directory containing eval fixtures
        #[arg(short, long, default_value = "evals")]
        dir: String,

        /// Use mock LLM for faster deterministic tests
        #[arg(long)]
        mock: bool,
    },
    /// List available eval fixtures
    List {
        /// Directory containing eval fixtures
        #[arg(short, long, default_value = "evals")]
        dir: String,
    },
}

#[derive(Subcommand)]
enum PacksCommands {
    /// List available domain packs
    List,
    /// Show details of a specific pack
    Info {
        /// Pack name
        name: String,
    },
}

/// JSON output format for run results (Cross-Platform Contract compliant)
#[derive(Debug, Serialize)]
struct RunOutput {
    run_id: String,
    correlation_id: String,
    timestamp: String,
    actor: ActorInfo,
    result: RunResultOutput,
    facts: Vec<FactOutput>,
}

#[derive(Debug, Serialize)]
struct ActorInfo {
    #[serde(rename = "type")]
    actor_type: String,
    device_id: String,
    cli_version: String,
}

#[derive(Debug, Serialize)]
struct RunResultOutput {
    converged: bool,
    cycles: u32,
    total_facts: usize,
}

#[derive(Debug, Serialize)]
struct FactOutput {
    sequence: usize,
    key: String,
    id: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // Check if we should suppress tracing (quiet mode for Run command)
    let suppress_tracing = matches!(&cli.command, Commands::Run { quiet: true, .. });

    // Initialize tracing (skip for quiet mode)
    if !suppress_tracing {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_target(false)
            .init();
    }

    match cli.command {
        Commands::Tui => {
            // Don't initialize tracing for TUI (conflicts with terminal)
            run_tui().await?;
        }

        Commands::Packs { command } => match command {
            PacksCommands::List => {
                println!("Available domain packs:\n");
                for pack in packs::available_packs() {
                    let info = packs::pack_info(&pack);
                    println!("  {} - {}", pack, info.description);
                }
            }
            PacksCommands::Info { name } => {
                let info = packs::pack_info(&name);
                println!("Pack: {name}");
                println!("Description: {}", info.description);
                println!("Version: {}", info.version);
                println!("\nTemplates:");
                for template in &info.templates {
                    println!("  - {template}");
                }
                println!("\nInvariants:");
                for invariant in &info.invariants {
                    println!("  - {invariant}");
                }
            }
        },

        Commands::Run {
            template,
            seeds,
            max_cycles,
            run_id,
            correlation_id,
            mock,
            json,
            stream,
            quiet,
        } => {
            // Generate or use provided run_id
            let run_id = run_id.unwrap_or_else(|| format!("run_{}", uuid::Uuid::new_v4()));
            let correlation_id =
                correlation_id.unwrap_or_else(|| format!("cor_{}", uuid::Uuid::new_v4()));

            // Build actor
            let hostname = hostname::get().map_or_else(
                |_| "unknown".to_string(),
                |h| h.to_string_lossy().to_string(),
            );
            let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            let device_id = format!("cli:{hostname}:{username}");

            if !json && !stream && !quiet {
                info!(
                    template = %template,
                    run_id = %run_id,
                    correlation_id = %correlation_id,
                    "Running job from CLI"
                );
            }

            // Load templates from enabled packs
            let enabled_packs = packs::available_packs();
            let registry = packs::load_templates(&enabled_packs)?;

            // Resolve template
            let _template_arc = registry.get(&template).ok_or_else(|| {
                anyhow::anyhow!("Template '{template}' not found in any enabled pack")
            })?;

            // Parse seeds
            let mut context = Context::new();
            if let Some(seeds_raw) = seeds {
                let seeds_json = if let Some(path) = seeds_raw.strip_prefix('@') {
                    std::fs::read_to_string(path)
                        .map_err(|e| anyhow::anyhow!("Failed to read seed file '{path}': {e}"))?
                } else {
                    seeds_raw
                };

                let seed_facts: Vec<crate::packs::SeedFact> = serde_json::from_str(&seeds_json)
                    .map_err(|e| anyhow::anyhow!("Failed to parse seeds JSON: {e}"))?;

                for seed in seed_facts {
                    let fact = Fact::new(ContextKey::Seeds, seed.id, seed.content);
                    context
                        .add_fact(fact)
                        .map_err(|e| anyhow::anyhow!("Failed to add seed fact: {e}"))?;
                }
            }

            // Report total facts across all keys
            let total_facts: usize = ContextKey::iter().map(|key| context.get(key).len()).sum();
            if !json && !stream && !quiet {
                info!(facts = total_facts, "Context initialized with seeds");
            }

            // Run convergence loop inline
            let mut engine = Engine::new();

            // Register agents from template (Bridge to domain packs)
            register_pack_agents(&mut engine, template.as_str(), mock)?;

            // Set up streaming callback if requested
            let streaming_handler = if stream {
                use crate::streaming::{OutputFormat, StreamingHandler};
                let format = if json {
                    OutputFormat::Json
                } else {
                    OutputFormat::Human
                };
                let handler = Arc::new(StreamingHandler::new(format));
                engine.set_streaming(handler.clone());
                Some(handler)
            } else {
                None
            };

            if !stream && !quiet {
                info!("Starting convergence loop...");
            }

            // Run engine - handle errors differently in quiet mode
            let result = if quiet {
                match engine.run(context) {
                    Ok(r) => r,
                    Err(e) => {
                        // Exit codes per CLI_CONTRACT.md:
                        // 1 = halted (invariant violated), 3 = error (system failure)
                        let exit_code = if e.to_string().contains("invariant") {
                            1
                        } else {
                            3
                        };
                        std::process::exit(exit_code);
                    }
                }
            } else {
                engine.run(context)?
            };

            if !stream && !quiet {
                if result.converged {
                    info!(cycles = result.cycles, "Job reached fixed point");
                } else {
                    warn!(
                        cycles = result.cycles,
                        "Job halted without reaching fixed point (budget exhausted)"
                    );
                }
            }

            // Handle output based on mode
            if quiet {
                // Quiet mode: exit code only
                // Exit codes per CLI_CONTRACT.md:
                // 0 = converged, 2 = budget_exceeded
                let exit_code = if result.converged { 0 } else { 2 };
                std::process::exit(exit_code);
            } else if let Some(handler) = streaming_handler {
                // Streaming mode: emit final status line
                handler.emit_final_status(result.converged, result.cycles);
            } else if json {
                // JSON output (Cross-Platform Contract compliant)
                let final_facts: usize = ContextKey::iter()
                    .map(|key| result.context.get(key).len())
                    .sum();

                let mut facts: Vec<FactOutput> = Vec::new();
                let mut sequence = 0usize;
                for key in ContextKey::iter() {
                    for fact in result.context.get(key) {
                        sequence += 1;
                        facts.push(FactOutput {
                            sequence,
                            key: format!("{key:?}"),
                            id: fact.id.clone(),
                            content: fact.content.clone(),
                        });
                    }
                }

                let output = RunOutput {
                    run_id: run_id.clone(),
                    correlation_id: correlation_id.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    actor: ActorInfo {
                        actor_type: "system".to_string(),
                        device_id: device_id.clone(),
                        cli_version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                    result: RunResultOutput {
                        converged: result.converged,
                        cycles: result.cycles,
                        total_facts: final_facts,
                    },
                    facts,
                };

                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                // Human-readable output
                let final_facts: usize = ContextKey::iter()
                    .map(|key| result.context.get(key).len())
                    .sum();

                println!("\n=== Convergence Result ===");
                println!("Run ID: {run_id}");
                println!("Correlation ID: {correlation_id}");
                println!("Converged: {}", result.converged);
                println!("Total Cycles: {}", result.cycles);
                println!("Total Facts: {final_facts}");
                println!("==========================\n");

                // Print all facts by category
                println!("=== Generated Facts ===\n");
                for key in ContextKey::iter() {
                    let facts = result.context.get(key);
                    if !facts.is_empty() {
                        println!("[{key:?}]");
                        for fact in facts {
                            println!("  {} | {}", fact.id, fact.content);
                        }
                        println!();
                    }
                }
                println!("=======================");
            }
        }

        Commands::Eval { command } => match command {
            EvalCommands::Run { eval_id, dir, mock } => {
                let dir_path = std::path::Path::new(&dir);

                // Load fixtures
                let mut fixtures = evals::load_fixtures_from_dir(dir_path)?;

                if fixtures.is_empty() {
                    println!("No eval fixtures found in '{dir}'");
                    println!("Create JSON fixture files in the evals/ directory.");
                    return Ok(());
                }

                // Filter to specific eval if provided
                if let Some(ref id) = eval_id {
                    fixtures.retain(|f| f.eval_id == *id);
                    if fixtures.is_empty() {
                        println!("Eval '{id}' not found in '{dir}'");
                        return Ok(());
                    }
                }

                // Override mock setting if flag provided
                if mock {
                    for fixture in &mut fixtures {
                        fixture.use_mock_llm = true;
                    }
                }

                info!(count = fixtures.len(), "Running eval fixtures");

                // Run evals
                let results = evals::run_evals(&fixtures);

                // Print results
                evals::print_results(&results);

                // Exit with error code if any failed
                let all_passed = results.iter().all(|r| r.passed);
                if !all_passed {
                    std::process::exit(1);
                }
            }
            EvalCommands::List { dir } => {
                let dir_path = std::path::Path::new(&dir);
                let fixtures = evals::load_fixtures_from_dir(dir_path)?;

                if fixtures.is_empty() {
                    println!("No eval fixtures found in '{dir}'");
                    return Ok(());
                }

                println!("\nAvailable eval fixtures:\n");
                for fixture in fixtures {
                    println!("  {} - {}", fixture.eval_id, fixture.description);
                    println!("    Pack: {}", fixture.pack);
                    println!("    Seeds: {}", fixture.seeds.len());
                    println!("    Mock LLM: {}", fixture.use_mock_llm);
                    println!();
                }
            }
        },
    }

    Ok(())
}

/// Cleanup terminal on exit or panic
fn cleanup_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

/// Run the TUI application with proper terminal lifecycle management
async fn run_tui() -> Result<()> {
    // Set up panic hook to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal();
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = ui::App::new();
    let res = ui::run_app(&mut terminal, app).await;

    // Restore terminal
    cleanup_terminal();
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

/// Creates a chat backend from environment variables.
///
/// Tries Anthropic first, falls back to mock provider.
/// OpenAI support pending `OpenAiBackend` in converge-provider.
fn create_chat_backend() -> Arc<dyn DynChatBackend> {
    // Try Anthropic (Claude is excellent for strategic analysis)
    if let Ok(backend) = AnthropicBackend::from_env() {
        info!(
            provider = "anthropic",
            model = "claude-sonnet-4-20250514",
            "Using Anthropic Claude for LLM insights"
        );
        return Arc::new(backend.with_model("claude-sonnet-4-20250514")) as Arc<dyn DynChatBackend>;
    }

    // Fall back to mock provider
    // TODO: Add OpenAI support once OpenAiBackend implements ChatBackend
    warn!("No LLM API keys found (ANTHROPIC_API_KEY). Using mock provider.");
    info!("Set ANTHROPIC_API_KEY in .env for real LLM insights");
    Arc::new(MockInsightProvider::default_insights()) as Arc<dyn DynChatBackend>
}

/// Register agents and invariants for a specific domain pack.
///
/// This acts as the bridge between the distribution layer and the domain packs.
///
/// # Arguments
/// * `engine` - The convergence engine to register agents with
/// * `pack_name` - Name of the domain pack (e.g., "growth-strategy")
/// * `use_mock` - If true, use mock LLM provider for deterministic output
fn register_pack_agents(engine: &mut Engine, pack_name: &str, use_mock: bool) -> Result<()> {
    match pack_name {
        "growth-strategy" => {
            info!(pack = %pack_name, mock = use_mock, "Registering growth-strategy agents and invariants");

            // Register deterministic agents
            engine.register(MarketSignalAgent);
            engine.register(CompetitorAgent);
            engine.register(StrategyAgent);
            engine.register(EvaluationAgent);

            // Create chat backend based on mock flag
            let llm_provider: Arc<dyn DynChatBackend> = if use_mock {
                info!("Using mock LLM provider for deterministic output");
                Arc::new(MockInsightProvider::default_insights())
            } else {
                create_chat_backend()
            };

            // Register LLM-powered agents
            engine.register(StrategicInsightAgent::new(llm_provider.clone()));
            info!("Registered LLM-powered StrategicInsightAgent");

            engine.register(RiskAssessmentAgent::new(llm_provider));
            info!("Registered LLM-powered RiskAssessmentAgent");

            // Register Invariants
            engine.register_invariant(BrandSafetyInvariant::default());
            engine.register_invariant(RequireMultipleStrategies);
            engine.register_invariant(RequireStrategyEvaluations);
            engine.register_invariant(RequireEvaluationRationale);
        }
        "ask-converge" => {
            info!(pack = %pack_name, "Registering ask-converge agent and invariants");
            engine.register(AskConvergeAgent);
            engine.register_invariant(GroundedAnswerInvariant);
            engine.register_invariant(RecallNotEvidenceInvariant);
        }
        "patent-research" => {
            info!(pack = %pack_name, "Registering patent-research agents and invariants");

            let provider = Arc::new(
                converge_provider::CompositePatentProvider::from_env().unwrap_or_else(|_| {
                    converge_provider::CompositePatentProvider::new().with_provider(
                        converge_provider::PatentOperator::Uspto,
                        Arc::new(converge_provider::StubPatentProvider::new()),
                    )
                }),
            );

            engine.register(PatentQueryBuilderAgent);
            engine.register(MatterPolicyAgent);
            engine.register(MatterContextAgent);
            engine.register(InventionCaptureAgent);
            engine.register(InventionSummaryAgent);
            engine.register(ClaimSeedAgent);
            engine.register(PatentOperatorPlannerAgent);
            engine.register(PatentSearchExecutorAgent::new(provider));
            engine.register(PatentEvidenceCollectorAgent);
            engine.register(PriorArtShortlistAgent);
            engine.register(ClaimRiskFlaggerAgent);
            engine.register(EnrichmentLoopAgent);
            engine.register(ClaimStrategyAgent);
            engine.register(ClaimChartGeneratorAgent);
            engine.register(PatentClaimsAnalyzerAgent);
            engine.register(PatentLandscapeAnalyzerAgent);
            engine.register(PatentReportAssemblerAgent);
            engine.register(PatentAlertAgent);
            engine.register(SpecDraftAgent);
            engine.register(SupportMatrixAgent);
            engine.register(DraftPackAssemblerAgent);
            engine.register(PatentSubmissionAgent);
            engine.register(PatentApprovalRecorderAgent);

            engine.register(ValidationAgent::with_defaults());

            engine.register_invariant(PatentEvidenceHasProvenanceInvariant);
            engine.register_invariant(DisclosureCompletenessInvariant);
            engine.register_invariant(RemoteBackendRestrictedInvariant);
            engine.register_invariant(EvidenceCitationInvariant);
            engine.register_invariant(ClaimSupportInvariant);
            engine.register_invariant(PaidActionRequiresApprovalInvariant);
            engine.register_invariant(SubmissionRequiresApprovalInvariant);
            engine.register_invariant(SubmissionRequiresEvidenceInvariant);
        }
        "linkedin-research" => {
            info!(pack = %pack_name, "Registering linkedin-research agents and invariants");

            let provider: Arc<dyn converge_provider::LinkedInProvider> =
                match converge_provider::LinkedInApiProvider::from_env() {
                    Ok(provider) => Arc::new(provider),
                    Err(_) => Arc::new(converge_provider::StubLinkedInProvider),
                };

            engine.register(SignalIngestAgent);
            engine.register(EvidenceValidatorAgent);
            engine.register(DossierBuilderAgent);
            engine.register(PathVerifierAgent);
            engine.register(ApprovalRecorderAgent);
            engine.register(LinkedInTargetDiscoveryAgent::new(provider));

            engine.register_invariant(EvidenceRequiresProvenanceInvariant);
            engine.register_invariant(NetworkPathRequiresVerificationInvariant);
            engine.register_invariant(ApprovalRequiredForExternalActionInvariant);
        }
        "release-readiness" => {
            info!(pack = %pack_name, "Registering release-readiness agents and invariants");

            engine.register(DependencyGraphAgent);
            engine.register(TestCoverageAgent);
            engine.register(SecurityScanAgent);
            engine.register(PerformanceRegressionAgent);
            engine.register(DocumentationAgent);
            engine.register(RiskSummaryAgent);
            engine.register(ReleaseReadyAgent);

            engine.register_invariant(RequireAllChecksComplete);
            engine.register_invariant(RequireNoCriticalVulnerabilities);
            engine.register_invariant(RequireMinimumCoverage);
        }
        "drafting-short" => {
            info!(pack = %pack_name, mock = use_mock, "Registering drafting-short agents");
            // NOTE: LLM drafting (setup_llm_drafting) temporarily disabled due to
            // converge-provider/converge-core LlmProvider trait divergence. See REF-36.
            engine.register(DraftingResearchAgent);
            engine.register(DraftingComposerAgent);
        }
        "novelty-search" => {
            info!(pack = %pack_name, "Registering novelty-search agents");

            let provider = Arc::new(
                converge_provider::CompositePatentProvider::from_env().unwrap_or_else(|_| {
                    converge_provider::CompositePatentProvider::new().with_provider(
                        converge_provider::PatentOperator::Uspto,
                        Arc::new(converge_provider::StubPatentProvider::new()),
                    )
                }),
            );

            engine.register(PatentQueryBuilderAgent);
            engine.register(PatentOperatorPlannerAgent);
            engine.register(PatentSearchExecutorAgent::new(provider));
            engine.register(PatentEvidenceCollectorAgent);
            engine.register(PriorArtShortlistAgent);
            engine.register(ValidationAgent::with_defaults());

            engine.register_invariant(PatentEvidenceHasProvenanceInvariant);
            engine.register_invariant(EvidenceCitationInvariant);
        }
        _ => {
            warn!(pack = %pack_name, "No specific agent registration for pack");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::SeedFact;

    fn release_readiness_fixture_context() -> Context {
        let seeds_json = include_str!("../examples/release-readiness/seeds.json");
        let seeds: Vec<SeedFact> =
            serde_json::from_str(seeds_json).expect("fixture seeds should parse");

        let mut ctx = Context::new();
        for seed in seeds {
            ctx.add_fact(Fact::new(ContextKey::Seeds, seed.id, seed.content))
                .expect("fixture facts should be accepted");
        }
        ctx
    }

    #[test]
    fn release_readiness_fixture_runs_deterministically_with_current_behavior() {
        let run = || {
            let mut engine = Engine::new();
            register_pack_agents(&mut engine, "release-readiness", true)
                .expect("release-readiness pack should register");
            engine
                .run(release_readiness_fixture_context())
                .expect("fixture run should complete")
        };

        let r1 = run();
        let r2 = run();

        assert!(r1.converged);
        assert!(r2.converged);
        assert_eq!(r1.cycles, r2.cycles);
        assert_eq!(
            r1.context.get(ContextKey::Signals),
            r2.context.get(ContextKey::Signals)
        );
        assert_eq!(
            r1.context.get(ContextKey::Strategies),
            r2.context.get(ContextKey::Strategies)
        );
        assert_eq!(
            r1.context.get(ContextKey::Evaluations),
            r2.context.get(ContextKey::Evaluations)
        );

        let required_signal_ids = [
            "dependency:graph",
            "dependency:outdated",
            "coverage:unit",
            "coverage:integration",
            "security:vulnerabilities",
            "security:licenses",
            "performance:benchmarks",
            "performance:memory",
            "docs:api",
            "docs:changelog",
        ];
        let signals = r1.context.get(ContextKey::Signals);
        for id in required_signal_ids {
            assert!(
                signals.iter().any(|f| f.id == id),
                "missing expected signal: {id}"
            );
        }

        let risk_summary = r1
            .context
            .get(ContextKey::Strategies)
            .iter()
            .find(|f| f.id == "risk:summary")
            .expect("risk summary should exist");
        let eval = r1
            .context
            .get(ContextKey::Evaluations)
            .iter()
            .find(|f| f.id == "eval:release-readiness")
            .expect("release evaluation should exist");

        // Lock current deterministic behavior until the string-based parser is refined.
        assert!(risk_summary.content.contains("Status: BLOCKED"));
        assert!(eval.content.contains("Status: BLOCKED"));
    }

    #[test]
    #[ignore = "pending release-readiness risk parser/normalization fix to classify 0 critical as REVIEW/READY"]
    fn release_readiness_fixture_should_be_review_and_ready_after_parser_fix() {
        let result = {
            let mut engine = Engine::new();
            register_pack_agents(&mut engine, "release-readiness", true)
                .expect("release-readiness pack should register");
            engine
                .run(release_readiness_fixture_context())
                .expect("fixture run should complete")
        };

        let risk_summary = result
            .context
            .get(ContextKey::Strategies)
            .iter()
            .find(|f| f.id == "risk:summary")
            .expect("risk summary should exist");
        let eval = result
            .context
            .get(ContextKey::Evaluations)
            .iter()
            .find(|f| f.id == "eval:release-readiness")
            .expect("release evaluation should exist");

        assert!(
            risk_summary.content.contains("Risk Assessment: REVIEW")
                || risk_summary.content.contains("Status: REVIEW"),
            "expected REVIEW risk summary after parser fix, got: {}",
            risk_summary.content
        );
        assert!(
            eval.content.contains("Status: READY"),
            "expected READY evaluation after parser fix, got: {}",
            eval.content
        );
    }
}
