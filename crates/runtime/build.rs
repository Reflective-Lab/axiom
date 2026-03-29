// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Build script for protobuf code generation.
//!
//! Compiles proto files when the `grpc` feature is enabled and proto files exist.
//! Pre-generated code is checked in under src/ledger/generated/ and src/grpc/generated/.

fn main() {
    #[cfg(feature = "grpc")]
    {
        let schema_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema/proto");
        let context_proto = format!("{schema_dir}/context.proto");
        let converge_proto = format!("{schema_dir}/converge.proto");

        if std::path::Path::new(&context_proto).exists() {
            println!("cargo:rerun-if-changed={context_proto}");
            tonic_build::configure()
                .build_server(false)
                .build_client(true)
                .out_dir("src/ledger/generated")
                .compile(&[&context_proto], &[schema_dir])
                .expect("Failed to compile context.proto");
        }

        if std::path::Path::new(&converge_proto).exists() {
            println!("cargo:rerun-if-changed={converge_proto}");
            tonic_build::configure()
                .build_server(true)
                .build_client(true)
                .out_dir("src/grpc/generated")
                .compile(&[&converge_proto], &[schema_dir])
                .expect("Failed to compile converge.proto");
        }
    }
}
