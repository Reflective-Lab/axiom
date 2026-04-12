// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Authentication, authorization, cryptography, and secrets management.
//!
//! Extracted from converge-runtime to be a standalone security crate.
//! All security concerns in one place instead of scattered across the runtime.
//!
//! # Modules
//!
//! - [`auth`] — Firebase auth, JWT validation, identity, provider abstraction
//! - [`crypto`] — Field encryption, keystore, in-memory key management
//! - [`secrets`] — Secret resolution (GCP Secret Manager, environment)
//! - [`identity`] — File-based identity management
//! - [`interceptor`] — gRPC auth interceptor
//! - [`http_auth`] — Axum HTTP auth middleware

pub mod auth;
pub mod crypto;
pub mod secrets;
pub mod identity;
pub mod interceptor;
pub mod http_auth;
