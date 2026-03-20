// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Build script for protobuf code generation.
//!
//! Compiles proto files when the `grpc` feature is enabled and proto files exist.
//! Pre-generated code is checked in under src/ledger/generated/ and src/grpc/generated/.

fn main() {
    #[cfg(feature = "grpc")]
    {
        let context_proto = std::path::Path::new("proto/context.proto");
        let converge_proto = std::path::Path::new("proto/converge.proto");

        if context_proto.exists() {
            println!("cargo:rerun-if-changed=proto/context.proto");
            tonic_build::configure()
                .build_server(false)
                .build_client(true)
                .out_dir("src/ledger/generated")
                .compile(&["proto/context.proto"], &["proto"])
                .expect("Failed to compile context.proto");
        }

        if converge_proto.exists() {
            println!("cargo:rerun-if-changed=proto/converge.proto");
            tonic_build::configure()
                .build_server(true)
                .build_client(true)
                .out_dir("src/grpc/generated")
                .compile(&["proto/converge.proto"], &["proto"])
                .expect("Failed to compile converge.proto");
        }
    }
}
