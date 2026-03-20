# AGENTS.md — Converge AI Assistant Guide

> Converge is a vision for **semantic governance**. We move from fragmented intent to unified, converged states through a deterministic alignment engine. Our mission is to provide a stable foundation for complex decision-making where human authority and AI agency coexist in a transparent, explainable ecosystem.

**For AI coding assistants (Claude, Gemini, Codex, Cursor, etc.)**

This document provides comprehensive guidance for AI assistants working on the Converge codebase. It consolidates architecture, coding standards, and development patterns into a single reference.

---

## Knowledge Base

All project documentation lives in `../converge-business/`. Do NOT create documentation files in this repo — reference converge-business instead.

- **Best practices:** `../converge-business/strategy/`
- **Domain packs:** `../converge-business/packs/`
- **Architecture docs:** `../converge-business/knowledgebase/platform-docs/architecture/`
- **Development docs:** `../converge-business/knowledgebase/platform-docs/development/`
- **Component specs:** `../converge-business/knowledgebase/components/`
- **Use cases:** `../converge-business/knowledgebase/platform-docs/use-cases/`

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Core Philosophy](#core-philosophy)
3. [Architecture Fundamentals](#architecture-fundamentals)
4. [Code Standards](#code-standards)
5. [Development Workflow](#development-workflow)
6. [File Organization](#file-organization)
7. [Testing Requirements](#testing-requirements)
8. [Common Patterns](#common-patterns)
9. [What NOT to Do](#what-not-to-do)

---

## Project Overview

**Converge** is a correctness-first, context-driven multi-agent runtime system written in Rust. It enables building systems where agents collaborate through shared context to reach provable convergence.

**Key Characteristics:**

- **Not a workflow engine** — No predefined steps or control flow
- **Not an actor system** — No message passing or mailboxes
- **Not event-driven** — No implicit control flow
- **Convergence-based** — Execution proceeds until fixed point
- **Deterministic** — Same inputs produce same outputs
- **Explainable** — Every outcome is traceable

**Workspace Structure:**

```
converge-core/      # Core engine (private library)
converge-provider/  # LLM provider implementations
converge-domain/    # Domain-specific agents and use cases
converge-runtime/   # HTTP/gRPC/TUI server
converge-tool/      # Development utilities (Gherkin parsing)
```

---

## Core Philosophy

### Design Tenets (Non-Negotiable)

1. **Explicit Authority** — Every root intent has a single semantic authority. Nothing decides implicitly.

2. **Convergence Over Control Flow** — The system progresses by reaching a fixed point, not by executing predefined steps.

3. **Append-Only Truth** — Facts are added, never mutated. History is preserved.

4. **Agents Suggest, Engines Decide** — Agents contribute ideas. The engine enforces truth.

5. **Safety by Construction** — Invalid states must be unrepresentable.

6. **Transparent Determinism** — Every outcome must be explainable and reconstructible.

7. **Human Authority Is First-Class** — Humans pause, approve, and decide. Waiting is a valid state.

8. **No Hidden Work** — No background tasks, no silent retries, no eventual outcomes.

9. **Scale by Intent Replication** — Scale comes from running more intents, not from distributing authority.

### Core Principles

- **Context is the API** — Agents collaborate through data, not calls
- **Agents never call each other** — All communication via shared context
- **Convergence is mandatory** — Execution proceeds until fixed point
- **LLMs are tools, never authorities** — LLM outputs are suggestions, not facts
- **Gherkin constrains behavior** — It does not control execution
- **Correctness over availability** — Wrong answers are worse than no answers

---

## Architecture Fundamentals

### Agent Trait

All agents implement this trait:

```rust
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> &[ContextKey];
    fn accepts(&self, ctx: &Context) -> bool;  // Pure, no side effects
    fn execute(&self, ctx: &Context) -> AgentEffect;  // Read-only
}
```

**Critical Rules:**

- `accepts()` must be **pure** — no side effects, no mutations
- `execute()` is **read-only** — cannot mutate context directly
- Agents **never call other agents** — only read context and emit effects
- Effects are merged by the engine in deterministic order
- **Idempotency must be context-based** — check for existing contributions in context, not internal state
- **LLM agents must check both `Proposals` and `target_key`** for idempotency (see LLM Integration Pattern below)

### Context Keys

The shared context uses typed keys:

- `Seeds` — Initial inputs
- `Hypotheses` — Proposed ideas
- `Strategies` — Action plans
- `Constraints` — Limitations and rules
- `Signals` — Observations and data
- `Competitors` — Competitive intelligence
- `Evaluations` — Assessments and ratings
- `Proposals` — LLM-generated suggestions (require validation)
- `Diagnostic` — Error and debugging information

### Type Boundary: ProposedFact vs Fact

**Critical separation:**

- `ProposedFact` — Suggestions from non-authoritative sources (e.g., LLMs)
- `Fact` — Validated, authoritative assertions
- LLMs can only emit `ProposedFact`
- Explicit validation required to promote `ProposedFact` → `Fact`
- This boundary is enforced by the type system

### Engine Execution Model

1. **Register agents** — Builds dependency index
2. **Initialize context** — From RootIntent
3. **Convergence loop:**
   - Find eligible agents (dirty keys + dependencies)
   - Execute in parallel (Rayon)
   - Merge effects serially (deterministic by AgentId order)
   - Detect fixed point (no new facts = converged)
4. **Return ConvergeResult**

### LLM Integration Pattern

LLM agents follow this pattern:

1. **Specify requirements** — Cost, latency, capabilities via `AgentRequirements`
2. **Model selection** — `ProviderRegistry` selects appropriate model
3. **Create provider** — Via `create_provider()`
4. **Instantiate agent** — `LlmAgent` with provider and config
5. **Emit proposals** — LLM outputs go to `ContextKey::Proposals`
6. **Validation** — `ValidationAgent` promotes proposals to facts

**Example:**

```rust
use converge_domain::llm_utils::{create_llm_agent, requirements};

let agent = create_llm_agent(
    "MarketAnalyst",
    "You are a market analyst.",
    "Analyze: {context}",
    ContextKey::Signals,
    vec![ContextKey::Seeds],
    requirements::analysis(),
    &registry,
)?;
```

**Critical: LlmAgent Idempotency Pattern**

`LlmAgent` uses context-based idempotency following the canonical pattern from `ENGINE_EXECUTION_MODEL.md`. However, **LLM agents emit to `Proposals` first**, so the idempotency check must check **both** places:

```rust
fn accepts(&self, ctx: &Context) -> bool {
    // Precondition: at least one input dependency has data
    let has_input = self.config.dependencies.iter().any(|k| ctx.has(*k));
    if !has_input {
        return false;
    }

    // Idempotency: check if we've already contributed
    let my_prefix = format!("{}-", self.name);
    
    // Check Proposals (pending contributions before validation)
    let has_pending_proposal = ctx
        .get(ContextKey::Proposals)
        .iter()
        .any(|f| {
            // Proposal IDs are: "proposal:{target_key}:{agent_name}-{uuid}"
            f.id.contains(&my_prefix)
        });
    
    // Check target_key (validated contributions after validation)
    let has_validated_fact = ctx
        .get(self.config.target_key)
        .iter()
        .any(|f| f.id.starts_with(&my_prefix));

    // Run if we haven't contributed (no pending proposal AND no validated fact)
    !has_pending_proposal && !has_validated_fact
}
```

**Why both checks are needed:**

- Agent emits to `Proposals` (encoded as `proposal:{target_key}:{agent_name}-{uuid}`)
- ValidationAgent promotes to `target_key` (with ID `{agent_name}-{uuid}`)
- Agent must check both to avoid duplicate execution
- This ensures idempotency throughout the proposal → validation → fact pipeline

**Known Issue**: Current implementation only checks `target_key`. This can cause agents to miss execution opportunities in multi-step pipelines. See `docs/use-cases/GROWTH_STRATEGY_FLOW_ANALYSIS.md` for details.

---

## Code Standards

### Rust Edition & Toolchain

- **Rust Edition 2024** for all new code
- **Minimum Rust 1.76+**
- Never downgrade toolchain for stale dependencies

### Error Handling (Strict)

- **NO `unwrap()`, `expect()`, or `panic!`** in production paths
- Use `thiserror` for domain errors
- One error enum per bounded context/module
- Prefer `Result<T, DomainError>` internally
- Map to transport errors (HTTP/gRPC) only at boundaries
- **NO `anyhow` in libraries or domain layers** — only at bin boundaries

### Code Quality

Always enable clippy & fmt:

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

**Immutability:**

- Default to `let`, not `let mut`, unless required
- Prefer iterator transforms over mutating loops
- Avoid unnecessary `clone()` — justify at ownership boundaries

**Data Modeling:**

- Use newtypes for IDs: `struct IntentId(String);` not `String`
- Prefer enums over boolean flags or magic strings
- Keep structs focused — avoid "god structs"
- Derive traits intentionally (don't cargo-cult `Clone`, `Debug`)

**Async & Concurrency:**

- Never hold locks across `.await`
- Use `tokio::sync` primitives in async code
- Spawn tasks only when concurrency is required
- Add timeouts around external calls (`tokio::time::timeout`)

### Technology Stack (Mandatory)

**Core Framework:**

- **Axum** — Default API framework (internal & external)
- **Tokio** — Only async runtime allowed (no mixed runtimes)
- **Tonic** — gRPC for internal service-to-service communication

**Data Layer:**

- **SurrealDB** — Primary system-of-record
- **Qdrant** — Production vector search
- **LanceDB** — Local/embedded vector workflows
- **Apache Arrow + DataFusion** — Analytics workloads

**Messaging & Workflows:**

- **NATS** — Default message bus (with JetStream for durability)
- **Temporal** — Long-running workflows, retries, distributed coordination

**Secrets & Config:**

- **config crate** — Layered config (base.yaml → env-specific → env vars → secrets)
- **Google Secret Manager** (GCP) or **Vault** (self-host) — No plain `.env` in production
- **OpenRouter** — Default LLM API aggregator

**Observability (Mandatory):**

- **OpenTelemetry** — Traces
- **Prometheus** — Metrics
- **tracing** — Structured logging
- Every request creates a root span in handler
- Every public service/repository method must run inside a span

---

## Development Workflow

### Build & Test Commands

```bash
# Building
just build              # Build all crates (release)
just build-core         # Build converge-core only
just build-runtime      # Build runtime server

# Testing
just test               # Run all tests
just test-core          # Test core only
just test-runtime       # Test runtime only

# Running
just run-server         # Start HTTP server
just run-server-trace   # Start with RUST_LOG=info

# Code Quality
just fmt                # Format code
just lint               # Run clippy with -D warnings
```

### Git Commit Format

```
domain(subsystem): description

Examples:
- chore(engine): Update converge-core with eligibility efficiency
- feat(domain): Add LLM-enabled growth strategy agents
- fix(provider): Handle rate limit errors gracefully
```

### Testing Requirements

- **TDD**: Red → Green → Refactor
- Target >80% code coverage for new code
- Use `proptest` for property-based testing
- All changes must pass `cargo clippy -- -D warnings`
- Unit tests in `#[cfg(test)]` modules
- Integration tests in `<crate>/tests/`

---

## File Organization

### Documentation Structure

```
docs/
├── architecture/          # System architecture & execution model
├── agents/                # Agent model & LLM integration
├── governance/            # Design tenets & principles
├── testing/               # Testing & invariants
├── product/               # Product guide & strategy
├── deployment/            # Tech stack & deployment
├── use-cases/             # Use case examples
├── reference/             # Reference comparisons
├── development/           # Implementation details
├── assistant-guides/      # AI assistant guides
├── internal/              # Internal documentation
└── public/                # Public API documentation
```

### Key Documentation Files

| Need | Location |
|------|----------|
| Design principles | `docs/governance/GOVERNANCE.md` |
| Architecture | `docs/architecture/ARCHITECTURE.md` |
| Agent lifecycle | `docs/agents/AGENT_LIFECYCLE.md` |
| LLM integration | `docs/agents/LLM_INTEGRATION.md` |
| Use cases | `docs/use-cases/` |
| Development status | `docs/development/STATUS.md` |
| Locked decisions | `docs/development/DECISIONS.md` |
| Contribution rules | `CONTRIBUTING.md` |

---

## Common Patterns

### Creating a New Agent

```rust
pub struct MyAgent;

impl Agent for MyAgent {
    fn name(&self) -> &str {
        "MyAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds, ContextKey::Signals]
    }

    fn accepts(&self, ctx: &Context) -> bool {
        // Pure predicate: check if dependencies are satisfied
        ctx.has(ContextKey::Seeds) && !ctx.has(ContextKey::Strategies)
    }

    fn execute(&self, ctx: &Context) -> AgentEffect {
        // Read-only: extract data from context
        let seeds = ctx.get(ContextKey::Seeds);
        
        // Process and emit effects
        let strategies: Vec<Fact> = seeds
            .iter()
            .map(|seed| Fact {
                key: ContextKey::Strategies,
                id: format!("strategy:{}", seed.id),
                content: format!("Strategy for {}", seed.content),
            })
            .collect();
        
        AgentEffect::with_facts(strategies)
    }
}
```

### Creating an LLM Agent

```rust
use converge_domain::llm_utils::{create_llm_agent, requirements};
use converge_provider::ProviderRegistry;

let registry = ProviderRegistry::from_env();
let agent = create_llm_agent(
    "MarketAnalyst",
    "You are a market analyst.",
    "Analyze market: {context}",
    ContextKey::Signals,
    vec![ContextKey::Seeds],
    requirements::analysis(),  // Cost, latency, capabilities
    &registry,
)?;
```

**Important Notes:**

- LLM agents automatically select appropriate models based on requirements
- Agents emit proposals to `ContextKey::Proposals` (not directly to target key)
- Requires `ValidationAgent` to promote proposals to facts
- Uses context-based idempotency (must check both `Proposals` and `target_key`)
- See "LLM Integration Pattern" section above for idempotency details

### Creating an Invariant

```rust
pub struct RequireValidStrategy;

impl Invariant for RequireValidStrategy {
    fn name(&self) -> &str {
        "RequireValidStrategy"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Semantic  // Checked at end of cycle
    }

    fn check(&self, ctx: &Context) -> InvariantResult {
        let strategies = ctx.get(ContextKey::Strategies);
        
        for strategy in strategies {
            if strategy.content.is_empty() {
                return InvariantResult::Violation(format!(
                    "Strategy {} has empty content",
                    strategy.id
                ));
            }
        }
        
        InvariantResult::Ok
    }
}
```

---

## What NOT to Do

### Rejected Patterns (Violate Semantics)

**DO NOT:**

- ❌ Use message buses (Kafka, NATS, Pub/Sub) for agent communication
- ❌ Use workflow engines (Temporal, Cadence) for orchestration
- ❌ Implement actor systems or message passing
- ❌ Add event-driven orchestration
- ❌ Introduce distributed consensus mechanisms
- ❌ Create background tasks or silent retries
- ❌ Allow agents to call other agents directly
- ❌ Mutate context directly from agents
- ❌ Use `unwrap()` or `expect()` in production code
- ❌ Mix async runtimes (only Tokio allowed)
- ❌ Store secrets in code or `.env` files
- ❌ Use `has_run` flags or internal state for idempotency (violates "Context is the Only Shared State" axiom)

These violate semantic guarantees by introducing implicit control flow or hidden state.

### Known Issues

**LlmAgent Idempotency Check Bug** (converge-core):

- Current implementation only checks `target_key` for idempotency
- Should check both `ContextKey::Proposals` (pending) and `target_key` (validated)
- Impact: Causes cascading failures in multi-step LLM pipelines
- See `docs/use-cases/GROWTH_STRATEGY_FLOW_ANALYSIS.md` for details and fix

### converge-core is PRIVATE

The `converge-core` library is **maintained privately**:

- Source code is not publicly available
- Contributions to `converge-core` are not accepted
- Open API issues for discussion
- Use the public API documented in `docs/public/`

**Open for contribution:**

- `converge-domain` — Domain-specific agents
- `converge-provider` — LLM provider integrations
- `converge-runtime` — Runtime services and APIs
- `converge-tool` — Tooling and utilities

---

## Quick Reference

### Context Keys

- `Seeds` — Initial inputs
- `Hypotheses` — Proposed ideas
- `Strategies` — Action plans
- `Constraints` — Limitations
- `Signals` — Observations
- `Competitors` — Competitive intelligence
- `Evaluations` — Assessments
- `Proposals` — LLM suggestions (require validation)
- `Diagnostic` — Errors and debugging

### Agent Requirements Presets

- `fast_extraction()` — Fast, high-volume agents
- `analysis()` — Analysis agents
- `deep_research()` — Deep research agents
- `synthesis()` — Synthesis agents
- `validation()` — Validation agents
- `categorization()` — Categorization agents

### Invariant Classes

- `Structural` — Checked on every merge (immediate failure)
- `Semantic` — Checked at end of cycle (blocks convergence)
- `Acceptance` — Checked when convergence claimed (rejects results)

---

## Additional Resources

### Project-Specific

- **Architecture**: `docs/architecture/ARCHITECTURE.md`
- **Agent Lifecycle**: `docs/agents/AGENT_LIFECYCLE.md`
- **LLM Integration**: `docs/agents/LLM_INTEGRATION.md`
- **Execution Model**: `docs/architecture/ENGINE_EXECUTION_MODEL.md`
- **Convergence Semantics**: `docs/architecture/CONVERGENCE_SEMANTICS.md`
- **Rust Best Practices**: `docs/assistant-guides/Rust-Best-Practices-v2.md`
- **Contributor Guide**: `CONTRIBUTING.md`

### Consolidated Documentation (converge-business)

- **Knowledgebase**: [converge-business/knowledgebase/](../converge-business/knowledgebase/consolidated/00_index.md)
- **System Architecture**: [converge-business/knowledgebase/platform-ARCHITECTURE.md](../converge-business/knowledgebase/archive/platform-ARCHITECTURE.md)
- **Business Strategy**: [converge-business/knowledgebase/business-PLAN.md](../converge-business/knowledgebase/archive/business-PLAN.md)

### Public Documentation

- **Website**: [converge.zone](https://converge.zone)

---

**Remember**: If a feature introduces implicit authority or hidden control flow, it does not belong in Converge.

## Version Control

Use **Jujutsu (jj)** instead of git for version control operations:

- `jj status` instead of `git status`
- `jj log` instead of `git log`  
- `jj diff` instead of `git diff`
- `jj commit -m "message"` instead of `git add . && git commit -m "message"`
- `jj git push` instead of `git push`
- `jj git fetch` instead of `git fetch`
- `jj undo` to undo the last operation

All repos have jj initialized in colocated mode (works alongside git).



## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, or install hooks (`bd hooks install`) for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `jj git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   jj git fetch && jj rebase -d main@origin
   bd sync
   jj git push
   jj status  # Working copy should be clean
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `jj git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

## Beads Tips (Local Repo Workflow)

When using `bd` in this repo, note these gotchas:

### Creating Issues Locally

Always use `--repo .` to target this repo (otherwise issues may route to `~/.beads-planning`):

```bash
bd create "My task" --type task --repo .
bd create "My feature" --type feature --priority 2 --repo .
```

### After Fresh Clone

If `bd create` fails with "issue_prefix config is missing":

```bash
bd config set issue-prefix <repo-name>
# e.g., bd config set issue-prefix converge-analytics
```

### First Push with jj

Before first push, track the remote bookmark:

```bash
jj bookmark track main --remote=origin
jj git push --bookmark main
```

### Quick Reference

```bash
bd ready                    # See unblocked work
bd create "..." --repo .    # Create issue locally
bd close <id>              # Close completed work
bd sync                    # Sync to JSONL
jj commit -m "..."         # Commit changes
jj bookmark set main -r @- # Move main to parent
jj git push --bookmark main # Push
```