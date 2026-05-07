---
tags: [architecture, intent, organism]
source: llm
---

# Intent Compilation

The `intent` module compiles a parsed [`TruthDocument`](../Concepts/Truth%20Documents.md)
into `organism_pack::IntentPacket` — the runtime contract organism consumes.
Truth is axiom's; mechanism is organism's. This module is the boundary.

## API

```rust
pub fn compile_intent(doc: &TruthDocument) -> Result<IntentPacket, CompileError>
pub fn compile_intent_from_source(src: &str) -> Result<IntentPacket, CompileFromSourceError>
```

`compile_intent_from_source` is a convenience that calls `parse_truth_document`
then `compile_intent`. Use it when you have raw `.truths` text and don't need
the intermediate `TruthDocument`.

## Field mapping

| Truth governance block | IntentPacket field | Notes |
|---|---|---|
| `Intent.outcome` (or `Intent.goal` fallback) | `outcome` | Required. Empty/whitespace fails with `MissingOutcome`. |
| `Authority.actor` | `authority[0]` as `"actor: <name>"` | Prefixed onto the authority list. |
| `Authority.may` | `authority[..]` | Appended after actor. |
| `Authority.must_not` | `forbidden[..]` with `reason: "authority"` | Deduplicated against `Constraint.must_not`. |
| `Authority.requires_approval` | `constraints[..]` as `"requires_approval: <action>"` | Folded into constraints. |
| `Authority.expires` | `expires` | RFC-3339 timestamp; `YYYY-MM-DD` accepted as midnight UTC. Missing → `now + 24h`. |
| `Constraint.budget` | `constraints[..]` as `"budget: <value>"` | |
| `Constraint.cost_limit` | `constraints[..]` as `"cost_limit: <value>"` | |
| `Constraint.must_not` | `forbidden[..]` with `reason: "constraint"` | Authority entries take precedence on duplicates. |
| `Exception.escalates_to` ⊕ `Exception.requires` (any) | `expiry_action: Escalate` | Otherwise `Halt`. |
| Synthetic `"reversibility: irreversible"` line in constraints | `reversibility: Irreversible` | `partial` also recognized. Default `Reversible`. |

The Gherkin body is **not** folded into the IntentPacket. It is the
validation/simulation surface, not the runtime contract. Callers that need the
body retain the full `TruthDocument` alongside the compiled packet.

## Errors

```rust
pub enum CompileError {
    MissingOutcome,
    ExpiryParse { value: String, message: String },
}

pub enum CompileFromSourceError {
    ParseFailed(gherkin::ValidationError),  // truth source malformed
    CompileFailed(CompileError),            // structurally fine, missing fields
}
```

## End-to-end caller flow

```rust
use axiom_truth::{compile_intent, parse_truth_document};
use organism_runtime::Runtime;

let truth   = parse_truth_document(source)?;
let intent  = compile_intent(&truth)?;
let receipt = Runtime::new().admit_intent(&intent, actor, src, &mut ctx)?;
```

Three steps, three crates: `axiom-truth` parses, `axiom-truth` compiles,
`organism-runtime` admits. Helms's `truth-catalog` hand-rolls IntentPackets
field-by-field today — when it bumps off `axiom-truth v0.6.0`, this is the
flow it should adopt.

## Why this lives in axiom

Truths belong in axiom. Organism's runtime is mechanism — it operates on
`IntentPacket`s and knows nothing about Truth representations. Putting the
bridge in axiom keeps the dependency arrow honest:

```
axiom-truth → organism-pack    (Truth produces; organism contract consumed)
```

Before 0.8.0 the arrow ran the other way: `organism-intent::bridge` parsed
axiom's `TruthDocument` directly, forcing organism to depend on axiom. The
inversion (Phase 2 of the May 2026 refactor) moved the bridge here.

## Tests

Inline in `src/intent.rs` — 17 tests covering each block, expiry formats,
reversibility overrides, exception escalation, deduplication, and a full
round-trip from `.truths` source.
