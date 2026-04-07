// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # converge-storage
//!
//! Object storage abstraction for Converge. Provides a unified interface
//! for storing and retrieving blobs (parquet files, model weights, artifacts)
//! across local filesystem, S3-compatible stores (AWS S3, `MinIO`, `RustFS`),
//! and Google Cloud Storage.
//!
//! ## Storage URI Convention
//!
//! - `file:///path/to/dir` or plain paths → local filesystem
//! - `s3://bucket` → S3-compatible (AWS, `MinIO`, `RustFS`)
//! - `gs://bucket` → Google Cloud Storage
//!
//! Object key prefixes are configured separately via [`StorageConfig::prefix`].
//!
//! ## Feature Flags
//!
//! - `local` (default) — local filesystem backend
//! - `s3` — S3-compatible backend (AWS, `MinIO`, `RustFS`)
//! - `gcs` — Google Cloud Storage backend
//! - `all-backends` — all of the above

mod config;
mod error;
mod uri;

#[cfg(feature = "gcs")]
mod gcs;
#[cfg(feature = "local")]
mod local;
#[cfg(feature = "s3")]
mod s3;

pub use config::StorageConfig;
pub use error::StorageError;
pub use uri::StorageUri;

use std::sync::Arc;

pub use object_store::path::Path as ObjectPath;
pub use object_store::{self, GetResult, ObjectStore, PutResult};

#[cfg(any(feature = "s3", test))]
fn resolve_s3_options<'a>(
    config: &'a StorageConfig,
    endpoint: Option<&'a String>,
    region: Option<&'a String>,
) -> (Option<&'a str>, Option<&'a str>) {
    (
        config.endpoint.as_deref().or(endpoint.map(String::as_str)),
        config.region.as_deref().or(region.map(String::as_str)),
    )
}

/// Build an [`ObjectStore`] from a [`StorageConfig`].
///
/// Returns a type-erased `Arc<dyn ObjectStore>` suitable for use across
/// async boundaries. The concrete backend is selected by the URI scheme.
///
/// # Errors
///
/// Returns [`StorageError`] if the URI scheme is unsupported or if the
/// backend cannot be configured (e.g., missing credentials).
pub fn build_store(config: &StorageConfig) -> Result<Arc<dyn ObjectStore>, StorageError> {
    match &config.uri {
        #[cfg(feature = "local")]
        StorageUri::Local(path) => local::build(path),

        #[cfg(feature = "s3")]
        StorageUri::S3 {
            bucket,
            endpoint,
            region,
        } => {
            let (endpoint, region) = resolve_s3_options(config, endpoint.as_ref(), region.as_ref());
            s3::build(bucket, endpoint, region)
        }

        #[cfg(feature = "gcs")]
        StorageUri::Gcs { bucket } if config.public => gcs::build_public(bucket),

        #[cfg(feature = "gcs")]
        StorageUri::Gcs { bucket } => gcs::build(bucket),

        #[allow(unreachable_patterns)]
        other => Err(StorageError::UnsupportedScheme(other.scheme().to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_s3_options_override_uri_values() {
        let config = StorageConfig {
            uri: StorageUri::S3 {
                bucket: "bucket".to_string(),
                endpoint: Some("http://uri-endpoint:9000".to_string()),
                region: Some("uri-region".to_string()),
            },
            prefix: None,
            public: false,
            endpoint: Some("http://config-endpoint:9000".to_string()),
            region: Some("config-region".to_string()),
        };

        let StorageUri::S3 {
            endpoint, region, ..
        } = &config.uri
        else {
            unreachable!();
        };

        let (resolved_endpoint, resolved_region) =
            resolve_s3_options(&config, endpoint.as_ref(), region.as_ref());

        assert_eq!(resolved_endpoint, Some("http://config-endpoint:9000"));
        assert_eq!(resolved_region, Some("config-region"));
    }

    #[test]
    fn uri_s3_options_are_used_as_fallback() {
        let config = StorageConfig {
            uri: StorageUri::S3 {
                bucket: "bucket".to_string(),
                endpoint: Some("http://uri-endpoint:9000".to_string()),
                region: Some("uri-region".to_string()),
            },
            prefix: None,
            public: false,
            endpoint: None,
            region: None,
        };

        let StorageUri::S3 {
            endpoint, region, ..
        } = &config.uri
        else {
            unreachable!();
        };

        let (resolved_endpoint, resolved_region) =
            resolve_s3_options(&config, endpoint.as_ref(), region.as_ref());

        assert_eq!(resolved_endpoint, Some("http://uri-endpoint:9000"));
        assert_eq!(resolved_region, Some("uri-region"));
    }
}
