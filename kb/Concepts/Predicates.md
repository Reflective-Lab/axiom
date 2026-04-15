---
tags: [concepts]
source: llm
---

# Predicates

The `predicate` module parses Gherkin steps into semantic predicates — structured representations that drive code generation.

## Predicate Types

| Predicate | Pattern | Example |
|---|---|---|
| `CountAtLeast` | "contains at least N facts" | "Then Seeds contains at least 3 facts" |
| `CountAtMost` | "contains at most N facts" | "Then Strategies contains at most 10 facts" |
| `ContentMustNotContain` | "must not contain forbidden term" | "Then it must not contain forbidden term 'TBD'" |
| `ContentMustContain` | "must include field X" | "Then it must include field 'risk_assessment'" |
| `CrossReference` | "for each X exists Y" | "Then for each Signal exists Hypothesis" |
| `HasFacts` | "any fact under key X" | "Then any fact under key Evaluations" |
| `RequiredFields` | "must include" (with table) | Step with data table of required fields |
| `Custom` | (unrecognized) | Falls through as freeform description |

## Known Context Keys

Predicates reference context keys from the Converge engine: Seeds, Hypotheses, Strategies, Constraints, Signals, Competitors, Evaluations.

## Role in Pipeline

Predicates are the bridge between human-readable Gherkin and machine-executable WASM. The [[Concepts/Code Generation|codegen module]] consumes predicates to produce Rust check expressions.
