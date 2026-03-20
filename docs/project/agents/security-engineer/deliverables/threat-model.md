# Converge Platform — Threat Model

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering]
> Effective date: [Pending approval]
> Review cadence: Every release, or upon material architecture changes
> Related: `risk-assessment.md` (quantitative scoring), `control-catalog.md` (mitigations)

---

## 1. Purpose

This document identifies and describes the threat surfaces unique to the Converge semantic governance platform. It serves as the reference for security reviews, release gates, and architecture decisions.

## 2. System Overview

Converge is a multi-agent convergence engine. Agents contribute facts to a shared context. The engine runs convergence loops, checking invariants at each cycle, until the context stabilizes or a budget is exhausted.

**Key architectural components:**

| Component | Crate | Trust Level | Description |
|-----------|-------|-------------|-------------|
| Convergence Engine | converge-core | Trusted | Runs convergence loops, enforces invariants, manages context |
| Type System | converge-traits | Trusted | Defines Fact, ProposedFact, Agent, Invariant types |
| LLM Providers | converge-provider | Untrusted output | Wraps LLM API calls, produces ProposedFacts |
| Policy Engine | converge-policy | Trusted | Cedar-based authorization decisions |
| WASM Runtime | converge-runtime | Semi-trusted | Loads and executes WASM modules |
| Experience Store | converge-experience | Trusted (data integrity dependent) | SurrealDB/LanceDB persistence |
| Analytics | converge-analytics | Trusted | Telemetry and metrics collection |

## 3. Trust Boundaries

### 3.1 Primary Trust Boundary: ProposedFact → Fact

**The most critical boundary in the platform.**

```
LLM Response (untrusted) → ProposedFact → TryFrom validation → Fact (trusted)
                                              ↓ (fail)
                                         Rejection + diagnostic
```

- LLM outputs are NEVER trusted. All LLM responses produce `ProposedFact`, never `Fact`.
- `TryFrom<ProposedFact> for Fact` is the ONLY promotion path.
- Validation includes: confidence range (finite, 0.0–1.0), content non-empty.
- Post-promotion, structural invariants provide defense-in-depth.
- **Status:** Core boundary implemented (REF-8, conditional pass). NaN bypass fixed.

### 3.2 Agent Identity Boundary

```
Agent Registration (trusted) → Agent ID → Context Contribution (verified identity)
```

- Each agent must have a verifiable identity.
- Context contributions must be tagged with verified agent identity.
- Context keys have an ownership model — agents can only write to authorized keys.
- **Status:** Not yet implemented (REF-10, HIGH priority).

### 3.3 WASM Module Boundary

```
Module Source (external) → Signature Verification → WASM Sandbox → Execution
```

- WASM modules are external code executing within the platform.
- Modules must be signed and verified before loading.
- Sandbox must prevent escape (memory, filesystem, network access).
- **Status:** Not yet implemented (REF-6, HIGH priority).

### 3.4 External Integration Boundary

```
External System (CRM, webhook) → TLS + HMAC → Schema Validation → Internal Processing
```

- All external integrations must authenticate (HMAC signatures for webhooks).
- All traffic must use TLS 1.2+.
- Payload schema validation rejects unexpected fields.
- **Status:** Not yet implemented (REF-16, HIGH priority).

## 4. Threat Surfaces

### T1: LLM Prompt Injection via Context

**Severity: CRITICAL** | **Tracking: REF-8**

- **Attack:** Attacker crafts input that, when processed by an LLM agent, causes the LLM to emit a response designed to corrupt convergence results.
- **Vector:** LLM response → ProposedFact → (if validation bypassed) → Fact → corrupted context.
- **Impact:** Corrupted convergence results. All downstream decisions based on poisoned context.
- **Mitigations:**
  - ProposedFact/Fact type separation (implemented)
  - TryFrom validation with NaN/infinity guards (implemented)
  - Structural invariants as defense-in-depth (implemented)
  - Content-based injection detection invariants (available, user-configurable)
- **Residual risk:** Content-level prompt injection that produces plausible-looking ProposedFacts with valid confidence scores. Requires domain-specific invariants to catch.

### T2: Agent Impersonation / Context Key Spoofing

**Severity: HIGH** | **Tracking: REF-10**

- **Attack:** A malicious or compromised agent impersonates another agent and writes context keys under a false identity.
- **Vector:** Agent constructs Facts with another agent's identity → corrupted context attribution.
- **Impact:** Targeted manipulation of convergence outcomes. Audit trail corruption.
- **Mitigations (planned):**
  - Cryptographic or runtime-enforced agent identity
  - Context key ownership model
  - Engine rejects contributions from unverified agents
- **Current gap:** `Fact::new()` is public. Any agent can construct a Fact directly via `effect.facts`, bypassing the ProposedFact validation path. No agent classification (trusted vs untrusted) is enforced at the engine level.

### T3: WASM Module Tampering

**Severity: HIGH** | **Tracking: REF-6**

- **Attack:** Attacker replaces or modifies a WASM module to execute malicious code within the platform.
- **Vector:** Compromised module source → unsigned module loaded → arbitrary execution.
- **Impact:** Full platform compromise. Data exfiltration. Context corruption.
- **Mitigations (planned):**
  - Module signature verification (checksums + signing keys)
  - WASM sandbox enforcement (memory limits, no filesystem/network)
  - Module allowlisting

