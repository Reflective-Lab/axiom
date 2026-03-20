# converge-traits

The public trait contract for the Converge platform. Every agent implementation in the ecosystem compiles against this crate.

## Current State (2026-03-12)

- **Version**: 0.3.0 (local commit bb6d28b, not pushed)
- **Edition**: 2024
- **Status**: In review. Not frozen for 1.0.

## Two Layers

1. **Convergence Engine Contract**: Agent, Context, ContextKey, Fact, ProposedFact, AgentEffect, Invariant, InvariantClass, InvariantResult
2. **Backend Abstraction Layer**: Backend, BackendKind, Capability, BackendRequirements, BackendSelector, BackendError

## Dependencies

- serde (derive) + thiserror only. No async, no I/O.

## Path to 1.0 Freeze

1. converge-core must compile against these traits (core doesn't exist yet)
2. At least one Wave 2 crate validates Agent trait ergonomics
3. Kira Novak should review the trait surface
4. Resolve git push (HTTPS auth blocker)

## Location

`/Users/kpernyer/repo/all-converge/converge-traits/`
