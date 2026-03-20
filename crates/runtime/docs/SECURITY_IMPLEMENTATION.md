# Converge Security Implementation Specification

## Overview

Implement Zero Trust security across the Converge runtime with:
- mTLS for all service communication (gRPC + NATS)
- SPIFFE/SPIRE for service identity
- Passkey authentication for users → JWT for API access
- Field-level E2E encryption for sensitive data

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                         Web Client                                │
│  Passkey Auth → JWT → gRPC-Web calls with Authorization header  │
└─────────────────────────────┬────────────────────────────────────┘
                              │ HTTPS + JWT
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│                      API Gateway / Edge                           │
│  - Terminates TLS                                                 │
│  - Validates JWT (user identity)                                  │
│  - Issues internal mTLS connection to services                    │
└─────────────────────────────┬────────────────────────────────────┘
                              │ mTLS (SPIFFE SVID)
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│                     gRPC Services (Rust/Tonic)                    │
│  - AuthInterceptor validates: service identity + user JWT        │
│  - PolicyEngine checks authorization per method                   │
│  - Encrypted fields decrypted only at authorized boundaries      │
└─────────────────────────────┬────────────────────────────────────┘
                              │ mTLS + NKey
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│                           NATS                                    │
│  - TLS required for all connections                               │
│  - Client cert identity → subject permissions                     │
│  - JetStream for durable audit log                               │
└──────────────────────────────────────────────────────────────────┘
```

---

## 1. Service Identity (SPIFFE/SPIRE)

### Dependencies
```toml
[dependencies]
spiffe = "0.6"
```

### Service Identity Format
```
spiffe://converge.zone/service/<service-name>
```

Examples:
- `spiffe://converge.zone/service/pricing`
- `spiffe://converge.zone/service/inventory`
- `spiffe://converge.zone/service/gateway`

### Implementation

```rust
// src/identity/spiffe.rs
use spiffe::{WorkloadApiClient, X509Svid};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ServiceIdentity {
    svid: Arc<RwLock<X509Svid>>,
    client: WorkloadApiClient,
}

impl ServiceIdentity {
    pub async fn new() -> Result<Self, IdentityError> {
        let client = WorkloadApiClient::default().await?;
        let svid = client.fetch_x509_svid().await?;

        let identity = Self {
            svid: Arc::new(RwLock::new(svid)),
            client,
        };

        // Start background rotation
        identity.start_rotation();

        Ok(identity)
    }

    pub async fn spiffe_id(&self) -> String {
        self.svid.read().await.spiffe_id().to_string()
    }

    pub async fn tls_config(&self) -> rustls::ClientConfig {
        let svid = self.svid.read().await;
        // Build rustls config with SVID cert + key
        build_mtls_config(&svid)
    }

    fn start_rotation(&self) {
        let svid = self.svid.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
                if let Ok(new_svid) = client.fetch_x509_svid().await {
                    *svid.write().await = new_svid;
                }
            }
        });
    }
}
```

---

## 2. gRPC mTLS + Auth Interceptor

### Dependencies
```toml
[dependencies]
tonic = { version = "0.12", features = ["tls"] }
rustls = "0.23"
jsonwebtoken = "9"
```

### Server Setup with mTLS

```rust
// src/server/grpc.rs
use tonic::transport::{Server, ServerTlsConfig, Identity, Certificate};

pub async fn start_grpc_server(
    identity: &ServiceIdentity,
    config: &ServerConfig,
) -> Result<(), ServerError> {
    let svid = identity.svid.read().await;

    let tls_config = ServerTlsConfig::new()
        .identity(Identity::from_pem(
            svid.cert_chain_pem(),
            svid.private_key_pem(),
        ))
        .client_ca_root(Certificate::from_pem(svid.bundle_pem()))
        .client_auth_optional(false); // Require client certs

    let auth_interceptor = AuthInterceptor::new(
        config.jwt_secret.clone(),
        config.policy_engine.clone(),
    );

    Server::builder()
        .tls_config(tls_config)?
        .add_service(MyServiceServer::with_interceptor(
            MyServiceImpl::new(),
            auth_interceptor,
        ))
        .serve(config.addr)
        .await?;

    Ok(())
}
```

