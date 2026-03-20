//! Job repository

use chrono::{DateTime, Utc};
use firestore::*;
use serde::{Deserialize, Serialize};

use crate::gcp::FirestoreError;

/// Generate a unique document ID (timestamp + random suffix)
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let random: u32 = rand::random();
    format!("{timestamp:x}{random:08x}")
}

/// Job status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    #[default]
    Pending,
    Running,
    Converged,
    Failed,
    Cancelled,
}

/// Job document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Job ID (auto-generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// User ID who created the job
    pub user_id: String,

    /// Job status
    #[serde(default)]
    pub status: JobStatus,

    /// Root intent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_id: Option<String>,

    /// Number of convergence cycles
    #[serde(default)]
    pub cycles: u32,

    /// Maximum allowed cycles
    #[serde(default = "default_max_cycles")]
    pub max_cycles: u32,

    /// Seed facts (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seeds: Option<serde_json::Value>,

    /// Final context (JSON, populated after convergence)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Agents used in this job
    #[serde(default)]
    pub agents: Vec<String>,

    /// LLM provider used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,

    /// LLM model used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_model: Option<String>,

    /// Execution duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Started timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    /// Completed timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

fn default_max_cycles() -> u32 {
    100
}

impl Job {
    /// Create a new pending job
    pub fn new(user_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            user_id: user_id.into(),
            status: JobStatus::Pending,
            intent_id: None,
            cycles: 0,
            max_cycles: default_max_cycles(),
            seeds: None,
            context: None,
            error: None,
            agents: Vec::new(),
            llm_provider: None,
            llm_model: None,
            duration_ms: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        }
    }

    /// Mark job as running
    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark job as converged
    pub fn complete(&mut self, context: serde_json::Value, cycles: u32) {
        self.status = JobStatus::Converged;
        self.context = Some(context);
        self.cycles = cycles;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        if let Some(started) = self.started_at {
            self.duration_ms = Some((Utc::now() - started).num_milliseconds() as u64);
        }
    }

    /// Mark job as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = JobStatus::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        if let Some(started) = self.started_at {
            self.duration_ms = Some((Utc::now() - started).num_milliseconds() as u64);
        }
    }

    /// Mark job as cancelled
    pub fn cancel(&mut self) {
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

/// Job repository for Firestore
pub struct JobRepository {
    db: FirestoreDb,
    collection: String,
}

impl JobRepository {
    const COLLECTION: &'static str = "jobs";

    /// Create a new job repository
    pub fn new(db: FirestoreDb) -> Self {
        Self {
            db,
            collection: Self::COLLECTION.to_string(),
        }
    }

    /// Create a new job
    pub async fn create(&self, job: &Job) -> Result<String, FirestoreError> {
        let id = generate_id();
        let mut job_with_id = job.clone();
        job_with_id.id = Some(id.clone());

        self.db
            .fluent()
            .insert()
            .into(&self.collection)
            .document_id(&id)
            .object(&job_with_id)
            .execute::<Job>()
            .await?;
        Ok(id)
    }

    /// Get a job by ID
    pub async fn get(&self, id: &str) -> Result<Option<Job>, FirestoreError> {
        Ok(self
            .db
            .fluent()
            .select()
            .by_id_in(&self.collection)
            .obj()
            .one(id)
            .await?)
    }

    /// Update a job
    pub async fn update(&self, id: &str, job: &Job) -> Result<(), FirestoreError> {
        let mut updated = job.clone();
        updated.updated_at = Utc::now();

        let _: Job = self
            .db
            .fluent()
            .update()
            .in_col(&self.collection)
            .document_id(id)
            .object(&updated)
            .execute()
            .await?;
        Ok(())
    }

    /// Delete a job
    pub async fn delete(&self, id: &str) -> Result<(), FirestoreError> {
        self.db
            .fluent()
            .delete()
            .from(self.collection.as_str())
            .document_id(id)
            .execute()
            .await?;
        Ok(())
    }

    /// List jobs for a user
    pub async fn list_by_user(
        &self,
        user_id: &str,
        limit: u32,
    ) -> Result<Vec<Job>, FirestoreError> {
        Ok(self
            .db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .filter(|q| q.field("user_id").eq(user_id))
            .order_by([("created_at", FirestoreQueryDirection::Descending)])
            .limit(limit)
            .obj()
            .query()
            .await?)
    }

    /// List jobs by status
    pub async fn list_by_status(
        &self,
        status: JobStatus,
        limit: u32,
    ) -> Result<Vec<Job>, FirestoreError> {
        let status_str = match status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Converged => "converged",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        };

        Ok(self
            .db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .filter(|q| q.field("status").eq(status_str))
            .order_by([("updated_at", FirestoreQueryDirection::Descending)])
            .limit(limit)
            .obj()
            .query()
            .await?)
    }

    /// Get pending jobs (for worker polling)
    pub async fn get_pending(&self, limit: u32) -> Result<Vec<Job>, FirestoreError> {
        self.list_by_status(JobStatus::Pending, limit).await
    }

    /// Get running jobs (for monitoring)
    pub async fn get_running(&self) -> Result<Vec<Job>, FirestoreError> {
        self.list_by_status(JobStatus::Running, 100).await
    }

    /// Count jobs by status for a user
    pub async fn count_by_user_and_status(
        &self,
        user_id: &str,
        status: JobStatus,
    ) -> Result<usize, FirestoreError> {
        let jobs: Vec<Job> = self.list_by_user(user_id, 1000).await?;
        Ok(jobs.iter().filter(|j| j.status == status).count())
    }

    /// Store context snapshot (stored in job_snapshots collection with composite ID)
    pub async fn save_snapshot(
        &self,
        job_id: &str,
        cycle: u32,
        context: &serde_json::Value,
    ) -> Result<(), FirestoreError> {
        #[derive(Serialize, Deserialize)]
        struct Snapshot {
            job_id: String,
            cycle: u32,
            context: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let snapshot = Snapshot {
            job_id: job_id.to_string(),
            cycle,
            context: context.clone(),
            created_at: Utc::now(),
        };

        // Use composite document ID: job_id_cycle-XXXX
        let doc_id = format!("{}_{:04}", job_id, cycle);
        self.db
            .fluent()
            .insert()
            .into("job_snapshots")
            .document_id(&doc_id)
            .object(&snapshot)
            .execute::<Snapshot>()
            .await?;
        Ok(())
    }
}
