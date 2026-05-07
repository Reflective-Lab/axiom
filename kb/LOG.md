---
tags: [log]
source: llm
---

# Axiom — Mutation Log

## 2026-05-07

- Added `intent` module — `compile_intent(&TruthDocument) -> IntentPacket` plus `compile_intent_from_source` convenience. Bridge logic moved here from `organism-intent::bridge` (deleted) to invert the dependency arrow: `axiom-truth → organism-pack` instead of `organism → axiom-truth`. 17 tests inline. Axiom now depends on `organism-pack` 1.5.1.
- Updated Architecture/System Overview.md — added `intent` to module map, documented the Runtime Intent pipeline, recorded the new dependency direction.
- Added Architecture/API Surfaces.md — canonical public API reference, including the `.truths` source → `parse_truth_document` → `compile_intent` → `IntentPacket` flow.
- Created Architecture/Intent Compilation.md — full field mapping (Authority/Constraint/Exception → IntentPacket), error model, caller flow.
- Updated top-level `lib.rs` doc comment — added Compiling to runtime intent section with caller example.
- Aligned AGENTS.md, MILESTONES.md, and Justfile workflow recipes to the 0.8.1 surface: `converge-provider` + `converge-manifold-adapters` + `organism-pack`; `just focus`, `just sync`, and `just status` now exist.
- Bumped axiom-truth 0.7.0 → 0.8.1 (new public module + new dep is a minor bump).

## 2026-05-04

- Updated Concepts/Simulation.md — documented opt-in domain profiles so core simulation stays domain-neutral while downstream layers can enable richer checks.

## 2026-04-15

- Created kb/ structure: Home, INDEX, LOG, Philosophy/, Concepts/, Architecture/, Building/, Workflow/
- Created Philosophy/Why Axiom.md — problem statement and design rationale
- Created Philosophy/Truth-Driven Development.md — the methodology
- Created Concepts/Truth Documents.md — format specification
- Created Concepts/Validation Pipeline.md — three-check pipeline
- Created Concepts/Predicates.md — step parsing and predicate types
- Created Concepts/Simulation.md — pre-flight analysis
- Created Concepts/Code Generation.md — WASM compilation pipeline
- Created Concepts/Policy Lens.md — Cedar policy coverage
- Created Concepts/Guidance.md — heading quality feedback
- Created Concepts/JTBD.md — Jobs-to-be-Done integration
- Created Architecture/System Overview.md — module map and pipeline
- Created Architecture/Converge Contract.md — relationship to Converge
- Created Architecture/WASM Compilation.md — build pipeline details
- Created Building/Getting Started.md — setup and development
- Created Building/Writing Truths.md — authoring guide
- Created Building/CLI Reference.md — cz command reference
- Created Workflow/Daily Journey.md — development workflow

## 2026-04-17

- Created AGENTS.md — canonical agent entrypoint (philosophy, public surface, rules, architecture, workflows)
- Updated MILESTONES.md — axiom-specific milestones starting with v0.5 Foundation Hardening
- Refactored justfile — removed converge-specific tasks, focused on axiom development workflow
