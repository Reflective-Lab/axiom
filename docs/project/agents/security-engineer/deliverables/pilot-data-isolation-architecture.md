# Pilot Data Isolation Architecture — Converge

> Version: 1.0
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering]
> Effective date: cw-4 (Mar 30 - Apr 3, 2026)
> SOC 2 mapping: CC6.1, CC6.3, CC6.7, C1.1, C1.2, PI1.1
> Related issues: REF-15, REF-7, REF-48
> Engineering Plan: MVP-7 (No customer data leaves their environment)

---

## 1. Purpose

Define the architecture that ensures complete data isolation between pilot customers. Each pilot customer's data must be stored, processed, and exported in isolation — no cross-customer data leakage, no shared state, no co-mingled storage.

This document is the implementation spec for MVP-7 acceptance criteria and the operational guide for Dex (infrastructure), Eli (engine integration), and Leo (onboarding playbook).

## 2. Deployment Model Decision

**Decision: Single-tenant workspaces on shared infrastructure** for the first 3-4 pilots.

Rationale:
- Full single-tenant deployment (separate compute) is premature at 3-4 customers
- Multi-tenant with logical isolation introduces cross-customer risk we can't afford pre-SOC 2
- Workspace-per-customer on shared compute is the right tradeoff: strong isolation boundaries, minimal ops overhead

This decision should be revisited at 10+ customers or when SOC 2 Type II requires demonstrable tenant boundary enforcement.

## 3. Workspace Architecture

### 3.1 Directory Structure

```
pilot-workspaces/
  {customer-slug}/                    # e.g., "acme-corp-20260401"
    workspace.toml                    # workspace config (non-secret)
    .access-control.toml              # authorized identities
    data/
      runs/                           # convergence run outputs
        {run-id}.json                 # individual run data (Restricted)
      telemetry/                      # pilot metrics (Confidential)
        {date}.json
      audit/                          # access audit trail (Confidential)
        access.log                    # append-only access log
      exports/                        # anonymized exports (Internal)
        {export-id}.json
    secrets.ref                       # pointers to Secret Manager (never actual values)
    .gitignore                        # excludes data/ from source control
```

### 3.2 Naming Convention

Format: `{company-slug}-{start-date}`

- `company-slug`: lowercase, alphanumeric + hyphens only, max 32 chars
- `start-date`: `YYYYMMDD` format
- Example: `acme-corp-20260401`

Validated by `create-workspace.sh` (see Section 7).

### 3.3 workspace.toml

```toml
[customer]
slug = "acme-corp-20260401"
display_name = "Acme Corp"
pilot_start = "2026-04-01"
pilot_end = "2026-06-30"
data_retention_days = 90   # from pilot_end

[engine]
max_agents = 10
max_runs_per_day = 100
convergence_timeout_seconds = 120

[integration]
webhook_endpoint = ""      # configured during onboarding
webhook_secret_ref = "sm://converge/pilots/acme-corp/webhook-hmac"

[security]
encryption_key_ref = "sm://converge/pilots/acme-corp/data-key"
allowed_ip_ranges = []     # optional IP allowlisting
```

### 3.4 .access-control.toml

```toml
# Only these identities can access this workspace's data.
# Enforced by the access layer (Section 5).

[[authorized]]
identity = "agent:convergence-engine"
permissions = ["read", "write"]
scope = "data/runs/*"

[[authorized]]
identity = "agent:telemetry-exporter"
permissions = ["read", "write"]
scope = "data/telemetry/*"

[[authorized]]
identity = "role:pilot-team"
permissions = ["read"]
scope = "data/*"

[[authorized]]
identity = "role:pilot-team"
permissions = ["read", "write"]
scope = "data/exports/*"

[[authorized]]
identity = "agent:disposal-script"
permissions = ["delete"]
scope = "*"
```

## 4. Data Classification per Workspace

Per the Data Classification Policy (v0.1):

