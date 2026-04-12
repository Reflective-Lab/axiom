// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Observability for Converge — audit, telemetry, and metrics.
//!
//! Extracted from converge-runtime to be a standalone crate.
//!
//! - [`audit`] — Durable security/operational event trail via NATS JetStream
//! - [`telemetry`] — OpenTelemetry distributed tracing (Jaeger, OTLP)
//! - [`metrics`] — Prometheus metrics (HTTP, gRPC, convergence counters)
//!
//! `converge-experience` remains a separate crate — it tracks convergence-level
//! events (fact promotion, proposal rejection), not generic observability.
//!
//! Source files need import refactoring before this compiles standalone.

pub mod audit;
pub mod telemetry;
pub mod metrics;
