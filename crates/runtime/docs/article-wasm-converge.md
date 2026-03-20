# WASM and the Converge Agent Framework: A Match Made in Sandbox Heaven

*How end-users define business invariants in plain Gherkin and run them as sandboxed WASM modules inside a multi-agent convergence engine.*

---

## The Problem: Extensibility Without Anarchy

Multi-agent AI systems face a fundamental tension. The framework author designs the engine, the convergence loop, the safety invariants. But the *end-user* knows their business. They know that strategies must never mention "spam," that invoices require dual approval above €50k, that escalation paths must exist before any access-control rollout ships.

The traditional answer is configuration files. YAML. JSON schemas. Maybe a plugin API with a fat SDK. Each approach trades off the same axes: expressiveness versus safety, power versus isolation.

What if end-users could describe their business rules in natural language — actual Gherkin specs — and have those rules compiled to native-speed sandboxed code that runs inside the agent collaboration loop? Not as interpreted scripts. Not as prompt injections. As compiled, metered, capability-gated, deterministic WASM modules.

That's what we built in Converge.

## What Converge Actually Does

Converge is a multi-agent convergence engine. Agents don't chat. They *converge*. Each agent reads from a shared, append-only Context, proposes new facts, and the engine merges, validates, and iterates until invariants hold and no agent has anything left to say.

The architecture enforces nine axioms. Three matter most for understanding why WASM fits:

1. **Agents Suggest, Engine Decides.** Agents emit proposals. The engine validates and promotes them to facts. No agent can mutate the shared context directly.

2. **Safety by Construction.** Invariants aren't advisory. They're structural. A violated structural invariant halts the merge. A violated acceptance invariant rejects the entire convergence result.

3. **Transparent Determinism.** Every decision is traceable. Every promotion is recorded. Every agent execution produces a trace link with provenance.

The core trait that makes this work is minimal:

```rust
pub trait Invariant: Send + Sync {
    fn name(&self) -> &str;
    fn class(&self) -> InvariantClass; // Structural, Semantic, or Acceptance
    fn check(&self, ctx: &Context) -> InvariantResult;
}
```

Three methods. No async. No side effects. Pure functions from context to verdict. This purity is what makes the WASM story possible.

## JTBD in Gherkin: Business Rules as Specifications

Converge uses `.truth` files — a branded Gherkin dialect where `Feature:` becomes `Truth:`:

```gherkin
Truth: Growth Strategy Pack

  @invariant @structural @id:brand_safety
  Scenario: Strategies must not contain brand-unsafe terms
    Given any fact under key "Strategies"
    Then it must not contain any forbidden term:
      | term         | reason              |
      | spam         | illegal marketing   |
      | bot army     | fake engagement     |
      | guaranteed   | overpromising       |

  @invariant @acceptance @id:require_multiple_strategies
  Scenario: At least 2 strategies must exist at convergence
    Given the engine halts with reason "Converged"
    Then the Context key "Strategies" contains at least 2 facts
```

Today, these specs are hand-compiled to Rust structs that implement `Invariant`. The framework ships with invariants like `BrandSafetyInvariant`, `RequireMultipleStrategies`, `RequireEvaluationRationale`. They live in `converge-domain`, they're tested, they work.

But they're static. If a customer needs "every proposal must reference a compliance framework," that's a code change, a release cycle, a deployment. The Gherkin exists as documentation. The actual enforcement is hand-coded Rust.

WASM closes this gap.

## Why WASM Is the Right Sandbox

We evaluated every reasonable sandboxing technology. Lua. V8 isolates. eBPF. Custom bytecode interpreters. WASM won on every axis that matters for a safety-critical agent framework.

### Determinism

A WASM module with the same bytes, receiving the same input bytes, produces the same output bytes. Always. There is no `Math.random()`, no file I/O, no network access, no ambient authority. The module sees exactly what the host provides through explicit imports.

This aligns perfectly with Converge's **Transparent Determinism** axiom. If you record the module hash and the input hash, you can replay the execution and get the identical result. This is not true of any JavaScript sandbox, any Lua environment, or any container-based isolation.

### Fuel Metering: Guaranteed Termination

Wasmtime's fuel mechanism counts instructions. You set a fuel budget before calling the guest. When fuel runs out, the module traps — immediately, deterministically, without the host needing a separate watchdog timer.

```rust
store.set_fuel(1_000_000)?; // ~1M instructions
let result = check_invariant.call(&mut store, (ctx_ptr, ctx_len))?;
let consumed = initial_fuel - store.get_fuel()?;
```

