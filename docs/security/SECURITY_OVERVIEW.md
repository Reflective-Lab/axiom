# Security Overview

## Objective

Converge is a correctness-first multi-agent runtime. The security objective of
this repository is to provide a strong engineering baseline for controlled
enterprise deployments, while being explicit about which controls are built
into the codebase and which controls must be supplied by operators.

## Current Security Posture

Repository and code-level controls currently present or documented include:

- Rust workspace linting with `unsafe_code = "forbid"`
- CI enforcement for build, test, format, and clippy checks
- dependency review with `cargo-deny`
- policy-oriented design in `converge-policy`
- audit-oriented/event-sourced components in `converge-experience`
- authentication, identity, audit, crypto, and interceptor modules in the runtime
- optional secret zeroization support in `converge-provider`

## Control Areas

### Secure development

- pull requests are expected for changes to `main`
- code quality gates run in GitHub Actions
- dependency risk can be checked through `cargo-deny`
- this repository now includes dedicated security workflows for dependency,
  secret, and code scanning

### Authentication and authorization

- the runtime architecture includes JWT validation, identity handling, and
  policy evaluation modules
- authorization decisions should be enforced at service boundaries, not only in
  client code
- enterprise deployments should integrate with a managed identity provider and
  short-lived credentials

### Secrets handling

- secrets are expected to come from environment variables or external secret
  providers, not from source control
- production deployments should use a managed secret store or cloud KMS
- repository scanning is required to reduce accidental secret disclosure

### Auditability

- the project includes audit/event concepts, logging modules, and provenance
  models intended to support traceability
- operators should forward audit events to immutable or append-only storage with
  retention controls appropriate to their environment

### Data protection

- encryption in transit and at rest should be enabled by deployers
- sensitive-field handling and key management must be configured in the target
  environment
- personal data minimization should be applied before data enters agent
  workflows whenever possible

## Shared Responsibility Model

### Maintained by this repository

- source code controls and secure coding baseline
- dependency policy and supply-chain scanning configuration
- reference guidance for runtime security design
- repository-level vulnerability reporting channel

### Required from deployers/integrators

- production identity and access management
- network isolation and perimeter controls
- infrastructure patching and host hardening
- encryption key custody and rotation
- data retention, deletion, and lawful-basis decisions
- customer-specific compliance mapping and contractual terms

## Minimum Enterprise Recommendations

- require branch protection and mandatory status checks
- enable GitHub secret scanning and push protection
- enable GitHub Advanced Security or equivalent code scanning
- run workloads in isolated tenants or isolated namespaces
- use managed KMS-backed secrets and key rotation
- keep audit logs centralized and tamper-evident
- document data classification for every ingestion path
- require human approval for high-risk or externally acting agents
