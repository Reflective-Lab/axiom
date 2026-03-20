//! Domain Packs for Solver Gate
//!
//! Each pack provides:
//! - Typed input/output schemas
//! - Domain-specific solvers
//! - Invariant definitions
//! - Test scenarios
//!
//! ## Available Packs
//!
//! - [`meeting_scheduler`] - Meeting time selection with preferences
//! - [`inventory_rebalancing`] - Inventory transfer planning
//!
//! ## Example
//!
//! ```rust,ignore
//! use converge_optimization::packs::{PackRegistry, Pack};
//! use converge_optimization::gate::ProblemSpec;
//!
//! let registry = PackRegistry::with_builtins();
//! let pack = registry.get("meeting-scheduler").unwrap();
//! let result = pack.solve(&spec)?;
//! ```

pub mod registry;
pub mod testing;
pub mod traits;

// Fully implemented packs
pub mod meeting_scheduler;
pub mod inventory_rebalancing;

// Stub packs (types + placeholder solver)
pub mod anomaly_triage;
pub mod backlog_prioritization;
pub mod budget_allocation;
pub mod capacity_planning;
pub mod inventory_replenishment;
pub mod lead_routing;
pub mod pricing_guardrails;
pub mod shipping_choice;
pub mod vendor_shortlist;

pub use registry::*;
pub use testing::{ExpectedOutcome, ScenarioResult, TestScenario};
pub use traits::*;
