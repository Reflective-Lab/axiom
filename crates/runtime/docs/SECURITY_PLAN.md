# Security Implementation Plan

## Layer Analysis

### What Belongs in converge-runtime

| Component | Rationale |
|-----------|-----------|
| **gRPC mTLS Server** | Core transport security for this service |
| **Auth Interceptor** | Request-level identity extraction and validation |
| **Policy Engine** | Authorization decisions for this service's methods |
| **NATS mTLS Client** | Secure messaging from this service |
| **Audit Logger** | Emit audit events to JetStream |
| **Field Decryption** | Decrypt at service boundary (authorized endpoint) |
| **JWT Validation** | Validate tokens issued by auth service |
| **SPIFFE Client** | Consume SVIDs from SPIRE agent |

### What Belongs Elsewhere

| Component | Where | Rationale |
|-----------|-------|-----------|
| **SPIRE Agent** | Infrastructure (K8s DaemonSet / systemd) | Runs as infrastructure, not application code |
| **SPIRE Server** | Infrastructure (separate deployment) | Certificate authority, not runtime concern |
| **Passkey Registration/Auth** | **converge-auth** (new service) | User-facing auth is separate concern |
| **JWT Issuance** | **converge-auth** | Token minting is auth service responsibility |
| **Key Management Service** | Cloud KMS (GCP/AWS) | Never store KEKs in application |
| **NATS Server Config** | Infrastructure (Helm/Terraform) | Server-side permissions are infra config |
| **Policy Definitions** | Config repo / Policy service | Policies change independently of code |
| **Web Client Passkey Flow** | **converge-www** (frontend) | Browser WebAuthn API calls |

### Shared Libraries (converge-core or new crate)

| Component | Crate | Rationale |
|-----------|-------|-----------|
| **Field Encryption/Decryption** | `converge-crypto` | Reusable across services |
| **Audit Event Types** | `converge-audit` | Consistent audit schema |
| **Policy Types** | `converge-policy` | Shared policy definitions |

---

## Architecture Decision: Auth Service

**Option A: Embed passkey auth in converge-runtime**
- Pros: Simpler deployment, fewer services
- Cons: Mixes concerns, harder to scale auth independently

**Option B: Separate converge-auth service** ← Recommended
- Pros: Single responsibility, can scale independently, cleaner security boundary
- Cons: Additional service to deploy

**Decision: Option B**

The auth flow becomes:
```
Browser → converge-www (passkey UI)
       ↓
converge-auth (WebAuthn verify → JWT issue)
       ↓
Browser stores JWT → sends with gRPC-Web calls
       ↓
converge-runtime (validates JWT, enforces policy)
```

---

## Implementation Phases

### Phase 1: Transport Security (converge-runtime)
**Goal:** mTLS for gRPC, secure NATS connections

**Tasks:**
1. Add dependencies to Cargo.toml
2. Create `src/identity/` module - SPIFFE SVID client
3. Update `src/grpc/server.rs` - Add ServerTlsConfig with client auth
4. Create `src/nats/` module - mTLS NATS client
5. Add security config to `src/config.rs`

**Dependencies to add:**
```toml
# TLS and certificates
rustls = "0.23"
rustls-pemfile = "2.0"
tokio-rustls = "0.26"
x509-parser = "0.16"

# SPIFFE (optional - can start with file-based certs)
spiffe = { version = "0.6", optional = true }

# NATS
async-nats = "0.35"
```

**New feature flag:**
```toml
security = ["rustls", "rustls-pemfile", "tokio-rustls", "x509-parser"]
nats = ["async-nats"]
spiffe = ["dep:spiffe", "security"]
```

**Files to create:**
- `src/identity/mod.rs`
- `src/identity/spiffe.rs` (SPIFFE workload API client)
- `src/identity/file.rs` (file-based certs for dev/testing)
- `src/nats/mod.rs`
- `src/nats/client.rs`

