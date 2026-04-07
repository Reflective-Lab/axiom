// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("unsupported storage scheme: {0}")]
    UnsupportedScheme(String),

    #[error("invalid storage URI: {0}")]
    InvalidUri(String),

    #[error("local path does not exist: {}", .0.display())]
    LocalPathNotFound(PathBuf),

    #[error("object store error: {0}")]
    ObjectStore(#[from] object_store::Error),

    #[error("configuration error: {0}")]
    Config(String),
}
