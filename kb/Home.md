---
tags: [moc]
source: mixed
---

# Axiom — The Truth Layer

Axiom validates, simulates, and compiles business specifications into enforceable WASM invariants for [[Architecture/Converge Contract|Converge]]. It sits beneath Helm as the truth-definition surface and above Organism and Converge in the authoring and validation story. Where Converge enforces correctness at runtime, Axiom ensures specifications are correct *before* they reach the engine.

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
- [[Concepts/JTBD]] — Jobs-to-be-Done metadata in truth files

## Architecture

- [[Architecture/System Overview]] — modules, pipeline, dependencies
- [[Architecture/Converge Contract]] — how Axiom relates to Converge
- [[Architecture/WASM Compilation]] — the Rust → WASM build pipeline

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
