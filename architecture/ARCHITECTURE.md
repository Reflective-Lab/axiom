# Converge — Architecture

Converge is a **pure Rust Agent OS** for building correctness-first,
context-driven, multi-agent systems that provably converge.

## Core Axioms

1. Context is the only shared state
2. Agents never call each other
3. Execution is graph-based, not linear
4. Context evolution is monotonic
5. Convergence is explicit and observable
6. Correctness is verified semantically
7. LLMs may suggest, never decide

## System View

```
┌──────────────────────────────────────────┐
│ Specification Layer (Gherkin)            │
│ Behavioral invariants & success criteria │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Orchestration Layer                      │
│ Execution graph & convergence engine     │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Agent Layer                              │
│ Deterministic · LLM · Solver · IO        │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Context Layer                            │
│ Typed, shared, monotonic job state       │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Tool Layer                               │
│ LLMs · Search · Solvers · APIs · MCP     │
└──────────────────────────────────────────┘
```

## Workspace Layout

```
crates/
  traits/         Public trait contract (no deps)
  core/           Engine, context, types (no internal deps)
  provider/       LLM backends (Anthropic, OpenAI, Gemini, Ollama, …)
  domain/         Domain agents and packs
  experience/     Event-sourced audit ledger
  knowledge/      Vector search knowledge base
  mcp/            Model Context Protocol (client + server)
  optimization/   Constraint solving (optional OR-Tools FFI)
  analytics/      Polars + Burn analytics (publish = false)
  llm/            Local inference kernel (publish = false)
  policy/         Cedar policy engine (publish = false)
  runtime/        HTTP/gRPC servers (publish = false)
  remote/         gRPC client library
  tool/           Dev tools, Gherkin validator (`cz` binary)
  application/    Single `converge` binary (publish = false)

schema/
  proto/          Protocol Buffer definitions
  openapi/        OpenAPI specifications

examples/         Standalone example crates
architecture/     This directory
```

## Dependency Graph (leaf → root)

```
converge-traits          (no deps)
converge-core            (no internal deps)
converge-mcp             (no internal deps)
converge-provider        → core, traits
converge-domain          → core, provider
converge-experience      → core
converge-knowledge       → mcp (server feature)
ortools-sys              (no deps, FFI)
converge-optimization    → ortools-sys (optional)
converge-analytics       → core, domain, provider
converge-llm             → core, domain, provider
converge-policy          → core
converge-tool            → core, provider
converge-remote          (no internal deps, gRPC client)
converge-runtime         → core, provider, tool
converge-application     → core, provider, domain, tool, mcp, knowledge, …
```

## Execution Model

### Basic convergence loop
```
initialize context from RootIntent

repeat
  determine eligible agents     (pure, side-effect free)
  filter by active packs        (only agents in activated packs run)
  execute eligible agents       (parallel, read-only context via ContextView)
  collect AgentEffects          (buffered facts + proposals)
  promote proposals → facts     (promotion gate validates confidence, provenance)
  merge effects into context    (serialized, deterministic)
  evaluate criteria             (CriterionEvaluator checks success conditions)
until convergence or termination
```

### Application-level truth execution
```
Application builds TypesRootIntent from TruthDefinition
  → intent carries: active_packs, success_criteria, budget, constraints

Application creates Engine, registers agents in packs
  → engine.register_in_pack("compliance-pack", screener_agent)

Application calls run_with_types_intent_and_hooks()
  → engine runs convergence loop
  → CriterionEvaluator checks each criterion after convergence
  → ExperienceEventObserver captures events during the run

Engine returns ConvergeResult
  → context: final state with all facts
  → criteria_outcomes: per-criterion Met/Unmet/Blocked/Indeterminate
  → stop_reason: Converged | CriteriaMet | HumanInterventionRequired | BudgetExhausted

Application projects ConvergeResult into domain state
  → reads facts from context, writes to its own storage
```

**Convergence**: `Contextₙ₊₁ == Contextₙ` — no new facts, no new intents, no state change.

**Termination**: convergence reached, criteria met, budgets exhausted, invariants violated, or human intervention required.