**Files to modify:**
- `Cargo.toml` - add dependencies
- `src/lib.rs` - export new modules
- `src/config.rs` - add SecurityConfig, NatsConfig
- `src/grpc/server.rs` - add TLS config

---

### Phase 2: Request Authentication (converge-runtime)
**Goal:** Extract and validate identity on every request

**Tasks:**
1. Create `src/interceptor/` module
2. Implement auth interceptor (extract service ID from cert, user ID from JWT)
3. Create `src/auth/` module for JWT validation
4. Add auth errors to `src/error.rs`
5. Wire interceptor into gRPC server

**Dependencies to add:**
```toml
jsonwebtoken = "9"
```

**Files to create:**
- `src/interceptor/mod.rs`
- `src/interceptor/auth.rs`
- `src/auth/mod.rs`
- `src/auth/jwt.rs`
- `src/auth/identity.rs` (VerifiedIdentity struct)

**Files to modify:**
- `src/error.rs` - add AuthError variants
- `src/grpc/server.rs` - add interceptor to service

---

### Phase 3: Authorization (converge-runtime)
**Goal:** Policy-based access control

**Tasks:**
1. Create `src/policy/` module
2. Implement policy engine with YAML config
3. Integrate policy checks into auth interceptor
4. Add policy config loading

**Files to create:**
- `src/policy/mod.rs`
- `src/policy/engine.rs`
- `src/policy/loader.rs`
- `config/policy.yaml` (default policy)

**Files to modify:**
- `src/config.rs` - add PolicyConfig
- `src/interceptor/auth.rs` - call policy engine

---

### Phase 4: Audit Logging (converge-runtime)
**Goal:** Durable audit trail via NATS JetStream

**Tasks:**
1. Create `src/audit/` module
2. Implement audit logger (publish to JetStream)
3. Define audit event schema
4. Add audit calls to interceptor and handlers

**Files to create:**
- `src/audit/mod.rs`
- `src/audit/logger.rs`
- `src/audit/events.rs`

**Files to modify:**
- `src/interceptor/auth.rs` - log auth decisions
- `src/grpc/server.rs` - log method calls

---

### Phase 5: Field Encryption (converge-runtime or converge-crypto)
**Goal:** E2E encryption for sensitive Protobuf fields

**Decision point:** Create separate `converge-crypto` crate or embed in runtime?
- If only runtime needs it → embed
- If multiple services need it → separate crate

**Tasks:**
1. Create `src/crypto/` module (or new crate)
2. Implement ChaCha20-Poly1305 field encryption
3. Create KeyStore trait and Cloud KMS implementation
4. Integrate into service handlers

**Dependencies to add:**
```toml
chacha20poly1305 = "0.10"
async-trait = "0.1"
```

**Files to create:**
- `src/crypto/mod.rs`
- `src/crypto/field.rs` (encrypt/decrypt functions)
- `src/crypto/keystore.rs` (KeyStore trait)
- `src/crypto/kms.rs` (Cloud KMS implementation)
- `src/crypto/memory.rs` (in-memory for testing)

---

### Phase 6: Auth Service (NEW: converge-auth)
**Goal:** Passkey authentication and JWT issuance

**This is a separate service.** Create new repo/workspace member.

**Tasks:**
1. Create converge-auth service skeleton
2. Implement WebAuthn registration/authentication
3. Implement JWT issuance
4. Add credential storage (Firestore)
5. Deploy as separate service

**This phase is OUT OF SCOPE for converge-runtime.**

---

## Dependency Summary

### Phase 1 (Transport)
```toml
rustls = "0.23"
rustls-pemfile = "2.0"
tokio-rustls = "0.26"
x509-parser = "0.16"
async-nats = "0.35"
spiffe = { version = "0.6", optional = true }
```

### Phase 2 (Authentication)
```toml
jsonwebtoken = "9"
```

