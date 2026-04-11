// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Build script for protobuf code generation.
//!
//! Compiles `context.proto` when the `grpc` feature is enabled and the proto file
//! exists. The shared `converge.v1` contract now comes from `converge-protocol`.

fn main() {
    #[cfg(feature = "grpc")]
    {
        let schema_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema/proto");
        let context_proto = format!("{schema_dir}/context.proto");
        if std::path::Path::new(&context_proto).exists() {
            println!("cargo:rerun-if-changed={context_proto}");
            tonic_build::configure()
                .build_server(false)
                .build_client(true)
                .out_dir("src/ledger/generated")
                .compile(&[&context_proto], &[schema_dir])
                .expect("Failed to compile context.proto");
        }
    }
}