**Guarantees**: determinism, termination (budgets), isolation (agents can't affect each other), auditability (full provenance on every fact and proposal).

## Feature Gates

The workspace uses Cargo features so consumers only pull what they need.

**Individual crates** (library consumers):
```toml
converge-core = "1.1"                                        # minimal
converge-provider = { version = "1.1", features = ["anthropic"] }  # + LLM
converge-knowledge = { version = "1.1", features = ["mcp"] }      # + knowledge + MCP
```

**Umbrella binary** (`converge-application`):
| Feature        | What it enables                        | Heavy deps        |
|----------------|----------------------------------------|-------------------|
| `tui`          | Interactive terminal UI (default)      | ratatui, crossterm|
| `knowledge`    | Knowledge base commands                | —                 |
| `mcp`          | MCP server (`converge mcp serve`)      | axum, tower       |
| `llm`          | Local inference kernel                 | burn, llama-burn  |
| `analytics`    | Analytics pipeline                     | polars, burn      |
| `optimization` | Constraint solving                     | ortools-sys (FFI) |
| `full`         | Everything                             | all of the above  |

## API Surface

### Engine
```rust
let mut engine = Engine::new();
engine.register(agent);                          // global agent
engine.register_in_pack("pack-id", agent);       // pack-scoped agent
engine.run(context);                             // basic convergence
engine.run_with_types_intent_and_hooks(          // application-level truth execution
    context, &intent, TypesRunHooks {
        criterion_evaluator: Some(evaluator),
        event_observer: Some(observer),
    },
);
```

### Context
```rust
let context = Context::new();
context.has(ContextKey::Seeds);
context.get(ContextKey::Seeds);          // → iterator of &Fact
context.get(ContextKey::Evaluations);
context.get(ContextKey::Diagnostic);
context.add_fact(fact);
```

### Agent trait
```rust
trait Agent {
    fn name(&self) -> &str;
    fn dependencies(&self) -> &[ContextKey];
    fn accepts(&self, ctx: &dyn ContextView) -> bool;
    fn execute(&self, ctx: &dyn ContextView) -> AgentEffect;
}
```

Agents never call other agents. Agents never mutate context directly. Agents only emit effects.

### AgentEffect
```rust
// An agent can emit facts AND proposals in a single execution
AgentEffect {
    facts: vec![...],       // validated, authoritative
    proposals: vec![...],   // need promotion gate validation
}
AgentEffect::with_proposal(proposed_fact)  // convenience
AgentEffect::with_fact(fact)               // convenience
AgentEffect::empty()                       // nothing to contribute
```

### ProposedFact
```rust
ProposedFact {
    key: ContextKey::Evaluations,
    id: "compliance:screen:acme".into(),
    content: payload_json,
    confidence: 0.85,           // how confident the agent is (0.0–1.0)
    provenance: "agent:screener".into(),  // who proposed it
}
```

### Truth Execution
```rust
// Application declares truths
trait TruthCatalog {
    fn list_truths(&self) -> Vec<TruthDefinition>;
    fn find_truth(&self, key: &str) -> Option<TruthDefinition>;
}

// Application evaluates criteria
trait CriterionEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult;
}

// Four-way typed result
enum CriterionResult {
    Met { evidence: Vec<FactId> },
    Blocked { reason: String, approval_ref: Option<String> },
    Unmet { reason: String },
    Indeterminate,
}

// Durable context across runs
trait ContextStore {
    fn load_context(&self, scope_id: &str) -> impl Future<Output = Result<Option<Context>>>;
    fn save_context(&self, scope_id: &str, context: &Context) -> impl Future<Output = Result<()>>;
}
```

### ConvergeResult
```rust
ConvergeResult {
    context: Context,              // final state
    cycles: u32,                   // how many cycles ran
    converged: bool,               // did it reach a fixed point?
    stop_reason: StopReason,       // why it stopped
    criteria_outcomes: Vec<CriterionOutcome>,  // per-criterion results
}

enum StopReason {
    Converged,
    CriteriaMet { .. },
    HumanInterventionRequired { criteria, approval_refs },
    CycleBudgetExhausted { .. },
    FactBudgetExhausted { .. },
    // ...
}
```

## Schema

Protocol definitions live in `schema/`:
- `schema/proto/converge.proto` — Main gRPC API (bidirectional streaming for mobile/CLI)
- `schema/proto/knowledge.proto` — Knowledge base service
- `schema/proto/kernel.proto` — LLM reasoning kernel (GPU-isolated)

## One-sentence takeaway

> Converge runs agents in parallel, but commits knowledge serially to guarantee convergence.