### Auth Interceptor

```rust
// src/interceptor/auth.rs
use tonic::{Request, Status, service::Interceptor};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

#[derive(Clone)]
pub struct AuthInterceptor {
    jwt_secret: DecodingKey,
    policy_engine: Arc<PolicyEngine>,
}

#[derive(Debug, Clone)]
pub struct VerifiedIdentity {
    pub service_id: String,      // From mTLS cert (SPIFFE ID)
    pub user_id: Option<String>, // From JWT sub claim
    pub permissions: Vec<String>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // 1. Extract service identity from mTLS peer cert
        let peer_certs = request.peer_certs()
            .ok_or_else(|| Status::unauthenticated("no client certificate"))?;

        let service_id = extract_spiffe_id(&peer_certs[0])
            .map_err(|_| Status::unauthenticated("invalid SPIFFE ID"))?;

        // 2. Extract user identity from JWT (if present)
        let user_claims = request.metadata()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .and_then(|token| self.validate_jwt(token).ok());

        // 3. Check policy
        let method = request.uri().path();
        let decision = self.policy_engine.evaluate(PolicyRequest {
            service_id: &service_id,
            user_id: user_claims.as_ref().map(|c| c.sub.as_str()),
            method,
            user_permissions: user_claims.as_ref()
                .map(|c| c.permissions.as_slice())
                .unwrap_or(&[]),
        });

        if !decision.allowed {
            return Err(Status::permission_denied(format!(
                "policy denied: {}",
                decision.reason
            )));
        }

        // 4. Attach verified identity
        request.extensions_mut().insert(VerifiedIdentity {
            service_id,
            user_id: user_claims.map(|c| c.sub),
            permissions: decision.granted_permissions,
        });

        Ok(request)
    }
}

impl AuthInterceptor {
    fn validate_jwt(&self, token: &str) -> Result<JwtClaims, JwtError> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<JwtClaims>(token, &self.jwt_secret, &validation)?;
        Ok(token_data.claims)
    }
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
    exp: i64,
    permissions: Vec<String>,
}

fn extract_spiffe_id(cert: &Certificate) -> Result<String, CertError> {
    // Parse X.509, extract SAN URI with spiffe:// prefix
    let x509 = x509_parser::parse_x509_certificate(cert.as_ref())?.1;

    for san in x509.subject_alternative_name()?.value.general_names.iter() {
        if let GeneralName::UniformResourceIdentifier(uri) = san {
            if uri.starts_with("spiffe://") {
                return Ok(uri.to_string());
            }
        }
    }

    Err(CertError::NoSpiffeId)
}
```

---

## 3. Policy Engine

### Simple Policy Definition

```rust
// src/policy/engine.rs

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone)]
struct PolicyRule {
    service_pattern: String,      // e.g., "spiffe://converge.zone/service/gateway"
    method_pattern: String,       // e.g., "/converge.pricing.v1.PricingService/*"
    requires_user: bool,
    required_permissions: Vec<String>,
}

pub struct PolicyRequest<'a> {
    pub service_id: &'a str,
    pub user_id: Option<&'a str>,
    pub method: &'a str,
    pub user_permissions: &'a [String],
}

pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: String,
    pub granted_permissions: Vec<String>,
}

impl PolicyEngine {
    pub fn evaluate(&self, req: PolicyRequest) -> PolicyDecision {
        for rule in &self.rules {
            if !matches_pattern(&rule.service_pattern, req.service_id) {
                continue;
            }
            if !matches_pattern(&rule.method_pattern, req.method) {
                continue;
            }

            // Found matching rule
            if rule.requires_user && req.user_id.is_none() {
                return PolicyDecision {
                    allowed: false,
                    reason: "user authentication required".into(),
                    granted_permissions: vec![],
                };
            }

            // Check required permissions
            let has_permissions = rule.required_permissions.iter()
                .all(|p| req.user_permissions.contains(p));

            if !has_permissions {
                return PolicyDecision {
                    allowed: false,
                    reason: format!(
                        "missing permissions: {:?}",
                        rule.required_permissions
                    ),
                    granted_permissions: vec![],
                };
            }

            return PolicyDecision {
                allowed: true,
                reason: "policy matched".into(),
                granted_permissions: req.user_permissions.to_vec(),
            };
        }

        // Default deny
        PolicyDecision {
            allowed: false,
            reason: "no matching policy rule".into(),
            granted_permissions: vec![],
        }
    }
}
```

