// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::StorageError;

/// Identifies the storage backend.
///
/// Bucket-local object key prefixes are configured separately via
/// [`crate::StorageConfig::prefix`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub enum StorageUri {
    Local(PathBuf),
    S3 {
        bucket: String,
        endpoint: Option<String>,
        region: Option<String>,
    },
    Gcs {
        bucket: String,
    },
}

impl StorageUri {
    #[must_use]
    pub fn scheme(&self) -> &str {
        match self {
            Self::Local(_) => "file",
            Self::S3 { .. } => "s3",
            Self::Gcs { .. } => "gs",
        }
    }
}

impl FromStr for StorageUri {
    type Err = StorageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix("s3://") {
            let bucket = rest
                .split('/')
                .next()
                .ok_or_else(|| StorageError::InvalidUri(s.to_string()))?
                .to_string();
            if bucket.is_empty() {
                return Err(StorageError::InvalidUri(s.to_string()));
            }
            Ok(Self::S3 {
                bucket,
                endpoint: None,
                region: None,
            })
        } else if let Some(rest) = s.strip_prefix("gs://") {
            let bucket = rest
                .split('/')
                .next()
                .ok_or_else(|| StorageError::InvalidUri(s.to_string()))?
                .to_string();
            if bucket.is_empty() {
                return Err(StorageError::InvalidUri(s.to_string()));
            }
            Ok(Self::Gcs { bucket })
        } else if let Some(path) = s.strip_prefix("file://") {
            Ok(Self::Local(PathBuf::from(path)))
        } else {
            // Treat plain paths as local
            Ok(Self::Local(PathBuf::from(s)))
        }
    }
}

impl TryFrom<String> for StorageUri {
    type Error = StorageError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<StorageUri> for String {
    fn from(uri: StorageUri) -> Self {
        match uri {
            StorageUri::Local(path) => format!("file://{}", path.display()),
            StorageUri::S3 { bucket, .. } => format!("s3://{bucket}"),
            StorageUri::Gcs { bucket } => format!("gs://{bucket}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_local_path() {
        let uri: StorageUri = "./data/parquet".parse().unwrap();
        assert_eq!(uri, StorageUri::Local(PathBuf::from("./data/parquet")));
    }

    #[test]
    fn parse_file_uri() {
        let uri: StorageUri = "file:///tmp/data".parse().unwrap();
        assert_eq!(uri, StorageUri::Local(PathBuf::from("/tmp/data")));
    }

    #[test]
    fn parse_s3_uri() {
        let uri: StorageUri = "s3://my-bucket/prefix/data".parse().unwrap();
        assert!(matches!(uri, StorageUri::S3 { bucket, .. } if bucket == "my-bucket"));
    }

    #[test]
    fn parse_gs_uri() {
        let uri: StorageUri = "gs://my-bucket/path".parse().unwrap();
        assert!(matches!(uri, StorageUri::Gcs { bucket } if bucket == "my-bucket"));
    }

    #[test]
    fn empty_bucket_is_error() {
        assert!("s3://".parse::<StorageUri>().is_err());
        assert!("gs://".parse::<StorageUri>().is_err());
    }

    #[test]
    fn serializes_bucket_only_for_object_store_uris() {
        let s3 = StorageUri::S3 {
            bucket: "bucket".to_string(),
            endpoint: Some("http://localhost:9000".to_string()),
            region: Some("us-east-1".to_string()),
        };
        let gcs = StorageUri::Gcs {
            bucket: "bucket".to_string(),
        };

        assert_eq!(String::from(s3), "s3://bucket");
        assert_eq!(String::from(gcs), "gs://bucket");
    }
}
