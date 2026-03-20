//! Database repositories
//!
//! Provides repository patterns for persistent storage using Firestore.

mod jobs;
mod users;

pub use jobs::{Job, JobRepository, JobStatus};
pub use users::{User, UserRepository};

use crate::gcp::{FirestoreClient, FirestoreError, GcpConfig};

/// Database connection holder
pub struct Database {
    pub firestore: FirestoreClient,
    pub users: UserRepository,
    pub jobs: JobRepository,
}

impl Database {
    /// Create a new database connection
    pub async fn new(config: GcpConfig) -> Result<Self, FirestoreError> {
        let firestore = FirestoreClient::new(config).await?;

        Ok(Self {
            users: UserRepository::new(firestore.db().clone()),
            jobs: JobRepository::new(firestore.db().clone()),
            firestore,
        })
    }
}