### Policy Configuration (YAML)

```yaml
# config/policy.yaml
rules:
  # Gateway can call any service
  - service: "spiffe://converge.zone/service/gateway"
    method: "*"
    requires_user: false

  # Pricing service can only be called by gateway, requires user
  - service: "spiffe://converge.zone/service/gateway"
    method: "/converge.pricing.v1.PricingService/*"
    requires_user: true
    permissions: ["pricing:read"]

  # Internal service-to-service (no user required)
  - service: "spiffe://converge.zone/service/inventory"
    method: "/converge.warehouse.v1.WarehouseService/CheckStock"
    requires_user: false
```

---

## 4. NATS with mTLS

### Dependencies
```toml
[dependencies]
async-nats = "0.35"
```

### Secure NATS Client

```rust
// src/nats/client.rs
use async_nats::{Client, ConnectOptions};
use rustls::ClientConfig;

pub async fn connect_nats(
    identity: &ServiceIdentity,
    config: &NatsConfig,
) -> Result<Client, NatsError> {
    let svid = identity.svid.read().await;

    let tls_config = ClientConfig::builder()
        .with_root_certificates(load_root_certs(&svid.bundle_pem())?)
        .with_client_auth_cert(
            load_certs(&svid.cert_chain_pem())?,
            load_private_key(&svid.private_key_pem())?,
        )?;

    let client = ConnectOptions::new()
        .require_tls(true)
        .tls_client_config(tls_config)
        .connect(&config.servers)
        .await?;

    Ok(client)
}
```

### NATS Server Authorization Config

```conf
# nats-server.conf
authorization {
  users = [
    {
      # Pricing service identity
      user: "spiffe://converge.zone/service/pricing"
      permissions: {
        publish: ["pricing.>", "audit.pricing.>"]
        subscribe: ["pricing.requests.>", "_INBOX.>"]
      }
    }
    {
      # Inventory service identity
      user: "spiffe://converge.zone/service/inventory"
      permissions: {
        publish: ["inventory.>", "audit.inventory.>"]
        subscribe: ["inventory.requests.>", "pricing.events.>", "_INBOX.>"]
      }
    }
    {
      # Gateway - broader access
      user: "spiffe://converge.zone/service/gateway"
      permissions: {
        publish: ["*.requests.>"]
        subscribe: ["*.responses.>", "_INBOX.>"]
      }
    }
  ]
}
```

---

## 5. Field-Level E2E Encryption

### Dependencies
```toml
[dependencies]
chacha20poly1305 = "0.10"
rand = "0.8"
```

### Encryption Module

```rust
// src/crypto/field_encryption.rs
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encryption failed")]
    EncryptionFailed,
    #[error("decryption failed")]
    DecryptionFailed,
    #[error("invalid ciphertext")]
    InvalidCiphertext,
}

/// Encrypt sensitive data before storage or transmission
pub fn encrypt_field(plaintext: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(key.into());

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend(ciphertext);

    Ok(result)
}

/// Decrypt field - only at authorized service boundaries
pub fn decrypt_field(encrypted: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, CryptoError> {
    if encrypted.len() < NONCE_SIZE {
        return Err(CryptoError::InvalidCiphertext);
    }

    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)
}
```

