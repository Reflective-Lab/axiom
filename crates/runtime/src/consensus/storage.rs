//! Durable storage for Raft governance state and snapshots.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::raft::{LogIndex, RaftGovernanceCluster};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSnapshotMetadata {
    pub snapshot_id: String,
    pub created_at_unix_ms: u64,
    pub term: u64,
    pub commit_index: LogIndex,
    pub applied_index: LogIndex,
    pub last_log_index: LogIndex,
    pub member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSnapshot {
    pub metadata: GovernanceSnapshotMetadata,
    pub cluster: RaftGovernanceCluster,
}

impl GovernanceSnapshot {
    pub fn from_cluster(cluster: &RaftGovernanceCluster) -> Self {
        let created_at_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis() as u64);
        let metadata = GovernanceSnapshotMetadata {
            snapshot_id: format!(
                "raft-governance-{:020}-{:020}-{:020}",
                cluster.commit_index().get(),
                cluster.applied_index().get(),
                created_at_unix_ms
            ),
            created_at_unix_ms,
            term: cluster.current_term(),
            commit_index: cluster.commit_index(),
            applied_index: cluster.applied_index(),
            last_log_index: cluster.last_log_index(),
            member_count: cluster.members().len(),
        };

        Self {
            metadata,
            cluster: cluster.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("JSON error at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub trait DurableRaftStore: Send + Sync {
    fn persist_cluster(&self, cluster: &RaftGovernanceCluster) -> Result<(), StorageError>;
    fn load_cluster(&self) -> Result<Option<RaftGovernanceCluster>, StorageError>;
    fn create_snapshot(
        &self,
        cluster: &RaftGovernanceCluster,
    ) -> Result<GovernanceSnapshot, StorageError>;
    fn load_latest_snapshot(&self) -> Result<Option<GovernanceSnapshot>, StorageError>;
}

#[derive(Debug, Clone)]
pub struct JsonFileRaftStore {
    root: PathBuf,
}

impl JsonFileRaftStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn ensure_layout(&self) -> Result<(), StorageError> {
        fs::create_dir_all(self.snapshots_dir()).map_err(|source| StorageError::Io {
            path: self.snapshots_dir(),
            source,
        })
    }

    fn cluster_state_path(&self) -> PathBuf {
        self.root.join("raft-governance-cluster.json")
    }

    fn snapshots_dir(&self) -> PathBuf {
        self.root.join("snapshots")
    }

    fn snapshot_path(&self, snapshot: &GovernanceSnapshot) -> PathBuf {
        self.snapshots_dir()
            .join(format!("{}.json", snapshot.metadata.snapshot_id))
    }

    fn load_json<T>(&self, path: &Path) -> Result<Option<T>, StorageError>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path).map_err(|source| StorageError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let value = serde_json::from_slice(&bytes).map_err(|source| StorageError::Json {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Some(value))
    }

    fn write_json<T>(&self, path: &Path, value: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StorageError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let encoded = serde_json::to_vec_pretty(value).map_err(|source| StorageError::Json {
            path: path.to_path_buf(),
            source,
        })?;

        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, &encoded).map_err(|source| StorageError::Io {
            path: tmp_path.clone(),
            source,
        })?;
        fs::rename(&tmp_path, path).map_err(|source| StorageError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(())
    }

    fn latest_snapshot_path(&self) -> Result<Option<PathBuf>, StorageError> {
        let dir = self.snapshots_dir();
        if !dir.exists() {
            return Ok(None);
        }

        let entries = fs::read_dir(&dir).map_err(|source| StorageError::Io {
            path: dir.clone(),
            source,
        })?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| StorageError::Io {
                path: dir.clone(),
                source,
            })?;
            let path = entry.path();
            if path.extension() == Some(OsStr::new("json")) {
                files.push(path);
            }
        }

        files.sort();
        Ok(files.pop())
    }
}

impl DurableRaftStore for JsonFileRaftStore {
    fn persist_cluster(&self, cluster: &RaftGovernanceCluster) -> Result<(), StorageError> {
        self.ensure_layout()?;
        self.write_json(&self.cluster_state_path(), cluster)
    }

    fn load_cluster(&self) -> Result<Option<RaftGovernanceCluster>, StorageError> {
        self.ensure_layout()?;
        self.load_json(&self.cluster_state_path())
    }

    fn create_snapshot(
        &self,
        cluster: &RaftGovernanceCluster,
    ) -> Result<GovernanceSnapshot, StorageError> {
        self.ensure_layout()?;
        let snapshot = GovernanceSnapshot::from_cluster(cluster);
        self.write_json(&self.snapshot_path(&snapshot), &snapshot)?;
        Ok(snapshot)
    }

    fn load_latest_snapshot(&self) -> Result<Option<GovernanceSnapshot>, StorageError> {
        self.ensure_layout()?;
        let Some(path) = self.latest_snapshot_path()? else {
            return Ok(None);
        };
        self.load_json(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::raft::{GovernanceEvent, NodeId, RaftGovernanceCluster};
    use uuid::Uuid;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("converge-runtime-raft-store-{}", Uuid::new_v4()));
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

    fn sample_cluster() -> RaftGovernanceCluster {
        let mut cluster = RaftGovernanceCluster::bootstrap(
            [NodeId::new(1), NodeId::new(2), NodeId::new(3)].into_iter(),
        )
        .expect("cluster bootstrap should succeed");
        cluster
            .elect_leader(NodeId::new(1))
            .expect("leader election should succeed");
        let first = cluster
            .append_governance_event(
                NodeId::new(1),
                GovernanceEvent::ProposalCreated {
                    proposal_id: "p-1".to_owned(),
                    actor: "agent://planner".to_owned(),
                },
            )
            .expect("append should succeed");
        let _second = cluster
            .append_governance_event(
                NodeId::new(1),
                GovernanceEvent::PromotionApproved {
                    proposal_id: "p-1".to_owned(),
                    approver: "engine://governor".to_owned(),
                },
            )
            .expect("append should succeed");
        cluster
            .acknowledge_replication(NodeId::new(2), first.index)
            .expect("ack should succeed");
        cluster
            .acknowledge_replication(NodeId::new(2), cluster.last_log_index())
            .expect("ack should succeed");
        cluster
    }

    #[test]
    fn persist_and_load_cluster_round_trip() {
        let test_dir = TestDir::new();
        let store = JsonFileRaftStore::new(test_dir.path());
        let cluster = sample_cluster();

        store
            .persist_cluster(&cluster)
            .expect("persist should succeed");
        let loaded = store
            .load_cluster()
            .expect("load should succeed")
            .expect("cluster should exist");

        assert_eq!(loaded.commit_index(), cluster.commit_index());
        assert_eq!(loaded.applied_index(), cluster.applied_index());
        assert_eq!(loaded.current_term(), cluster.current_term());
        assert_eq!(loaded.log_entries().len(), cluster.log_entries().len());
    }

    #[test]
    fn create_and_load_latest_snapshot() {
        let test_dir = TestDir::new();
        let store = JsonFileRaftStore::new(test_dir.path());
        let cluster = sample_cluster();

        let created = store
            .create_snapshot(&cluster)
            .expect("snapshot should be created");
        let loaded = store
            .load_latest_snapshot()
            .expect("load should succeed")
            .expect("snapshot should exist");

        assert_eq!(loaded.metadata.snapshot_id, created.metadata.snapshot_id);
        assert_eq!(loaded.metadata.commit_index, cluster.commit_index());
        assert_eq!(
            loaded.cluster.log_entries().len(),
            cluster.log_entries().len()
        );
    }
}
