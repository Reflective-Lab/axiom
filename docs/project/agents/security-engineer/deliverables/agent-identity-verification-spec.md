# Agent Identity Verification & Context Key Spoofing Prevention

**Issue:** REF-10 | **Severity:** HIGH | **Author:** Ava Petrov | **Date:** 2026-03-13

## 1. Problem Statement

Converge agents are currently identified by string names (trait level) and monotonic `u32` IDs (runtime level). Neither is verified at registration or execution time. Any code path that can register an agent can claim any name, and context contributions carry no proof of origin.

**Threat:** An attacker who can register an agent (via library dependency, WASM module, or compromised provider) can:
- Impersonate a trusted agent by claiming its name
- Inject facts under a false identity, corrupting convergence results
- Write context keys that should belong to another agent
- Bypass HITL gates by spoofing the `decided_by` field in `GateDecision`

## 2. Current State Analysis

| Component | File | Gap |
|-----------|------|-----|
| `Agent` trait | `converge-traits/src/agent.rs` | `name()` returns `&str` — no uniqueness enforcement, no verification |
| `AgentId(u32)` | `converge-core/src/agent.rs` | Monotonic counter — predictable, no auth |
| `Context` trait | `converge-traits/src/context.rs` | Read-only view is good, but no write permission model |
| `AgentEffect` | `converge-traits/src/agent.rs` | Effects carry agent name but no proof of origin |
| `Actor` | `converge-core/src/types/provenance.rs` | `id: String, kind: ActorKind` — unverified string |
| `GateDecision` | `converge-core/src/gates/hitl.rs` | `decided_by: String` — application layer must verify |
| `AuthorityGrant` | `converge-core/src/gates/boundary.rs` | Framework exists, `pub(crate)` constructors, but not enforced in promotion flow |

**What works well:**
- `Fact` private constructor via `PromotionGate` — cannot forge facts outside the gate
- `ValidationToken` zero-sized forgery prevention — `ValidationReport` cannot be fabricated
- `ProposedFact.source_agent` field — audit trail exists
- `PromotionRecord` with full provenance — decisions are traceable

## 3. Architecture Specification

### 3.1 Agent Registration with Verified Identity

**Requirement:** Every agent registered in a convergence engine MUST have a verified, unique identity.

```rust
/// Verified agent identity — created only by the engine at registration time.
/// Cannot be constructed outside converge-core.
pub struct VerifiedAgentId {
    id: u32,                          // Monotonic (existing)
    name: String,                     // Human-readable (existing)
    fingerprint: AgentFingerprint,    // New: content-addressable identity
    registered_at: Timestamp,
    _token: RegistrationToken,        // Zero-sized, pub(crate) — prevents external construction
}

/// Content-addressable fingerprint derived from agent code/config.
/// For native agents: hash of type name + version.
/// For WASM agents: hash of module bytes (ties to REF-6 signing).
/// For LLM agents: hash of provider config + model identifier.
pub struct AgentFingerprint(ContentHash);

/// Registration must go through the engine, which:
/// 1. Verifies name uniqueness (reject duplicate names)
/// 2. Computes fingerprint
/// 3. Assigns monotonic ID
/// 4. Returns VerifiedAgentId (only way to get one)
impl Engine {
    pub fn register_agent(&mut self, agent: impl Agent) -> Result<VerifiedAgentId, RegistrationError> {
        // Enforce uniqueness
        if self.agents.contains_name(agent.name()) {
            return Err(RegistrationError::DuplicateName(agent.name().to_string()));
        }
        // Compute fingerprint
        let fingerprint = AgentFingerprint::from_agent(&agent);
        // Create verified ID (only possible here due to RegistrationToken)
        Ok(VerifiedAgentId::new(self.next_id(), agent.name(), fingerprint))
    }
}
```

**Acceptance Criteria:**
- [x] Maps to: "Each agent has a verifiable identity (cryptographic or runtime-enforced)"
- [x] Maps to: "converge-core rejects context contributions from unverified agents"

### 3.2 Context Contributions Tagged with Verified Identity

**Requirement:** Every `AgentEffect` MUST carry the `VerifiedAgentId` of the agent that produced it. The engine MUST reject effects that don't match the executing agent.

```rust
/// AgentEffect now carries verified identity proof.
pub struct AgentEffect {
    pub agent_id: VerifiedAgentId,   // Was: agent name string
    pub facts: Vec<ProposedFact>,    // Proposals only (existing)
    pub diagnostics: Vec<Diagnostic>,
}

/// Engine execution loop enforces identity:
impl Engine {
    fn execute_agent(&self, agent: &dyn Agent, id: &VerifiedAgentId, ctx: &Context) -> AgentEffect {
        let mut effect = agent.execute(ctx);
        // Engine stamps verified identity — agent cannot choose its own
        effect.agent_id = id.clone();
        // Verify all proposed facts carry correct source_agent
        for proposal in &mut effect.facts {
            proposal.source_agent = id.name.clone();
            proposal.source_fingerprint = Some(id.fingerprint.clone());
        }
        effect
    }
}
```

