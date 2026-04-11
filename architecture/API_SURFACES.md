# Converge API Surfaces

This document defines the supported external contracts for Converge.

If a type, trait, or module is not reachable through one of the surfaces below,
it is an implementation detail and must not be treated as a stable dependency.

If this document conflicts with older comments in the codebase, this document wins.

## Public Contracts

Converge exposes six external contracts:

1. Rust pack authoring
2. Rust provider capability routing
3. Rust semantic model
4. Rust in-process kernel embedding
5. Rust remote client
6. Remote wire protocol

These contracts are intentionally separate. A pack author should not need provider
selection APIs. A provider adapter should not need kernel internals. A remote
Rust consumer should not need CLI implementation code.

## Supported Rust Crates

### `converge-pack`

Purpose:
- author pure Converge packs, suggestors, and invariants

What external code may use:
- `Suggestor`
- `Context`
- `ContextKey`
- `AgentEffect`
- `Fact`
- `ProposedFact`
- `Invariant`
- `InvariantClass`
- `InvariantResult`

Status:
- canonical pack authoring crate
- authoritative fact creation is kernel-gated
- pack authoring is proposal-only

Important note:
- `Suggestor` is the canonical authoring trait, and it cannot emit authoritative facts.
- `Fact` is read-only outside kernel-authority code paths.
- `AgentEffect` is proposal-only.

### `converge-provider-api`

Purpose:
- describe provider identity, capabilities, and routing requirements

What external code may use:
- `Backend`
- `BackendKind`
- `Capability`
- `BackendError`
- `BackendErrorKind`
- `BackendRequirements`
- `BackendSelector`
- `ComplianceLevel`
- `CostClass`
- `DataSovereignty`

Status:
- canonical provider capability contract

### `converge-model`

Purpose:
- expose the curated semantic model shared across kernel consumers

What external code may use:
- governed semantic types such as `Proposal`, `TypesFact`, `PromotionRecord`,
  `TypesRootIntent`, `Criterion`, `StopReason`, and related IDs/newtypes

Status:
- stable curated semantic surface
- authority-bearing fact construction is not part of this contract

### `converge-kernel`

Purpose:
- embed the Converge kernel in-process

What external code may use:
- `Engine`
- `ConvergeResult`
- `ConvergeError`
- `RunResult`
- `HitlPause`
- `EngineHitlPolicy`
- `CriterionEvaluator`
- `ExperienceEventObserver`
- `TypesRunHooks`
- pack-facing traits re-exported for convenience

Status:
- canonical embedding surface
- current engine semantics enforce the single truth pipeline

### `converge-protocol`

Purpose:
- expose the generated Rust representation of the `converge.v1` wire contract

What external code may use:
- generated `v1::*` request, response, event, and service types
- `ConvergeService` client/server stubs generated from the protobuf contract
- re-exported `prost_types` for structured payloads

Status:
- canonical Rust representation of the remote wire contract
- semver must track the public `converge.v1` protocol

### `converge-client`

Purpose:
- connect to remote Converge runtimes from Rust without depending on CLI code

What external code may use:
- `ConvergeClient`
- `ClientError`
- `messages`
- `protocol` and `v1` re-exports for typed requests and events

Status:
- canonical remote Rust SDK
- intentionally thin over `converge-protocol`

## Deprecated Compatibility Surface

### `converge-traits`

Status:
- deprecated compatibility facade
- `publish = false`

Rule:
- no new code may depend on `converge-traits`
- controlled downstreams must migrate to `converge-pack` and
  `converge-provider-api`

## Remote Protocol

The current public network contract is the protobuf package in:

- [schema/proto/converge.proto](/Users/kpernyer/dev/work/converge/schema/proto/converge.proto)

This is the external client/server protocol for mobile, CLI, and remote systems.
The canonical Rust packaging of that contract is `converge-protocol`.

Status:
- public wire contract
- versioned by protobuf package (`converge.v1`)

The following is not the general external protocol:

- [schema/proto/kernel.proto](/Users/kpernyer/dev/work/converge/schema/proto/kernel.proto)

`kernel.proto` is an internal service boundary for the GPU/kernel service and must
not be treated as the general external Converge API.

## Internal Crates

These crates are implementation and reference code, not stable external contracts:

- `converge-core`
- `converge-provider`
- `converge-domain`
- `converge-runtime`
- `converge-application`
- `converge-storage`
- `converge-llm`
- `converge-optimization`
- `converge-analytics`
- `converge-mcp`
- `converge-knowledge`
- `converge-experience`
- `converge-tool`
- `converge-remote`

External code may use them experimentally inside controlled repos, but they are not
the promised public surface and may change without preserving compatibility.

## Downstream Rules

### Rust pack/module authors

Allowed dependencies:
- `converge-pack`
- `converge-model`
- `converge-provider-api` only if the pack truly needs to express provider routing

Not allowed:
- `converge-core`
- `converge-runtime`
- `converge-storage`
- direct dependency on implementation crates for authoring contracts

### Embedded applications

Allowed dependencies:
- `converge-kernel`
- `converge-model`
- `converge-pack`

### Provider adapters

Allowed dependencies:
- `converge-provider-api`

### Remote systems

Allowed contract:
- `converge-client` for the stable Rust SDK
- `converge-protocol` when raw gRPC/protobuf types are required
- protobuf/OpenAPI surface derived from `converge.v1` for non-Rust consumers

Not allowed:
- runtime internals
- kernel service internals

## Controlled Repo Mapping

These are the intended targets for the currently known downstreams:

- `organism` -> `converge-pack` + `converge-model`
- `saas-killer` -> `converge-kernel` + `converge-model`
- `wolfgang` -> `converge-provider-api` and `converge-client` across the runtime boundary

## Semver Rules

Semver promises apply only to:

- `converge-pack`
- `converge-provider-api`
- `converge-model`
- `converge-kernel`
- `converge-protocol`
- `converge-client`
- `converge.v1` wire protocol

The following are explicit non-promises:

- undocumented items in internal crates
- broad re-exports from `converge-core`
- deprecated `converge-traits`
- runtime implementation details

## Regression Gate

Any change touching pack/core/protocol/client/runtime semantics must preserve:

- compile-time authority enforcement
- the observation -> proposal -> promotion -> fact pipeline
- deterministic convergence and bounded termination
- observation-first remote vocabulary

Required commands:

- `cargo test -p converge-pack --test compile_fail`
- `cargo test -p converge-core --test compile_fail --test truth_pipeline --test negative --test properties`
- `cargo test -p converge-client --test messages`

Note:
- the pack compile-fail suite must run in isolation because Cargo feature
  unification enables `kernel-authority` when `converge-core` participates in
  the same build graph
