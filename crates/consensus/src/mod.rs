//! Runtime consensus primitives.
//!
//! Converge uses consensus for cluster agreement on execution facts and ordering.
//! This module models infrastructure mechanics (leader election, quorum commits,
//! replicated ordering) and intentionally does not perform semantic validation.

pub mod raft;
pub mod service;
pub mod storage;

#[cfg(feature = "openraft")]
pub mod openraft_adapter;

pub use service::{
    GovernanceConsensus, GovernanceConsensusBackendKind, GovernanceConsensusError,
    InMemoryRaftGovernanceEngine, SharedGovernanceConsensus,
};
pub use storage::{
    DurableRaftStore, GovernanceSnapshot, GovernanceSnapshotMetadata, JsonFileRaftStore,
    StorageError,
};
