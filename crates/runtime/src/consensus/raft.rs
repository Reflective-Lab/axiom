//! In-memory Raft mechanics for Converge runtime governance events.
//!
//! This module is a Converge-specific scaffold for runtime consensus:
//! - `converge-ledger` (Elixir) can provide append-only local/context ledgering
//!   and Lamport timestamps for scalable distributed ordering hints.
//! - This module provides quorum-backed commit ordering and leader authority for
//!   the subset of runtime events that must become cluster facts.
//!
//! It intentionally models Raft-like mechanics (leader, log, quorum commit,
//! state-machine apply) without implementing a networked Raft transport.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct NodeId(u64);

impl NodeId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for NodeId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct LogIndex(u64);

impl LogIndex {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for LogIndex {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaftRole {
    Leader,
    Follower,
    Candidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LamportTimestamp {
    pub counter: u64,
    pub node: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LamportClock {
    node: NodeId,
    counter: u64,
}

impl Default for LamportClock {
    fn default() -> Self {
        Self {
            node: NodeId::new(0),
            counter: 0,
        }
    }
}

impl LamportClock {
    pub const fn seeded(node: NodeId, counter: u64) -> Self {
        Self { node, counter }
    }

    pub const fn node(self) -> NodeId {
        self.node
    }

    pub const fn counter(self) -> u64 {
        self.counter
    }

    pub fn observe(&mut self, observed: LamportTimestamp) {
        self.counter = self.counter.max(observed.counter);
    }

    pub fn tick(&mut self) -> LamportTimestamp {
        self.counter = self.counter.saturating_add(1);
        LamportTimestamp {
            counter: self.counter,
            node: self.node,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceEvent {
    ProposalCreated {
        proposal_id: String,
        actor: String,
    },
    ValidationPassed {
        proposal_id: String,
        validator: String,
    },
    ValidationFailed {
        proposal_id: String,
        validator: String,
        reason: String,
    },
    PromotionApproved {
        proposal_id: String,
        approver: String,
    },
    PromotionRejected {
        proposal_id: String,
        approver: String,
        reason: String,
    },
    RollbackIssued {
        proposal_id: String,
        operator: String,
        reason: String,
    },
    OverrideGranted {
        proposal_id: String,
        operator: String,
        reason: String,
    },
}

impl GovernanceEvent {
    pub fn proposal_id(&self) -> &str {
        match self {
            Self::ProposalCreated { proposal_id, .. }
            | Self::ValidationPassed { proposal_id, .. }
            | Self::ValidationFailed { proposal_id, .. }
            | Self::PromotionApproved { proposal_id, .. }
            | Self::PromotionRejected { proposal_id, .. }
            | Self::RollbackIssued { proposal_id, .. }
            | Self::OverrideGranted { proposal_id, .. } => proposal_id,
        }
    }

    pub const fn kind(&self) -> GovernanceEventKind {
        match self {
            Self::ProposalCreated { .. } => GovernanceEventKind::ProposalCreated,
            Self::ValidationPassed { .. } => GovernanceEventKind::ValidationPassed,
            Self::ValidationFailed { .. } => GovernanceEventKind::ValidationFailed,
            Self::PromotionApproved { .. } => GovernanceEventKind::PromotionApproved,
            Self::PromotionRejected { .. } => GovernanceEventKind::PromotionRejected,
            Self::RollbackIssued { .. } => GovernanceEventKind::RollbackIssued,
            Self::OverrideGranted { .. } => GovernanceEventKind::OverrideGranted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceEventKind {
    ProposalCreated,
    ValidationPassed,
    ValidationFailed,
    PromotionApproved,
    PromotionRejected,
    RollbackIssued,
    OverrideGranted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Validated,
    ValidationFailed,
    Promoted,
    Rejected,
    RolledBack,
    Overridden,
}

impl ProposalStatus {
    pub const fn from_event(event: &GovernanceEvent) -> Self {
        match event {
            GovernanceEvent::ProposalCreated { .. } => Self::Pending,
            GovernanceEvent::ValidationPassed { .. } => Self::Validated,
            GovernanceEvent::ValidationFailed { .. } => Self::ValidationFailed,
            GovernanceEvent::PromotionApproved { .. } => Self::Promoted,
            GovernanceEvent::PromotionRejected { .. } => Self::Rejected,
            GovernanceEvent::RollbackIssued { .. } => Self::RolledBack,
            GovernanceEvent::OverrideGranted { .. } => Self::Overridden,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceLogEntry {
    pub term: u64,
    pub index: LogIndex,
    pub leader: NodeId,
    pub lamport: LamportTimestamp,
    pub event: GovernanceEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalProjection {
    pub status: ProposalStatus,
    pub last_event: GovernanceEventKind,
    pub last_index: LogIndex,
    pub last_term: u64,
    pub history_len: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionLedgerState {
    proposals: BTreeMap<String, ProposalProjection>,
}

impl PromotionLedgerState {
    pub fn proposal(&self, proposal_id: &str) -> Option<&ProposalProjection> {
        self.proposals.get(proposal_id)
    }

    pub fn proposals(&self) -> &BTreeMap<String, ProposalProjection> {
        &self.proposals
    }

    fn apply(&mut self, entry: &GovernanceLogEntry) {
        let proposal_id = entry.event.proposal_id().to_owned();
        let projection = self
            .proposals
            .entry(proposal_id)
            .or_insert_with(|| ProposalProjection {
                status: ProposalStatus::Pending,
                last_event: GovernanceEventKind::ProposalCreated,
                last_index: LogIndex::default(),
                last_term: 0,
                history_len: 0,
            });

        projection.status = ProposalStatus::from_event(&entry.event);
        projection.last_event = entry.event.kind();
        projection.last_index = entry.index;
        projection.last_term = entry.term;
        projection.history_len = projection.history_len.saturating_add(1);
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RaftError {
    #[error("cluster requires at least one member")]
    EmptyCluster,
    #[error("node {node:?} is not a cluster member")]
    UnknownNode { node: NodeId },
    #[error("node {node:?} is not leader; current leader is {leader:?}")]
    NotLeader {
        node: NodeId,
        leader: Option<NodeId>,
    },
    #[error("replication acknowledgment must come from a follower, got leader {node:?}")]
    InvalidAckFromLeader { node: NodeId },
    #[error("replication acknowledgment for {index:?} is beyond local log {last_index:?}")]
    AckBeyondLog {
        index: LogIndex,
        last_index: LogIndex,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftGovernanceCluster {
    members: BTreeSet<NodeId>,
    roles: BTreeMap<NodeId, RaftRole>,
    match_index: BTreeMap<NodeId, LogIndex>,
    current_term: u64,
    leader: Option<NodeId>,
    log: Vec<GovernanceLogEntry>,
    commit_index: LogIndex,
    applied_index: LogIndex,
    lamport_clock: LamportClock,
    ledger: PromotionLedgerState,
}

impl RaftGovernanceCluster {
    pub fn bootstrap<I>(members: I) -> Result<Self, RaftError>
    where
        I: IntoIterator<Item = NodeId>,
    {
        let members: BTreeSet<NodeId> = members.into_iter().collect();
        if members.is_empty() {
            return Err(RaftError::EmptyCluster);
        }

        let roles = members
            .iter()
            .copied()
            .map(|node| (node, RaftRole::Follower))
            .collect();
        let match_index = members
            .iter()
            .copied()
            .map(|node| (node, LogIndex::default()))
            .collect();

        Ok(Self {
            members,
            roles,
            match_index,
            current_term: 0,
            leader: None,
            log: Vec::new(),
            commit_index: LogIndex::default(),
            applied_index: LogIndex::default(),
            lamport_clock: LamportClock::default(),
            ledger: PromotionLedgerState::default(),
        })
    }

    pub fn members(&self) -> &BTreeSet<NodeId> {
        &self.members
    }

    pub fn quorum_size(&self) -> usize {
        (self.members.len() / 2) + 1
    }

    pub const fn current_term(&self) -> u64 {
        self.current_term
    }

    pub const fn leader(&self) -> Option<NodeId> {
        self.leader
    }

    pub fn role(&self, node: NodeId) -> Result<RaftRole, RaftError> {
        self.ensure_member(node)?;
        Ok(self.roles.get(&node).copied().unwrap_or(RaftRole::Follower))
    }

    pub fn lamport_clock(&self) -> LamportClock {
        self.lamport_clock
    }

    pub fn observe_remote_timestamp(&mut self, observed: LamportTimestamp) {
        self.lamport_clock.observe(observed);
    }

    pub fn elect_leader(&mut self, node: NodeId) -> Result<(), RaftError> {
        self.ensure_member(node)?;

        self.current_term = self.current_term.saturating_add(1);
        self.leader = Some(node);

        for (member, role) in &mut self.roles {
            *role = if *member == node {
                RaftRole::Leader
            } else {
                RaftRole::Follower
            };
        }

        let seed = self
            .lamport_clock
            .counter()
            .max(self.log.last().map_or(0, |entry| entry.lamport.counter));
        self.lamport_clock = LamportClock::seeded(node, seed);

        for member in &self.members {
            let reset_index = if *member == node {
                self.last_log_index()
            } else {
                self.commit_index
            };
            self.match_index.insert(*member, reset_index);
        }

        Ok(())
    }

    pub fn append_governance_event(
        &mut self,
        node: NodeId,
        event: GovernanceEvent,
    ) -> Result<GovernanceLogEntry, RaftError> {
        self.ensure_leader(node)?;

        let next_index = LogIndex::new(self.log.len() as u64 + 1);
        let entry = GovernanceLogEntry {
            term: self.current_term,
            index: next_index,
            leader: node,
            lamport: self.lamport_clock.tick(),
            event,
        };

        self.log.push(entry.clone());
        self.match_index.insert(node, next_index);
        self.advance_commit_index();

        Ok(entry)
    }

    pub fn acknowledge_replication(
        &mut self,
        node: NodeId,
        index: LogIndex,
    ) -> Result<(), RaftError> {
        self.ensure_member(node)?;
        if Some(node) == self.leader {
            return Err(RaftError::InvalidAckFromLeader { node });
        }

        let last_index = self.last_log_index();
        if index > last_index {
            return Err(RaftError::AckBeyondLog { index, last_index });
        }

        let current = self.match_index.get(&node).copied().unwrap_or_default();
        if index > current {
            self.match_index.insert(node, index);
        }

        self.advance_commit_index();
        Ok(())
    }

    pub fn log_entries(&self) -> &[GovernanceLogEntry] {
        &self.log
    }

    pub fn committed_entries(&self) -> &[GovernanceLogEntry] {
        let end = self.commit_index.get() as usize;
        &self.log[..end]
    }

    pub const fn commit_index(&self) -> LogIndex {
        self.commit_index
    }

    pub const fn applied_index(&self) -> LogIndex {
        self.applied_index
    }

    pub fn ledger(&self) -> &PromotionLedgerState {
        &self.ledger
    }

    pub fn last_log_index(&self) -> LogIndex {
        LogIndex::new(self.log.len() as u64)
    }

    fn ensure_member(&self, node: NodeId) -> Result<(), RaftError> {
        if self.members.contains(&node) {
            Ok(())
        } else {
            Err(RaftError::UnknownNode { node })
        }
    }

    fn ensure_leader(&self, node: NodeId) -> Result<(), RaftError> {
        self.ensure_member(node)?;
        if Some(node) == self.leader {
            Ok(())
        } else {
            Err(RaftError::NotLeader {
                node,
                leader: self.leader,
            })
        }
    }

    fn advance_commit_index(&mut self) {
        let mut new_commit = self.commit_index;
        let last = self.last_log_index().get();

        for raw_index in (self.commit_index.get() + 1)..=last {
            let candidate = LogIndex::new(raw_index);
            let replicated = self
                .members
                .iter()
                .filter(|member| {
                    self.match_index.get(*member).copied().unwrap_or_default() >= candidate
                })
                .count();

            if replicated >= self.quorum_size() {
                new_commit = candidate;
            }
        }

        if new_commit > self.commit_index {
            self.commit_index = new_commit;
            self.apply_commits();
        }
    }

    fn apply_commits(&mut self) {
        while self.applied_index < self.commit_index {
            let next_raw = self.applied_index.get() + 1;
            let offset = (next_raw - 1) as usize;
            let entry = &self.log[offset];
            self.ledger.apply(entry);
            self.applied_index = LogIndex::new(next_raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster(member_ids: &[u64]) -> RaftGovernanceCluster {
        let members = member_ids.iter().copied().map(NodeId::new);
        RaftGovernanceCluster::bootstrap(members).expect("cluster bootstrap should succeed")
    }

    fn proposal_created(proposal_id: &str) -> GovernanceEvent {
        GovernanceEvent::ProposalCreated {
            proposal_id: proposal_id.to_owned(),
            actor: "agent://planner".to_owned(),
        }
    }

    fn promotion_approved(proposal_id: &str) -> GovernanceEvent {
        GovernanceEvent::PromotionApproved {
            proposal_id: proposal_id.to_owned(),
            approver: "engine://governor".to_owned(),
        }
    }

    #[test]
    fn single_node_cluster_commits_immediately() {
        let mut cluster = cluster(&[1]);
        cluster
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");

        let entry = cluster
            .append_governance_event(NodeId::new(1), proposal_created("p-1"))
            .expect("append should succeed");

        assert_eq!(entry.index, LogIndex::new(1));
        assert_eq!(cluster.commit_index(), LogIndex::new(1));
        assert_eq!(cluster.applied_index(), LogIndex::new(1));

        let projection = cluster
            .ledger()
            .proposal("p-1")
            .expect("proposal should exist after apply");
        assert_eq!(projection.status, ProposalStatus::Pending);
        assert_eq!(projection.last_event, GovernanceEventKind::ProposalCreated);
    }

    #[test]
    fn append_requires_leader_authority() {
        let mut cluster = cluster(&[1, 2, 3]);
        cluster
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");

        let error = cluster
            .append_governance_event(NodeId::new(2), proposal_created("p-2"))
            .expect_err("follower append must fail");

        assert_eq!(
            error,
            RaftError::NotLeader {
                node: NodeId::new(2),
                leader: Some(NodeId::new(1)),
            }
        );
        assert!(cluster.log_entries().is_empty());
    }

    #[test]
    fn quorum_commit_waits_for_replication_ack() {
        let mut cluster = cluster(&[1, 2, 3]);
        cluster
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");

        cluster
            .append_governance_event(NodeId::new(1), proposal_created("p-3"))
            .expect("append should succeed");

        assert_eq!(cluster.commit_index(), LogIndex::new(0));
        assert_eq!(cluster.applied_index(), LogIndex::new(0));

        cluster
            .acknowledge_replication(NodeId::new(2), LogIndex::new(1))
            .expect("follower ack should succeed");

        assert_eq!(cluster.commit_index(), LogIndex::new(1));
        assert_eq!(cluster.applied_index(), LogIndex::new(1));
    }

    #[test]
    fn follower_ack_for_higher_index_commits_multiple_entries_in_order() {
        let mut cluster = cluster(&[1, 2, 3]);
        cluster
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");

        let first = cluster
            .append_governance_event(NodeId::new(1), proposal_created("p-4"))
            .expect("first append should succeed");
        let second = cluster
            .append_governance_event(NodeId::new(1), promotion_approved("p-4"))
            .expect("second append should succeed");

        cluster
            .acknowledge_replication(NodeId::new(2), LogIndex::new(2))
            .expect("follower ack should succeed");

        assert_eq!(first.index, LogIndex::new(1));
        assert_eq!(second.index, LogIndex::new(2));
        assert_eq!(cluster.commit_index(), LogIndex::new(2));
        assert_eq!(cluster.applied_index(), LogIndex::new(2));
        assert_eq!(cluster.committed_entries().len(), 2);

        let projection = cluster
            .ledger()
            .proposal("p-4")
            .expect("proposal should exist after apply");
        assert_eq!(projection.status, ProposalStatus::Promoted);
        assert_eq!(projection.last_index, LogIndex::new(2));
        assert_eq!(projection.history_len, 2);
    }

    #[test]
    fn lamport_clock_advances_after_observing_remote_timestamp() {
        let mut cluster = cluster(&[1, 2, 3]);
        cluster
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");

        cluster.observe_remote_timestamp(LamportTimestamp {
            counter: 40,
            node: NodeId::new(2),
        });

        let first = cluster
            .append_governance_event(NodeId::new(1), proposal_created("p-5"))
            .expect("append should succeed");
        let second = cluster
            .append_governance_event(NodeId::new(1), proposal_created("p-6"))
            .expect("append should succeed");

        assert_eq!(first.lamport.counter, 41);
        assert_eq!(first.lamport.node, NodeId::new(1));
        assert_eq!(second.lamport.counter, 42);
    }
}