### Key Management Interface

```rust
// src/crypto/key_store.rs

#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Get encryption key for a customer
    async fn get_key(&self, customer_id: &str) -> Result<[u8; 32], KeyError>;

    /// Rotate key (re-encrypt existing data required)
    async fn rotate_key(&self, customer_id: &str) -> Result<[u8; 32], KeyError>;
}

/// Cloud KMS-backed key store (production)
pub struct CloudKmsKeyStore {
    kms_client: CloudKmsClient,
    key_ring: String,
}

#[async_trait]
impl KeyStore for CloudKmsKeyStore {
    async fn get_key(&self, customer_id: &str) -> Result<[u8; 32], KeyError> {
        // Key ID derived from customer ID
        let key_name = format!("{}/cryptoKeys/{}", self.key_ring, customer_id);

        // Use KMS to decrypt the data encryption key (DEK)
        // DEK is stored encrypted alongside customer data
        let encrypted_dek = self.get_encrypted_dek(customer_id).await?;
        let dek = self.kms_client.decrypt(&key_name, &encrypted_dek).await?;

        let mut key = [0u8; 32];
        key.copy_from_slice(&dek);
        Ok(key)
    }
}
```

### Usage in Service Handler

```rust
// src/service/handler.rs
use crate::crypto::{encrypt_field, decrypt_field};
use crate::interceptor::VerifiedIdentity;

impl MyService for MyServiceImpl {
    async fn process_sensitive_data(
        &self,
        request: Request<SensitiveRequest>,
    ) -> Result<Response<SensitiveResponse>, Status> {
        // Get verified identity from interceptor
        let identity = request.extensions()
            .get::<VerifiedIdentity>()
            .ok_or_else(|| Status::internal("missing identity"))?;

        let req = request.into_inner();

        // Get customer's encryption key
        let key = self.key_store
            .get_key(&req.customer_id)
            .await
            .map_err(|e| Status::internal(format!("key error: {e}")))?;

        // Decrypt incoming encrypted field
        let plaintext = decrypt_field(&req.encrypted_payload, &key)
            .map_err(|_| Status::invalid_argument("decryption failed"))?;

        // Process...
        let result = self.processor.handle(&plaintext).await?;

        // Re-encrypt for response
        let encrypted_result = encrypt_field(&result, &key)
            .map_err(|_| Status::internal("encryption failed"))?;

        // Audit log (encrypted data never logged in plaintext)
        self.audit.log(AuditEvent {
            service_id: &identity.service_id,
            user_id: identity.user_id.as_deref(),
            action: "process_sensitive_data",
            customer_id: &req.customer_id,
            success: true,
        }).await;

        Ok(Response::new(SensitiveResponse {
            encrypted_result,
        }))
    }
}
```

---

## 6. Passkey Authentication (Auth Service)

### Dependencies
```toml
[dependencies]
webauthn-rs = "0.5"
jsonwebtoken = "9"
```

### Passkey Registration + Authentication

