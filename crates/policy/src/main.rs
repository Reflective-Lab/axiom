//! converge-policy HTTP server
//!
//! Thin Axum shell over the policy engine library.
//! Endpoints:
//!   POST /decide           — policy or delegation decision
//!   POST /issue-delegation — issue a scoped authority token
//!   GET  /pubkey           — Ed25519 public key for delegation verification

use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose};
use converge_policy::{
    PolicyEngine,
    decision::{DecisionMode, PolicyDecision, PolicyOutcome},
    delegation,
    types::DecideRequest,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::Serialize;
use std::{fs, net::SocketAddr, sync::Arc};
use thiserror::Error;
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Error)]
enum ServerError {
    #[error("policy: {0}")]
    Policy(String),
    #[error("delegation: {0}")]
    Delegation(String),
    #[error("engine: {0}")]
    Engine(String),
    #[error("server: {0}")]
    #[allow(dead_code)]
    Server(String),
    #[cfg(feature = "redis-tracking")]
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match &self {
            ServerError::Delegation(_) => StatusCode::BAD_REQUEST,
            #[cfg(feature = "redis-tracking")]
            ServerError::Redis(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(serde_json::json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}

// -------------------- HTTP response types --------------------

#[derive(Debug, Serialize)]
struct DecideResp {
    outcome: PolicyOutcome,
    reason: Option<String>,
    mode: DecisionMode,
}

impl From<PolicyDecision> for DecideResp {
    fn from(d: PolicyDecision) -> Self {
        Self {
            outcome: d.outcome,
            reason: d.reason,
            mode: d.mode,
        }
    }
}

#[derive(Debug, Serialize)]
struct PubKeyResp {
    pubkey_b64: String,
}

// -------------------- App State --------------------

#[derive(Clone)]
struct AppState {
    engine: Arc<PolicyEngine>,
    signing_key: Arc<SigningKey>,
    verifying_key: Arc<VerifyingKey>,
    #[cfg(feature = "redis-tracking")]
    redis_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::new();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("logging already initialized: {err}");
    }

    let policy_text = fs::read_to_string("policies/policy.cedar")?;
    let engine = PolicyEngine::from_policy_str(&policy_text)
        .map_err(|err| ServerError::Policy(err.to_string()))?;

    let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let verifying_key = signing_key.verifying_key();

    let state = AppState {
        engine: Arc::new(engine),
        signing_key: Arc::new(signing_key),
        verifying_key: Arc::new(verifying_key),
        #[cfg(feature = "redis-tracking")]
        redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into()),
    };

    let app = Router::new()
        .route("/decide", post(decide))
        .route("/issue-delegation", post(issue_delegation))
        .route("/pubkey", get(pubkey))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    info!(%addr, "converge-policy listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// -------------------- Endpoints --------------------

async fn decide(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(input): Json<DecideRequest>,
) -> Result<Json<DecideResp>, ServerError> {
    let decision = if let Some(del_b64) = input.delegation_b64.as_ref() {
        // Delegation fast path
        let result = delegation::verify(del_b64, &state.verifying_key, &input);
        let (outcome, reason) = match result {
            Ok(true) => (PolicyOutcome::Promote, None),
            Ok(false) => (
                PolicyOutcome::Reject,
                Some("delegation check failed".into()),
            ),
            Err(reason) => (PolicyOutcome::Reject, Some(reason)),
        };
        PolicyDecision::delegation(
            outcome,
            reason,
            input.principal.id.clone(),
            input.action.clone(),
            input.resource.id.clone(),
        )
    } else {
        // Cedar policy path
        state
            .engine
            .evaluate(&input)
            .map_err(|err| ServerError::Engine(err.to_string()))?
    };

    #[cfg(feature = "redis-tracking")]
    if input.observe.unwrap_or(false) {
        if let Err(err) = record_decision(&state.redis_url, &decision).await {
            warn!(error = %err, "failed to record decision");
        }
    }

    Ok(Json(decision.into()))
}

async fn issue_delegation(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<delegation::IssueDelegationReq>,
) -> Result<Json<delegation::IssueDelegationResp>, ServerError> {
    let resp = delegation::issue(&state.signing_key, req).map_err(ServerError::Delegation)?;
    Ok(Json(resp))
}

async fn pubkey(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<PubKeyResp>, ServerError> {
    let pub_b64 = general_purpose::STANDARD_NO_PAD.encode(state.verifying_key.to_bytes());
    Ok(Json(PubKeyResp {
        pubkey_b64: pub_b64,
    }))
}

// -------------------- Redis tracking --------------------

#[cfg(feature = "redis-tracking")]
async fn record_decision(redis_url: &str, decision: &PolicyDecision) -> Result<(), ServerError> {
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("last:agent:{}", decision.principal_id);
    redis::pipe()
        .hset(&key, "resource", &decision.resource_id)
        .hset(&key, "action", &decision.action)
        .hset(&key, "outcome", format!("{:?}", decision.outcome))
        .hset(
            &key,
            "ts",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        )
        .expire(&key, 3600)
        .query_async::<_, ()>(&mut con)
        .await?;
    Ok(())
}
