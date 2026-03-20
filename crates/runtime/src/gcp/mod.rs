//! Google Cloud Platform integration
//!
//! Provides clients and utilities for GCP services:
//! - Firestore for user management and job state
//! - Service Directory for gRPC service discovery

mod config;
mod firestore;
mod service_directory;

pub use config::GcpConfig;
pub use firestore::{FirestoreClient, FirestoreError};
pub use service_directory::{ServiceDirectory, ServiceEndpoint};