**Key principle:** The agent never stamps its own identity. The engine does, using the `VerifiedAgentId` from registration. This prevents impersonation even if an agent's `execute()` method tries to claim a different identity.

**Acceptance Criteria:**
- [x] Maps to: "Context contributions are tagged with verified agent identity — cannot be forged"

### 3.3 Context Key Ownership Model

**Requirement:** Context keys have declared ownership. Only the owning agent (or the engine itself) can write facts to its keys.

**Design choice:** Declarative ownership via agent `dependencies()` and a new `contributions()` method, enforced at the engine level.

```rust
/// Extended Agent trait — agents declare what they read AND write.
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> &[ContextKey];       // Read permissions (existing)
    fn contributions(&self) -> &[ContextKey];       // NEW: Write permissions
    fn accepts(&self, ctx: &Context) -> bool;
    fn execute(&self, ctx: &Context) -> AgentEffect;
}

/// Engine enforces write permissions during effect merge:
impl Engine {
    fn merge_effect(&mut self, effect: &AgentEffect, agent: &dyn Agent) -> Result<(), MergeError> {
        let allowed_keys: HashSet<ContextKey> = agent.contributions().iter().cloned().collect();
        for proposal in &effect.facts {
            if !allowed_keys.contains(&proposal.target_key) {
                return Err(MergeError::UnauthorizedContextKey {
                    agent: effect.agent_id.name.clone(),
                    attempted_key: proposal.target_key,
                    allowed_keys: allowed_keys.iter().cloned().collect(),
                });
            }
        }
        // Merge proceeds only if all proposals target allowed keys
        Ok(())
    }
}
```

**Ownership rules:**
1. `ContextKey::Seeds` — engine-only (set at initialization, immutable after)
2. `ContextKey::Proposals` — any agent can write (staging area by design)
3. All other keys — agent must declare in `contributions()`
4. Engine rejects proposals targeting undeclared keys
5. Overlap is allowed (multiple agents can contribute to `Hypotheses`) — this is intentional for convergence

**Acceptance Criteria:**
- [x] Maps to: "Context keys have an ownership model — only the owning agent can write to its keys"
- [x] Maps to: "Test: agent A cannot write context keys belonging to agent B"

### 3.4 Unregistered Agent Rejection

**Requirement:** The engine MUST reject any attempt to contribute to context from an unregistered agent.

```rust
/// The engine's agent registry is the single source of truth.
/// No VerifiedAgentId = no execution, no effect merge.
impl Engine {
    fn cycle(&mut self, ctx: &mut Context) -> CycleResult {
        for (id, agent) in &self.registered_agents {
            // Only registered agents with VerifiedAgentId can execute
            if agent.accepts(ctx) {
                let effect = self.execute_agent(agent.as_ref(), id, ctx);
                self.merge_effect(&effect, agent.as_ref())?;
            }
        }
        // No path exists for unregistered agents to contribute
        self.evaluate_convergence(ctx)
    }
}
```

This is already partially true (only registered agents are in the engine loop), but must be explicitly enforced:
- No public API to inject effects without going through `register_agent()` first
- `Context` mutation methods are `pub(crate)` only
- `AgentEffect` merge requires `VerifiedAgentId` which can only come from registration

**Acceptance Criteria:**
- [x] Maps to: "Test: unregistered agent cannot contribute to context"

### 3.5 HITL Gate Identity Verification

**Requirement:** `GateDecision.decided_by` MUST be verified at the application boundary before being passed to the engine.

```rust
/// Application layer provides a verified human identity.
/// This is NOT in converge-core — it's in the application (converge-application).
pub struct VerifiedHumanDecision {
    pub gate_id: GateId,
    pub verdict: GateVerdict,
    pub decided_by: VerifiedHumanId,   // Was: String
    pub decided_at: Timestamp,
    pub auth_method: AuthMethod,        // How identity was verified
}

pub enum AuthMethod {
    OAuth2 { provider: String, token_hash: ContentHash },
    Saml { idp: String, assertion_hash: ContentHash },
    ApiKey { key_hash: ContentHash },
    LocalTrusted,  // Dev/test only — must be gated by build flag
}

/// converge-core accepts only VerifiedHumanDecision, not raw GateDecision.
impl HitlGate {
    pub fn record_decision(&mut self, decision: VerifiedHumanDecision) -> Result<(), GateError> {
        // Decision already verified at application boundary
        // Record in audit trail with auth method
        self.decisions.push(decision);
        Ok(())
    }
}
```

