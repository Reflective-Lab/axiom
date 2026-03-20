//! Policy-based authorization module.
//!
//! Provides role/service/method-based access control with YAML configuration.

mod engine;
mod loader;
mod types;

pub use engine::PolicyEngine;
pub use loader::{PolicyLoader, PolicyLoaderError};
pub use types::{Effect, Policy, PolicyError, Principal, Rule};
