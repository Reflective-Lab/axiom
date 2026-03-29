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

```
initialize context from RootIntent

repeat
  determine eligible agents   (pure, side-effect free)
  execute eligible agents     (parallel, read-only context)
  collect effects             (buffered, not applied)
  merge effects into context  (serialized, deterministic)
  apply pruning rules
until convergence or termination
```

**Convergence**: `Contextₙ₊₁ == Contextₙ` — no new facts, no new intents, no state change.

**Termination**: convergence reached, budgets exhausted, or invariants violated.

**Guarantees**: determinism, termination (budgets), isolation (agents can't affect each other), auditability (full provenance).

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
- `Engine::new()` → create engine
- `engine.register(agent)` → register an agent
- `engine.run(context)` → execute until convergence

### Context
- `Context::new()` → empty context
- `context.has(key)` → check key existence
- `context.get(key)` → retrieve facts

### Agent trait
```rust
trait Agent {
    fn accepts(&self, ctx: &Context) -> bool;
    fn dependencies(&self) -> Vec<ContextKey>;
    fn execute(&self, ctx: &Context) -> AgentEffect;
}
```

Agents never call other agents. Agents never mutate context directly. Agents only emit effects.

## Schema

Protocol definitions live in `schema/`:
- `schema/proto/converge.proto` — Main gRPC API (bidirectional streaming for mobile/CLI)
- `schema/proto/knowledge.proto` — Knowledge base service
- `schema/proto/kernel.proto` — LLM reasoning kernel (GPU-isolated)

## One-sentence takeaway

> Converge runs agents in parallel, but commits knowledge serially to guarantee convergence.
