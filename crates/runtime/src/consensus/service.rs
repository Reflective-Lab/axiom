//! Runtime-facing consensus service API for governance events.

use std::sync::{Arc, Mutex};

use thiserror::Error;

use super::raft::{
    GovernanceEvent, GovernanceLogEntry, LogIndex, NodeId, RaftError, RaftGovernanceCluster,
};
use super::storage::{DurableRaftStore, GovernanceSnapshot, StorageError};

pub type SharedGovernanceConsensus = Arc<dyn GovernanceConsensus>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceConsensusBackendKind {
    InMemoryRaft,
    #[cfg(feature = "openraft")]
    OpenRaftAdapter,
}

#[derive(Debug, Error)]
pub enum GovernanceConsensusError {
    #[error(transparent)]
    Raft(#[from] RaftError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("consensus state lock poisoned")]
    LockPoisoned,
}

pub trait GovernanceConsensus: Send + Sync {
    fn backend_kind(&self) -> GovernanceConsensusBackendKind;
    fn cluster_state(&self) -> Result<RaftGovernanceCluster, GovernanceConsensusError>;
    fn elect_leader(&self, node: NodeId) -> Result<(), GovernanceConsensusError>;
    fn append_governance_event(
        &self,
        node: NodeId,
        event: GovernanceEvent,
    ) -> Result<GovernanceLogEntry, GovernanceConsensusError>;
    fn acknowledge_replication(
        &self,
        node: NodeId,
        index: LogIndex,
    ) -> Result<(), GovernanceConsensusError>;
    fn create_snapshot(&self) -> Result<Option<GovernanceSnapshot>, GovernanceConsensusError>;
    fn restore_from_store(&self) -> Result<bool, GovernanceConsensusError>;
}

pub struct InMemoryRaftGovernanceEngine {
    cluster: Mutex<RaftGovernanceCluster>,
    store: Option<Arc<dyn DurableRaftStore>>,
}

impl InMemoryRaftGovernanceEngine {
    pub fn new(cluster: RaftGovernanceCluster) -> Self {
        Self {
            cluster: Mutex::new(cluster),
            store: None,
        }
    }

    pub fn with_store(cluster: RaftGovernanceCluster, store: Arc<dyn DurableRaftStore>) -> Self {
        Self {
            cluster: Mutex::new(cluster),
            store: Some(store),
        }
    }

    pub fn store(&self) -> Option<&Arc<dyn DurableRaftStore>> {
        self.store.as_ref()
    }

    fn lock_cluster(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, RaftGovernanceCluster>, GovernanceConsensusError> {
        self.cluster
            .lock()
            .map_err(|_| GovernanceConsensusError::LockPoisoned)
    }

    fn persist_locked(
        &self,
        cluster: &RaftGovernanceCluster,
    ) -> Result<(), GovernanceConsensusError> {
        if let Some(store) = &self.store {
            store.persist_cluster(cluster)?;
        }
        Ok(())
    }
}

impl GovernanceConsensus for InMemoryRaftGovernanceEngine {
    fn backend_kind(&self) -> GovernanceConsensusBackendKind {
        GovernanceConsensusBackendKind::InMemoryRaft
    }

    fn cluster_state(&self) -> Result<RaftGovernanceCluster, GovernanceConsensusError> {
        let cluster = self.lock_cluster()?;
        Ok(cluster.clone())
    }

    fn elect_leader(&self, node: NodeId) -> Result<(), GovernanceConsensusError> {
        let mut cluster = self.lock_cluster()?;
        cluster.elect_leader(node)?;
        self.persist_locked(&cluster)?;
        Ok(())
    }

    fn append_governance_event(
        &self,
        node: NodeId,
        event: GovernanceEvent,
    ) -> Result<GovernanceLogEntry, GovernanceConsensusError> {
        let mut cluster = self.lock_cluster()?;
        let entry = cluster.append_governance_event(node, event)?;
        self.persist_locked(&cluster)?;
        Ok(entry)
    }

    fn acknowledge_replication(
        &self,
        node: NodeId,
        index: LogIndex,
    ) -> Result<(), GovernanceConsensusError> {
        let mut cluster = self.lock_cluster()?;
        cluster.acknowledge_replication(node, index)?;
        self.persist_locked(&cluster)?;
        Ok(())
    }

    fn create_snapshot(&self) -> Result<Option<GovernanceSnapshot>, GovernanceConsensusError> {
        let cluster = self.lock_cluster()?;
        let Some(store) = &self.store else {
            return Ok(None);
        };
        let snapshot = store.create_snapshot(&cluster)?;
        store.persist_cluster(&cluster)?;
        Ok(Some(snapshot))
    }

    fn restore_from_store(&self) -> Result<bool, GovernanceConsensusError> {
        let Some(store) = &self.store else {
            return Ok(false);
        };

        let restored = if let Some(cluster) = store.load_cluster()? {
            Some(cluster)
        } else {
            store
                .load_latest_snapshot()?
                .map(|snapshot| snapshot.cluster)
        };

        let Some(cluster) = restored else {
            return Ok(false);
        };

        let mut guard = self.lock_cluster()?;
        *guard = cluster;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::consensus::raft::{GovernanceEvent, NodeId};
    use crate::consensus::storage::JsonFileRaftStore;
    use uuid::Uuid;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("converge-runtime-raft-service-{}", Uuid::new_v4()));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn bootstrap_cluster() -> RaftGovernanceCluster {
        RaftGovernanceCluster::bootstrap([NodeId::new(1), NodeId::new(2), NodeId::new(3)])
            .expect("bootstrap should succeed")
    }

    #[test]
    fn create_snapshot_without_store_returns_none() {
        let engine = InMemoryRaftGovernanceEngine::new(bootstrap_cluster());
        let snapshot = engine
            .create_snapshot()
            .expect("snapshot operation should succeed");
        assert!(snapshot.is_none());
    }

    #[test]
    fn persistent_store_round_trip_restore() {
        let test_dir = TestDir::new();
        let store: Arc<dyn DurableRaftStore> = Arc::new(JsonFileRaftStore::new(test_dir.path()));

        let engine =
            InMemoryRaftGovernanceEngine::with_store(bootstrap_cluster(), Arc::clone(&store));
        engine
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");
        let first = engine
            .append_governance_event(
                NodeId::new(1),
                GovernanceEvent::ProposalCreated {
                    proposal_id: "p-restore".to_owned(),
                    actor: "agent://planner".to_owned(),
                },
            )
            .expect("append should succeed");
        engine
            .acknowledge_replication(NodeId::new(2), first.index)
            .expect("ack should succeed");
        let snapshot = engine
            .create_snapshot()
            .expect("snapshot should succeed")
            .expect("snapshot should exist with store");
        assert_eq!(snapshot.metadata.commit_index.get(), 1);

        let restored_engine =
            InMemoryRaftGovernanceEngine::with_store(bootstrap_cluster(), Arc::clone(&store));
        let restored = restored_engine
            .restore_from_store()
            .expect("restore should succeed");
        assert!(restored);

        let restored_cluster = restored_engine
            .cluster_state()
            .expect("cluster state should be readable");
        assert_eq!(restored_cluster.commit_index().get(), 1);
        assert_eq!(restored_cluster.applied_index().get(), 1);
        assert_eq!(restored_cluster.log_entries().len(), 1);
    }
}
