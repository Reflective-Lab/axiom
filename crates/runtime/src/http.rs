// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! HTTP server implementation using Axum.

#[cfg(feature = "auth")]
use axum::middleware::from_fn;
use axum::{Json, Router, routing::get};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{Level, info};

use utoipa::OpenApi;

use crate::api::ApiDoc;
use crate::config::HttpConfig;
use crate::error::RuntimeError;
use crate::handlers;
use crate::pilot;
use crate::sse;
use crate::state::AppState;

/// HTTP server for Converge Runtime.
pub struct HttpServer {
    config: HttpConfig,
    state: AppState,
}

impl HttpServer {
    /// Create a new HTTP server.
    pub fn new(config: HttpConfig, state: AppState) -> Self {
        Self { config, state }
    }

    /// Start the HTTP server.
    pub async fn start(self) -> Result<(), RuntimeError> {
        let addr = self.config.bind;
        info!(%addr, "Starting HTTP server");

        // Build router with middleware and OpenAPI docs
        let protected = Router::new()
            .merge(handlers::protected_router(self.state.clone()))
            .merge(pilot::router())
            .merge(sse::router().with_state(self.state.clone()));
        #[cfg(feature = "auth")]
        let protected = protected.layer(from_fn(crate::http_auth::require_auth));

        let app = Router::new()
            .merge(handlers::public_router())
            .merge(protected)
            .route(
                "/api-docs/openapi.json",
                get(|| async { Json(ApiDoc::openapi()) }),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|_request: &axum::http::Request<_>| {
                                tracing::span!(
                                    Level::INFO,
                                    "http_request",
                                    method = %_request.method(),
                                    path = %_request.uri().path(),
                                )
                            })
                            .on_response(
                                |_response: &axum::response::Response<_>,
                                 latency: std::time::Duration,
                                 _span: &tracing::Span| {
                                    tracing::event!(
                                        Level::INFO,
                                        latency_ms = latency.as_millis(),
                                        status = %_response.status(),
                                        "HTTP response"
                                    );
                                },
                            ),
                    )
                    .layer(RequestBodyLimitLayer::new(self.config.max_body_size))
                    .layer(CompressionLayer::new()),
            );

        // Start listening
        let listener = TcpListener::bind(addr).await?;
        info!(%addr, "HTTP server listening");

        // Serve requests
        axum::serve(listener, app).await?;

        Ok(())
    }
}
