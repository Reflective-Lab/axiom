// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Billing module — payment capability adapter for Converge Runtime.
//!
//! Implements Stripe's Agentic Commerce Protocol (ACP) for two use cases:
//! 1. **Agent payments**: AI agents making payments on behalf of users with
//!    Shared Payment Tokens, scoped authorization, and human-in-the-loop consent.
//! 2. **SaaS billing**: Getting paid for Converge itself via payment links
//!    and subscription management.
//!
//! This module is feature-gated behind the `billing` feature flag and follows
//! existing converge-runtime patterns: Axum handlers, `RuntimeError` integration,
//! and environment-based configuration.
//!
//! # Architecture
//!
//! - **`types`**: Stripe ACP types (checkout sessions, line items, webhooks)
//! - **`client`**: HTTP client wrapping `reqwest` for Stripe API calls
//! - **`webhook`**: HMAC-SHA256 signature verification and event dispatch
//! - **`handlers`**: Axum HTTP handlers mounted under `/api/v1/billing/`

pub mod client;
pub mod handlers;
pub mod types;
pub mod webhook;

#[cfg(all(feature = "billing", feature = "gcp"))]
pub mod ledger;

// Re-exports for convenience
pub use client::StripeClient;
pub use handlers::billing_router;
pub use types::{
    BillingError, CheckoutSession, CheckoutStatus, CompleteCheckoutRequest, CreateCheckoutRequest,
    Customer, LineItem, MeterEvent, MeterEventResponse, PaymentLink, UpdateCheckoutRequest,
    WebhookEvent,
};
pub use webhook::{WebhookDispatcher, verify_webhook_signature};

#[cfg(all(feature = "billing", feature = "gcp"))]
pub use ledger::CreditLedger;
#[cfg(all(feature = "billing", feature = "gcp"))]
pub use types::{CreditBalance, CreditTransaction, CreditTransactionKind};
