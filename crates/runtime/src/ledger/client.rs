// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! LedgerClient - gRPC client for converge-ledger.

use std::collections::HashMap;

use tonic::Status;
use tonic::transport::Channel;

use super::generated::context_service_client::ContextServiceClient;
use super::generated::{
    AppendRequest, Entry, GetRequest, GetResponse, LoadRequest, LoadResponse, SnapshotRequest,
    SnapshotResponse, WatchEvent, WatchRequest,
};

/// Error type for ledger operations.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    /// Connection error.
    #[error("connection error: {0}")]
    Connection(#[from] tonic::transport::Error),

    /// gRPC status error.
    #[error("grpc error: {0}")]
    Grpc(#[from] Status),

    /// Invalid response from server.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

/// Result type for ledger operations.
pub type LedgerResult<T> = Result<T, LedgerError>;

/// gRPC client for the converge-ledger append-only store.
///
/// The client provides async methods for all ledger operations:
/// - `append`: Add entries to a context
/// - `get`: Retrieve entries from a context
/// - `snapshot`: Create a portable backup
/// - `load`: Restore from a snapshot
/// - `watch`: Stream new entries as they arrive
#[derive(Debug, Clone)]
pub struct LedgerClient {
    client: ContextServiceClient<Channel>,
}

impl LedgerClient {
    /// Connect to a ledger server.
    ///
    /// # Arguments
    ///
    /// * `addr` - The server address (e.g., "http://localhost:50051")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = LedgerClient::connect("http://localhost:50051").await?;
    /// ```
    pub async fn connect(addr: impl Into<String>) -> LedgerResult<Self> {
        let client = ContextServiceClient::connect(addr.into()).await?;
        Ok(Self { client })
    }

    /// Create a client from an existing channel.
    ///
    /// Useful when you want to configure the channel manually (timeouts, TLS, etc.).
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            client: ContextServiceClient::new(channel),
        }
    }

    /// Append an entry to a context.
    ///
    /// Each entry is assigned:
    /// - A unique ID
    /// - A monotonic sequence number
    /// - A Lamport clock timestamp
    /// - A content hash for integrity verification
    ///
    /// # Arguments
    ///
    /// * `context_id` - The context identifier (typically Root Intent ID)
    /// * `key` - The context key (e.g., "facts", "intents", "traces")
    /// * `payload` - The entry payload (opaque bytes)
    ///
    /// # Returns
    ///
    /// The created entry with all assigned fields.
    pub async fn append(
        &self,
        context_id: impl Into<String>,
        key: impl Into<String>,
        payload: impl Into<Vec<u8>>,
    ) -> LedgerResult<Entry> {
        self.append_with_metadata(context_id, key, payload, HashMap::new())
            .await
    }

    /// Append an entry with metadata.
    ///
    /// Same as `append`, but allows attaching metadata to the entry.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The context identifier
    /// * `key` - The context key
    /// * `payload` - The entry payload
    /// * `metadata` - Optional metadata (e.g., agent_id, cycle_number)
    pub async fn append_with_metadata(
        &self,
        context_id: impl Into<String>,
        key: impl Into<String>,
        payload: impl Into<Vec<u8>>,
        metadata: HashMap<String, String>,
    ) -> LedgerResult<Entry> {
        let request = AppendRequest {
            context_id: context_id.into(),
            key: key.into(),
            payload: payload.into(),
            metadata,
        };

        let response = self.client.clone().append(request).await?;
        response
            .into_inner()
            .entry
            .ok_or_else(|| LedgerError::InvalidResponse("missing entry in response".to_string()))
    }

    /// Get entries from a context.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The context identifier
    /// * `key` - Optional key filter (None = all keys)
    /// * `after_sequence` - Only return entries with sequence > this value
    /// * `limit` - Maximum number of entries (None = no limit)
    ///
    /// # Returns
    ///
    /// A tuple of (entries, latest_sequence).
    pub async fn get(
        &self,
        context_id: impl Into<String>,
        key: Option<String>,
        after_sequence: Option<u64>,
        limit: Option<u32>,
    ) -> LedgerResult<GetResponse> {
        let request = GetRequest {
            context_id: context_id.into(),
            key: key.unwrap_or_default(),
            after_sequence: after_sequence.unwrap_or(0),
            limit: limit.unwrap_or(0),
        };

        let response = self.client.clone().get(request).await?;
        Ok(response.into_inner())
    }

    /// Create a snapshot of the entire context.
    ///
    /// The snapshot is a portable blob that can be used to restore
    /// the context on another ledger instance.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The context identifier
    ///
    /// # Returns
    ///
    /// The snapshot response containing the blob and metadata.
    pub async fn snapshot(&self, context_id: impl Into<String>) -> LedgerResult<SnapshotResponse> {
        let request = SnapshotRequest {
            context_id: context_id.into(),
        };

        let response = self.client.clone().snapshot(request).await?;
        Ok(response.into_inner())
    }

    /// Load a context from a snapshot.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The target context identifier
    /// * `snapshot` - The snapshot blob from a previous `snapshot()` call
    /// * `fail_if_exists` - If true, fails if context already has entries
    ///
    /// # Returns
    ///
    /// The load response with entry count and latest sequence.
    pub async fn load(
        &self,
        context_id: impl Into<String>,
        snapshot: Vec<u8>,
        fail_if_exists: bool,
    ) -> LedgerResult<LoadResponse> {
        let request = LoadRequest {
            context_id: context_id.into(),
            snapshot,
            fail_if_exists,
        };

        let response = self.client.clone().load(request).await?;
        Ok(response.into_inner())
    }

    /// Watch for new entries in a context.
    ///
    /// Returns a streaming response that yields entries as they are appended.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The context identifier
    /// * `key` - Optional key filter
    /// * `from_sequence` - Start watching from this sequence (0 = from beginning)
    ///
    /// # Returns
    ///
    /// A streaming response of `WatchEvent`s.
    pub async fn watch(
        &self,
        context_id: impl Into<String>,
        key: Option<String>,
        from_sequence: Option<u64>,
    ) -> LedgerResult<tonic::Streaming<WatchEvent>> {
        let request = WatchRequest {
            context_id: context_id.into(),
            key: key.unwrap_or_default(),
            from_sequence: from_sequence.unwrap_or(0),
        };

        let response = self.client.clone().watch(request).await?;
        Ok(response.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_error_display() {
        let err = LedgerError::InvalidResponse("test error".to_string());
        assert_eq!(err.to_string(), "invalid response: test error");
    }
}
