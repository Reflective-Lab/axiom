# Webhook Authentication & Integration Security Spec — Converge

> Version: 1.0
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering]
> Implementation target: cw-4 (Mar 30 - Apr 3, 2026)
> SOC 2 mapping: CC6.1, CC6.6, CC6.7, CC7.2
> Related issues: REF-16, REF-15 (data isolation), REF-7 (secrets management)

---

## 1. Purpose

Define security requirements for all webhook and integration endpoints in the Converge pilot platform. These controls MUST ship as day-1 requirements with the webhook integration framework (cw-4), not as follow-up work.

## 2. Scope

| Integration Type | Direction | Examples |
|-----------------|-----------|----------|
| Inbound webhooks | External → Converge | CRM events, trigger events |
| Outbound API calls | Converge → External | CRM updates, notification delivery |
| Zapier/iPaaS | Bidirectional | Zapier triggers and actions |
| HITL notifications | Converge → Slack | Pause/approve/reject notifications |

## 3. Inbound Webhook Authentication

### 3.1 HMAC Signature Verification (Required)

All inbound webhooks MUST include an HMAC-SHA256 signature.

**Protocol:**

1. Sender computes: `HMAC-SHA256(webhook_secret, request_body)`
2. Sender sends signature in header: `X-Converge-Signature-256: sha256=<hex-digest>`
3. Receiver computes the same HMAC over the raw request body
4. Receiver compares in constant-time; rejects on mismatch

**Implementation:**

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_webhook_signature(
    secret: &[u8],
    body: &[u8],
    signature_header: &str,
) -> Result<(), WebhookError> {
    let expected = signature_header
        .strip_prefix("sha256=")
        .ok_or(WebhookError::MalformedSignature)?;

    let expected_bytes = hex::decode(expected)
        .map_err(|_| WebhookError::MalformedSignature)?;

    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| WebhookError::InvalidSecret)?;
    mac.update(body);
    let computed = mac.finalize().into_bytes();

    if computed.as_slice().ct_eq(&expected_bytes).into() {
        Ok(())
    } else {
        Err(WebhookError::SignatureVerificationFailed)
    }
}
```

**Critical requirements:**
- Constant-time comparison (via `subtle` crate) — prevents timing attacks
- Raw body bytes used for HMAC (not parsed/re-serialized JSON)
- Reject requests with missing or malformed signature header (401)
- Log failed verification attempts with source IP (but not the body)

### 3.2 Replay Prevention

**Control:** Timestamp-based replay prevention.

1. Sender includes `X-Converge-Timestamp` header (Unix epoch seconds)
2. Receiver rejects requests older than 5 minutes (300 seconds)
3. HMAC is computed over `timestamp.body` (timestamp concatenated with body)

```rust
pub fn verify_webhook_with_replay_prevention(
    secret: &[u8],
    body: &[u8],
    signature_header: &str,
    timestamp_header: &str,
    max_age_seconds: u64,
) -> Result<(), WebhookError> {
    let timestamp: u64 = timestamp_header.parse()
        .map_err(|_| WebhookError::MalformedTimestamp)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now.saturating_sub(timestamp) > max_age_seconds {
        return Err(WebhookError::TimestampExpired);
    }

    // HMAC over "timestamp.body"
    let mut payload = timestamp_header.as_bytes().to_vec();
    payload.push(b'.');
    payload.extend_from_slice(body);

    verify_webhook_signature(secret, &payload, signature_header)
}
```

### 3.3 Payload Schema Validation

**Control:** Reject webhooks with unexpected fields or malformed structure.

- Define a strict JSON schema per webhook event type
- Reject unknown fields (do not silently ignore)
- Maximum payload size: 1 MB
- Content-Type must be `application/json`
- Reject payloads that fail schema validation with 400 (not 500)

```rust
pub fn validate_webhook_payload(
    body: &[u8],
    event_type: &str,
) -> Result<serde_json::Value, WebhookError> {
    if body.len() > MAX_WEBHOOK_PAYLOAD_SIZE {
        return Err(WebhookError::PayloadTooLarge);
    }

    let value: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| WebhookError::MalformedPayload(e.to_string()))?;

    // Schema validation per event type
    let schema = get_schema_for_event(event_type)?;
    schema.validate(&value)
        .map_err(|e| WebhookError::SchemaViolation(e.to_string()))?;

    Ok(value)
}

const MAX_WEBHOOK_PAYLOAD_SIZE: usize = 1_048_576; // 1 MB
```

### 3.4 Rate Limiting

- Per-workspace rate limit: 100 requests/minute (configurable in workspace.toml)
- Per-IP rate limit: 1000 requests/minute (defense against DDoS)
- Return `429 Too Many Requests` with `Retry-After` header

## 4. Outbound API Security

### 4.1 TLS Requirements

All outbound API calls MUST use TLS 1.2 or higher.

**Enforcement:**
- HTTP client configured with `min_tls_version = TLS_1_2`
- Certificate validation enabled (no `DANGER_ACCEPT_INVALID_CERTS`)
- Certificate pinning optional per integration (configurable)

```rust
let client = reqwest::Client::builder()
    .min_tls_version(reqwest::tls::Version::TLS_1_2)
    .build()?;
