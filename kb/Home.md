---
tags: [moc]
source: mixed
---

# Axiom — The Truth Layer

Axiom validates, simulates, and compiles business specifications into enforceable WASM invariants and runtime intent for [[Architecture/Converge Contract|Converge]] and Organism. It sits beneath Helm as the truth-definition surface and above Organism and Converge in the authoring and proof story. Where Converge enforces correctness at runtime, Axiom ensures specifications are correct before they reach the engine and can explain the path from truth to fixed point.

**Start here:**
- [[Philosophy/Why Axiom]] — what problem this solves
- [[Philosophy/Truth-Driven Development]] — the methodology
- [[Concepts/Truth Documents]] — the specification format

## Philosophy

- [[Philosophy/Why Axiom]] — specifications must be validated before they execute
- [[Philosophy/Truth-Driven Development]] — from business intent to WASM invariant

## Concepts

- [[Concepts/Truth Documents]] — `.truths` files, governance blocks, scenario tags
- [[Concepts/Validation Pipeline]] — business sense, compilability, conventions
- [[Concepts/Predicates]] — semantic step parsing and predicate extraction
- [[Concepts/Simulation]] — pre-flight analysis and convergence readiness
- [[Concepts/Code Generation]] — from predicates to WASM invariant modules
- [[Concepts/Policy Lens]] — Cedar policy coverage analysis
- [[Concepts/Guidance]] — LLM-powered heading and spec quality feedback
- [[Concepts/JTBD]] — Jobs-to-Be-Done as source input and legacy truth metadata
- [[Concepts/Truth Package]] — v0.10 manifest spine from JTBD to auditable runtime contract

## Marquee

- [[Marquee/Round-Driven Formation Design]] — v0.11 candidate JTBD and staged `AxiomRunReport` shape for dynamic Formations

## Architecture

- [[Architecture/System Overview]] — modules, pipeline, dependencies
- [[Architecture/API Surfaces]] — public Rust and crate boundaries
- [[Architecture/Axiom as Verifier]] — doctrine for JTBD-to-contract compilation, verifier reports, and layer boundaries
- [[Architecture/Clause IDs and Decoder Spine]] — v0.10 decision for deterministic JTBD clause identity and decoder rules
- [[Architecture/Intent Compilation]] — TruthDocument to IntentPacket mapping
- [[Architecture/Truth-to-Formation Run Proof]] — v0.9 path from truth to Organism Formation to Converge fixed point
- [[Architecture/Converge Contract]] — how Axiom relates to Converge
- [[Architecture/WASM Compilation]] — Rust → WASM build details and the Axiom/Helm/Converge responsibility boundary

## Building

- [[Building/Getting Started]] — setup, build, test
- [[Building/Writing Truths]] — authoring `.truths` files
- [[Building/CLI Reference]] — `cz` commands

## Workflow

- [[Workflow/Daily Journey]] — how to use Axiom in development

## Experiments

- [[Experiments/INDEX]] — hypothesis-driven development with evidence logging
- [[Experiments/LOG]] — mutation log of all experiments

## Meta

- [[INDEX]] — entity catalog
- [[LOG]] — mutation log