```rust
// src/auth/passkey.rs
use webauthn_rs::prelude::*;

pub struct PasskeyAuth {
    webauthn: Webauthn,
    credential_store: Arc<dyn CredentialStore>,
    jwt_secret: [u8; 32],
}

impl PasskeyAuth {
    pub fn new(config: &AuthConfig) -> Result<Self, AuthError> {
        let rp_id = "converge.zone";
        let rp_origin = Url::parse("https://converge.zone")?;

        let webauthn = WebauthnBuilder::new(rp_id, &rp_origin)?
            .rp_name("Converge")
            .build()?;

        Ok(Self {
            webauthn,
            credential_store: config.credential_store.clone(),
            jwt_secret: config.jwt_secret,
        })
    }

    /// Start authentication - returns challenge for client
    pub async fn start_authentication(
        &self,
        user_id: &str,
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication), AuthError> {
        let credentials = self.credential_store
            .get_credentials(user_id)
            .await?;

        let (challenge, auth_state) = self.webauthn
            .start_passkey_authentication(&credentials)?;

        Ok((challenge, auth_state))
    }

    /// Complete authentication - returns JWT for API access
    pub async fn complete_authentication(
        &self,
        user_id: &str,
        response: &PublicKeyCredential,
        auth_state: &PasskeyAuthentication,
    ) -> Result<AuthToken, AuthError> {
        let auth_result = self.webauthn
            .finish_passkey_authentication(response, auth_state)?;

        // Update credential counter (replay protection)
        self.credential_store
            .update_counter(user_id, &auth_result.cred_id, auth_result.counter)
            .await?;

        // Load user permissions
        let permissions = self.credential_store
            .get_permissions(user_id)
            .await?;

        // Issue JWT
        let token = self.issue_jwt(user_id, &permissions)?;

        Ok(token)
    }

    fn issue_jwt(&self, user_id: &str, permissions: &[String]) -> Result<AuthToken, AuthError> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(1);

        let claims = JwtClaims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            permissions: permissions.to_vec(),
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(&self.jwt_secret),
        )?;

        Ok(AuthToken {
            token,
            expires_at: exp.timestamp(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: i64,
}
```

---

## 7. Audit Logging via NATS JetStream

```rust
// src/audit/logger.rs
use async_nats::jetstream::{self, Context};
use serde::Serialize;

pub struct AuditLogger {
    js: Context,
    stream: String,
}

#[derive(Debug, Serialize)]
pub struct AuditEvent<'a> {
    pub timestamp: i64,
    pub service_id: &'a str,
    pub user_id: Option<&'a str>,
    pub action: &'a str,
    pub resource: &'a str,
    pub success: bool,
    pub metadata: serde_json::Value,
}

impl AuditLogger {
    pub async fn new(nats: &Client) -> Result<Self, AuditError> {
        let js = jetstream::new(nats.clone());

        // Ensure stream exists
        js.get_or_create_stream(jetstream::stream::Config {
            name: "AUDIT".into(),
            subjects: vec!["audit.>".into()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            max_age: Duration::from_secs(90 * 24 * 60 * 60), // 90 days
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        }).await?;

        Ok(Self { js, stream: "AUDIT".into() })
    }

    pub async fn log(&self, event: AuditEvent<'_>) -> Result<(), AuditError> {
        let subject = format!("audit.{}.{}", event.service_id, event.action);
        let payload = serde_json::to_vec(&event)?;

        self.js.publish(subject, payload.into()).await?;

        Ok(())
    }
}
```

---

## Implementation Order

1. **Service Identity** - SPIFFE/SPIRE integration, cert loading
2. **gRPC mTLS** - Server + client TLS config with mutual auth
3. **Auth Interceptor** - Extract identities, basic policy check
4. **Policy Engine** - Load rules, evaluate requests
5. **NATS mTLS** - Secure NATS client, subject permissions
6. **Field Encryption** - ChaCha20-Poly1305, key store interface
7. **Passkey Auth** - WebAuthn registration/authentication, JWT issuance
8. **Audit Logging** - JetStream stream, structured events

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_encryption_roundtrip() {
        let key = [0u8; 32]; // Use random in real tests
        let plaintext = b"sensitive customer data";

        let encrypted = encrypt_field(plaintext, &key).unwrap();
        let decrypted = decrypt_field(&encrypted, &key).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_policy_deny_without_user() {
        let engine = PolicyEngine::from_config(/* ... */);

        let decision = engine.evaluate(PolicyRequest {
            service_id: "spiffe://converge.zone/service/gateway",
            user_id: None,
            method: "/converge.pricing.v1.PricingService/GetPrice",
            user_permissions: &[],
        });

        assert!(!decision.allowed);
    }
}
```