| Data | Classification | Handling |
|------|---------------|----------|
| Convergence run outputs (`data/runs/`) | **Restricted** | Encrypted at rest, per-customer key, access-logged |
| Customer workflow details in context | **Restricted** | Never leaves workspace boundary |
| Telemetry metrics (`data/telemetry/`) | **Confidential** | Encrypted, aggregatable only after anonymization |
| Access audit log (`data/audit/`) | **Confidential** | Append-only, tamper-evident |
| Anonymized exports (`data/exports/`) | **Internal** | Can leave workspace after anonymization verification |
| workspace.toml (config) | **Internal** | No secrets, can be version-controlled |
| secrets.ref (pointers) | **Confidential** | Pointers only, actual secrets in Secret Manager |

## 5. Isolation Controls

### 5.1 Storage Isolation

**Control:** Each customer workspace is a separate directory tree. No shared data directories.

**Enforcement:**
- All engine operations receive a `workspace_root` parameter scoped to one customer
- No code path accepts a bare path — all paths are resolved relative to `workspace_root`
- Path traversal prevention: reject any resolved path that escapes `workspace_root`

```rust
fn resolve_workspace_path(workspace_root: &Path, relative: &Path) -> Result<PathBuf, SecurityError> {
    let resolved = workspace_root.join(relative).canonicalize()?;
    if !resolved.starts_with(workspace_root.canonicalize()?) {
        return Err(SecurityError::PathTraversal {
            attempted: relative.to_path_buf(),
            workspace: workspace_root.to_path_buf(),
        });
    }
    Ok(resolved)
}
```

### 5.2 Encryption at Rest

**Control:** All Restricted and Confidential data encrypted with per-customer keys.

**Implementation (phased):**

| Phase | Approach | When |
|-------|----------|------|
| Phase 1 (cw-4) | Filesystem-level encryption via GCP CMEK or host-level dm-crypt | First pilot |
| Phase 2 (post-pilot) | Application-level envelope encryption with per-customer DEKs | Scale |

Phase 1 is sufficient for first pilots because:
- Single host deployment means filesystem encryption covers all at-rest data
- Per-customer key in Secret Manager enables key rotation and customer-specific revocation
- Application-level encryption adds latency with no security benefit at single-host scale

**Key management:**
- Each customer has a dedicated encryption key in Google Secret Manager (or Vault)
- Key path: `sm://converge/pilots/{customer-slug}/data-key`
- Key rotation: quarterly or on customer request
- Key destruction: per data retention policy (90 days post-pilot-end)

### 5.3 Access Control

**Control:** Only authorized identities can access workspace data.

**Enforcement layers:**

1. **OS-level:** Workspace directories owned by dedicated service account, `chmod 700`
2. **Application-level:** Access control check against `.access-control.toml` before any data read/write
3. **Audit:** All access attempts logged to `data/audit/access.log`

### 5.4 Network Isolation

**Control:** Pilot customer integrations (webhooks) are per-workspace.

- Webhook secrets are workspace-scoped (see REF-16 for webhook auth spec)
- No cross-workspace webhook routing
- Outbound API calls tagged with workspace context for firewall rules

### 5.5 Process Isolation

**Control:** Convergence engine runs are workspace-scoped.

- Engine receives `workspace_root` at run initialization
- All agent context, proposals, and facts are stored within workspace boundary
- InMemoryExperienceStore is instantiated per-workspace (not shared)
- LLM API calls do not include cross-customer context

## 6. Audit Trail

### 6.1 Access Log Format

```json
{
  "timestamp": "2026-04-01T10:30:00Z",
  "identity": "agent:convergence-engine",
  "action": "write",
  "path": "data/runs/run-abc123.json",
  "result": "allowed",
  "workspace": "acme-corp-20260401",
  "bytes": 4096
}
```

### 6.2 Audit Requirements

- Append-only: log file opened in append mode, no truncation
- Tamper evidence: SHA-256 hash chain (each entry includes hash of previous entry)
- Retention: audit logs retained for 1 year (exceeds data retention for compliance evidence)
- Export: audit logs can be exported for SOC 2 evidence collection

### 6.3 Monitored Events

