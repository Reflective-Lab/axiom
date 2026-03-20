//! Converge Policy — Cedar-based Policy Decision Point for the Converge gate model
//!
//! Evaluates agent authority, commitment constraints, and phase gate requirements
//! as deterministic, auditable policy decisions.
//!
//! Two decision paths:
//! - **Policy mode**: Cedar policy evaluation against principal/resource/context
//! - **Delegation mode**: Ed25519-signed, time-scoped authority tokens

pub mod decision;
pub mod delegation;
pub mod engine;
pub mod types;

pub use decision::{PolicyDecision, PolicyOutcome};
pub use delegation::Delegation;
pub use engine::PolicyEngine;
pub use types::{ContextIn, DecideRequest, PrincipalIn, ResourceIn};