```

### 4.2 Outbound Authentication

Converge → external API calls use per-integration credentials:

| Auth Method | Storage | Rotation |
|-------------|---------|----------|
| API key (header) | Secret Manager | Per pilot engagement |
| OAuth 2.0 bearer | Secret Manager (refresh token) | Auto-refresh, rotate per engagement |
| HMAC signing | Secret Manager | Per pilot engagement |

**No credentials in:**
- Source code
- Config files (.toml, .json, .yaml)
- Environment variables
- Log output

### 4.3 Outbound Data Minimization

- Only send the minimum data required by the integration
- No raw convergence state in outbound payloads — only structured results
- Customer PII is never sent outbound unless explicitly configured and consented
- Outbound payloads logged (without secrets) for audit trail

## 5. Zapier / iPaaS Integration

### 5.1 OAuth Token Management

- Zapier OAuth tokens stored in Secret Manager: `sm://converge/pilots/{customer-slug}/zapier-oauth`
- Token refresh automated; refresh token never logged
- Token scope limited to minimum required permissions
- Separate token per customer workspace (no shared tokens)

### 5.2 Zapier Trigger Security

- Zapier polling triggers: authenticated endpoint with per-customer API key
- Zapier webhook triggers: same HMAC verification as Section 3.1
- Zapier action calls: validated against expected schema before processing

## 6. Secret Management

All integration secrets follow the SecretProvider trait (REF-7):

| Secret | Storage Path | Rotation |
|--------|-------------|----------|
| Webhook HMAC key | `sm://converge/pilots/{slug}/webhook-hmac` | Per pilot engagement |
| Outbound API key | `sm://converge/pilots/{slug}/integration-{name}-key` | Per pilot engagement |
| OAuth refresh token | `sm://converge/pilots/{slug}/integration-{name}-oauth` | Auto-refresh |
| Zapier OAuth token | `sm://converge/pilots/{slug}/zapier-oauth` | Auto-refresh |

**Key generation:**
- HMAC keys: 256 bits, cryptographically random (`rand::rngs::OsRng`)
- API keys: 256 bits, hex-encoded, prefixed with `cvg_pilot_`

**Key lifecycle:**
1. Generated during workspace creation (create-workspace.sh)
2. Shared with customer via secure channel (not email)
3. Rotated on customer request or quarterly
4. Destroyed when workspace is disposed (pilot-data-dispose.sh)

## 7. Error Handling

Security-sensitive error responses MUST NOT leak internal details:

| Scenario | HTTP Status | Response Body |
|----------|-------------|---------------|
| Missing signature | 401 | `{"error": "authentication_required"}` |
| Invalid signature | 401 | `{"error": "authentication_failed"}` |
| Expired timestamp | 401 | `{"error": "authentication_failed"}` |
| Schema violation | 400 | `{"error": "invalid_payload", "details": "<schema error>"}` |
| Payload too large | 413 | `{"error": "payload_too_large"}` |
| Rate limited | 429 | `{"error": "rate_limited", "retry_after": <seconds>}` |
| Internal error | 500 | `{"error": "internal_error"}` |

**Never return:** stack traces, secret values, internal paths, database errors, or agent identity details in error responses.

## 8. Logging & Monitoring

### 8.1 What to Log

| Event | Log Level | Details |
|-------|-----------|---------|
| Successful webhook received | Info | event_type, source_ip, workspace, timestamp |
| Failed signature verification | Warning | source_ip, workspace, failure_reason |
| Schema validation failure | Warning | event_type, workspace, violation |
| Rate limit exceeded | Warning | source_ip, workspace, rate |
| Outbound API call | Info | destination, workspace, status_code |
| Secret rotation | Info | secret_path (not value), workspace |

### 8.2 What to NEVER Log

- Webhook body contents (may contain customer data)
- HMAC secrets or API keys
- OAuth tokens (access or refresh)
- Raw error stack traces from customer-facing endpoints

## 9. Test Plan

### T1: HMAC Verification

```
Given: webhook with valid HMAC signature
When: signature is verified
Then: request is accepted (200)

Given: webhook with invalid HMAC signature
When: signature is verified
Then: request is rejected (401)
And: failed attempt is logged with source_ip
```

### T2: Replay Prevention

```
Given: webhook with timestamp older than 5 minutes
When: timestamp is checked
Then: request is rejected (401) with "authentication_failed"
```

### T3: Schema Validation

```
Given: webhook with unexpected fields
When: payload is validated
Then: request is rejected (400) with schema violation details
```

### T4: TLS Enforcement

```
Given: outbound API call configured
When: destination only supports TLS 1.0/1.1
Then: connection is refused
And: error logged with destination
```

### T5: Secret Isolation

```
Given: workspaces for customer-a and customer-b
When: webhook arrives for customer-a
Then: only customer-a's HMAC secret is used for verification
And: customer-b's secret is never accessed
```

### T6: Rate Limiting

```
Given: rate limit of 100 req/min per workspace
When: 101st request arrives within 1 minute
Then: 429 returned with Retry-After header
And: previous 100 requests were all processed
```

## 10. Implementation Checklist (cw-4)

- [ ] HMAC verification middleware (Rust, `hmac` + `subtle` crates)
- [ ] Replay prevention middleware (timestamp check)
- [ ] Payload schema validation (per event type)
- [ ] Rate limiting (per-workspace + per-IP)
- [ ] TLS 1.2+ enforcement on outbound client
- [ ] Secret Manager integration for webhook keys
- [ ] Error response sanitization (no internal details)
- [ ] Logging configuration (Section 8)
- [ ] Integration tests (T1-T6)
- [ ] Security review before first pilot webhook is configured

## 11. Dependencies

| Dependency | Status | Impact |
|------------|--------|--------|
| REF-7: SecretProvider trait | **Done** | Used for all integration secret storage |
| REF-15: Data isolation architecture | **In review** | Per-workspace secret paths |
| Webhook framework (Eli + Leo, cw-4) | **Planned** | This spec defines security layer for that framework |
| `hmac`, `sha2`, `subtle` crates | Available | Core crypto dependencies |

---

**Review requested from:** Ren Akiyama (architecture approval), Eli Marsh (integration with webhook framework), Dex Tanaka (TLS/infra)