This maps directly to Converge's existing budget model. The engine already has `CycleBudget`, `FactBudget`, `TokenBudget`, and `ExecutionBudget` — all with `tick()` → `Option<StopReason>` semantics. WASM fuel is just another budget dimension. Same pattern, same guarantees.

No tenant-supplied code can infinite-loop your convergence engine. The fuel number is the proof.

### Capability-Based Security

WASM modules have *zero* capabilities by default. They can compute. That's it. Every interaction with the outside world goes through explicitly linked host functions.

In Converge, we define three capabilities:

| Capability | Host Function | What It Does |
|---|---|---|
| `ReadContext` | `host_read_context(key)` | Read facts from a specific context category |
| `Log` | `host_log(level, ptr, len)` | Write structured log entries |
| `Clock` | `host_now_millis()` | Read monotonic elapsed time |

The module declares which capabilities it needs in its manifest. The host checks the declaration against the tenant's policy. If the policy doesn't grant `Clock`, the module can't read the time. If it doesn't grant `ReadContext`, it can't see the context at all.

This is the **Explicit Authority** axiom made mechanical. No implicit capabilities. No ambient access. Every permission is declared, audited, and logged.

### Near-Native Speed

Wasmtime uses Cranelift, a production-grade compiler backend. WASM invariants run at near-native speed — we're talking microseconds for a typical `check_invariant` call, not milliseconds. For structural invariants that run after every merge in a convergence loop that may iterate dozens of times, this matters.

### Industry Validation at Scale

We're not the first to bet on WASM for extensibility in production systems:

**Shopify Functions** runs over 100,000 WASM module invocations per minute in production. Merchants write custom discount logic, shipping rules, and payment validations that execute inside Shopify's checkout pipeline. Same pattern: user-defined business logic, sandboxed execution, production-critical path.

**Microsoft's Wassette** project (2025) combines WASM sandboxing with the Model Context Protocol for AI agent tool execution. Their thesis is identical to ours: AI agents need a way to run user-supplied logic safely.

**NVIDIA** has publicly endorsed WASM as the sandboxing layer for agentic AI, specifically for the scenario where agents need to execute user-defined tools without compromising the host system.

**Fermyon, Cosmonic, and the WASI ecosystem** are building the standards (WASI Preview 2, the Component Model) that make WASM a first-class runtime target beyond the browser.

The pattern is converging (pun intended): wherever you need user-extensibility in a safety-critical path, WASM is becoming the answer.

## The Converge WASM Contract

The contract between Converge and guest WASM modules is constitutional — it enforces every axiom at the ABI level.

### Module Manifest: Self-Declaration

Every module exports a `converge_manifest()` function that returns a JSON self-declaration:

```json
{
  "name": "escalation-jtbd",
  "version": "1.2.0",
  "kind": "Invariant",
  "invariant_class": "Acceptance",
  "capabilities": ["ReadContext", "Log"],
  "requires_human_approval": false,
  "jtbd": {
    "truth_id": "escalation.truth",
    "actor": "Ops Manager",
    "job_functional": "Escalate delayed rollout"
  }
}
```

The manifest links back to the source Gherkin via `jtbd` — maintaining traceability from business specification to running code.

### Content-Addressed Identity

Modules are identified by their SHA-256 content hash. Same bytes = same identity. This enables:

