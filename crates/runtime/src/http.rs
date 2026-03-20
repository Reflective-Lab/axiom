// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! HTTP server implementation using Axum.

use axum::Router;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::{Level, info};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
        let app = Router::new()
            .merge(handlers::router(self.state.clone()))
            .merge(pilot::router())
            .merge(sse::router().with_state(self.state))
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .layer(
                ServiceBuilder::new()
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|_request: &axum::http::Request<_>| {
                                tracing::span!(
                                    Level::INFO,
                                    "http_request",
                                    method = %_request.method(),
                                    uri = %_request.uri(),
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
                    .layer(CompressionLayer::new())
                    .layer(CorsLayer::permissive()),
            );

        // Start listening
        let listener = TcpListener::bind(addr).await?;
        info!(%addr, "HTTP server listening");

        // Serve requests
        axum::serve(listener, app).await?;

        Ok(())
    }
}
