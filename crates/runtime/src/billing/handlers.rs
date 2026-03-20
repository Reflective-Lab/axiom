// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Axum HTTP handlers for the billing module.
//!
//! All endpoints are mounted under `/api/v1/billing/`.

use axum::{
    Router,
    body::Bytes,
    extract::{Json, Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use tracing::{info, warn};

use super::types::{
    CheckoutSession, CompleteCheckoutRequest, CreateCheckoutRequest, UpdateCheckoutRequest,
    WebhookEvent,
};
use super::webhook::{WebhookDispatcher, verify_webhook_signature};
use crate::error::RuntimeError;
use crate::state::AppState;

/// Create a new checkout session.
#[utoipa::path(
    post,
    path = "/api/v1/billing/checkouts",
    tag = "billing",
    request_body = CreateCheckoutRequest,
    responses(
        (status = 201, description = "Checkout session created", body = CheckoutSession),
        (status = 400, description = "Invalid request", body = crate::error::RuntimeErrorResponse),
        (status = 402, description = "Payment required", body = crate::error::RuntimeErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
pub async fn create_checkout_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateCheckoutRequest>,
) -> Result<(axum::http::StatusCode, Json<CheckoutSession>), RuntimeError> {
    let billing = state
        .billing_client()
        .ok_or_else(|| RuntimeError::Config("Billing not configured".to_string()))?;

    info!(
        items = request.line_items.len(),
        currency = %request.currency,
        "Creating checkout session"
    );

    let session = billing.create_checkout(&request).await.map_err(|e| {
        warn!(error = %e, "Failed to create checkout session");
        RuntimeError::from(e)
    })?;

    info!(session_id = %session.id, "Checkout session created");
    Ok((axum::http::StatusCode::CREATED, Json(session)))
}

/// Get a checkout session by ID.
#[utoipa::path(
    get,
    path = "/api/v1/billing/checkouts/{id}",
    tag = "billing",
    params(
        ("id" = String, Path, description = "Checkout session ID")
    ),
    responses(
        (status = 200, description = "Checkout session found", body = CheckoutSession),
        (status = 404, description = "Not found", body = crate::error::RuntimeErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
pub async fn get_checkout_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CheckoutSession>, RuntimeError> {
    let billing = state
        .billing_client()
        .ok_or_else(|| RuntimeError::Config("Billing not configured".to_string()))?;

    let session = billing.get_checkout(&id).await.map_err(|e| {
        warn!(session_id = %id, error = %e, "Failed to get checkout session");
        RuntimeError::from(e)
    })?;

    Ok(Json(session))
}

/// Update a checkout session.
#[utoipa::path(
    put,
    path = "/api/v1/billing/checkouts/{id}",
    tag = "billing",
    params(
        ("id" = String, Path, description = "Checkout session ID")
    ),
    request_body = UpdateCheckoutRequest,
    responses(
        (status = 200, description = "Checkout session updated", body = CheckoutSession),
        (status = 400, description = "Invalid request", body = crate::error::RuntimeErrorResponse),
        (status = 404, description = "Not found", body = crate::error::RuntimeErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
pub async fn update_checkout_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateCheckoutRequest>,
) -> Result<Json<CheckoutSession>, RuntimeError> {
    let billing = state
        .billing_client()
        .ok_or_else(|| RuntimeError::Config("Billing not configured".to_string()))?;

    let session = billing.update_checkout(&id, &request).await.map_err(|e| {
        warn!(session_id = %id, error = %e, "Failed to update checkout session");
        RuntimeError::from(e)
    })?;

    info!(session_id = %id, "Checkout session updated");
    Ok(Json(session))
}

/// Complete a checkout session with a shared payment token.
#[utoipa::path(
    post,
    path = "/api/v1/billing/checkouts/{id}/complete",
    tag = "billing",
    params(
        ("id" = String, Path, description = "Checkout session ID")
    ),
    request_body = CompleteCheckoutRequest,
    responses(
        (status = 200, description = "Checkout completed", body = CheckoutSession),
        (status = 400, description = "Invalid request", body = crate::error::RuntimeErrorResponse),
        (status = 402, description = "Payment failed", body = crate::error::RuntimeErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
pub async fn complete_checkout_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CompleteCheckoutRequest>,
) -> Result<Json<CheckoutSession>, RuntimeError> {
    let billing = state
        .billing_client()
        .ok_or_else(|| RuntimeError::Config("Billing not configured".to_string()))?;

    info!(session_id = %id, "Completing checkout session");

    let session = billing
        .complete_checkout(&id, &request)
        .await
        .map_err(|e| {
            warn!(session_id = %id, error = %e, "Failed to complete checkout");
            RuntimeError::from(e)
        })?;

    info!(session_id = %id, status = ?session.status, "Checkout completed");
    Ok(Json(session))
}

/// Cancel a checkout session.
#[utoipa::path(
    post,
    path = "/api/v1/billing/checkouts/{id}/cancel",
    tag = "billing",
    params(
        ("id" = String, Path, description = "Checkout session ID")
    ),
    responses(
        (status = 200, description = "Checkout canceled", body = CheckoutSession),
        (status = 404, description = "Not found", body = crate::error::RuntimeErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
pub async fn cancel_checkout_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CheckoutSession>, RuntimeError> {
    let billing = state
        .billing_client()
        .ok_or_else(|| RuntimeError::Config("Billing not configured".to_string()))?;

    info!(session_id = %id, "Canceling checkout session");

    let session = billing.cancel_checkout(&id).await.map_err(|e| {
        warn!(session_id = %id, error = %e, "Failed to cancel checkout");
        RuntimeError::from(e)
    })?;

    info!(session_id = %id, "Checkout canceled");
    Ok(Json(session))
}

/// Handle incoming Stripe webhooks.
///
/// Verifies the webhook signature and dispatches the event.
#[utoipa::path(
    post,
    path = "/api/v1/billing/webhooks/stripe",
    tag = "billing",
    responses(
        (status = 200, description = "Webhook processed"),
        (status = 401, description = "Invalid signature", body = crate::error::RuntimeErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
pub async fn stripe_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<axum::http::StatusCode, RuntimeError> {
    let webhook_secret = state
        .billing_webhook_secret()
        .ok_or_else(|| RuntimeError::Config("Webhook secret not configured".to_string()))?;

    let sig_header = headers
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            RuntimeError::Authentication("Missing Stripe-Signature header".to_string())
        })?;

    // Verify signature
    verify_webhook_signature(&body, sig_header, &webhook_secret).map_err(RuntimeError::from)?;

    // Parse event
    let event: WebhookEvent = serde_json::from_slice(&body)
        .map_err(|e| RuntimeError::Billing(format!("Invalid webhook payload: {e}")))?;

    info!(
        event_id = %event.id,
        event_type = %event.event_type,
        "Processing Stripe webhook"
    );

    // Dispatch event
    let dispatcher = WebhookDispatcher::from_state(&state);
    dispatcher.dispatch(&event).await;

    Ok(axum::http::StatusCode::OK)
}

// =============================================================================
// Credit balance endpoints (billing + gcp features)
// =============================================================================

/// Get credit balance for a user.
#[cfg(all(feature = "billing", feature = "gcp"))]
#[utoipa::path(
    get,
    path = "/api/v1/billing/credits/balance",
    tag = "billing",
    params(
        ("user_id" = String, Query, description = "User ID to check balance for")
    ),
    responses(
        (status = 200, description = "Credit balance", body = super::types::CreditBalance),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
#[cfg(all(feature = "billing", feature = "gcp"))]
pub async fn get_balance_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<super::types::CreditBalance>, RuntimeError> {
    let user_id = params
        .get("user_id")
        .ok_or_else(|| RuntimeError::Config("Missing user_id query parameter".to_string()))?;

    let ledger = state
        .credit_ledger()
        .ok_or_else(|| RuntimeError::Config("Credit ledger not configured".to_string()))?;

    let balance = ledger
        .get_balance(user_id)
        .await
        .map_err(|e| RuntimeError::Billing(e.to_string()))?;

    Ok(Json(balance))
}

/// List credit transactions for a user.
#[cfg(all(feature = "billing", feature = "gcp"))]
#[utoipa::path(
    get,
    path = "/api/v1/billing/credits/transactions",
    tag = "billing",
    params(
        ("user_id" = String, Query, description = "User ID"),
        ("limit" = Option<u32>, Query, description = "Max transactions to return (default 50)")
    ),
    responses(
        (status = 200, description = "Credit transactions", body = Vec<super::types::CreditTransaction>),
        (status = 500, description = "Internal server error", body = crate::error::RuntimeErrorResponse)
    )
)]
#[cfg(all(feature = "billing", feature = "gcp"))]
pub async fn list_transactions_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<super::types::CreditTransaction>>, RuntimeError> {
    let user_id = params
        .get("user_id")
        .ok_or_else(|| RuntimeError::Config("Missing user_id query parameter".to_string()))?;

    let limit: u32 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);

    let ledger = state
        .credit_ledger()
        .ok_or_else(|| RuntimeError::Config("Credit ledger not configured".to_string()))?;

    let transactions = ledger
        .list_transactions(user_id, limit)
        .await
        .map_err(|e| RuntimeError::Billing(e.to_string()))?;

    Ok(Json(transactions))
}

/// Build the billing router.
///
/// All routes are under `/api/v1/billing/`.
pub fn billing_router(state: AppState) -> Router<()> {
    let router = Router::new()
        .route("/api/v1/billing/checkouts", post(create_checkout_handler))
        .route(
            "/api/v1/billing/checkouts/:id",
            get(get_checkout_handler).put(update_checkout_handler),
        )
        .route(
            "/api/v1/billing/checkouts/:id/complete",
            post(complete_checkout_handler),
        )
        .route(
            "/api/v1/billing/checkouts/:id/cancel",
            post(cancel_checkout_handler),
        )
        .route(
            "/api/v1/billing/webhooks/stripe",
            post(stripe_webhook_handler),
        );

    #[cfg(all(feature = "billing", feature = "gcp"))]
    let router = router
        .route("/api/v1/billing/credits/balance", get(get_balance_handler))
        .route(
            "/api/v1/billing/credits/transactions",
            get(list_transactions_handler),
        );

    router.with_state(state)
}
