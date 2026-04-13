---
source: mixed
---
# ADR-002: Truth Pipeline

## Status

Accepted

## Context

Converge currently contains two competing models:

1. a legacy model where pack code can construct `Fact` values and emit them
   directly through `AgentEffect.facts`
2. a governed model where proposals are validated and promoted before becoming
   authoritative facts

Running both models in parallel breaks the axioms:

- agents suggest, engine decides
- explicit authority
- safety by construction

It also makes the type system unable to prevent authority bypass.

## Decision

There is exactly one legal truth pipeline in Converge:

`Observation -> DraftProposal -> ValidatedProposal -> Fact`

### Definitions

- `Observation`: raw provider or human/system input, not yet a decision
- `DraftProposal`: a suggested claim, plan, evaluation, or action candidate
- `ValidatedProposal`: a proposal that has passed validation and carries proof
- `Fact`: authoritative, append-only, governed truth with complete promotion record

### Hard rules

1. external pack code must not construct authoritative `Fact` values
2. there is no implicit conversion from proposal to fact
3. validation and promotion are distinct operations
4. only the kernel promotion path may create authoritative facts
5. proposals, facts, and observations must not be encoded into each other to
   smuggle state across type boundaries

## Consequences

### Required breaking changes

- remove `AgentEffect.facts`
- remove `AgentEffect::with_fact`
- remove `AgentEffect::with_facts`
- remove external/public authoritative `Fact::new`
- remove direct `TryFrom<ProposedFact> for Fact` style bypasses
- stop treating proposal queues as facts under `ContextKey::Proposals`

### Required structural changes

- split context storage by semantic tier instead of string conventions
- make validation proof objects unforgeable outside the kernel
- make promotion records mandatory on authoritative facts

### Result

After this change, doing the wrong thing becomes a compile-time error rather than
an internal convention.
