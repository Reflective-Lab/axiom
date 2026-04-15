---
tags: [concepts]
source: llm
---

# Validation Pipeline

Axiom validates `.truths` files through three independent checks, each catching a different class of error.

## The Three Checks

### 1. Business Sense (LLM)

Does the spec describe a meaningful business invariant? An LLM evaluates whether the truth file expresses something worth enforcing — not a process description, not a tautology, not an implementation detail.

### 2. Compilability (LLM)

Can this spec be translated to a Rust WASM invariant? The LLM checks whether the scenarios contain checkable predicates that can be compiled to code.

### 3. Conventions (Local)

Does the file follow Converge patterns? Deterministic checks for:
- Feature/Truth heading is present and descriptive
- Scenarios exist with steps
- Intent and Authority blocks present when governance tags are used
- Tags are from the known set

## Result

Each check produces `ValidationIssue` items with:
- `IssueCategory` — BusinessSense, Compilability, Convention, Syntax
- `Severity` — Info, Warning, Error
- Confidence score (0.0–1.0)

The combined result is a `SpecValidation` with an overall `is_valid` flag.

## Offline Mode

Both LLM checks can be skipped via CLI flags (`--skip-business-sense`, `--skip-compilability`, `--conventions-only`), leaving only the deterministic convention check. The [[Concepts/Guidance|guidance module]] also falls back to local heuristics when no LLM is available.