- **Deduplication** across tenants (if two customers compile the same Gherkin, it's one module)
- **Deterministic replay** (same hash + same input = same output, always)
- **Cache-friendly storage** (compile once, instantiate many)

### Lifecycle: Type-State Progression

Modules progress through a strict lifecycle:

```
Uploaded → Compiled → Validated → Active → Retired
                                    ↘ Rejected
```

A module cannot be called until it reaches `Active`. Reaching `Active` requires passing ABI version check, manifest parsing, capability audit, and quota verification. Each transition is audited. The **Safety by Construction** axiom becomes a state machine.

### The Guest Sees a Projection, Not the Truth

The guest receives a `GuestContext` — a simplified, read-only JSON projection of the engine's `Context`. No provenance metadata. No internal IDs. No access to the `Proposals` or `Diagnostic` context keys (those are host-internal). The guest sees facts. It checks them. It returns a verdict.

```rust
// What the guest receives
pub struct GuestContext {
    pub facts: HashMap<String, Vec<GuestFact>>,
    pub version: u64,
    pub cycle: u32,
}

// What the guest returns
pub struct GuestInvariantResult {
    pub ok: bool,
    pub reason: Option<String>,
    pub fact_ids: Vec<String>,
}
```

The **Agents Suggest, Engine Decides** axiom means the guest never mutates context. It reads and responds. That's the entire interface.

### Full Observability

Every invocation produces an `ExecutionTrace`:

```rust
pub struct ExecutionTrace {
    pub module_id: ModuleId,       // content hash
    pub fuel_consumed: u64,         // instruction count
    pub peak_memory_bytes: u64,     // memory high-water mark
    pub duration_us: u64,           // wall-clock time
    pub host_calls: Vec<HostCallRecord>, // every host function call
    pub result_bytes: u64,          // response size
    pub outcome: InvocationOutcome, // Ok | Trapped | QuotaExceeded | ...
}
```

The **System Tells Truth About Itself** axiom is satisfied structurally. You cannot run a WASM module without producing a trace. The trace records what happened, not what was intended.

## The Pipeline: From Gherkin to Running WASM

The full pipeline that we're building looks like this:

```
                 converge-tool              converge-runtime
┌──────────┐    ┌──────────────┐    ┌────────────────────────────┐
│ .truth   │───►│ Tag Parser   │───►│ WASM Manifest Builder      │
│ file     │    │ ScenarioMeta │    │ (tags + JTBD → manifest)   │
└──────────┘    └──────┬───────┘    └────────────┬───────────────┘
                       │                          │
                ┌──────▼───────┐           ┌──────▼───────┐
                │ Predicate    │           │ Rust Codegen │
                │ Parser       │──────────►│ (Gherkin →   │
                │ (Given/Then) │           │  impl check) │
                └──────────────┘           └──────┬───────┘
                                                  │
                                           ┌──────▼───────┐
                                           │ WASM Compile │
                                           │ (rustc →     │
                                           │  .wasm)      │
                                           └──────┬───────┘
                                                  │
                                           ┌──────▼───────┐
                                           │ WasmEngine   │
                                           │ (wasmtime    │
                                           │  sandbox)    │
                                           └──────────────┘
```

**Step 1: Tag Extraction.** The Gherkin parser reads `.truth` files and extracts structured tags. `@invariant @structural @id:brand_safety` becomes `ScenarioMeta { kind: Invariant, class: Structural, id: "brand_safety" }`. This is the bridge between the human-readable spec and the compilation pipeline.

**Step 2: Predicate Parsing.** `Given any fact under key "Strategies"` and `Then it must not contain any forbidden term` are parsed into semantic predicates — structured representations of what to check and how.

**Step 3: Code Generation.** Predicates are compiled to Rust source code implementing the `check_invariant` guest export. The generated code is minimal: deserialize context, apply predicate logic, serialize result.

**Step 4: WASM Compilation.** The generated Rust is compiled to `.wasm` via `rustc --target wasm32-unknown-unknown`. The resulting binary is content-hashed, manifested, and stored.

**Step 5: Sandboxed Execution.** The `WasmEngine` compiles the module with Cranelift, links host functions, sets fuel quotas, and instantiates. The module runs inside the convergence loop alongside native invariants — `Box<dyn Invariant>` doesn't care whether the implementation is Rust or WASM.

## What This Means

A Converge customer can write:

```gherkin
@invariant @acceptance @id:compliance_framework_ref
Scenario: Every proposal must reference a compliance framework
  Given any fact under key "Strategies"
  Then it must contain a field "compliance_ref" with a non-empty value
```

And without any framework release, without any code review from the Converge team, without any deployment pipeline change, that invariant is compiled, sandboxed, and running in their convergence engine within minutes. The fuel budget guarantees it terminates. The capability model guarantees it can't exfiltrate data. The content-addressed identity guarantees reproducibility. The execution trace guarantees auditability.

The framework author doesn't need to anticipate every business rule. The end-user doesn't need to learn Rust, or understand the engine internals, or trust that a YAML config file will actually enforce anything. They write what they mean in Gherkin — the same language they already use for acceptance criteria — and the system makes it real.

WASM makes this possible because it's the only technology that simultaneously delivers:
- **Deterministic execution** (same input → same output, always)
- **Guaranteed termination** (fuel metering, not timeouts)
- **Zero ambient authority** (capability-based, not permission-based)
- **Near-native performance** (Cranelift compilation, not interpretation)
- **Language agnosticism** (Rust today, any language tomorrow)

For a framework built on the principle that agents suggest and engines decide, that truth is append-only, and that every action must be transparent — WASM isn't just a good fit. It's the only sandbox that enforces those axioms *by construction*, not by convention.

---

*Kenneth Pernyer is the creator of Converge, a multi-agent convergence engine for structured business reasoning. The WASM runtime contract and engine described in this article are part of Converge v0.2, available at [converge.zone](https://converge.zone).*
