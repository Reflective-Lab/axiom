// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Converge Runtime Server
//!
//! Provides HTTP and gRPC interfaces for the Converge engine.

// Scaffolding code - allow pedantic lints during development
#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::unused_async)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::no_effect_underscore_binding)]

// Use the library crate
use converge_runtime::config::Config;
use converge_runtime::http::HttpServer;
use converge_runtime::state::AppState;

#[cfg(feature = "gcp")]
use converge_runtime::db;
#[cfg(feature = "gcp")]
use converge_runtime::gcp;

#[cfg(feature = "grpc")]
use converge_runtime::grpc;

#[cfg(feature = "metrics")]
use converge_runtime::metrics;

#[cfg(feature = "telemetry")]
use converge_runtime::telemetry;

use anyhow::Result;
use tracing::info;
#[cfg(feature = "gcp")]
use tracing::warn;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Initialize Sentry (must be done before tracing)
    #[cfg(feature = "sentry")]
    let _sentry_guard = init_sentry();

    // Initialize tracing with optional Sentry integration
    init_tracing();

    info!("Starting Converge Runtime");

    // Initialize Prometheus metrics
    #[cfg(feature = "metrics")]
    {
        match metrics::init() {
            Ok(_) => info!("Prometheus metrics initialized"),
            Err(e) => tracing::warn!(error = %e, "Failed to initialize metrics"),
        }
    }

    #[cfg(feature = "sentry")]
    if std::env::var("SENTRY_DSN").is_ok() {
        info!("Sentry error tracking enabled");
    }

    // Load configuration
    let config = Config::load()?;
    info!(?config, "Configuration loaded");

    // Initialize application state
    #[cfg(feature = "gcp")]
    let app_state = {
        let gcp_config = gcp::GcpConfig::from_env();
        info!(
            local_dev = gcp_config.is_local(),
            "GCP configuration loaded"
        );

        match db::Database::new(gcp_config).await {
            Ok(database) => {
                info!("Database connection established");
                AppState::with_database(database)
            }
            Err(e) => {
                warn!(error = %e, "Failed to connect to database, running without persistence");
                AppState::new()
            }
        }
    };

    #[cfg(not(feature = "gcp"))]
    let app_state = AppState::new();

    // Configure billing (Stripe) if available
    #[cfg(feature = "billing")]
    let app_state = if let Some(ref billing_config) = config.billing {
        info!("Billing (Stripe) enabled");
        app_state.with_billing(billing_config)
    } else {
        app_state
    };

    // Configure credit ledger if both billing and database are available
    #[cfg(all(feature = "billing", feature = "gcp"))]
    let app_state = {
        let firestore_db = app_state.db.as_ref().map(|db| db.firestore.db().clone());
        if app_state.billing.is_some() {
            if let Some(fdb) = firestore_db {
                info!("Credit ledger enabled");
                app_state.with_credit_ledger(fdb)
            } else {
                app_state
            }
        } else {
            app_state
        }
    };

    // Start HTTP server (always enabled)
    let http_server = HttpServer::new(config.http.clone(), app_state);
    let http_handle = tokio::spawn(async move {
        if let Err(e) = http_server.start().await {
            tracing::error!(error = %e, "HTTP server failed");
        }
    });

    // TODO: Start gRPC server when grpc feature is enabled
    #[cfg(feature = "grpc")]
    {
        let grpc_config = config.grpc();
        let grpc_server = grpc::GrpcServer::new(grpc_config);
        let grpc_handle = tokio::spawn(async move {
            if let Err(e) = grpc_server.start().await {
                tracing::error!(error = %e, "gRPC server failed");
            }
        });
        tokio::select! {
            _ = http_handle => {},
            _ = grpc_handle => {},
        }
        return Ok(());
    }

    // Default: just wait for HTTP server
    http_handle
        .await
        .map_err(|e| anyhow::anyhow!("HTTP server task failed: {e}"))?;

    Ok(())
}

// =============================================================================
// Initialization Functions
// =============================================================================

/// Initialize Sentry error tracking.
///
/// Returns a guard that must be kept alive for the duration of the program.
/// When dropped, Sentry will flush any pending events.
#[cfg(feature = "sentry")]
fn init_sentry() -> sentry::ClientInitGuard {
    sentry::init((
        std::env::var("SENTRY_DSN").ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: std::env::var("SENTRY_ENVIRONMENT")
                .ok()
                .map(std::borrow::Cow::Owned),
            // Capture 100% of transactions for performance monitoring in dev
            // Reduce this in production
            traces_sample_rate: std::env::var("SENTRY_TRACES_SAMPLE_RATE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            // Attach stack traces to all messages
            attach_stacktrace: true,
            // Send default PII (IP addresses, etc.) - disable in production if needed
            send_default_pii: std::env::var("SENTRY_SEND_PII")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            // Auto-session tracking
            auto_session_tracking: true,
            // Server name from hostname
            server_name: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .map(std::borrow::Cow::Owned),
            ..Default::default()
        },
    ))
}

/// Initialize tracing subscriber with optional Sentry and OpenTelemetry integration.
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false);

    // Build the subscriber with all enabled layers
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    // Add Sentry layer if enabled
    #[cfg(feature = "sentry")]
    let subscriber = {
        let sentry_layer =
            sentry_tracing::layer().event_filter(|metadata| match *metadata.level() {
                tracing::Level::ERROR | tracing::Level::WARN => sentry_tracing::EventFilter::Event,
                tracing::Level::INFO => sentry_tracing::EventFilter::Breadcrumb,
                _ => sentry_tracing::EventFilter::Ignore,
            });
        subscriber.with(sentry_layer)
    };

    // Add OpenTelemetry layer if enabled
    #[cfg(feature = "telemetry")]
    let subscriber = {
        match telemetry::init() {
            Ok(tracer) => {
                let otel_layer = telemetry::layer(tracer);
                subscriber.with(Some(otel_layer))
            }
            Err(e) => {
                eprintln!("Failed to initialize OpenTelemetry: {e}");
                subscriber.with(None::<tracing_opentelemetry::OpenTelemetryLayer<_, _>>)
            }
        }
    };

    subscriber.init();
}