### T4: Secrets Leakage

**Severity: HIGH** | **Tracking: REF-7**

- **Attack:** LLM provider API keys or other secrets exposed via logs, config files, error messages, or source code.
- **Vector:** Secrets in .env files, hardcoded in source, or logged in stack traces.
- **Impact:** Unauthorized API usage. Financial exposure. Provider account compromise.
- **Mitigations (planned):**
  - Google Secret Manager or Vault for all secrets
  - Pre-commit hooks scanning for secrets
  - Log sanitization (no secrets, tokens, or PII in logs)

### T5: Query Injection (SurrealDB/LanceDB)

**Severity: HIGH** | **Tracking: REF-9**

- **Attack:** Attacker injects malicious queries via user-controlled input that reaches the experience store.
- **Vector:** Unsanitized input → query string interpolation → arbitrary data access or modification.
- **Impact:** Data exfiltration. Cross-customer data leakage. Data corruption.
- **Mitigations (planned):**
  - Parameterized queries only
  - Input validation at all external boundaries
  - Principle of least privilege for database connections

### T6: Invariant Bypass

**Severity: HIGH** | **Tracking: implicit**

- **Attack:** Attacker finds a code path that skips invariant checking, allowing invalid context to persist.
- **Vector:** Edge case in convergence loop, error handling that swallows invariant failures, or budget exhaustion that skips final check.
- **Impact:** Invalid convergence results accepted as valid.
- **Mitigations:**
  - Budget enforcement terminates loops (implemented)
  - Structural invariants run post-merge every cycle (implemented)
  - Final convergence check before returning results (implemented)

### T7: Serialization Boundary Bypass

**Severity: MEDIUM** | **Tracking: future**

- **Attack:** Attacker crafts a serialized Fact payload and feeds it to a deserialization endpoint, bypassing ProposedFact validation.
- **Vector:** `Fact` derives `Deserialize` → direct deserialization possible → injected fact.
- **Impact:** Validation boundary bypassed for any fact accepted from external serialized sources.
- **Mitigations (planned):**
  - No external deserialization endpoints currently exist
  - When converge-experience or converge-remote introduce external fact ingestion, deserialization must route through ProposedFact validation
  - Consider: custom Deserialize impl that always produces ProposedFact

### T8: Pilot Customer Data Leakage

**Severity: HIGH** | **Tracking: REF-15, REF-16**

- **Attack:** Cross-customer data leakage via shared storage, misconfigured access controls, or unencrypted data at rest.
- **Vector:** `pilot-data/{customer-id}/` directories with insufficient isolation.
- **Impact:** Commercially sensitive customer workflow data exposed. Trust breach. Contract termination.
- **Mitigations (planned):**
  - Per-customer access controls (separate storage or encrypted per-customer keys)
  - Encryption at rest for JSON data files
  - Access audit trail
  - PII anonymization for aggregated exports (pilot-anonymize.sh — reviewed and fixed)

## 5. Attack Trees

### 5.1 Corrupt Convergence Results

```
Goal: Inject false fact into convergence output
├── Via LLM prompt injection (T1)
│   ├── Bypass TryFrom validation → BLOCKED (NaN fixed, range checks)
│   ├── Pass TryFrom but bypass invariants → BLOCKED (structural invariants)
│   └── Pass TryFrom AND invariants (plausible content) → RESIDUAL RISK
├── Via agent impersonation (T2)
│   ├── Construct Fact directly (Fact::new public) → OPEN (REF-10)
│   └── Spoof context key ownership → OPEN (REF-10)
├── Via WASM module tampering (T3)
│   └── Replace module with malicious version → OPEN (REF-6)
└── Via serialization bypass (T7)
    └── Deserialize crafted Fact payload → LOW RISK (no endpoints today)
```

### 5.2 Exfiltrate Data

```
Goal: Access customer data without authorization
├── Via query injection (T5) → OPEN (REF-9)
├── Via pilot data directory access (T8) → OPEN (REF-15)
├── Via webhook interception (T8) → OPEN (REF-16)
└── Via secrets leakage → LLM API access (T4) → OPEN (REF-7)
```

## 6. Priority Mitigations

| Priority | Threat | Issue | Status |
|----------|--------|-------|--------|
| 1 | LLM boundary (T1) | REF-8 | In review — conditional pass |
| 2 | Agent identity (T2) | REF-10 | Todo |
| 3 | Secrets management (T4) | REF-7 | Todo |
| 4 | Pilot data isolation (T8) | REF-15 | Todo |
| 5 | WASM signing (T3) | REF-6 | Todo |
| 6 | Query injection (T5) | REF-9 | Todo |
| 7 | Webhook auth (T8) | REF-16 | Todo |

## 7. Review Schedule

- **Release gate:** Full threat model review against changes in each release
- **Quarterly:** Risk scoring refresh and new threat identification
- **Architecture change:** Any new crate, external integration, or trust boundary change triggers review
- **Incident:** Post-incident threat model update

---

*This document is the authoritative reference for Converge security threats. All security backlog issues (REF-6 through REF-18) trace back to threats identified here.*