| Event | Severity | Alert |
|-------|----------|-------|
| Successful data access | Info | No |
| Denied access attempt | Warning | Yes — notify security |
| Path traversal attempt | Critical | Yes — immediate alert |
| Cross-workspace access attempt | Critical | Yes — immediate alert |
| Workspace created/destroyed | Info | No |
| Encryption key rotated | Info | No |

## 7. Operational Scripts

### 7.1 create-workspace.sh

Creates a new pilot workspace with correct structure, permissions, and encryption key.

```bash
#!/usr/bin/env bash
# Usage: ./scripts/create-workspace.sh <customer-slug> <pilot-start-date>
# Example: ./scripts/create-workspace.sh acme-corp 2026-04-01

set -euo pipefail

CUSTOMER_SLUG="$1"
PILOT_START="$2"
WORKSPACE_NAME="${CUSTOMER_SLUG}-$(echo "$PILOT_START" | tr -d '-')"
WORKSPACE_ROOT="pilot-workspaces/${WORKSPACE_NAME}"

# Validate slug format
if ! [[ "$CUSTOMER_SLUG" =~ ^[a-z0-9][a-z0-9-]{0,30}[a-z0-9]$ ]]; then
    echo "ERROR: Invalid customer slug. Use lowercase alphanumeric + hyphens, 2-32 chars." >&2
    exit 1
fi

# Prevent duplicate workspaces
if [[ -d "$WORKSPACE_ROOT" ]]; then
    echo "ERROR: Workspace already exists: ${WORKSPACE_ROOT}" >&2
    exit 1
fi

# Create directory structure
mkdir -p "${WORKSPACE_ROOT}/data/"{runs,telemetry,audit,exports}

# Create .gitignore
cat > "${WORKSPACE_ROOT}/.gitignore" << 'GITIGNORE'
data/
secrets.ref
GITIGNORE

# Create workspace.toml (template)
cat > "${WORKSPACE_ROOT}/workspace.toml" << TOML
[customer]
slug = "${WORKSPACE_NAME}"
display_name = ""
pilot_start = "${PILOT_START}"
pilot_end = ""
data_retention_days = 90

[engine]
max_agents = 10
max_runs_per_day = 100
convergence_timeout_seconds = 120

[integration]
webhook_endpoint = ""
webhook_secret_ref = "sm://converge/pilots/${CUSTOMER_SLUG}/webhook-hmac"

[security]
encryption_key_ref = "sm://converge/pilots/${CUSTOMER_SLUG}/data-key"
allowed_ip_ranges = []
TOML

# Create access control template
cat > "${WORKSPACE_ROOT}/.access-control.toml" << 'ACL'
[[authorized]]
identity = "agent:convergence-engine"
permissions = ["read", "write"]
scope = "data/runs/*"

[[authorized]]
identity = "agent:telemetry-exporter"
permissions = ["read", "write"]
scope = "data/telemetry/*"

[[authorized]]
identity = "role:pilot-team"
permissions = ["read"]
scope = "data/*"

[[authorized]]
identity = "role:pilot-team"
permissions = ["read", "write"]
scope = "data/exports/*"

[[authorized]]
identity = "agent:disposal-script"
permissions = ["delete"]
scope = "*"
ACL

# Initialize audit log
echo '{"event":"workspace_created","workspace":"'"${WORKSPACE_NAME}"'","timestamp":"'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'"}' > "${WORKSPACE_ROOT}/data/audit/access.log"

# Set permissions
chmod 700 "${WORKSPACE_ROOT}/data"
chmod 600 "${WORKSPACE_ROOT}/.access-control.toml"

echo "Workspace created: ${WORKSPACE_ROOT}"
echo ""
echo "Next steps:"
echo "  1. Set display_name and pilot_end in workspace.toml"
echo "  2. Create encryption key: gcloud secrets create converge-pilots-${CUSTOMER_SLUG}-data-key ..."
echo "  3. Create webhook secret: gcloud secrets create converge-pilots-${CUSTOMER_SLUG}-webhook-hmac ..."
echo "  4. Add to pilot-workspaces/.gitignore if not already covered"
```

### 7.2 Integration with Existing Scripts

