---
tags: [building]
---
# Getting Started

## Add Converge to Your Project

```toml
[dependencies]
converge-kernel = "3.0.1"
```

That's enough to embed the Converge engine in-process.

Need more?

```toml
converge-pack = "3.0.1"       # Author suggestors and invariants
converge-model = "3.0.1"      # Curated semantic types
converge-domain = "3.0.1"     # Pre-built domain packs
converge-client = "3.0.1"     # Remote Rust client
```

See [[Building/Crate Catalog]] for the full list.

## First Run

```rust
use converge_kernel::{
    AgentEffect, Context, ContextKey, ContextView, Engine, ProposedFact, Suggestor,
};

struct SeedSuggestor;

impl Suggestor for SeedSuggestor {
    fn name(&self) -> &str { "seed" }
    fn dependencies(&self) -> &[ContextKey] { &[] }
    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.has(ContextKey::Seeds)
    }
    fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        AgentEffect::with_proposal(ProposedFact {
            key: ContextKey::Seeds,
            id: "observation-1".into(),
            content: "Monthly active users grew 15%".into(),
            confidence: 0.95,
            provenance: "suggestor:seed".into(),
        })
    }
}

let mut engine = Engine::new();
engine.register_suggestor(SeedSuggestor);
let result = engine.run(Context::new())?;

assert!(result.converged);
assert!(result.context.has(ContextKey::Seeds));
```

## Build Commands

If you're working on the Converge repo itself:

| Command | What it does |
|---|---|
| `just build` | `cargo build --release` |
| `just build-quick` | `cargo build --profile quick-release` |
| `just lint` | `cargo fmt --check && cargo clippy --all-targets -- -D warnings` |
| `just fix-lint` | Auto-fix lint issues |
| `just test` | `cargo test --all-targets` (default members) |
| `just test-all` | `cargo test --all-targets --workspace` |
| `just doc` | `cargo doc --no-deps --workspace` |
| `just example hello-convergence` | Run an example |

## Next Steps

1. Read [[Philosophy/Why Converge]] and [[Philosophy/Nine Axioms]] — understand the model before building
2. Read [[Building/Writing Agents]] — implement the `Suggestor` trait
3. Study [[Concepts/Context and Facts]] — understand the shared state model
4. Explore [[Concepts/Domain Packs]] — use pre-built suggestors for cross-cutting concerns

See also: [[Building/Crate Catalog]], [[Building/Context Keys]]