## 4. Implementation Phasing

| Phase | Scope | Target | Owner |
|-------|-------|--------|-------|
| **Phase 1** (Wave 1) | `VerifiedAgentId` + `RegistrationToken` + name uniqueness enforcement | cw-3 | Eli Marsh |
| **Phase 2** (Wave 1) | `contributions()` trait method + engine-level write permission enforcement | cw-4 | Eli Marsh |
| **Phase 3** (Wave 2) | `AgentFingerprint` for native + LLM agents | cw-5 | Eli/Kira |
| **Phase 4** (Wave 2) | `VerifiedHumanDecision` in converge-application | cw-6 | TBD |
| **Phase 5** (Wave 4) | WASM module fingerprint via signed checksums (ties to REF-6) | Wave 4 | TBD |

**Critical path:** Phase 1 and Phase 2 must ship before first pilot. Without verified agent identity, convergence results cannot be trusted.

## 5. Test Specification

### T1: Duplicate Agent Name Rejection
```
Given: Engine with agent "analyst" registered
When: Second agent named "analyst" attempts registration
Then: RegistrationError::DuplicateName returned
And: Second agent is NOT in the registry
```

### T2: Unregistered Agent Cannot Contribute
```
Given: Engine with agents ["analyst", "strategist"]
When: AgentEffect with agent_id referencing "attacker" is injected
Then: Effect is rejected (no public API path exists)
And: Context is unchanged
```

### T3: Agent Cannot Write to Undeclared Context Key
```
Given: Agent "analyst" with contributions() = [Hypotheses, Evaluations]
When: "analyst" produces ProposedFact targeting ContextKey::Strategies
Then: MergeError::UnauthorizedContextKey returned
And: Proposal is NOT added to context
```

### T4: Agent Cannot Impersonate Another Agent
```
Given: Agents "analyst" and "strategist" registered
When: "analyst" execute() returns effect with source_agent = "strategist"
Then: Engine overwrites source_agent with "analyst" (verified identity)
And: ProposedFact.source_agent == "analyst" in context
```

### T5: Seeds Key is Immutable After Initialization
```
Given: Engine initialized with seeds ["intent A"]
When: Any agent produces ProposedFact targeting ContextKey::Seeds
Then: MergeError::ImmutableKey returned
```

### T6: VerifiedAgentId Cannot Be Constructed Externally
```
Given: Code outside converge-core crate
When: Attempting VerifiedAgentId { id: 0, name: "fake", ... }
Then: Compilation error (RegistrationToken is pub(crate))
```

### T7: Fingerprint Changes on Agent Modification
```
Given: Agent "analyst" v1 registered with fingerprint F1
When: Agent "analyst" v2 (different code) attempts registration
Then: Fingerprint F2 != F1
And: Engine detects version change (can be used for hot-reload validation)
```

## 6. Threat Mitigations

| Threat | Mitigation | Phase |
|--------|-----------|-------|
| Agent impersonation | `VerifiedAgentId` + engine-stamped identity | Phase 1 |
| Context key spoofing | `contributions()` + engine-level enforcement | Phase 2 |
| Unregistered agent injection | `RegistrationToken` + no public effect merge API | Phase 1 |
| HITL decision spoofing | `VerifiedHumanDecision` at application boundary | Phase 4 |
| WASM module tampering | `AgentFingerprint` from signed module hash | Phase 5 |
| Name collision attacks | Duplicate name rejection at registration | Phase 1 |
| Effect replay attacks | `VerifiedAgentId` includes registration timestamp | Phase 1 |

## 7. Dependencies

- **REF-6** (WASM signing): Phase 5 fingerprinting depends on WASM module signatures
- **REF-8** (LLM boundary): DONE — `ProposedFact`/`Fact` boundary is the foundation this builds on
- **REF-11** (Security traits): `IdentityVerified` trait should compose with `VerifiedAgentId`
- **converge-policy** (Cedar): Phase 2 write permissions could be expressed as Cedar policies (Wave 2)

## 8. Open Questions

1. **Should `contributions()` be static or dynamic?** Static (declared at registration) is simpler and more secure. Dynamic (per-cycle) allows more flexibility but is harder to audit. **Recommendation: static.**
2. **Should fingerprints be stored in the experience store?** Yes — enables cross-run identity correlation and tamper detection. Deferred to Phase 3.
3. **Hot-reload semantics:** If an agent's fingerprint changes mid-run, should the engine reject it or re-register? **Recommendation: reject and require explicit re-registration.**
