//! OpenRaft adapter seam for Converge runtime governance consensus.
//!
//! This module is feature-gated and intentionally thin. It exposes the same
//! runtime-facing `GovernanceConsensus` API so callers can swap the in-memory
//! mechanics engine for a real OpenRaft-backed transport/storage integration
//! later without changing `AppState` or handler call sites.

use std::sync::Arc;

use super::raft::{GovernanceEvent, GovernanceLogEntry, LogIndex, NodeId, RaftGovernanceCluster};
use super::service::{
    GovernanceConsensus, GovernanceConsensusBackendKind, GovernanceConsensusError,
    SharedGovernanceConsensus,
};
use super::storage::GovernanceSnapshot;

pub struct OpenRaftGovernanceAdapter {
    cluster_name: String,
    fallback: SharedGovernanceConsensus,
}

impl OpenRaftGovernanceAdapter {
    pub fn stub(cluster_name: impl Into<String>, fallback: SharedGovernanceConsensus) -> Self {
        Self {
            cluster_name: cluster_name.into(),
            fallback,
        }
    }

    pub fn cluster_name(&self) -> &str {
        &self.cluster_name
    }

    pub fn into_shared(self) -> Arc<dyn GovernanceConsensus> {
        Arc::new(self)
    }
}

impl GovernanceConsensus for OpenRaftGovernanceAdapter {
    fn backend_kind(&self) -> GovernanceConsensusBackendKind {
        GovernanceConsensusBackendKind::OpenRaftAdapter
    }

    fn cluster_state(&self) -> Result<RaftGovernanceCluster, GovernanceConsensusError> {
        self.fallback.cluster_state()
    }

    fn elect_leader(&self, node: NodeId) -> Result<(), GovernanceConsensusError> {
        self.fallback.elect_leader(node)
    }

    fn append_governance_event(
        &self,
        node: NodeId,
        event: GovernanceEvent,
    ) -> Result<GovernanceLogEntry, GovernanceConsensusError> {
        self.fallback.append_governance_event(node, event)
    }

    fn acknowledge_replication(
        &self,
        node: NodeId,
        index: LogIndex,
    ) -> Result<(), GovernanceConsensusError> {
        self.fallback.acknowledge_replication(node, index)
    }

    fn create_snapshot(&self) -> Result<Option<GovernanceSnapshot>, GovernanceConsensusError> {
        self.fallback.create_snapshot()
    }

    fn restore_from_store(&self) -> Result<bool, GovernanceConsensusError> {
        self.fallback.restore_from_store()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::consensus::raft::{GovernanceEvent, NodeId, RaftGovernanceCluster};
    use crate::consensus::service::InMemoryRaftGovernanceEngine;

    #[test]
    fn openraft_adapter_uses_same_api_surface() {
        let cluster =
            RaftGovernanceCluster::bootstrap([NodeId::new(1), NodeId::new(2), NodeId::new(3)])
                .expect("bootstrap should succeed");
        let fallback: SharedGovernanceConsensus =
            Arc::new(InMemoryRaftGovernanceEngine::new(cluster));
        let adapter = OpenRaftGovernanceAdapter::stub("cluster-a", Arc::clone(&fallback));

        adapter
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");
        let first = adapter
            .append_governance_event(
                NodeId::new(1),
                GovernanceEvent::ProposalCreated {
                    proposal_id: "p-openraft".to_owned(),
                    actor: "agent://planner".to_owned(),
                },
            )
            .expect("append should succeed");
        adapter
            .acknowledge_replication(NodeId::new(2), first.index)
            .expect("ack should succeed");

        let cluster = adapter
            .cluster_state()
            .expect("cluster state should be readable");
        assert_eq!(cluster.commit_index().get(), 1);
    }
}
