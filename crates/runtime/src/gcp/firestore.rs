//! Firestore client wrapper
//!
//! Provides a high-level interface for Firestore operations with automatic
//! authentication using Application Default Credentials.

use firestore::*;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

use super::GcpConfig;

/// Firestore error types
#[derive(Error, Debug)]
pub enum FirestoreError {
    #[error("Firestore client error: {0}")]
    Client(#[from] firestore::errors::FirestoreError),

    #[error("Document not found: {collection}/{id}")]
    NotFound { collection: String, id: String },

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Firestore client wrapper
pub struct FirestoreClient {
    db: FirestoreDb,
    config: GcpConfig,
}

impl FirestoreClient {
    /// Create a new Firestore client
    ///
    /// Uses Application Default Credentials for authentication.
    /// On Cloud Run, this automatically uses the service account.
    /// Locally, use `gcloud auth application-default login`.
    pub async fn new(config: GcpConfig) -> Result<Self, FirestoreError> {
        let db = FirestoreDb::new(&config.project_id).await?;
        Ok(Self { db, config })
    }

    /// Get the underlying Firestore database
    pub fn db(&self) -> &FirestoreDb {
        &self.db
    }

    /// Get the GCP configuration
    pub fn config(&self) -> &GcpConfig {
        &self.config
    }

    // ========================================
    // Generic CRUD operations
    // ========================================

    /// Create a document with a specific ID
    pub async fn create<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        collection: &str,
        id: &str,
        data: &T,
    ) -> Result<(), FirestoreError> {
        self.db
            .fluent()
            .insert()
            .into(collection)
            .document_id(id)
            .object(data)
            .execute::<T>()
            .await?;
        Ok(())
    }

    /// Get a document by ID
    pub async fn get<T: DeserializeOwned + Send + Sync>(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<T, FirestoreError> {
        let doc: Option<T> = self
            .db
            .fluent()
            .select()
            .by_id_in(collection)
            .obj()
            .one(id)
            .await?;

        doc.ok_or_else(|| FirestoreError::NotFound {
            collection: collection.to_string(),
            id: id.to_string(),
        })
    }

    /// Get a document by ID, returning None if not found
    pub async fn get_opt<T: DeserializeOwned + Send + Sync>(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<T>, FirestoreError> {
        Ok(self
            .db
            .fluent()
            .select()
            .by_id_in(collection)
            .obj()
            .one(id)
            .await?)
    }

    /// Update a document (merge with existing)
    pub async fn update<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        collection: &str,
        id: &str,
        data: &T,
    ) -> Result<(), FirestoreError> {
        let _: T = self
            .db
            .fluent()
            .update()
            .in_col(collection)
            .document_id(id)
            .object(data)
            .execute()
            .await?;
        Ok(())
    }

    /// Upsert a document (create or replace)
    pub async fn upsert<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        collection: &str,
        id: &str,
        data: &T,
    ) -> Result<(), FirestoreError> {
        let _: T = self
            .db
            .fluent()
            .update()
            .in_col(collection)
            .document_id(id)
            .object(data)
            .execute()
            .await?;
        Ok(())
    }

    /// Delete a document
    pub async fn delete(&self, collection: &str, id: &str) -> Result<(), FirestoreError> {
        self.db
            .fluent()
            .delete()
            .from(collection)
            .document_id(id)
            .execute()
            .await?;
        Ok(())
    }

    /// List documents in a collection
    pub async fn list<T: DeserializeOwned + Send + Sync>(
        &self,
        collection: &str,
        limit: Option<usize>,
    ) -> Result<Vec<T>, FirestoreError> {
        let mut query = self.db.fluent().select().from(collection);

        if let Some(limit) = limit {
            query = query.limit(limit as u32);
        }

        Ok(query.obj().query().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = GcpConfig::default();
        assert_eq!(config.project_id, "hey-sh-production");
        assert_eq!(config.firestore_database, "(default)");
    }
}