### Phase 5 (Encryption)
```toml
chacha20poly1305 = "0.10"
async-trait = "0.1"
```

---

## Module Structure After Implementation

```
src/
├── lib.rs
├── main.rs
├── config.rs                 # + SecurityConfig, NatsConfig, PolicyConfig
├── error.rs                  # + AuthError, CryptoError, PolicyError
│
├── identity/                 # NEW
│   ├── mod.rs
│   ├── spiffe.rs            # SPIFFE workload API client
│   └── file.rs              # File-based certs (dev/test)
│
├── interceptor/             # NEW
│   ├── mod.rs
│   └── auth.rs              # Auth interceptor
│
├── auth/                    # NEW
│   ├── mod.rs
│   ├── jwt.rs               # JWT validation
│   └── identity.rs          # VerifiedIdentity type
│
├── policy/                  # NEW
│   ├── mod.rs
│   ├── engine.rs            # Policy evaluation
│   └── loader.rs            # YAML policy loading
│
├── crypto/                  # NEW
│   ├── mod.rs
│   ├── field.rs             # Field encryption
│   ├── keystore.rs          # KeyStore trait
│   ├── kms.rs               # Cloud KMS impl
│   └── memory.rs            # In-memory impl (test)
│
├── nats/                    # NEW
│   ├── mod.rs
│   └── client.rs            # mTLS NATS client
│
├── audit/                   # NEW
│   ├── mod.rs
│   ├── logger.rs            # JetStream audit logger
│   └── events.rs            # Audit event types
│
├── grpc/                    # MODIFIED
│   ├── mod.rs
│   ├── server.rs            # + TLS config, interceptor
│   └── generated/
│
└── ... (existing modules unchanged)
```

---

## Configuration Structure

```yaml
# config/default.yaml (additions)
security:
  # Certificate source: "spiffe" or "file"
  identity_source: "file"

  # File-based certs (when identity_source = "file")
  cert_path: "/etc/converge/tls/server.crt"
  key_path: "/etc/converge/tls/server.key"
  ca_path: "/etc/converge/tls/ca.crt"

  # JWT validation
  jwt_secret: "${JWT_SECRET}"  # From environment
  jwt_issuer: "https://auth.converge.zone"
  jwt_audience: "converge-runtime"

nats:
  servers:
    - "nats://nats.converge.svc:4222"
  # Uses same certs as gRPC when security.identity_source = "file"

policy:
  # Policy source: "file" or "remote"
  source: "file"
  path: "config/policy.yaml"

audit:
  stream: "AUDIT"
  subject_prefix: "audit.runtime"
```

---

## Testing Strategy

### Unit Tests
- Policy engine evaluation
- JWT validation (valid/expired/invalid signature)
- Field encryption roundtrip
- Certificate parsing

### Integration Tests
- gRPC with mTLS (self-signed certs)
- NATS publish/subscribe with TLS
- Full auth flow with mock JWT

### E2E Tests (requires infrastructure)
- SPIFFE/SPIRE integration
- Cloud KMS key retrieval
- JetStream audit persistence

---

## Open Questions

1. **Policy storage:** File-based YAML or remote policy service (OPA/Cedar)?
   - Start with file-based, migrate to remote later if needed

2. **Key rotation:** How to handle DEK rotation for encrypted fields?
   - Store key version with ciphertext, support decryption with old keys

3. **SPIFFE bootstrap:** How does runtime get initial SVID before SPIRE agent?
   - Fall back to file-based certs, SPIRE optional enhancement

4. **Audit retention:** How long to keep audit logs?
   - 90 days in JetStream, archive to cold storage for compliance

---

## Next Action

Start with **Phase 1: Transport Security**

1. Add dependencies to Cargo.toml
2. Create identity module with file-based cert support
3. Update gRPC server with TLS
4. Test with self-signed certificates

This establishes the foundation without requiring SPIFFE infrastructure.