| Script | Integration |
|--------|-------------|
| `pilot-data-dispose.sh` | Extend to accept `--workspace <path>` and destroy entire workspace tree + Secret Manager keys |
| `pilot-anonymize.sh` | Already operates on single customer scope — add workspace path input |
| `pilot-pii-scan.sh` | Add workspace-scoped scanning mode |
| `pilot-data-precommit.sh` | Add `pilot-workspaces/` to scanned paths |

## 8. Test Plan

### T1: Cross-Workspace Isolation

```
Given: workspaces for customer-a and customer-b exist
When: engine running as customer-a attempts to read customer-b/data/runs/run1.json
Then: SecurityError::PathTraversal returned, access denied
And: audit log records denied access with critical severity
```

### T2: Path Traversal Prevention

```
Given: workspace for customer-a at pilot-workspaces/customer-a-20260401/
When: code resolves path "../../customer-b-20260401/data/runs/run1.json"
Then: SecurityError::PathTraversal returned
And: resolved path does NOT start with customer-a workspace root
```

### T3: Encryption at Rest Verification

```
Given: workspace with encryption key configured
When: convergence run writes output to data/runs/
Then: raw bytes on disk are not readable without decryption key
And: reading through application layer returns valid JSON
```

### T4: Access Audit Completeness

```
Given: workspace with audit logging enabled
When: 10 read operations and 5 write operations performed
Then: audit log contains exactly 15 entries
And: each entry has timestamp, identity, action, path, result
And: hash chain is valid (each entry hash includes previous hash)
```

### T5: Export Isolation

```
Given: workspaces for customer-a and customer-b with run data
When: anonymization export runs for customer-a
Then: export output contains ONLY customer-a data
And: no customer-b identifiers, slugs, or data appear in export
```

### T6: Workspace Lifecycle

```
Given: workspace created via create-workspace.sh
When: pilot ends and disposal runs
Then: all data/ contents are securely deleted
And: Secret Manager keys are destroyed
And: audit log is archived (retained 1 year per compliance)
And: workspace directory is removed
```

## 9. Implementation Timeline

| Phase | Scope | Target | Owner |
|-------|-------|--------|-------|
| **Phase 1** | Workspace directory structure, create-workspace.sh, .gitignore coverage | cw-4 | Ava (spec) + Dex (infra) |
| **Phase 2** | Path traversal prevention in engine, access control enforcement | cw-4 | Eli (engine) + Ava (review) |
| **Phase 3** | Filesystem encryption setup (CMEK or dm-crypt) | cw-4 | Dex (infra) + Ava (key mgmt) |
| **Phase 4** | Audit trail implementation, hash chain | cw-4 | Sam (telemetry) + Ava (review) |
| **Phase 5** | Integration testing (T1-T6) | cw-5 | Sam (QA) + Ava (security review) |

## 10. Dependencies

| Dependency | Status | Impact |
|------------|--------|--------|
| REF-7: Secrets management (SecretProvider trait) | **Done** | Per-customer key storage path defined |
| REF-10: Agent identity verification | **In review** | Needed for access control identity verification |
| REF-48: Pilot directory access controls | **Done** | .gitignore and precommit hooks in place |
| REF-16: Webhook authentication | **Todo** | Per-workspace webhook secrets |
| REF-17: Data retention policy | **Done** | 90-day retention window defined |
| REF-18: PII anonymization | **Done** | Anonymization scripts operational |

## 11. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Filesystem encryption adds latency | Low | Medium | Benchmark during cw-4; CMEK has <1ms overhead |
| Path traversal bug in new code | Medium | Critical | Property-based testing (T2), code review gate |
| Audit log disk exhaustion | Low | Medium | Log rotation with archival, size monitoring |
| Secret Manager availability | Low | High | Local encrypted fallback for development; prod requires SM |
| Workspace creation race condition | Low | Low | Mutex/lock file in create-workspace.sh |

---

**Review requested from:** Ren Akiyama (architecture approval), Dex Tanaka (infrastructure feasibility), Eli Marsh (engine integration), Leo Marin (onboarding alignment)
