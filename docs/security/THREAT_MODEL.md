# Threat Model

## Scope

This threat model covers the open source repository and the reference runtime
shape described here. It is a design-time model, not a substitute for a
deployment-specific risk assessment.

## Key Assets

- prompts, inputs, and model outputs
- agent workflow definitions and policies
- API keys, tokens, and service credentials
- audit logs and provenance records
- customer knowledge-base or retrieval content
- deployment configuration and infrastructure state

## Primary Threats

### Secret exposure

Risks:

- credentials committed to git
- credentials leaked in logs, prompts, or examples
- long-lived secrets reused across environments

Mitigations:

- secret scanning in CI
- managed secret storage
- least-privilege credentials
- key rotation and environment separation

### Prompt and data exfiltration

Risks:

- sensitive data sent to external model providers
- retrieval layers exposing data outside allowed context
- overbroad logging of prompts and outputs

Mitigations:

- provider allowlists and vendor review
- data classification before ingestion
- prompt/log redaction where appropriate
- tenant and environment isolation

### Authorization bypass

Risks:

- missing policy checks on internal endpoints
- relying on UI-only restrictions
- excessive service permissions

Mitigations:

- server-side authorization
- short-lived identity tokens
- policy-as-code reviews
- negative tests for denied actions

### Supply-chain compromise

Risks:

- vulnerable dependencies
- malicious transitive packages
- build-time compromise through third-party actions

Mitigations:

- dependency scanning
- lockfile review
- pinned GitHub Actions where practical
- SBOM generation for release evidence

### Unsafe autonomous behavior

Risks:

- agents taking destructive or externally visible actions without approval
- unreviewed high-impact recommendations
- policy drift between environments

Mitigations:

- human-in-the-loop approval for high-risk actions
- environment-specific guardrails
- audit trails and replayable evidence
- explicit action boundaries in product documentation

## Trust Boundaries

- repository and CI boundary
- runtime service boundary
- external model/provider boundary
- customer data source boundary
- operator infrastructure boundary

## Assumptions

- operators control deployment infrastructure
- customer-specific access policies are configured outside this repository
- regulated workloads require additional review
- third-party model providers may introduce legal and security obligations

## Residual Risk

Even with the controls documented here, residual risks remain around model
behavior, vendor ecosystems, deployment misconfiguration, and over-collection of
sensitive data. These risks must be accepted, transferred, or mitigated by the
deploying organization.
