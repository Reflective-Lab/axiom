// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Generated protobuf and gRPC contracts for Converge.
//!
//! This crate is the Rust representation of the versioned `converge.v1` wire
//! protocol defined in `schema/proto/converge.proto`.

pub use prost;
pub use prost_types;
pub use tonic;

pub mod v1 {
    #![allow(clippy::default_trait_access)]
    #![allow(clippy::doc_markdown)]
    #![allow(clippy::missing_errors_doc)]
    #![allow(clippy::must_use_candidate)]
    #![allow(clippy::similar_names)]

    tonic::include_proto!("converge.v1");
}
